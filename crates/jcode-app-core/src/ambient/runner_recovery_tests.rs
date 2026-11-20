//! Delivery through the runner: stale targets never wake, acknowledgement is
//! recorded as acceptance only, and a failed delivery is never redelivered.

use super::*;
use crate::ambient::{AmbientManager, ItemState, ScheduleRequest};

fn request_for(session_id: &str) -> ScheduleRequest {
    ScheduleRequest {
        wake_in_minutes: None,
        wake_at: Some(chrono::Utc::now() - chrono::Duration::seconds(1)),
        context: "Follow up".into(),
        priority: Priority::Normal,
        target: ScheduleTarget::Session {
            session_id: session_id.into(),
        },
        created_by_session: session_id.into(),
        working_dir: None,
        task_description: Some("Follow up".into()),
        relevant_files: vec![],
        git_branch: None,
        additional_context: None,
    }
}

fn history_state(id: &str) -> (ItemState, Vec<crate::ambient::DeliveryAttempt>) {
    let mgr = AmbientManager::new().expect("manager");
    let item = mgr
        .queue()
        .recent_history(100)
        .into_iter()
        .find(|item| item.id == id)
        .unwrap_or_else(|| panic!("{id} not in history"))
        .clone();
    (item.delivery.state, item.delivery.attempts)
}

#[tokio::test]
async fn revoked_target_is_suppressed_never_woken_and_recorded_cancelled() {
    let _guard = crate::storage::lock_test_env();
    let temp = tempfile::tempdir().expect("tempdir");
    let _home = EnvVarGuard::set_path("JCODE_HOME", temp.path());
    let _runtime = EnvVarGuard::set_path("JCODE_RUNTIME_DIR", temp.path());
    // A woken session would stream a response; the test provider has none, so
    // any delivery would be recorded as uncertain instead of cancelled.
    let provider: Arc<dyn Provider> = Arc::new(TestProvider);
    let mut session = Session::create_with_id(
        "session_revoked_target".into(),
        None,
        Some("Revoked".into()),
    );
    session.save().expect("save session");

    let mut mgr = AmbientManager::new().unwrap();
    let id = mgr.schedule(request_for(&session.id)).unwrap();
    // The stop lands after the item was claimed (below) but before delivery.
    let claimed = mgr.take_ready_direct_items();
    assert_eq!(claimed.len(), 1);
    let store = crate::server::session_control::store();
    store.register(&session.id, None).unwrap();
    store
        .request_stop(&session.id, &session.id, false, "stop-1")
        .unwrap();

    let runner = AmbientRunnerHandle::new(Arc::new(crate::safety::SafetySystem::new()));
    runner.deliver_ready_direct_items(&provider, claimed).await;

    let (state, attempts) = history_state(&id);
    assert_eq!(state, ItemState::Cancelled);
    assert!(attempts[0].acked_at.is_none());
    assert!(
        attempts[0]
            .note
            .as_deref()
            .unwrap()
            .contains("not runnable")
    );
    let reloaded = Session::load(&session.id).unwrap();
    assert!(
        reloaded.messages.is_empty(),
        "a revoked session must not receive the wake"
    );
}

#[tokio::test]
async fn acknowledged_delivery_is_recorded_as_delivered_and_not_redelivered() {
    let _guard = crate::storage::lock_test_env();
    let temp = tempfile::tempdir().expect("tempdir");
    let _home = EnvVarGuard::set_path("JCODE_HOME", temp.path());
    let _runtime = EnvVarGuard::set_path("JCODE_RUNTIME_DIR", temp.path());
    let streaming = StreamingTestProvider::default();
    streaming.queue_response(vec![
        StreamEvent::TextDelta("Done.".to_string()),
        StreamEvent::MessageEnd { stop_reason: None },
    ]);
    let provider: Arc<dyn Provider> = Arc::new(streaming);
    let mut session =
        Session::create_with_id("session_ack_target".into(), None, Some("Ack".into()));
    session.save().expect("save session");

    let mut mgr = AmbientManager::new().unwrap();
    let id = mgr.schedule(request_for(&session.id)).unwrap();
    let claimed = mgr.take_ready_direct_items();
    let runner = AmbientRunnerHandle::new(Arc::new(crate::safety::SafetySystem::new()));
    runner.deliver_ready_direct_items(&provider, claimed).await;

    let (state, attempts) = history_state(&id);
    assert_eq!(state, ItemState::Delivered, "{:?}", attempts[0].note);
    assert_eq!(attempts.len(), 1);
    assert_eq!(
        attempts[0].acked_target.as_deref(),
        Some(session.id.as_str())
    );
    // Acceptance only: the record says nothing about the work's outcome.
    let mut mgr = AmbientManager::new().unwrap();
    assert!(mgr.take_ready_direct_items().is_empty());
    assert!(mgr.take_ready_items().is_empty());
}

#[tokio::test]
async fn failed_delivery_is_uncertain_and_not_retried_automatically() {
    let _guard = crate::storage::lock_test_env();
    let temp = tempfile::tempdir().expect("tempdir");
    let _home = EnvVarGuard::set_path("JCODE_HOME", temp.path());
    let _runtime = EnvVarGuard::set_path("JCODE_RUNTIME_DIR", temp.path());
    let provider: Arc<dyn Provider> = Arc::new(TestProvider);

    let mut mgr = AmbientManager::new().unwrap();
    let id = mgr.schedule(request_for("session_never_saved")).unwrap();
    let claimed = mgr.take_ready_direct_items();
    let runner = AmbientRunnerHandle::new(Arc::new(crate::safety::SafetySystem::new()));
    runner.deliver_ready_direct_items(&provider, claimed).await;

    let (state, attempts) = history_state(&id);
    assert_eq!(state, ItemState::Uncertain);
    assert!(attempts[0].acked_at.is_none());
    let mut mgr = AmbientManager::new().unwrap();
    assert!(mgr.take_ready_direct_items().is_empty());

    // Only an explicit retry re-queues it, as a new attempt.
    mgr.retry_schedule(&id).unwrap().expect("retry uncertain");
    let again = mgr.take_ready_direct_items();
    assert_eq!(again[0].delivery.attempts.len(), 2);
    assert!(again[0].delivery.attempts[1].manual_retry);
}

/// Register `session_id`, run one generation, stop and settle it, then resume
/// it as a new generation. Returns `(revoked_through, current generation)`.
fn stop_and_resume(session_id: &str) -> (u64, u64) {
    let store = crate::server::session_control::store();
    store.register(session_id, None).unwrap();
    let first = store.admit(session_id).unwrap();
    let stop = store
        .request_stop(session_id, session_id, false, "stop-generation")
        .unwrap();
    let settled = store.settle(session_id, stop.revision).unwrap();
    let second = store.resume(session_id, settled.revision).unwrap();
    assert!(second.generation > first.generation);
    (first.generation, second.generation)
}

async fn deliver_with_origin(session_id: &str, origin: u64, provider: Arc<dyn Provider>) -> String {
    let mut mgr = AmbientManager::new().unwrap();
    let id = mgr.schedule(request_for(session_id)).unwrap();
    let mut claimed = mgr.take_ready_direct_items();
    claimed[0].delivery.origin_generation = Some(origin);
    let runner = AmbientRunnerHandle::new(Arc::new(crate::safety::SafetySystem::new()));
    runner.deliver_ready_direct_items(&provider, claimed).await;
    id
}

#[tokio::test]
async fn item_from_a_revoked_generation_is_suppressed_even_after_resume() {
    let _guard = crate::storage::lock_test_env();
    let temp = tempfile::tempdir().expect("tempdir");
    let _home = EnvVarGuard::set_path("JCODE_HOME", temp.path());
    let _runtime = EnvVarGuard::set_path("JCODE_RUNTIME_DIR", temp.path());
    let provider: Arc<dyn Provider> = Arc::new(TestProvider);
    let mut session = Session::create_with_id("session_stale_gen".into(), None, Some("S".into()));
    session.save().unwrap();
    let (revoked, _current) = stop_and_resume(&session.id);

    // The session is runnable again, but the item was scheduled by the
    // generation the stop revoked.
    let id = deliver_with_origin(&session.id, revoked, provider).await;
    let (state, attempts) = history_state(&id);
    assert_eq!(state, ItemState::Cancelled);
    assert!(attempts[0].note.as_deref().unwrap().contains("was revoked"));
    assert!(Session::load(&session.id).unwrap().messages.is_empty());
}

#[tokio::test]
async fn item_from_a_newer_generation_than_the_revocation_still_delivers() {
    let _guard = crate::storage::lock_test_env();
    let temp = tempfile::tempdir().expect("tempdir");
    let _home = EnvVarGuard::set_path("JCODE_HOME", temp.path());
    let _runtime = EnvVarGuard::set_path("JCODE_RUNTIME_DIR", temp.path());
    let streaming = StreamingTestProvider::default();
    streaming.queue_response(vec![
        StreamEvent::TextDelta("Done.".to_string()),
        StreamEvent::MessageEnd { stop_reason: None },
    ]);
    let provider: Arc<dyn Provider> = Arc::new(streaming);
    let mut session = Session::create_with_id("session_new_gen".into(), None, Some("N".into()));
    session.save().unwrap();
    let (_revoked, current) = stop_and_resume(&session.id);

    let id = deliver_with_origin(&session.id, current, provider).await;
    let (state, attempts) = history_state(&id);
    assert_eq!(state, ItemState::Delivered, "{:?}", attempts[0].note);
}

#[tokio::test]
async fn cron_runner_loop_admits_delivers_once_and_skips_temporary_servers() {
    use crate::ambient::recurring::{MissedPolicy, NewSchedule, RecurringStore};
    let _guard = crate::storage::lock_test_env();
    let temp = tempfile::tempdir().expect("tempdir");
    let _home = EnvVarGuard::set_path("JCODE_HOME", temp.path());
    let _runtime = EnvVarGuard::set_path("JCODE_RUNTIME_DIR", temp.path());
    let streaming = StreamingTestProvider::default();
    streaming.queue_response(vec![
        StreamEvent::TextDelta("Done.".to_string()),
        StreamEvent::MessageEnd { stop_reason: None },
    ]);
    let provider: Arc<dyn Provider> = Arc::new(streaming);
    let mut session =
        Session::create_with_id("session_cron_target".into(), None, Some("Cron".into()));
    session.save().expect("save session");

    let store = RecurringStore::open_default().unwrap();
    let schedule = store
        .create(
            NewSchedule {
                name: None,
                cron: "* * * * *".into(),
                zone: "UTC".into(),
                missed: MissedPolicy::RunOnce,
                max_active: 1,
                token_cap: None,
                spend_cap_usd: None,
                template: request_for(&session.id),
            },
            chrono::Utc::now() - chrono::Duration::minutes(3),
        )
        .unwrap();
    let runner = AmbientRunnerHandle::new(Arc::new(crate::safety::SafetySystem::new()));

    // A temporary server never runs recurring schedules.
    crate::env::set_var("JCODE_TEMP_SERVER", "1");
    assert!(runner.tick_recurring_schedules().is_none());
    crate::env::remove_var("JCODE_TEMP_SERVER");
    assert!(AmbientManager::new().unwrap().queue().items().is_empty());

    // The shared daemon admits the occurrence through the one queue.
    assert!(runner.tick_recurring_schedules().is_some());
    let mut mgr = AmbientManager::new().unwrap();
    let claimed = mgr.take_ready_direct_items();
    assert_eq!(claimed.len(), 1);
    let key = claimed[0].id.clone();
    assert!(key.starts_with(&format!("cron:{}:", schedule.id)));
    runner.deliver_ready_direct_items(&provider, claimed).await;
    let (state, _) = history_state(&key);
    assert_eq!(state, ItemState::Delivered, "delivery, not task success");

    // A later pass (and a restarted store) never re-creates the same key.
    runner.tick_recurring_schedules();
    let reopened = RecurringStore::open_default().unwrap();
    let kept = reopened.get(&schedule.id).unwrap().unwrap();
    assert_eq!(kept.occurrences.iter().filter(|o| o.key == key).count(), 1);
    let mgr = AmbientManager::new().unwrap();
    let in_queue = mgr
        .queue()
        .items()
        .iter()
        .chain(mgr.queue().in_flight())
        .chain(mgr.queue().recent_history(usize::MAX))
        .filter(|item| item.id == key)
        .count();
    assert_eq!(in_queue, 1);
}

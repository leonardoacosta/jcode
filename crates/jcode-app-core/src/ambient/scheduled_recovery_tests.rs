//! Queue-level durability: claims stay persisted, recovery marks instead of
//! delivering, and manual retry keeps attempt history.

use super::*;
use crate::ambient::{AmbientManager, DeliveryRecord, Priority, ScheduleRequest, ScheduleTarget};

fn item(name: &str, minutes_from_now: i64) -> ScheduledItem {
    AmbientManager::scheduled_item(ScheduleRequest {
        wake_in_minutes: None,
        wake_at: Some(Utc::now() + chrono::Duration::minutes(minutes_from_now)),
        context: name.into(),
        priority: Priority::Normal,
        target: ScheduleTarget::Session {
            session_id: name.into(),
        },
        created_by_session: name.into(),
        working_dir: None,
        task_description: None,
        relevant_files: vec![],
        git_branch: None,
        additional_context: None,
    })
}

/// Pretend the claiming process died: its attempt is owned by nobody alive.
fn orphan_claims(queue: &mut ScheduledQueue) {
    queue
        .transact(|items| {
            for item in items {
                for attempt in &mut item.delivery.attempts {
                    attempt.owner = "legacy".into();
                }
            }
            Ok(())
        })
        .unwrap();
}

fn state_of(queue: &ScheduledQueue, id: &str) -> ItemState {
    queue
        .items()
        .iter()
        .chain(queue.in_flight())
        .chain(queue.recent_history(usize::MAX))
        .find(|item| item.id == id)
        .unwrap()
        .delivery
        .state
}

const GRACE: chrono::Duration = chrono::Duration::minutes(5);

#[test]
fn claim_keeps_item_persisted_as_dispatched() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("queue.json");
    let mut queue = ScheduledQueue::load(path.clone());
    let due = item("a", -1);
    let id = due.id.clone();
    queue.try_push(due).unwrap();
    assert_eq!(queue.take_ready_direct_items().len(), 1);
    // Crash here: nothing was delivered or settled, yet the record exists.
    let reloaded = ScheduledQueue::load(path);
    assert!(reloaded.items().is_empty());
    assert_eq!(reloaded.in_flight()[0].id, id);
    assert_eq!(reloaded.in_flight()[0].delivery.attempts.len(), 1);
}

#[test]
fn crash_before_ack_becomes_uncertain_and_is_never_redelivered() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("queue.json");
    let mut queue = ScheduledQueue::load(path.clone());
    let due = item("a", -1);
    let id = due.id.clone();
    queue.try_push(due).unwrap();
    queue.take_ready_direct_items();
    orphan_claims(&mut queue);

    let mut restarted = ScheduledQueue::load(path);
    let report = restarted.recover_startup(GRACE).unwrap();
    assert_eq!((report.uncertain, report.missed), (1, 0));
    assert_eq!(state_of(&restarted, &id), ItemState::Uncertain);
    assert!(restarted.take_ready_direct_items().is_empty());
    assert!(restarted.pop_ready().is_empty());
    // Recovery is idempotent and still delivers nothing.
    assert_eq!(
        restarted.recover_startup(GRACE).unwrap(),
        RecoveryReport::default()
    );
    assert!(restarted.pop_ready().is_empty());
}

#[test]
fn live_claim_by_this_process_is_left_to_its_owner() {
    let dir = tempfile::tempdir().unwrap();
    let mut queue = ScheduledQueue::load(dir.path().join("queue.json"));
    queue.try_push(item("a", -1)).unwrap();
    queue.take_ready_direct_items();
    assert_eq!(
        queue.recover_startup(GRACE).unwrap(),
        RecoveryReport::default()
    );
    assert_eq!(queue.in_flight().len(), 1);
}

#[test]
fn overdue_items_are_missed_with_delay_and_not_delivered() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("queue.json");
    let mut queue = ScheduledQueue::load(path);
    let overdue = item("late", -90);
    let within_grace = item("recent", -1);
    let future = item("future", 60);
    let (late_id, recent_id, future_id) = (
        overdue.id.clone(),
        within_grace.id.clone(),
        future.id.clone(),
    );
    for it in [overdue, within_grace, future] {
        queue.try_push(it).unwrap();
    }
    let report = queue.recover_startup(GRACE).unwrap();
    assert_eq!((report.uncertain, report.missed), (0, 1));
    assert_eq!(state_of(&queue, &late_id), ItemState::Missed);
    let missed = queue.recent_history(10)[0];
    let delay = missed.delivery.missed_delay_secs.unwrap();
    assert!((89 * 60..=91 * 60).contains(&delay), "delay was {delay}");
    assert!(missed.delivery.attempts.is_empty(), "never dispatched");
    assert_eq!(state_of(&queue, &recent_id), ItemState::Pending);
    assert_eq!(state_of(&queue, &future_id), ItemState::Pending);
}

#[test]
fn ambient_target_items_are_not_marked_missed() {
    let dir = tempfile::tempdir().unwrap();
    let mut queue = ScheduledQueue::load(dir.path().join("queue.json"));
    let mut ambient = item("late", -90);
    ambient.target = ScheduleTarget::Ambient;
    queue.try_push(ambient).unwrap();
    assert_eq!(
        queue.recover_startup(GRACE).unwrap(),
        RecoveryReport::default()
    );
    assert_eq!(queue.len(), 1);
}

#[test]
fn ack_moves_item_to_delivered_history_once() {
    let dir = tempfile::tempdir().unwrap();
    let mut queue = ScheduledQueue::load(dir.path().join("queue.json"));
    let due = item("a", -1);
    let id = due.id.clone();
    queue.try_push(due).unwrap();
    queue.take_ready_direct_items();
    let ack = Settlement::Acked {
        target: "a".into(),
        generation: Some(3),
    };
    assert!(queue.settle(&id, ack.clone()).unwrap());
    assert!(!queue.settle(&id, ack).unwrap(), "already settled");
    let delivered = queue.recent_history(10)[0];
    assert_eq!(delivered.delivery.state, ItemState::Delivered);
    let attempt = &delivered.delivery.attempts[0];
    assert_eq!(attempt.acked_target.as_deref(), Some("a"));
    assert_eq!(attempt.generation, Some(3));
    assert!(queue.pop_ready().is_empty());
    assert!(queue.take_ready_direct_items().is_empty());
}

#[test]
fn manual_retry_adds_attempt_and_preserves_history() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("queue.json");
    let mut queue = ScheduledQueue::load(path.clone());
    let due = item("a", -1);
    let id = due.id.clone();
    queue.try_push(due).unwrap();
    queue.take_ready_direct_items();
    orphan_claims(&mut queue);
    queue.recover_startup(GRACE).unwrap();

    // Not retryable while pending or acknowledged.
    assert!(queue.retry("missing").unwrap().is_none());
    let retried = queue.retry(&id).unwrap().unwrap();
    assert_eq!(retried.delivery.state, ItemState::Pending);
    assert!(
        queue.retry(&id).unwrap().is_none(),
        "retry is explicit, once"
    );

    let claimed = queue.take_ready_direct_items();
    let attempts = &claimed[0].delivery.attempts;
    assert_eq!(attempts.len(), 2);
    assert!(!attempts[0].manual_retry && attempts[1].manual_retry);
    assert!(
        attempts[0]
            .note
            .as_deref()
            .unwrap()
            .contains("no acknowledgement"),
        "first attempt keeps its uncertain note"
    );
    queue
        .settle(
            &id,
            Settlement::Acked {
                target: "a".into(),
                generation: None,
            },
        )
        .unwrap();
    let reloaded = ScheduledQueue::load(path);
    let done = reloaded.recent_history(10)[0];
    assert_eq!(done.delivery.attempts.len(), 2);
    assert!(done.delivery.attempts[0].acked_at.is_none());
    assert!(done.delivery.attempts[1].acked_at.is_some());
}

#[test]
fn cancel_pending_goes_to_history_and_dispatched_cannot_be_cancelled() {
    let dir = tempfile::tempdir().unwrap();
    let mut queue = ScheduledQueue::load(dir.path().join("queue.json"));
    let (pending, claimed) = (item("p", 60), item("c", -1));
    let (pending_id, claimed_id) = (pending.id.clone(), claimed.id.clone());
    queue.try_push(pending).unwrap();
    queue.try_push(claimed).unwrap();
    queue.take_ready_direct_items();
    assert!(queue.cancel(&claimed_id).is_err());
    assert_eq!(queue.cancel(&pending_id).unwrap().unwrap().id, pending_id);
    assert_eq!(state_of(&queue, &pending_id), ItemState::Cancelled);
    assert!(queue.cancel(&pending_id).unwrap().is_none());
}

#[test]
fn history_is_bounded_but_uncertain_is_kept() {
    let dir = tempfile::tempdir().unwrap();
    let mut queue = ScheduledQueue::load(dir.path().join("queue.json"));
    let uncertain = item("u", -1);
    let uncertain_id = uncertain.id.clone();
    queue.try_push(uncertain).unwrap();
    queue.take_ready_direct_items();
    queue
        .settle(&uncertain_id, Settlement::Uncertain { note: "n".into() })
        .unwrap();
    for n in 0..HISTORY_LIMIT + 5 {
        let it = item(&format!("d{n}"), -1);
        let id = it.id.clone();
        queue.try_push(it).unwrap();
        queue.take_ready_direct_items();
        queue
            .settle(
                &id,
                Settlement::Acked {
                    target: "t".into(),
                    generation: None,
                },
            )
            .unwrap();
    }
    assert_eq!(state_of(&queue, &uncertain_id), ItemState::Uncertain);
    let all = queue.recent_history(usize::MAX);
    assert_eq!(all.len(), HISTORY_LIMIT + 1);
    // Even a tiny window still shows the uncertain item.
    assert!(queue.recent_history(1).iter().any(|i| i.id == uncertain_id));
}

#[test]
fn pending_items_serialize_without_delivery_noise() {
    let json = serde_json::to_value(item("a", 5)).unwrap();
    assert!(json.get("delivery").is_none());
    let legacy: ScheduledItem = serde_json::from_value(json).unwrap();
    assert_eq!(legacy.delivery.state, ItemState::Pending);
}

fn set_claim(queue: &mut ScheduledQueue, owner: &str, age: chrono::Duration) {
    queue
        .transact(|items| {
            for item in items {
                for attempt in &mut item.delivery.attempts {
                    attempt.owner = owner.into();
                    attempt.dispatched_at = Utc::now() - age;
                }
            }
            Ok(())
        })
        .unwrap();
}

fn claimed_queue() -> (tempfile::TempDir, ScheduledQueue, String) {
    let dir = tempfile::tempdir().unwrap();
    let mut queue = ScheduledQueue::load(dir.path().join("queue.json"));
    let due = item("a", -1);
    let id = due.id.clone();
    queue.try_push(due).unwrap();
    queue.take_ready_direct_items();
    (dir, queue, id)
}

/// A live process that is not this one (the test runner's parent).
fn other_live_pid() -> u32 {
    std::os::unix::process::parent_id()
}

#[test]
fn pid_reuse_cannot_keep_a_claim_alive_forever() {
    let pid = other_live_pid();
    let real_start = crate::platform::process_start_time(pid);

    // Owner from another run whose pid is now held by an unrelated live
    // process: the start time differs, so the claim is abandoned.
    let (_dir, mut queue, id) = claimed_queue();
    set_claim(
        &mut queue,
        &format!("{pid}:1:deadbeef"),
        chrono::Duration::seconds(5),
    );
    if real_start.is_some() {
        assert_eq!(queue.recover_startup(GRACE).unwrap().uncertain, 1);
        assert_eq!(state_of(&queue, &id), ItemState::Uncertain);
    }

    // Owner without a start time cannot be verified: trusted while the claim
    // is young, abandoned once it is older than the bound.
    let (_dir, mut queue, id) = claimed_queue();
    set_claim(
        &mut queue,
        &format!("{pid}:deadbeef"),
        chrono::Duration::seconds(5),
    );
    assert_eq!(queue.recover_startup(GRACE).unwrap().uncertain, 0);
    set_claim(
        &mut queue,
        &format!("{pid}:deadbeef"),
        chrono::Duration::hours(2),
    );
    assert_eq!(queue.recover_startup(GRACE).unwrap().uncertain, 1);
    assert_eq!(state_of(&queue, &id), ItemState::Uncertain);
}

#[test]
fn a_claim_older_than_the_bound_is_abandoned_even_with_a_matching_owner() {
    let pid = other_live_pid();
    let Some(start) = crate::platform::process_start_time(pid) else {
        return;
    };
    let owner = format!("{pid}:{start}:deadbeef");
    let (_dir, mut queue, id) = claimed_queue();
    set_claim(&mut queue, &owner, chrono::Duration::seconds(5));
    assert_eq!(
        queue.recover_startup(GRACE).unwrap().uncertain,
        0,
        "genuinely alive"
    );
    set_claim(&mut queue, &owner, chrono::Duration::hours(2));
    assert_eq!(queue.recover_startup(GRACE).unwrap().uncertain, 1);
    assert_eq!(state_of(&queue, &id), ItemState::Uncertain);
}

#[test]
fn stuck_dispatched_claim_can_be_retried_or_cancelled_by_the_user() {
    for retry in [true, false] {
        let (_dir, mut queue, id) = claimed_queue();
        set_claim(
            &mut queue,
            &format!("{}:deadbeef", other_live_pid()),
            chrono::Duration::hours(2),
        );
        if retry {
            let retried = queue.retry(&id).unwrap().expect("stuck claim is retryable");
            assert_eq!(retried.delivery.state, ItemState::Pending);
            assert_eq!(retried.delivery.attempts.len(), 1, "history kept");
        } else {
            queue
                .cancel(&id)
                .unwrap()
                .expect("stuck claim is cancellable");
            assert_eq!(state_of(&queue, &id), ItemState::Cancelled);
        }
    }
    // A fresh claim by this process is still being delivered: not cancellable.
    let (_dir, mut queue, id) = claimed_queue();
    assert!(queue.cancel(&id).is_err());
    assert!(queue.retry(&id).unwrap().is_none());
}

#[test]
fn uncertain_and_missed_items_can_be_dismissed_keeping_attempts() {
    let (_dir, mut queue, id) = claimed_queue();
    orphan_claims(&mut queue);
    queue.recover_startup(GRACE).unwrap();
    let cancelled = queue.cancel(&id).unwrap().unwrap();
    assert_eq!(cancelled.delivery.state, ItemState::Cancelled);
    assert_eq!(cancelled.delivery.attempts.len(), 1);

    let dir = tempfile::tempdir().unwrap();
    let mut queue = ScheduledQueue::load(dir.path().join("queue.json"));
    let late = item("late", -90);
    let late_id = late.id.clone();
    queue.try_push(late).unwrap();
    queue.recover_startup(GRACE).unwrap();
    assert!(queue.cancel(&late_id).unwrap().is_some());
}

#[test]
fn legacy_queue_file_without_delivery_fields_loads_pending_and_claims() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("queue.json");
    let legacy = serde_json::json!([{
        "id": "legacy-1",
        "scheduled_for": (Utc::now() - chrono::Duration::seconds(5)).to_rfc3339(),
        "context": "old item",
        "priority": "Normal",
        "target": {"kind": "session", "session_id": "s"},
        "created_by_session": "s",
        "created_at": Utc::now().to_rfc3339(),
    }]);
    std::fs::write(&path, legacy.to_string()).unwrap();
    let mut queue = ScheduledQueue::load(path.clone());
    assert_eq!(queue.len(), 1);
    assert_eq!(queue.items()[0].delivery, DeliveryRecord::default());
    let claimed = queue.take_ready_direct_items();
    assert_eq!(claimed[0].id, "legacy-1");
    assert_eq!(ScheduledQueue::load(path).in_flight().len(), 1);
}

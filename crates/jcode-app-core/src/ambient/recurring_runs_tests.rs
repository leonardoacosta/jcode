//! 3b: run completion, active-run limits, token caps, restart reconciliation.
//! A fake probe stands in for the live turn registry and session ownership.

use super::*;
use crate::ambient::recurring_runs::*;
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};

const T0: &str = "2026-01-01T00:00:00Z";

fn t(text: &str) -> DateTime<Utc> {
    DateTime::parse_from_rfc3339(text)
        .unwrap()
        .with_timezone(&Utc)
}

fn at(minutes: i64) -> DateTime<Utc> {
    t(T0) + chrono::Duration::minutes(minutes)
}

fn template() -> ScheduleRequest {
    ScheduleRequest {
        wake_in_minutes: None,
        wake_at: None,
        context: "job".into(),
        priority: crate::ambient::Priority::Normal,
        target: crate::ambient::ScheduleTarget::Session {
            session_id: "s1".into(),
        },
        created_by_session: "s1".into(),
        working_dir: None,
        task_description: Some("job".into()),
        relevant_files: vec![],
        git_branch: None,
        additional_context: None,
    }
}

fn new_schedule() -> NewSchedule {
    NewSchedule {
        name: None,
        cron: "*/10 * * * *".into(),
        zone: "UTC".into(),
        missed: MissedPolicy::CatchUp { limit: 10 },
        max_active: 1,
        token_cap: None,
        spend_cap_usd: None,
        template: template(),
    }
}

#[derive(Default)]
struct FakeProbe {
    active: RefCell<HashSet<String>>,
    owners: HashMap<String, String>,
    fired: RefCell<Vec<String>>,
    /// Firing a cancel ends the turn, as the real signal does.
    cancel_ends_turn: bool,
}

impl Probe for FakeProbe {
    fn turn_active(&self, session: &str) -> bool {
        self.active.borrow().contains(session)
    }
    fn owned(&self, session: &str) -> Vec<String> {
        self.owners
            .iter()
            .filter(|(_, owner)| owner.as_str() == session)
            .map(|(child, _)| child.clone())
            .collect()
    }
    fn owner_chain(&self, session: &str) -> Vec<String> {
        let mut chain = vec![session.to_string()];
        let mut cur = session.to_string();
        while let Some(owner) = self.owners.get(&cur) {
            chain.push(owner.clone());
            cur = owner.clone();
        }
        chain
    }
    fn fire_cancel(&self, session: &str) {
        self.fired.borrow_mut().push(session.to_string());
        if self.cancel_ends_turn {
            self.active.borrow_mut().remove(session);
        }
    }
}

fn tracked(schedule: &Schedule, key: &str, session: &str, since: DateTime<Utc>) -> Tracked {
    Tracked {
        schedule_id: schedule.id.clone(),
        key: key.into(),
        session_id: session.into(),
        since,
        turn_seen: false,
        ended: false,
        token_cap: schedule.token_cap,
        stopping_since: None,
    }
}

fn push_run(store: &RecurringStore, id: &str, key: &str, session: &str, started: DateTime<Utc>) {
    store
        .transact(|all| {
            all.iter_mut()
                .find(|s| s.id == id)
                .unwrap()
                .runs
                .push(RunRecord {
                    key: key.into(),
                    session_id: session.into(),
                    state: RunState::Running,
                    started_at: started,
                    finished_at: None,
                    note: None,
                });
            Ok(())
        })
        .unwrap();
}

fn state_of(store: &RecurringStore, id: &str, key: &str) -> RunState {
    store
        .get(id)
        .unwrap()
        .unwrap()
        .runs
        .iter()
        .find(|r| r.key == key)
        .unwrap()
        .state
}

fn reconcile_store(
    store: &RecurringStore,
    now: DateTime<Utc>,
    tr: &mut Tracker,
    probe: &FakeProbe,
) {
    store
        .transact(|all| {
            reconcile_with(all, now, tr, probe);
            Ok(())
        })
        .unwrap();
}

struct Fx {
    dir: tempfile::TempDir,
    store: RecurringStore,
    queue: PathBuf,
}

fn fx() -> Fx {
    let dir = tempfile::tempdir().unwrap();
    let store = RecurringStore::new(dir.path().join("schedules.json"));
    let queue = dir.path().join("queue.json");
    Fx { dir, store, queue }
}

#[test]
fn cron_run_completes_only_after_its_own_turn_ends() {
    let f = fx();
    let id = f.store.create(new_schedule(), at(0)).unwrap().id;
    let key = format!("cron:{id}:2026-01-01T00:10:UTC");
    let sched = f.store.get(&id).unwrap().unwrap();
    let mut tr = Tracker::default();
    let probe = FakeProbe::default();

    // Delivered into a live, mid-turn session: the message joins that turn,
    // so that turn ending completes the run.
    tr.turn_started("s1");
    push_run(&f.store, &id, &key, "s1", at(10));
    tr.register(tracked(&sched, &key, "s1", at(10)), true);
    assert!(tr.turn_ended("s1", 0));
    reconcile_store(&f.store, at(11), &mut tr, &probe);
    assert_eq!(state_of(&f.store, &id, &key), RunState::Completed);

    // Idle session: delivery is acknowledged, then the turn starts and ends.
    let key2 = format!("cron:{id}:2026-01-01T00:20:UTC");
    push_run(&f.store, &id, &key2, "s1", at(20));
    tr.register(tracked(&sched, &key2, "s1", at(20)), false);
    reconcile_store(&f.store, at(21), &mut tr, &probe);
    assert_eq!(
        state_of(&f.store, &id, &key2),
        RunState::Running,
        "no turn yet"
    );
    tr.turn_started("s1");
    assert!(!tr.turn_ended("s1", 1), "another turn still active");
    reconcile_store(&f.store, at(23), &mut tr, &probe);
    assert_eq!(state_of(&f.store, &id, &key2), RunState::Running);
    assert!(tr.turn_ended("s1", 0));
    reconcile_store(&f.store, at(24), &mut tr, &probe);
    assert_eq!(state_of(&f.store, &id, &key2), RunState::Completed);
}

#[test]
fn cron_active_limit_holds_until_the_run_completes() {
    let f = fx();
    let id = f.store.create(new_schedule(), at(0)).unwrap().id; // max_active 1
    let tick_at = |m| tick(&f.store, f.queue.clone(), at(m), 100, &ProviderUsageBudget).unwrap();
    assert_eq!(tick_at(12).enqueued, 1);
    // Deliver and acknowledge: the queue slot frees, the run holds it.
    let mut q = ScheduledQueue::load(f.queue.clone());
    let item = q.take_ready_direct_items().remove(0);
    q.settle(
        &item.id,
        crate::ambient::Settlement::Acked {
            target: "s1".into(),
            generation: None,
        },
    )
    .unwrap();
    push_run(&f.store, &id, &item.id, "s1", at(12));
    let sched = f.store.get(&id).unwrap().unwrap();
    // tick() reconciles against the process-wide tracker, so drive that one.
    tracker().register(tracked(&sched, &item.id, "s1", at(12)), true);

    let report = tick_at(22);
    assert_eq!(
        report.enqueued, 0,
        "the running occurrence still holds the slot"
    );
    assert!(report.waiting >= 1);
    assert_eq!(state_of(&f.store, &id, &item.id), RunState::Running);

    // The turn ends; the next tick reconciles, frees capacity and admits.
    tracker().turn_ended("s1", 0);
    assert_eq!(tick_at(23).enqueued, 1);
    assert_eq!(state_of(&f.store, &id, &item.id), RunState::Completed);
}

#[test]
fn cron_token_cap_stops_a_run_confirms_it_and_dedupes_usage() {
    let f = fx();
    let mut new = new_schedule();
    new.token_cap = Some(100);
    let id = f.store.create(new, at(0)).unwrap().id;
    let key = format!("cron:{id}:2026-01-01T00:10:UTC");
    push_run(&f.store, &id, &key, "coord", at(10));
    let sched = f.store.get(&id).unwrap().unwrap();
    let mut tr = Tracker::default();
    tr.register(tracked(&sched, &key, "coord", at(10)), true);
    let mut probe = FakeProbe::default();
    probe.owners.insert("worker".into(), "coord".into());
    probe
        .active
        .borrow_mut()
        .extend(["coord".to_string(), "worker".to_string()]);

    // Usage from the owned worker counts toward the coordinator's run.
    apply_usage(&f.store, &mut tr, &probe, "evt-1", "worker", 60, at(11)).unwrap();
    apply_usage(&f.store, &mut tr, &probe, "evt-1", "worker", 60, at(11)).unwrap(); // replay
    assert_eq!(
        f.store.get(&id).unwrap().unwrap().tokens_used,
        60,
        "replay counted once"
    );
    assert!(probe.fired.borrow().is_empty());

    apply_usage(&f.store, &mut tr, &probe, "evt-2", "coord", 50, at(12)).unwrap();
    assert_eq!(
        f.store.get(&id).unwrap().unwrap().tokens_used,
        110,
        "one response may overshoot"
    );
    let mut fired = probe.fired.borrow().clone();
    fired.sort();
    assert_eq!(
        fired,
        vec!["coord", "worker"],
        "target and owned sessions are cancelled"
    );
    assert_eq!(state_of(&f.store, &id, &key), RunState::Stopping);

    // Not settled while a child turn is unsettled.
    reconcile_store(&f.store, at(13), &mut tr, &probe);
    assert_eq!(
        state_of(&f.store, &id, &key),
        RunState::Stopping,
        "unsettled child blocks settlement"
    );

    // A further event after the stop does not cancel twice.
    apply_usage(&f.store, &mut tr, &probe, "evt-3", "coord", 10, at(13)).unwrap();
    assert_eq!(probe.fired.borrow().len(), 2);

    // Both turns end: confirmed stopped.
    probe.active.borrow_mut().clear();
    reconcile_store(&f.store, at(14), &mut tr, &probe);
    assert_eq!(state_of(&f.store, &id, &key), RunState::LimitStopped);

    // An unconfirmed stop times out as unknown, never as stopped.
    let g = fx();
    let mut new = new_schedule();
    new.token_cap = Some(10);
    let gid = g.store.create(new, at(0)).unwrap().id;
    let gkey = format!("cron:{gid}:2026-01-01T00:10:UTC");
    push_run(&g.store, &gid, &gkey, "coord", at(10));
    let gs = g.store.get(&gid).unwrap().unwrap();
    let mut tr2 = Tracker::default();
    tr2.register(tracked(&gs, &gkey, "coord", at(10)), true);
    let stuck = FakeProbe::default();
    stuck.active.borrow_mut().insert("coord".into());
    apply_usage(&g.store, &mut tr2, &stuck, "e", "coord", 20, at(11)).unwrap();
    reconcile_store(
        &g.store,
        at(11) + chrono::Duration::minutes(10),
        &mut tr2,
        &stuck,
    );
    assert_eq!(state_of(&g.store, &gid, &gkey), RunState::Unknown);
}

#[test]
fn cron_usage_and_caps_are_ignored_without_a_tracked_capped_run() {
    let f = fx();
    let id = f.store.create(new_schedule(), at(0)).unwrap().id; // no cap
    let key = format!("cron:{id}:2026-01-01T00:10:UTC");
    let sched = f.store.get(&id).unwrap().unwrap();
    let mut tr = Tracker::default();
    tr.register(tracked(&sched, &key, "s1", at(10)), true);
    apply_usage(
        &f.store,
        &mut tr,
        &FakeProbe::default(),
        "e",
        "s1",
        1_000_000,
        at(11),
    )
    .unwrap();
    assert_eq!(f.store.get(&id).unwrap().unwrap().tokens_used, 0);
    apply_usage(
        &f.store,
        &mut tr,
        &FakeProbe::default(),
        "e",
        "unrelated",
        5,
        at(11),
    )
    .unwrap();
    assert_eq!(f.store.get(&id).unwrap().unwrap().tokens_used, 0);
}

#[test]
fn cron_restart_marks_unfinished_runs_unknown_and_never_redelivers() {
    let f = fx();
    let id = f.store.create(new_schedule(), at(0)).unwrap().id;
    let key = format!("cron:{id}:2026-01-01T00:10:UTC");
    f.store
        .transact(|all| {
            all[0].occurrences.push(OccurrenceRecord {
                key: key.clone(),
                local: at(10).naive_utc(),
                at: at(10),
                state: OccurrenceState::Enqueued,
            });
            Ok(())
        })
        .unwrap();
    push_run(&f.store, &id, &key, "s1", at(10));

    // A fresh process has no tracker: the persisted run is from a dead one.
    let mut tr = Tracker::default();
    reconcile_store(
        &f.store,
        at(10) + chrono::Duration::minutes(1),
        &mut tr,
        &FakeProbe::default(),
    );
    assert_eq!(
        state_of(&f.store, &id, &key),
        RunState::Running,
        "within the grace window"
    );
    reconcile_store(
        &f.store,
        at(10) + chrono::Duration::minutes(5),
        &mut tr,
        &FakeProbe::default(),
    );
    assert_eq!(state_of(&f.store, &id, &key), RunState::Unknown);

    // The occurrence is never re-created or redelivered.
    let before = f.store.get(&id).unwrap().unwrap().occurrences.len();
    tick(&f.store, f.queue.clone(), at(12), 100, &ProviderUsageBudget).unwrap();
    let after = f.store.get(&id).unwrap().unwrap();
    assert_eq!(after.occurrences.iter().filter(|o| o.key == key).count(), 1);
    assert!(after.occurrences.len() >= before);
    // An unknown run no longer holds capacity.
    assert!(after.runs.iter().all(|r| !r.state.active()));
}

#[test]
fn cron_a_run_without_a_completion_signal_times_out_as_unknown() {
    let f = fx();
    let id = f.store.create(new_schedule(), at(0)).unwrap().id;
    let key = format!("cron:{id}:2026-01-01T00:10:UTC");
    push_run(&f.store, &id, &key, "s1", at(10));
    let sched = f.store.get(&id).unwrap().unwrap();
    let mut tr = Tracker::default();
    tr.register(tracked(&sched, &key, "s1", at(10)), true);
    reconcile_store(
        &f.store,
        at(10) + RUN_TIMEOUT - chrono::Duration::minutes(1),
        &mut tr,
        &FakeProbe::default(),
    );
    assert_eq!(state_of(&f.store, &id, &key), RunState::Running);
    reconcile_store(
        &f.store,
        at(10) + RUN_TIMEOUT + chrono::Duration::minutes(1),
        &mut tr,
        &FakeProbe::default(),
    );
    assert_eq!(state_of(&f.store, &id, &key), RunState::Unknown);
}

#[test]
fn cron_spend_caps_stay_unavailable_while_token_caps_start() {
    let f = fx();
    let mut spend = new_schedule();
    spend.spend_cap_usd = Some(5.0);
    let sid = f.store.create(spend, at(0)).unwrap().id;
    let mut tokens = new_schedule();
    tokens.token_cap = Some(1_000);
    let tid = f.store.create(tokens, at(0)).unwrap().id;
    let report = tick(&f.store, f.queue.clone(), at(12), 100, &ProviderUsageBudget).unwrap();
    assert_eq!(report.enqueued, 1, "only the token-capped schedule starts");
    assert_eq!(
        f.store.get(&sid).unwrap().unwrap().occurrences[0].state,
        OccurrenceState::BudgetUnavailable
    );
    assert_eq!(
        f.store.get(&tid).unwrap().unwrap().occurrences[0].state,
        OccurrenceState::Enqueued
    );
    let _ = &f.dir;
}

#[test]
fn cron_token_caps_fail_closed_for_targets_whose_turns_cannot_be_observed() {
    for target in [
        crate::ambient::ScheduleTarget::Ambient,
        crate::ambient::ScheduleTarget::Spawn {
            parent_session_id: "p".into(),
        },
    ] {
        let f = fx();
        let mut new = new_schedule();
        new.token_cap = Some(1_000);
        new.template.target = target;
        let id = f.store.create(new, at(0)).unwrap().id;
        let report = tick(&f.store, f.queue.clone(), at(12), 100, &ProviderUsageBudget).unwrap();
        assert_eq!(
            report.enqueued, 0,
            "a cap that can never be counted must not start"
        );
        assert_eq!(
            f.store.get(&id).unwrap().unwrap().occurrences[0].state,
            OccurrenceState::BudgetUnavailable
        );
    }
    // Uncapped spawn and ambient schedules are unaffected.
    let f = fx();
    let mut new = new_schedule();
    new.template.target = crate::ambient::ScheduleTarget::Ambient;
    f.store.create(new, at(0)).unwrap();
    assert_eq!(
        tick(&f.store, f.queue.clone(), at(12), 100, &ProviderUsageBudget)
            .unwrap()
            .enqueued,
        1
    );
}

#[test]
fn cron_replayed_usage_never_cancels_twice_or_counts_twice() {
    let f = fx();
    let mut new = new_schedule();
    new.token_cap = Some(50);
    let id = f.store.create(new, at(0)).unwrap().id;
    let key = format!("cron:{id}:2026-01-01T00:10:UTC");
    push_run(&f.store, &id, &key, "coord", at(10));
    let sched = f.store.get(&id).unwrap().unwrap();
    let mut tr = Tracker::default();
    tr.register(tracked(&sched, &key, "coord", at(10)), true);
    let probe = FakeProbe::default();
    probe.active.borrow_mut().insert("coord".into());
    apply_usage(&f.store, &mut tr, &probe, "evt", "coord", 80, at(11)).unwrap();
    assert_eq!(probe.fired.borrow().len(), 1);
    // The same event replayed, even with the in-memory stop lost.
    let mut fresh = Tracker::default();
    fresh.register(tracked(&sched, &key, "coord", at(10)), true);
    apply_usage(&f.store, &mut fresh, &probe, "evt", "coord", 80, at(12)).unwrap();
    assert_eq!(
        probe.fired.borrow().len(),
        1,
        "a replay does not re-fire the cancel"
    );
    assert_eq!(f.store.get(&id).unwrap().unwrap().tokens_used, 80);
}

#[test]
fn cron_a_future_dated_run_does_not_hold_its_slot_forever() {
    let f = fx();
    let id = f.store.create(new_schedule(), at(0)).unwrap().id;
    let key = format!("cron:{id}:2026-01-01T00:10:UTC");
    // started_at is a day ahead of now (clock stepped back).
    push_run(
        &f.store,
        &id,
        &key,
        "s1",
        at(10) + chrono::Duration::days(1),
    );
    let mut tr = Tracker::default();
    reconcile_store(
        &f.store,
        at(10) + chrono::Duration::minutes(3),
        &mut tr,
        &FakeProbe::default(),
    );
    // Age is clamped to zero: still inside the grace window, then expires with
    // the clock instead of staying active for a day.
    assert_eq!(state_of(&f.store, &id, &key), RunState::Running);
    reconcile_store(
        &f.store,
        at(10) + chrono::Duration::days(1) + chrono::Duration::minutes(3),
        &mut tr,
        &FakeProbe::default(),
    );
    assert_eq!(state_of(&f.store, &id, &key), RunState::Unknown);
}

#[test]
fn cron_unrecognised_run_state_from_a_newer_build_still_loads() {
    let f = fx();
    let id = f.store.create(new_schedule(), at(0)).unwrap().id;
    let path = f.dir.path().join("schedules.json");
    let mut doc: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
    doc["schedules"][0]["runs"] = serde_json::json!([{
        "key": "k", "session_id": "s", "state": "quantum", "started_at": "2026-01-01T00:00:00Z"
    }]);
    std::fs::write(&path, serde_json::to_vec(&doc).unwrap()).unwrap();
    let s = f.store.get(&id).unwrap().unwrap();
    assert_eq!(s.runs[0].state, RunState::Unknown);
}

struct SyncProbe;
impl Probe for SyncProbe {
    fn turn_active(&self, _: &str) -> bool {
        true
    }
    fn owned(&self, _: &str) -> Vec<String> {
        Vec::new()
    }
    fn owner_chain(&self, s: &str) -> Vec<String> {
        vec![s.to_string()]
    }
    fn fire_cancel(&self, _: &str) {}
}

#[test]
fn cron_tick_and_usage_do_not_deadlock_on_the_tracker_and_store_locks() {
    use std::sync::{Arc, mpsc};
    let f = fx();
    let mut new = new_schedule();
    new.token_cap = Some(u64::MAX);
    new.max_active = 50;
    let id = f.store.create(new, at(0)).unwrap().id;
    let key = format!("cron:{id}:2026-01-01T00:10:UTC");
    push_run(&f.store, &id, &key, "coord", at(10));
    let sched = f.store.get(&id).unwrap().unwrap();
    tracker().register(tracked(&sched, &key, "coord", at(10)), true);

    let path = f.dir.path().join("schedules.json");
    let queue = f.queue.clone();
    let (tx, rx) = mpsc::channel();
    let ticker = {
        let (path, queue, tx) = (path.clone(), queue.clone(), tx.clone());
        std::thread::spawn(move || {
            let store = RecurringStore::new(path);
            for i in 0..40 {
                let _ = tick(&store, queue.clone(), at(12 + i), 100, &ProviderUsageBudget);
            }
            tx.send("tick").unwrap();
        })
    };
    let user = {
        let (path, tx) = (path.clone(), tx.clone());
        let probe = Arc::new(SyncProbe);
        std::thread::spawn(move || {
            let store = RecurringStore::new(path);
            for i in 0..40 {
                let chain = vec!["coord".to_string()];
                let snapshot = tracker().clone();
                let _ = usage_decision(
                    &store,
                    &snapshot,
                    probe.as_ref(),
                    &chain,
                    &format!("e{i}"),
                    1,
                );
            }
            tx.send("usage").unwrap();
        })
    };
    for _ in 0..2 {
        rx.recv_timeout(std::time::Duration::from_secs(60))
            .expect("tick and usage must both finish: no lock-order deadlock");
    }
    ticker.join().unwrap();
    user.join().unwrap();
    tracker().forget(&key);
}

#[test]
fn cron_manual_retry_of_a_finished_occurrence_starts_a_new_tracked_run() {
    let _guard = crate::storage::lock_test_env();
    let temp = tempfile::tempdir().unwrap();
    let prev = std::env::var_os("JCODE_HOME");
    crate::env::set_var("JCODE_HOME", temp.path());
    let store = RecurringStore::open_default().unwrap();
    let mut new = new_schedule();
    new.token_cap = Some(100);
    let id = store.create(new, at(0)).unwrap().id;
    let key = format!("cron:{id}:2026-01-01T00:10:UTC");
    // A first attempt that ended unknown.
    push_run(&store, &id, &key, "s1", at(10));
    store
        .transact(|all| {
            all[0].runs[0].state = RunState::Unknown;
            Ok(())
        })
        .unwrap();

    note_delivered(&key, "s1", at(40)).unwrap();
    let run = store
        .get(&id)
        .unwrap()
        .unwrap()
        .runs
        .into_iter()
        .find(|r| r.key == key)
        .unwrap();
    assert_eq!(run.state, RunState::Running, "a retry is a new active run");
    assert!(
        tracker().has_capped_runs(),
        "and it is tracked with its cap"
    );
    tracker().forget(&key);
    match prev {
        Some(v) => crate::env::set_var("JCODE_HOME", v),
        None => crate::env::remove_var("JCODE_HOME"),
    }
}

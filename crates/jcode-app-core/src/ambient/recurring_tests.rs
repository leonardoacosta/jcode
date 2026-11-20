//! Recurring-schedule behavior over real stores in a temp dir. Time is always
//! injected, so no test reads the wall clock for a decision.

use super::*;
use crate::ambient::{Priority, ScheduleTarget};

const T0: &str = "2026-01-01T00:00:00Z";

fn t(text: &str) -> DateTime<Utc> {
    DateTime::parse_from_rfc3339(text)
        .unwrap()
        .with_timezone(&Utc)
}

fn mins(base: &str, minutes: i64) -> DateTime<Utc> {
    t(base) + chrono::Duration::minutes(minutes)
}

fn template() -> ScheduleRequest {
    ScheduleRequest {
        wake_in_minutes: None,
        wake_at: None,
        context: "nightly".into(),
        priority: Priority::Normal,
        target: ScheduleTarget::Session {
            session_id: "s1".into(),
        },
        created_by_session: "s1".into(),
        working_dir: None,
        task_description: Some("nightly".into()),
        relevant_files: vec![],
        git_branch: None,
        additional_context: None,
    }
}

fn new_schedule(missed: MissedPolicy) -> NewSchedule {
    NewSchedule {
        name: None,
        cron: "*/10 * * * *".into(),
        zone: "UTC".into(),
        missed,
        max_active: 100,
        token_cap: None,
        spend_cap_usd: None,
        template: template(),
    }
}

struct Fixture {
    _dir: tempfile::TempDir,
    store: RecurringStore,
    queue_path: PathBuf,
}

impl Fixture {
    fn new() -> Self {
        let dir = tempfile::tempdir().unwrap();
        let store = RecurringStore::new(dir.path().join("schedules.json"));
        let queue_path = dir.path().join("queue.json");
        Self {
            _dir: dir,
            store,
            queue_path,
        }
    }

    fn tick(&self, now: DateTime<Utc>, global: u32) -> TickReport {
        tick(
            &self.store,
            self.queue_path.clone(),
            now,
            global,
            &NoHardEnforcement,
        )
        .unwrap()
    }

    fn queue(&self) -> ScheduledQueue {
        ScheduledQueue::load(self.queue_path.clone())
    }

    fn queue_ids(&self) -> Vec<String> {
        let queue = self.queue();
        let mut ids: Vec<String> = queue
            .items()
            .iter()
            .chain(queue.in_flight())
            .chain(queue.recent_history(usize::MAX))
            .map(|item| item.id.clone())
            .collect();
        ids.sort();
        ids
    }

    fn states(&self, id: &str) -> Vec<(String, OccurrenceState)> {
        self.store
            .get(id)
            .unwrap()
            .unwrap()
            .occurrences
            .into_iter()
            .map(|o| (o.local.format("%H:%M").to_string(), o.state))
            .collect()
    }
}

fn count(states: &[(String, OccurrenceState)], wanted: OccurrenceState) -> usize {
    states.iter().filter(|(_, state)| *state == wanted).count()
}

#[test]
fn cron_missed_run_policies_bound_catch_up() {
    // 68 minutes of downtime: occurrences at 00:10 .. 01:00 are all missed.
    let now = mins(T0, 68);

    let skip = Fixture::new();
    let id = skip
        .store
        .create(new_schedule(MissedPolicy::Skip), t(T0))
        .unwrap()
        .id;
    let report = skip.tick(now, 100);
    assert_eq!(report.enqueued, 0);
    assert!(skip.queue_ids().is_empty());
    assert_eq!(count(&skip.states(&id), OccurrenceState::Skipped), 6);

    let once = Fixture::new();
    let id = once
        .store
        .create(new_schedule(MissedPolicy::RunOnce), t(T0))
        .unwrap()
        .id;
    assert_eq!(once.tick(now, 100).enqueued, 1);
    let states = once.states(&id);
    assert_eq!(count(&states, OccurrenceState::Coalesced), 5);
    assert_eq!(
        states.last().unwrap(),
        &("01:00".to_string(), OccurrenceState::Enqueued),
        "only the latest missed occurrence runs"
    );

    let catch = Fixture::new();
    let id = catch
        .store
        .create(new_schedule(MissedPolicy::CatchUp { limit: 2 }), t(T0))
        .unwrap()
        .id;
    assert_eq!(catch.tick(now, 100).enqueued, 2);
    let states = catch.states(&id);
    assert_eq!(count(&states, OccurrenceState::Enqueued), 2);
    assert_eq!(count(&states, OccurrenceState::Skipped), 4);
    assert_eq!(states[0], ("00:10".to_string(), OccurrenceState::Enqueued));
    assert_eq!(states[1], ("00:20".to_string(), OccurrenceState::Enqueued));
}

#[test]
fn cron_decisions_are_persisted_before_anything_is_runnable() {
    let f = Fixture::new();
    let id = f
        .store
        .create(new_schedule(MissedPolicy::RunOnce), t(T0))
        .unwrap()
        .id;
    // The decision pass alone (no admission) already wrote its records.
    f.store
        .transact(|all| {
            all.iter_mut().for_each(|s| decide(s, mins(T0, 68)));
            Ok(())
        })
        .unwrap();
    assert!(f.queue_ids().is_empty(), "nothing runnable yet");
    let states = f.states(&id);
    assert_eq!(count(&states, OccurrenceState::Coalesced), 5);
    assert_eq!(count(&states, OccurrenceState::Due), 1);
}

#[test]
fn cron_clock_jumps_preserve_occurrence_identity() {
    let f = Fixture::new();
    let id = f
        .store
        .create(new_schedule(MissedPolicy::CatchUp { limit: 50 }), t(T0))
        .unwrap()
        .id;
    f.tick(mins(T0, 68), 100);
    let before = f.queue_ids();
    assert_eq!(before.len(), 6);
    let states_before = f.states(&id);

    // Backward jump across recorded occurrences: nothing is re-created.
    assert_eq!(
        f.tick(mins(T0, 30), 100),
        TickReport {
            enqueued: 0,
            waiting: 0,
            next_due: Some(mins(T0, 40)),
        }
    );
    assert_eq!(f.queue_ids(), before);
    // Returning to the later time re-creates nothing either.
    f.tick(mins(T0, 68), 100);
    assert_eq!(f.queue_ids(), before);
    assert_eq!(f.states(&id), states_before);

    // A forward jump continues in cron order with only new keys.
    f.tick(mins(T0, 68 + 30), 100);
    let after = f.queue_ids();
    assert_eq!(after.len(), 9);
    assert!(before.iter().all(|key| after.contains(key)));
    assert_eq!(
        after.iter().collect::<std::collections::HashSet<_>>().len(),
        9
    );
}

#[test]
fn cron_schedule_restart_preserves_state() {
    let f = Fixture::new();
    let mut new = new_schedule(MissedPolicy::RunOnce);
    new.name = Some("nightly".into());
    new.token_cap = Some(500);
    let id = f.store.create(new, t(T0)).unwrap().id;
    f.store.set_paused(&id, true).unwrap();
    f.store.set_paused(&id, false).unwrap();
    f.tick(mins(T0, 68), 100);
    f.store.set_paused(&id, true).unwrap();

    // A new store and queue over the same files (a restarted process).
    let reopened = RecurringStore::new(f._dir.path().join("schedules.json"));
    let s = reopened.get(&id).unwrap().unwrap();
    assert_eq!(s.name.as_deref(), Some("nightly"));
    assert_eq!(s.zone, "UTC");
    assert_eq!(s.cron, "*/10 * * * *");
    assert!(s.paused, "pause survives restart");
    assert_eq!(s.token_cap, Some(500));
    assert_eq!(s.evaluated_through, mins(T0, 68));
    assert_eq!(s.occurrences.len(), 6);
    // A schedule that never ran tick still has its next occurrence computed.
    assert_eq!(s.upcoming(mins(T0, 68), 1)[0].at, mins(T0, 70));
}

#[test]
fn cron_tick_crash_between_enqueue_and_record_does_not_duplicate() {
    let f = Fixture::new();
    let id = f
        .store
        .create(new_schedule(MissedPolicy::Skip), t(T0))
        .unwrap()
        .id;
    f.tick(mins(T0, 12), 100);
    assert_eq!(f.queue_ids().len(), 1);
    // Crash after the queue push but before the store recorded it.
    f.store
        .transact(|all| {
            for o in &mut all[0].occurrences {
                o.state = OccurrenceState::Due;
            }
            Ok(())
        })
        .unwrap();
    f.tick(mins(T0, 12), 100);
    assert_eq!(f.queue_ids().len(), 1, "same key is never pushed twice");
    assert_eq!(f.states(&id)[0].1, OccurrenceState::Enqueued);
}

#[test]
fn cron_concurrency_limits_leave_occurrences_pending() {
    // Per-schedule limit.
    let f = Fixture::new();
    let mut new = new_schedule(MissedPolicy::CatchUp { limit: 10 });
    new.max_active = 1;
    let id = f.store.create(new, t(T0)).unwrap().id;
    let report = f.tick(mins(T0, 32), 100);
    assert_eq!((report.enqueued, report.waiting), (1, 2));
    let states = f.states(&id);
    assert_eq!(
        count(&states, OccurrenceState::Due),
        2,
        "kept, identity intact"
    );
    assert_eq!(f.tick(mins(T0, 32), 100).enqueued, 0, "no duplicate starts");

    // Releasing capacity (claim + ack) lets exactly the next one in.
    let mut queue = f.queue();
    let claimed = queue.take_ready_direct_items();
    assert_eq!(claimed.len(), 1);
    assert_eq!(
        claimed[0].id,
        "cron:".to_string() + &id + ":2026-01-01T00:10:UTC"
    );
    queue
        .settle(
            &claimed[0].id,
            crate::ambient::Settlement::Acked {
                target: "s1".into(),
                generation: None,
            },
        )
        .unwrap();
    assert_eq!(f.tick(mins(T0, 32), 100).enqueued, 1);
    assert_eq!(count(&f.states(&id), OccurrenceState::Due), 1);

    // Global limit across two schedules; oldest due occurrence goes first.
    let g = Fixture::new();
    let a = g
        .store
        .create(new_schedule(MissedPolicy::CatchUp { limit: 10 }), t(T0))
        .unwrap()
        .id;
    let mut later = new_schedule(MissedPolicy::CatchUp { limit: 10 });
    later.cron = "5-35/10 * * * *".into();
    let b = g.store.create(later, t(T0)).unwrap().id;
    let report = g.tick(mins(T0, 32), 2);
    assert_eq!(report.enqueued, 2);
    let ids = g.queue_ids();
    assert!(
        ids.contains(&format!("cron:{a}:2026-01-01T00:05:UTC"))
            || ids.contains(&format!("cron:{b}:2026-01-01T00:05:UTC"))
    );
    let all_dues: usize = [&a, &b]
        .iter()
        .map(|id| count(&g.states(id), OccurrenceState::Due))
        .sum();
    assert_eq!(all_dues, 4, "six due, two admitted, four wait");
}

#[test]
fn cron_pause_blocks_admission_and_resume_applies_policy() {
    let f = Fixture::new();
    let id = f
        .store
        .create(new_schedule(MissedPolicy::RunOnce), t(T0))
        .unwrap()
        .id;
    f.tick(mins(T0, 12), 100);
    let queued = f.queue_ids();
    assert_eq!(
        queued.len(),
        1,
        "an active occurrence is not stopped by pause"
    );
    f.store.set_paused(&id, true).unwrap();
    f.tick(mins(T0, 127), 100);
    assert_eq!(f.queue_ids(), queued, "paused schedule admits nothing");
    assert_eq!(f.queue().in_flight().len() + f.queue().items().len(), 1);

    f.store.set_paused(&id, false).unwrap();
    f.tick(mins(T0, 127), 100);
    let ids = f.queue_ids();
    assert_eq!(ids.len(), 2, "run-once: the latest missed occurrence only");
    assert!(ids.contains(&format!("cron:{id}:2026-01-01T02:00:UTC")));
}

#[test]
fn cron_recovery_marks_unacknowledged_delivery_uncertain() {
    let f = Fixture::new();
    let id = f
        .store
        .create(new_schedule(MissedPolicy::Skip), t(T0))
        .unwrap()
        .id;
    f.tick(mins(T0, 12), 100);
    let key = format!("cron:{id}:2026-01-01T00:10:UTC");

    // Delivered but not acknowledged, then the claiming process dies.
    let mut queue = f.queue();
    assert_eq!(queue.take_ready_direct_items().len(), 1);
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
    let mut restarted = f.queue();
    let report = restarted
        .recover_startup(chrono::Duration::minutes(5))
        .unwrap();
    assert_eq!(report.uncertain, 1);
    let state = |q: &ScheduledQueue| {
        q.recent_history(usize::MAX)
            .into_iter()
            .find(|i| i.id == key)
            .map(|i| i.delivery.state)
    };
    assert_eq!(state(&restarted), Some(ItemState::Uncertain));

    // The scheduler never re-delivers it: same key, no new item, no new claim.
    f.tick(mins(T0, 12), 100);
    assert_eq!(f.queue_ids(), vec![key.clone()]);
    assert!(f.queue().take_ready_direct_items().is_empty());
    assert_eq!(state(&f.queue()), Some(ItemState::Uncertain));

    // Acknowledged-then-crash: the key stays attributed to its delivery.
    let g = Fixture::new();
    let gid = g
        .store
        .create(new_schedule(MissedPolicy::Skip), t(T0))
        .unwrap()
        .id;
    g.tick(mins(T0, 12), 100);
    let gkey = format!("cron:{gid}:2026-01-01T00:10:UTC");
    let mut q = g.queue();
    let item = q.take_ready_direct_items().remove(0);
    q.settle(
        &item.id,
        crate::ambient::Settlement::Acked {
            target: "s1".into(),
            generation: None,
        },
    )
    .unwrap();
    g.tick(mins(T0, 12), 100);
    assert_eq!(
        g.queue_ids(),
        vec![gkey],
        "delivered occurrence is never re-created"
    );
}

struct Enforced;
impl BudgetSource for Enforced {
    fn enforces(&self, _: &Schedule) -> bool {
        true
    }
}

#[test]
fn cron_budget_enforcement_blocks_or_stops_runs() {
    // No authoritative enforcement: a capped schedule fails closed.
    let f = Fixture::new();
    let mut new = new_schedule(MissedPolicy::Skip);
    new.spend_cap_usd = Some(5.0);
    let id = f.store.create(new, t(T0)).unwrap().id;
    let report = f.tick(mins(T0, 12), 100);
    assert_eq!(report.enqueued, 0);
    assert!(f.queue_ids().is_empty());
    assert_eq!(f.states(&id)[0].1, OccurrenceState::BudgetUnavailable);

    // With an enforcement source: runs until the deduplicated usage hits the cap.
    let g = Fixture::new();
    let mut new = new_schedule(MissedPolicy::Skip);
    new.token_cap = Some(100);
    let id = g.store.create(new, t(T0)).unwrap().id;
    let run = |now| tick(&g.store, g.queue_path.clone(), now, 100, &Enforced).unwrap();
    assert_eq!(run(mins(T0, 12)).enqueued, 1);
    let key = format!("cron:{id}:2026-01-01T00:10:UTC");
    assert!(g.store.record_usage(&id, "evt-1", &key, 60, 0.0).unwrap());
    assert!(
        !g.store.record_usage(&id, "evt-1", &key, 60, 0.0).unwrap(),
        "replay"
    );
    assert_eq!(g.store.get(&id).unwrap().unwrap().tokens_used, 60);
    assert_eq!(
        run(mins(T0, 22)).enqueued,
        1,
        "60 of 100 used: still allowed"
    );
    assert!(g.store.record_usage(&id, "evt-2", &key, 40, 0.0).unwrap());
    assert_eq!(run(mins(T0, 32)).enqueued, 0);
    let states = g.states(&id);
    assert_eq!(states.last().unwrap().1, OccurrenceState::BudgetExhausted);
    // Usage survives restart and a replay after it is still ignored.
    let reopened = RecurringStore::new(g._dir.path().join("schedules.json"));
    assert!(!reopened.record_usage(&id, "evt-2", &key, 40, 0.0).unwrap());
    assert_eq!(reopened.get(&id).unwrap().unwrap().tokens_used, 100);
}

#[test]
fn cron_definition_validation_rejects_without_persisting() {
    let f = Fixture::new();
    let mut cases = Vec::new();
    let mut bad = new_schedule(MissedPolicy::Skip);
    bad.cron = "61 * * * *".into();
    cases.push(bad);
    let mut bad = new_schedule(MissedPolicy::Skip);
    bad.zone = "Nowhere/City".into();
    cases.push(bad);
    let mut bad = new_schedule(MissedPolicy::Skip);
    bad.max_active = 0;
    cases.push(bad);
    cases.push(new_schedule(MissedPolicy::CatchUp { limit: 0 }));
    let mut bad = new_schedule(MissedPolicy::Skip);
    bad.token_cap = Some(0);
    cases.push(bad);
    let mut bad = new_schedule(MissedPolicy::Skip);
    bad.spend_cap_usd = Some(-1.0);
    cases.push(bad);
    let mut bad = new_schedule(MissedPolicy::Skip);
    bad.spend_cap_usd = Some(f64::NAN);
    cases.push(bad);
    for bad in cases {
        assert!(f.store.create(bad, t(T0)).is_err());
    }
    assert!(f.store.list().unwrap().is_empty());
    assert!(!f._dir.path().join("schedules.json").exists());
}

#[test]
fn cron_zone_is_kept_and_dst_applies_at_dispatch() {
    let f = Fixture::new();
    let mut new = new_schedule(MissedPolicy::Skip);
    new.cron = "30 2 * * *".into();
    new.zone = "America/New_York".into();
    let id = f.store.create(new, t("2026-03-06T12:00:00Z")).unwrap().id;
    assert_eq!(f.store.get(&id).unwrap().unwrap().zone, "America/New_York");
    // Walk through the spring-forward weekend, one tick per day.
    for day in 7..=11 {
        f.tick(t(&format!("2026-03-{day:02}T12:00:00Z")), 100);
    }
    // Skip policy, so inspect the persisted decisions. 03-08 02:30 does not exist.
    let locals: Vec<String> = f
        .store
        .get(&id)
        .unwrap()
        .unwrap()
        .occurrences
        .iter()
        .map(|o| o.local.format("%m-%d %H:%M").to_string())
        .collect();
    assert_eq!(
        locals,
        vec!["03-07 02:30", "03-09 02:30", "03-10 02:30", "03-11 02:30"]
    );
    assert!(f.queue_ids().is_empty());
}

#[test]
fn cron_short_outage_cannot_bypass_the_missed_policy() {
    // Every-minute schedule, 4 minutes of downtime: all four are inside the
    // grace window but only the latest may count as on time.
    let mut every = new_schedule(MissedPolicy::Skip);
    every.cron = "* * * * *".into();
    let f = Fixture::new();
    let id = f.store.create(every, t(T0)).unwrap().id;
    assert_eq!(
        f.tick(mins(T0, 4), 100).enqueued,
        1,
        "latest only under skip"
    );
    let states = f.states(&id);
    assert_eq!(count(&states, OccurrenceState::Skipped), 3);

    let mut every = new_schedule(MissedPolicy::RunOnce);
    every.cron = "* * * * *".into();
    let g = Fixture::new();
    let id = g.store.create(every, t(T0)).unwrap().id;
    assert_eq!(g.tick(mins(T0, 4), 100).enqueued, 1);
    assert_eq!(count(&g.states(&id), OccurrenceState::Coalesced), 3);
}

#[test]
fn cron_policy_applies_across_the_whole_span_not_per_chunk() {
    // 30 days of every-minute downtime is 43,200 occurrences: more than one
    // evaluation batch. run-once and catch-up must still be bounded overall.
    let span = 30 * 24 * 60;
    let mut every = new_schedule(MissedPolicy::RunOnce);
    every.cron = "* * * * *".into();
    let f = Fixture::new();
    let id = f.store.create(every, t(T0)).unwrap().id;
    assert_eq!(
        f.tick(mins(T0, span), 100).enqueued,
        1,
        "one run for the whole span"
    );
    let s = f.store.get(&id).unwrap().unwrap();
    assert!(s.occurrences.len() <= 101, "records stay bounded");
    assert_eq!(s.evaluated_through, mins(T0, span));

    let mut every = new_schedule(MissedPolicy::CatchUp { limit: 3 });
    every.cron = "* * * * *".into();
    let g = Fixture::new();
    g.store.create(every, t(T0)).unwrap();
    assert_eq!(
        g.tick(mins(T0, span), 100).enqueued,
        3,
        "limit holds across chunks"
    );
}

#[test]
fn cron_waiting_occurrences_are_bounded_while_capacity_is_full() {
    let mut every = new_schedule(MissedPolicy::RunOnce);
    every.cron = "* * * * *".into();
    every.max_active = 1;
    let f = Fixture::new();
    let id = f.store.create(every, t(T0)).unwrap().id;
    f.tick(mins(T0, 1), 100); // fills the one slot, never claimed
    for minute in 2..400 {
        f.tick(mins(T0, minute), 100);
    }
    let s = f.store.get(&id).unwrap().unwrap();
    let waiting = count(
        &s.occurrences
            .iter()
            .map(|o| (String::new(), o.state))
            .collect::<Vec<_>>(),
        OccurrenceState::Due,
    );
    assert_eq!(
        waiting, 1,
        "stale waiting work is coalesced, not queued forever"
    );
    assert!(s.occurrences.len() <= 101);
}

#[test]
fn cron_never_matching_expressions_are_rejected() {
    let f = Fixture::new();
    let mut bad = new_schedule(MissedPolicy::Skip);
    bad.cron = "0 0 31 2 *".into();
    assert!(f.store.create(bad, t(T0)).is_err());
    assert!(
        f.store
            .create(new_schedule(MissedPolicy::CatchUp { limit: 101 }), t(T0))
            .is_err(),
        "catch-up limit is bounded"
    );
    assert!(f.store.list().unwrap().is_empty());
}

#[test]
fn cron_corrupt_or_extreme_input_never_panics() {
    assert!(CronExpr::parse("5/4294967295 * * * *").is_ok());
    let expr = CronExpr::parse("* * * * *").unwrap();
    let zone = cron::parse_zone("UTC").unwrap();
    let far = DateTime::<Utc>::MAX_UTC - chrono::Duration::days(1);
    assert!(cron::next_occurrences(&expr, zone, far, Some(DateTime::<Utc>::MAX_UTC), 5).len() <= 5);
    assert!(cron::next_occurrences(&expr, zone, DateTime::<Utc>::MAX_UTC, None, 5).is_empty());
}

#[test]
fn cron_stale_store_handles_never_lose_updates() {
    let f = Fixture::new();
    let a = RecurringStore::new(f._dir.path().join("schedules.json"));
    let b = RecurringStore::new(f._dir.path().join("schedules.json"));
    let id1 = a
        .create(new_schedule(MissedPolicy::Skip), t(T0))
        .unwrap()
        .id;
    let id2 = b
        .create(new_schedule(MissedPolicy::Skip), t(T0))
        .unwrap()
        .id;
    a.set_paused(&id2, true).unwrap();
    b.set_paused(&id1, true).unwrap();
    tick(
        &a,
        f.queue_path.clone(),
        mins(T0, 12),
        100,
        &NoHardEnforcement,
    )
    .unwrap();
    let all = b.list().unwrap();
    assert_eq!(
        all.len(),
        2,
        "neither handle overwrote the other's schedule"
    );
    assert!(all.iter().all(|s| s.paused));
}

#[test]
fn cron_push_is_idempotent_even_after_history_pruning() {
    let f = Fixture::new();
    let id = f
        .store
        .create(new_schedule(MissedPolicy::Skip), t(T0))
        .unwrap()
        .id;
    f.tick(mins(T0, 12), 100);
    let key = format!("cron:{id}:2026-01-01T00:10:UTC");
    let mut q = f.queue();
    let item = q.take_ready_direct_items().remove(0);
    q.settle(
        &item.id,
        crate::ambient::Settlement::Acked {
            target: "s1".into(),
            generation: None,
        },
    )
    .unwrap();
    // Pruned out of history and the store lost its record of the push.
    q.transact(|items| {
        items.clear();
        Ok(())
    })
    .unwrap();
    let again = crate::ambient::AmbientManager::scheduled_item(template());
    let mut dup = again;
    dup.id = key.clone();
    assert!(
        q.try_push_unique(dup.clone()).unwrap(),
        "absent key is added"
    );
    assert!(
        !q.try_push_unique(dup).unwrap(),
        "present key is not added twice"
    );
}

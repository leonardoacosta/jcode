use super::{schedule::Schedule, store::*};
use chrono::{TimeZone, Utc};
use std::fs;

#[test]
fn interval_rejects_under_minute_and_overflow() {
    assert!(Schedule::Interval { seconds: 59 }.validate().is_err());
    assert!(Schedule::Interval { seconds: 60 }.validate().is_ok());
    assert!(Schedule::Interval { seconds: u64::MAX }.next_after(Utc::now()).is_err());
}

#[test]
fn interval_anchor_coalesces_elapsed_ticks() {
    let now = Utc.with_ymd_and_hms(2026, 1, 1, 0, 0, 0).unwrap();
    let schedule = Schedule::Interval { seconds: 60 };
    assert_eq!(schedule.next_from(now, now + chrono::Duration::seconds(60)).unwrap(), now + chrono::Duration::seconds(120));
    assert_eq!(schedule.next_from(now, now + chrono::Duration::seconds(3600)).unwrap(), now + chrono::Duration::seconds(3660));
}

#[test]
fn calendar_skips_dst_gap_and_uses_earlier_fold() {
    let schedule = Schedule::Calendar { weekdays: vec![6], time: "02:30".into(), timezone: "America/New_York".into() };
    let after = Utc.with_ymd_and_hms(2026, 3, 7, 0, 0, 0).unwrap();
    assert_eq!(schedule.next_after(after).unwrap().to_rfc3339(), "2026-03-15T06:30:00+00:00");
    let fold = Schedule::Calendar { weekdays: vec![6], time: "01:30".into(), timezone: "America/New_York".into() };
    assert_eq!(fold.next_after(Utc.with_ymd_and_hms(2026, 10, 31, 0, 0, 0).unwrap()).unwrap().to_rfc3339(), "2026-11-01T05:30:00+00:00");
}

#[test]
fn calendar_recovery_between_fold_instants_never_replays_fold() {
    let schedule = Schedule::Calendar { weekdays: vec![6], time: "01:30".into(), timezone: "America/New_York".into() };
    let first = Utc.with_ymd_and_hms(2026, 11, 1, 5, 30, 0).unwrap();
    assert_eq!(schedule.next_after(first).unwrap(), Utc.with_ymd_and_hms(2026, 11, 8, 6, 30, 0).unwrap());
}

#[test]
fn preview_returns_consecutive_matching_occurrences() {
    let schedule = Schedule::Calendar { weekdays: vec![0, 1, 2, 3, 4], time: "09:00".into(), timezone: "America/Chicago".into() };
    let now = Utc.with_ymd_and_hms(2026, 9, 25, 20, 0, 0).unwrap();
    let preview = schedule.preview(now, 3).unwrap();
    assert_eq!(preview.len(), 3);
    assert!(preview.windows(2).all(|pair| pair[0] < pair[1]));
}

fn demo_automation(dir: &std::path::Path, now: chrono::DateTime<Utc>) -> (Automation, std::path::PathBuf) {
    let skills = dir.join(".agents/skills");
    fs::create_dir_all(skills.join("demo")).unwrap();
    fs::write(skills.join("demo/SKILL.md"), "---\nname: demo\ndescription: demo\n---\n").unwrap();
    (Automation { id:"a".into(), skill:"demo".into(), arguments:String::new(), working_dir:dir.to_path_buf(), schedule:Schedule::Interval {seconds:60}, enabled:true, created_at:now, next_due:now, provider:None, model:None, error:None }, skills)
}

#[test]
fn claim_finish_persist_lock_and_utf8_truncation() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("state.json");
    let now = Utc.with_ymd_and_hms(2026, 1, 1, 0, 0, 0).unwrap();
    let mut store = Store::open(&path).unwrap();
    let (automation, skills) = demo_automation(dir.path(), now);
    store.create(automation, &skills).unwrap();
    assert!(Store::open(&path).is_err());
    let due = now + chrono::Duration::seconds(60);
    let (_, run) = store.claim_due(due).unwrap().unwrap();
    assert_eq!(store.automations()[0].next_due, due + chrono::Duration::seconds(60));
    store.finish(&run.id, RunStatus::Success, "fast", due + chrono::Duration::seconds(10)).unwrap();
    assert_eq!(store.automations()[0].next_due, due + chrono::Duration::seconds(60));
    let (_, run) = store.claim_due(due + chrono::Duration::seconds(60)).unwrap().unwrap();
    assert!(store.claim_due(due + chrono::Duration::hours(1)).unwrap().is_none());
    let long = format!("{}é", "x".repeat(64 * 1024 - 1));
    store.finish(&run.id, RunStatus::Success, &long, due + chrono::Duration::hours(1)).unwrap();
    assert!(store.runs().iter().find(|r| r.id == run.id).unwrap().truncated);
    assert_eq!(store.runs().iter().find(|r| r.id == run.id).unwrap().output.len(), 64 * 1024 - 1);
    assert_eq!(store.automations()[0].next_due, due + chrono::Duration::seconds(3660));
    drop(store);
    let mut store = Store::open(&path).unwrap();
    assert!(store.runs().iter().any(|stored| stored.id == run.id));
    store.recover(now).unwrap();
    assert!(store.runs().iter().any(|r| r.status == RunStatus::Success));
}

#[test]
fn restart_marks_claimed_run_interrupted_without_replay() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("state.json");
    let now = Utc.with_ymd_and_hms(2026, 1, 1, 0, 0, 0).unwrap();
    let mut store = Store::open(&path).unwrap();
    let (automation, skills) = demo_automation(dir.path(), now);
    store.create(automation, &skills).unwrap();
    let due=now+chrono::Duration::seconds(60);
    let (_, run)=store.claim_due(due).unwrap().unwrap();
    drop(store);
    let mut store=Store::open(&path).unwrap();
    store.recover(due+chrono::Duration::seconds(30)).unwrap();
    assert_eq!(store.runs().iter().find(|r|r.id==run.id).unwrap().status,RunStatus::Interrupted);
    assert!(store.claim_due(due+chrono::Duration::seconds(30)).unwrap().is_none());
}

#[test]
fn unknown_version_and_corruption_are_preserved() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("state.json");
    fs::write(&path, r#"{"version":987}"#).unwrap();
    assert!(Store::open(&path).is_err());
    assert_eq!(fs::read_to_string(&path).unwrap(), r#"{"version":987}"#);
}

#[test]
fn failed_persist_poison_blocks_later_claims() {
    let dir = tempfile::tempdir().unwrap();
    let parent = dir.path().join("state");
    fs::create_dir_all(&parent).unwrap();
    let path = parent.join("data.json");
    let mut store = Store::open(&path).unwrap();
    fs::remove_dir_all(&parent).unwrap();
    fs::write(&parent, "not a directory").unwrap();
    let now = Utc.with_ymd_and_hms(2026, 1, 1, 0, 0, 0).unwrap();
    assert!(store.recover(now).is_err());
    assert!(store.claim_due(now).is_err());
}

#[test]
fn terminal_retention_removes_oldest_and_keeps_running_run() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("state.json");
    let now = Utc.with_ymd_and_hms(2026, 1, 1, 0, 0, 0).unwrap();
    let automation = Automation { id:"a".into(), skill:"demo".into(), arguments:String::new(), working_dir:dir.path().into(), schedule:Schedule::Interval {seconds:60}, enabled:false, created_at:now, next_due:now, provider:None, model:None, error:None };
    let mut runs:Vec<_> = (0..1000).map(|i| Run { id:format!("r{i}"), automation_id:"a".into(), session_id:None, started_at:now + chrono::Duration::seconds(i), ended_at:Some(now + chrono::Duration::seconds(i)), status:RunStatus::Success, output:String::new(), truncated:false, skill_source:None }).collect();
    runs.push(Run { id:"active".into(), automation_id:"a".into(), session_id:None, started_at:now + chrono::Duration::days(1), ended_at:None, status:RunStatus::Running, output:String::new(), truncated:false, skill_source:None });
    let state = serde_json::json!({"version":1,"automations":[automation],"runs":runs});
    fs::write(&path, serde_json::to_vec(&state).unwrap()).unwrap();
    let mut store = Store::open(&path).unwrap();
    store.finish("active", RunStatus::Success, "last", now + chrono::Duration::days(2)).unwrap();
    assert_eq!(store.runs().len(), 1000);
    assert!(store.runs().iter().any(|r| r.id == "active"));
    assert!(!store.runs().iter().any(|r| r.id == "r0"));
}

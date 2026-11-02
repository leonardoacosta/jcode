use super::*;
use std::time::SystemTime;

fn policy() -> SessionRetentionConfig {
    SessionRetentionConfig {
        enabled: true,
        ..Default::default()
    }
}

fn meta(saved: bool, debug: bool, msgs: usize, live: bool, age_days: i64) -> SessionMeta {
    SessionMeta {
        parsed: Some(ParsedMeta {
            saved,
            is_debug: debug,
            message_count: msgs,
            live,
        }),
        modified: Utc::now() - Duration::days(age_days),
    }
}

#[test]
fn saved_debug_tiny_old_is_kept() {
    let v = classify(&meta(true, true, 0, false, 365), Utc::now(), &policy());
    assert_eq!(v, Verdict::Keep(Reason::Saved));
}

#[test]
fn old_debug_is_candidate() {
    let v = classify(&meta(false, true, 50, false, 8), Utc::now(), &policy());
    assert_eq!(v, Verdict::Candidate(Reason::DebugExpired));
}

#[test]
fn recent_debug_is_kept() {
    let v = classify(&meta(false, true, 0, false, 2), Utc::now(), &policy());
    assert_eq!(v, Verdict::Keep(Reason::Recent));
}

#[test]
fn old_tiny_is_candidate_but_big_old_kept() {
    let p = policy();
    assert_eq!(
        classify(&meta(false, false, 3, false, 15), Utc::now(), &p),
        Verdict::Candidate(Reason::TinyExpired)
    );
    assert_eq!(
        classify(&meta(false, false, 4, false, 400), Utc::now(), &p),
        Verdict::Keep(Reason::Recent)
    );
}

#[test]
fn unparseable_is_kept() {
    let m = SessionMeta {
        parsed: None,
        modified: Utc::now() - Duration::days(999),
    };
    assert_eq!(
        classify(&m, Utc::now(), &policy()),
        Verdict::Keep(Reason::Unparseable)
    );
}

#[test]
fn live_is_kept() {
    let v = classify(&meta(false, true, 0, true, 100), Utc::now(), &policy());
    assert_eq!(v, Verdict::Keep(Reason::Live));
}

fn temp_dir() -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "jcode-retention-test-{}-{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0)
    ));
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn write_session(dir: &Path, id: &str, saved: bool, debug: bool) {
    let mut s = Session::create_with_id(id.to_string(), None, None);
    s.saved = saved;
    s.is_debug = debug;
    s.status = SessionStatus::Closed;
    std::fs::write(
        dir.join(format!("{id}.json")),
        serde_json::to_vec(&s).unwrap(),
    )
    .unwrap();
}

#[test]
fn gc_dry_run_changes_nothing_and_apply_removes_only_candidates() {
    let dir = temp_dir();
    write_session(&dir, "debug_old", false, true);
    write_session(&dir, "saved_old", true, true);
    write_session(&dir, "plain_tiny", false, false);
    std::fs::write(dir.join("broken.json"), b"{not json").unwrap();
    // All files were written "now"; evaluate GC 30 days in the future.
    let later = Utc::now() + Duration::days(30);

    let disabled = SessionRetentionConfig::default();
    assert!(run_gc_in(&dir, later, &disabled, true).is_err());
    assert!(dir.join("debug_old.json").exists());

    let dry = run_gc_in(&dir, later, &policy(), false).unwrap();
    assert_eq!(dry.scanned, 4);
    assert_eq!(dry.candidates, 2);
    assert_eq!(dry.removed, 0);
    assert!(dir.join("debug_old.json").exists());
    assert!(dir.join("plain_tiny.json").exists());
    assert!(!dir.join(TRASH_DIR).exists());

    let applied = run_gc_in(&dir, later, &policy(), true).unwrap();
    assert_eq!(applied.removed, 2);
    assert!(!dir.join("debug_old.json").exists());
    assert!(!dir.join("plain_tiny.json").exists());
    assert!(dir.join("saved_old.json").exists());
    assert!(dir.join("broken.json").exists());
    let _ = std::fs::remove_dir_all(&dir);
}

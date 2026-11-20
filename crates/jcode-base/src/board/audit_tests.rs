use super::*;

pub(crate) fn intent(op: &str) -> AuditEvent {
    AuditEvent::Intent(IntentRecord {
        op_id: op.into(),
        ts: "2026-11-20T00:00:00Z".into(),
        world: "personal".into(),
        backend: "linear".into(),
        team: "team-1".into(),
        project: Some("proj-1".into()),
        issue_id: "i-1".into(),
        expected_prior_state: "s-todo".into(),
        requested_state: "s-done".into(),
        confirmation: ConfirmationRecord {
            confirmed_at: "2026-11-20T00:00:00Z".into(),
        },
    })
}

pub(crate) fn outcome(op: &str, result: AuditResult) -> AuditEvent {
    AuditEvent::Outcome(OutcomeRecord {
        op_id: op.into(),
        ts: "2026-11-20T00:00:01Z".into(),
        result,
    })
}

#[test]
fn appends_one_json_line_per_event_and_preserves_earlier_bytes() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("nested/board-audit.jsonl");
    let log = AuditLog::open(&path).unwrap();
    log.append(&intent("op1")).unwrap();
    let first = std::fs::read(&path).unwrap();
    assert!(first.ends_with(b"\n"));

    log.append(&outcome("op1", AuditResult::Applied)).unwrap();
    // A second handle (a later session) appends, never truncates.
    let again = AuditLog::open(&path).unwrap();
    again
        .append(&outcome("op1", AuditResult::Reconciled { applied: true }))
        .unwrap();
    let all = std::fs::read(&path).unwrap();
    assert!(all.starts_with(&first), "earlier bytes changed");
    let lines: Vec<AuditEvent> = String::from_utf8(all)
        .unwrap()
        .lines()
        .map(|l| serde_json::from_str(l).unwrap())
        .collect();
    assert_eq!(lines.len(), 3);
    assert_eq!(lines[0], intent("op1"));
    assert_eq!(
        lines[2],
        outcome("op1", AuditResult::Reconciled { applied: true })
    );
}

#[cfg(unix)]
#[test]
fn file_is_owner_only() {
    use std::os::unix::fs::PermissionsExt;
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("a.jsonl");
    let log = AuditLog::open(&path).unwrap();
    log.append(&intent("op")).unwrap();
    let mode = std::fs::metadata(&path).unwrap().permissions().mode() & 0o777;
    assert_eq!(mode, 0o600);
}

#[test]
fn unwritable_location_is_an_error() {
    let dir = tempfile::tempdir().unwrap();
    // The parent is a regular file, so the log cannot be created.
    let blocker = dir.path().join("blocker");
    std::fs::write(&blocker, "x").unwrap();
    assert!(AuditLog::open(&blocker.join("audit.jsonl")).is_err());
    // A directory in place of the file is not appendable either.
    let as_dir = dir.path().join("isdir");
    std::fs::create_dir(&as_dir).unwrap();
    assert!(AuditLog::open(&as_dir).is_err());
}

#[cfg(target_os = "linux")]
#[test]
fn append_fails_when_the_device_write_fails() {
    // /dev/full accepts open() and fails every write with ENOSPC.
    let log = AuditLog::open(Path::new("/dev/full")).expect("/dev/full opens on Linux");
    assert!(log.append(&intent("op")).is_err());
}

#[test]
fn records_hold_only_ids_states_and_times() {
    let json = serde_json::to_value(intent("op1")).unwrap();
    let mut keys: Vec<_> = json.as_object().unwrap().keys().cloned().collect();
    keys.sort();
    assert_eq!(
        keys,
        [
            "backend",
            "confirmation",
            "event",
            "expected_prior_state",
            "issue_id",
            "op_id",
            "project",
            "requested_state",
            "team",
            "ts",
            "world"
        ]
    );
    let json = serde_json::to_value(outcome("op1", AuditResult::Uncertain)).unwrap();
    let mut keys: Vec<_> = json.as_object().unwrap().keys().cloned().collect();
    keys.sort();
    assert_eq!(keys, ["event", "op_id", "result", "ts"]);
    for forbidden in [
        "title",
        "description",
        "token",
        "credential",
        "password",
        "comment",
        "label",
    ] {
        assert!(
            !serde_json::to_string(&intent("op1"))
                .unwrap()
                .contains(forbidden)
        );
    }
}

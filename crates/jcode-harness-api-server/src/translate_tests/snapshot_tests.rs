//! Whole-frame golden snapshots. Regenerate with `UPDATE_SNAPSHOTS=1`.
use super::*;
use std::path::PathBuf;

fn snapshot_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src/translate_tests/snapshots")
}

fn fixture(name: &str) -> Vec<Value> {
    let text = std::fs::read_to_string(snapshot_dir().join(name)).expect("fixture");
    serde_json::from_str(&text).expect("fixture json")
}

/// Compare `actual` to the golden file, or rewrite it when `UPDATE_SNAPSHOTS=1`.
pub(super) fn assert_snapshot(name: &str, actual: &Value) {
    let path = snapshot_dir().join(name);
    let actual = format!("{}\n", serde_json::to_string_pretty(actual).unwrap());
    if std::env::var_os("UPDATE_SNAPSHOTS").is_some_and(|v| v == "1") {
        std::fs::write(&path, &actual).unwrap();
        return;
    }
    let expected = std::fs::read_to_string(&path).unwrap_or_else(|_| {
        panic!(
            "missing snapshot {}; run with UPDATE_SNAPSHOTS=1",
            path.display()
        )
    });
    if expected != actual {
        let diff: String = expected
            .lines()
            .zip(actual.lines().chain(std::iter::repeat("")))
            .enumerate()
            .filter(|(_, (e, a))| e != a)
            .map(|(i, (e, a))| format!("line {}:\n- {e}\n+ {a}\n", i + 1))
            .collect();
        panic!(
            "snapshot {name} drifted (set UPDATE_SNAPSHOTS=1 to accept):\n{diff}\n--- full actual ---\n{actual}"
        );
    }
}

/// Redact process-global counters (legacy ids, text message ids) so
/// snapshots are stable regardless of test order.
fn redact(value: &mut Value) {
    match value {
        Value::Object(map) => {
            for (key, v) in map.iter_mut() {
                match (key.as_str(), &*v) {
                    ("message_id", Value::String(s)) if s.starts_with("text-") => {
                        *v = json!("text-<n>")
                    }
                    _ => redact(v),
                }
            }
        }
        Value::Array(items) => items.iter_mut().for_each(redact),
        _ => {}
    }
}

fn attached_state() -> BridgeState {
    BridgeState {
        session_id: Some("s1".into()),
        runtime_commands_supported: true,
        ..Default::default()
    }
}

#[test]
fn translate_request_frames_match_snapshot() {
    let mut state = attached_state();
    let frames: Vec<Value> = fixture("requests.input.json")
        .iter()
        .map(|request| {
            let out: Vec<Value> = state
                .api_request_to_legacy(request)
                .into_iter()
                .map(|o| match o {
                    Outbound::Legacy(mut v) => {
                        if v.get("id").is_some_and(Value::is_u64) {
                            v["id"] = json!("<legacy-id>");
                        }
                        json!({"legacy": v})
                    }
                    Outbound::Reply(f) => json!({"reply": serde_json::to_value(f).unwrap()}),
                })
                .collect();
            json!({"request": request, "outbound": out})
        })
        .collect();
    assert_snapshot("requests.snap.json", &Value::Array(frames));
}

#[test]
fn translate_event_frames_match_snapshot() {
    let mut state = attached_state();
    let frames: Vec<Value> = fixture("events.input.json")
        .iter()
        .map(|event| {
            let mut out = serde_json::to_value(state.legacy_event_to_api(event)).unwrap();
            redact(&mut out);
            json!({"event": event, "frames": out})
        })
        .collect();
    assert_snapshot("events.snap.json", &Value::Array(frames));
}

#[test]
#[should_panic(expected = "drifted")]
fn translate_snapshot_drift_fails_with_diff() {
    if std::env::var_os("UPDATE_SNAPSHOTS").is_some() {
        panic!("drifted (skipped while updating)");
    }
    assert_snapshot("events.snap.json", &json!(["deliberate drift"]));
}

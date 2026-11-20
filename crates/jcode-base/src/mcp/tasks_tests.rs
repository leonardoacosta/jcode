use super::*;
use serde_json::json;

fn task(status: &str) -> Value {
    json!({"resultType":"task","taskId":"786512e2","status":status,
           "createdAt":"2025-11-25T10:30:00Z","lastUpdatedAt":"2025-11-25T10:40:00Z",
           "ttlMs":60000,"pollIntervalMs":5000})
}

#[test]
fn parses_the_spec_create_task_result() {
    let t = parse_task(&task("working")).unwrap();
    assert_eq!(t.task_id, "786512e2");
    assert_eq!(t.status, RemoteStatus::Working);
    assert_eq!(t.ttl_ms, Some(60000));
    assert_eq!(t.poll_interval_ms, Some(5000));
    assert!(t.result.is_none() && t.error.is_none() && t.input_requests.is_none());
}

#[test]
fn null_ttl_means_unlimited_and_missing_poll_interval_is_none() {
    let mut v = task("working");
    v["ttlMs"] = Value::Null;
    v.as_object_mut().unwrap().remove("pollIntervalMs");
    let t = parse_task(&v).unwrap();
    assert_eq!(t.ttl_ms, None);
    assert_eq!(t.poll_interval_ms, None);
}

#[test]
fn inlines_result_error_and_input_requests() {
    let mut v = task("completed");
    v["result"] = json!({"content":[{"type":"text","text":"Hello"}],"isError":false});
    assert_eq!(
        parse_task(&v).unwrap().result.unwrap()["content"][0]["text"],
        "Hello"
    );
    let mut v = task("failed");
    v["error"] = json!({"code":-32603,"message":"rate limited"});
    assert_eq!(parse_task(&v).unwrap().error.unwrap()["code"], -32603);
    let mut v = task("input_required");
    v["inputRequests"] = json!({"name":{"method":"elicitation/create","params":{}}});
    assert!(parse_task(&v).unwrap().input_requests.is_some());
}

#[test]
fn rejects_malformed_handles() {
    for key in ["taskId", "status", "createdAt", "lastUpdatedAt"] {
        let mut v = task("working");
        v.as_object_mut().unwrap().remove(key);
        assert!(parse_task(&v).is_err(), "missing {key}");
    }
    let mut v = task("working");
    v["status"] = json!("paused");
    assert!(parse_task(&v).is_err(), "unknown status");
    let mut v = task("working");
    v["taskId"] = json!("");
    assert!(parse_task(&v).is_err(), "empty id");
    let mut v = task("working");
    v["taskId"] = json!("a\nb");
    assert!(parse_task(&v).is_err(), "control chars in id");
    let mut v = task("working");
    v["ttlMs"] = json!(-5);
    assert!(parse_task(&v).is_err(), "negative ttl");
    let mut v = task("working");
    v["pollIntervalMs"] = json!("fast");
    assert!(parse_task(&v).is_err(), "non numeric interval");
}

#[test]
fn oversized_inlined_payloads_are_replaced_not_persisted() {
    let mut v = task("completed");
    v["result"] = json!({"content":[{"type":"text","text":"x".repeat(MAX_INLINE_BYTES + 10)}]});
    let t = parse_task(&v).unwrap();
    assert_eq!(t.result.unwrap()["truncated"], true);
}

#[test]
fn status_terminality() {
    use RemoteStatus::*;
    for s in [Completed, Failed, Cancelled] {
        assert!(s.is_terminal());
    }
    for s in [Working, InputRequired] {
        assert!(!s.is_terminal());
    }
    assert_eq!(RemoteStatus::parse("input_required"), Some(InputRequired));
    assert_eq!(RemoteStatus::parse("expired"), None);
}

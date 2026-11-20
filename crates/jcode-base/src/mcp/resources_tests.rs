use super::*;
use std::sync::Mutex;

/// Scripted backend: pops one canned reply per call and records the requests.
struct Scripted {
    replies: Mutex<Vec<Result<Value>>>,
    seen: Mutex<Vec<(String, Option<Value>)>>,
}

impl Scripted {
    fn new(replies: Vec<Result<Value>>) -> Self {
        Self {
            replies: Mutex::new(replies.into_iter().rev().collect()),
            seen: Mutex::new(Vec::new()),
        }
    }
    fn seen(&self) -> Vec<(String, Option<Value>)> {
        self.seen.lock().unwrap().clone()
    }
}

#[async_trait]
impl Rpc for Scripted {
    async fn rpc(&self, method: &str, params: Option<Value>) -> Result<Value> {
        self.seen.lock().unwrap().push((method.to_string(), params));
        self.replies
            .lock()
            .unwrap()
            .pop()
            .expect("unexpected extra request")
    }
}

#[tokio::test]
async fn resources_list_follows_cursors_and_ignores_unknown_fields() {
    let rpc = Scripted::new(vec![
        Ok(json!({"resources": [
            {"uri": "file:///a", "name": "a", "mimeType": "text/plain", "size": 3, "future": {"x": 1}},
            {"name": "no uri, skipped"}
        ], "nextCursor": "c1", "extra": true})),
        Ok(json!({"resources": [{"uri": "file:///b"}]})),
    ]);
    let listing = list_paged(&rpc, "resources/list", "resources", parse_resource_info)
        .await
        .unwrap();
    assert_eq!(
        listing
            .items
            .iter()
            .map(|r| r.uri.as_str())
            .collect::<Vec<_>>(),
        ["file:///a", "file:///b"]
    );
    assert_eq!(listing.items[0].size, Some(3));
    assert!(!listing.truncated);
    let seen = rpc.seen();
    assert_eq!(seen[0].1, None);
    assert_eq!(seen[1].1, Some(json!({"cursor": "c1"})));
}

#[tokio::test]
async fn list_detects_cursor_cycle() {
    let rpc = Scripted::new(vec![
        Ok(json!({"prompts": [], "nextCursor": "same"})),
        Ok(json!({"prompts": [], "nextCursor": "same"})),
    ]);
    let err = list_paged(&rpc, "prompts/list", "prompts", parse_prompt_info)
        .await
        .unwrap_err();
    assert!(err.to_string().contains("cursor cycle"), "{err}");
}

#[tokio::test]
async fn list_enforces_page_limit() {
    let replies = (0..MAX_LIST_PAGES)
        .map(|i| Ok(json!({"prompts": [], "nextCursor": format!("c{i}")})))
        .collect();
    let rpc = Scripted::new(replies);
    let err = list_paged(&rpc, "prompts/list", "prompts", parse_prompt_info)
        .await
        .unwrap_err();
    assert!(err.to_string().contains("page limit"), "{err}");
}

#[tokio::test]
async fn list_caps_items_and_flags_truncation() {
    let many: Vec<Value> = (0..MAX_LIST_ITEMS + 5)
        .map(|i| json!({"uri": format!("u{i}")}))
        .collect();
    let rpc = Scripted::new(vec![Ok(json!({"resources": many, "nextCursor": "more"}))]);
    let listing = list_paged(&rpc, "resources/list", "resources", parse_resource_info)
        .await
        .unwrap();
    assert_eq!(listing.items.len(), MAX_LIST_ITEMS);
    assert!(listing.truncated);
    assert_eq!(rpc.seen().len(), 1, "no further pages after the cap");
}

#[tokio::test]
async fn prompts_list_parses_arguments() {
    let rpc = Scripted::new(vec![Ok(json!({"prompts": [
        {"name": "p", "description": "d", "arguments": [
            {"name": "x", "required": true, "description": "dx"}, {"name": "y"}, {"bad": 1}
        ]}
    ]}))]);
    let listing = list_paged(&rpc, "prompts/list", "prompts", parse_prompt_info)
        .await
        .unwrap();
    let p = &listing.items[0];
    assert_eq!(p.arguments.len(), 2);
    assert!(p.arguments[0].required && !p.arguments[1].required);
}

#[tokio::test]
async fn read_truncates_text_on_char_boundary_and_reports_original_size() {
    // 3-byte chars so the byte cap falls mid-character.
    let text = "€".repeat(MAX_TEXT_BYTES);
    let rpc = Scripted::new(vec![Ok(json!({"contents": [
        {"uri": "file:///big", "mimeType": "text/plain", "text": text}
    ]}))]);
    let read = read_resource_via(&rpc, "file:///big").await.unwrap();
    let c = &read.contents[0];
    assert!(c.truncated);
    assert_eq!(c.text_bytes, MAX_TEXT_BYTES * 3);
    let kept = c.text.as_deref().unwrap();
    assert!(kept.len() <= MAX_TEXT_BYTES);
    assert!(kept.chars().all(|ch| ch == '€'));
    assert_eq!(rpc.seen()[0].1, Some(json!({"uri": "file:///big"})));
}

#[tokio::test]
async fn read_never_inlines_blobs() {
    let blob = "QUJD".repeat(1000);
    let rpc = Scripted::new(vec![Ok(json!({"contents": [
        {"uri": "file:///img", "mimeType": "image/png", "blob": blob}
    ]}))]);
    let read = read_resource_via(&rpc, "file:///img").await.unwrap();
    let c = &read.contents[0];
    assert_eq!(c.text, None);
    assert_eq!(c.blob_bytes, Some(4000));
    assert_eq!(c.mime_type.as_deref(), Some("image/png"));
    assert!(!format!("{read:?}").contains("QUJDQUJD"));
}

#[tokio::test]
async fn read_caps_number_of_contents() {
    let entries: Vec<Value> = (0..MAX_PARTS + 3)
        .map(|i| json!({"uri": format!("u{i}"), "text": "t"}))
        .collect();
    let rpc = Scripted::new(vec![Ok(json!({"contents": entries}))]);
    let read = read_resource_via(&rpc, "u").await.unwrap();
    assert_eq!(read.contents.len(), MAX_PARTS);
    assert!(read.contents_truncated);
}

#[tokio::test]
async fn read_error_and_missing_contents_are_errors() {
    let rpc = Scripted::new(vec![Err(anyhow::anyhow!(
        "MCP error -32002: Resource not found"
    ))]);
    let err = read_resource_via(&rpc, "x").await.unwrap_err();
    assert!(err.to_string().contains("-32002"), "{err}");
    let rpc = Scripted::new(vec![Ok(json!({"nope": []}))]);
    assert!(read_resource_via(&rpc, "x").await.is_err());
}

#[tokio::test]
async fn prompt_get_renders_content_kinds_and_bounds_text() {
    let long = "a".repeat(MAX_TEXT_BYTES + 10);
    let rpc = Scripted::new(vec![Ok(json!({
        "description": "demo",
        "messages": [
            {"role": "user", "content": {"type": "text", "text": "hi", "annotations": {}}},
            {"role": "assistant", "content": {"type": "text", "text": long}},
            {"role": "user", "content": {"type": "image", "mimeType": "image/png", "data": "QUJD"}},
            {"role": "user", "content": {"type": "resource", "resource": {"uri": "f:///r", "mimeType": "x/y", "blob": "QUJD"}}},
            {"role": "user", "content": {"type": "resource", "resource": {"uri": "f:///t", "text": "embedded"}}},
            {"role": "user"}
        ]
    }))]);
    let args = BTreeMap::from([("k".to_string(), "v".to_string())]);
    let result = get_prompt_via(&rpc, "p", &args).await.unwrap();
    assert_eq!(result.description.as_deref(), Some("demo"));
    assert_eq!(result.messages[0].text, "hi");
    assert!(result.messages[1].truncated);
    assert_eq!(result.messages[1].text.len(), MAX_TEXT_BYTES);
    assert_eq!(result.messages[2].text, "[image: image/png (4 bytes)]");
    assert_eq!(result.messages[3].text, "[resource f:///r: x/y (4 bytes)]");
    assert_eq!(result.messages[4].text, "embedded");
    assert_eq!(result.messages[5].text, "");
    assert_eq!(
        rpc.seen()[0].1,
        Some(json!({"name": "p", "arguments": {"k": "v"}}))
    );
}

#[tokio::test]
async fn prompt_get_omits_empty_arguments_and_propagates_errors() {
    let rpc = Scripted::new(vec![Err(anyhow::anyhow!(
        "MCP error -32602: Unknown prompt"
    ))]);
    let err = get_prompt_via(&rpc, "nope", &BTreeMap::new())
        .await
        .unwrap_err();
    assert!(err.to_string().contains("Unknown prompt"));
    assert_eq!(rpc.seen()[0].1, Some(json!({"name": "nope"})));
}

#[test]
fn missing_capability_is_a_clear_error_naming_it() {
    let none = ServerCapabilities::default();
    let err = require_capability("srv", &none, "resources", "resources/read")
        .unwrap_err()
        .to_string();
    assert!(
        err.contains("'resources' capability")
            && err.contains("srv")
            && err.contains("resources/read"),
        "{err}"
    );
    let err = require_capability("srv", &none, "prompts", "prompts/list")
        .unwrap_err()
        .to_string();
    assert!(err.contains("'prompts' capability"), "{err}");

    // A server advertising only prompts still cannot serve resources.
    let prompts_only: ServerCapabilities = serde_json::from_value(json!({"prompts": {}})).unwrap();
    assert!(require_capability("srv", &prompts_only, "prompts", "prompts/get").is_ok());
    assert!(require_capability("srv", &prompts_only, "resources", "resources/list").is_err());
    let both: ServerCapabilities =
        serde_json::from_value(json!({"resources": {"subscribe": true}, "prompts": {}})).unwrap();
    assert!(require_capability("srv", &both, "resources", "resources/read").is_ok());
}

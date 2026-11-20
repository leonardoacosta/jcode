use super::*;
use serde_json::json;

#[test]
fn spec_value_encoding_examples() {
    // The five examples from the 2026-07-28 Streamable HTTP page.
    assert_eq!(encode_header_value("us-west1"), "us-west1");
    assert_eq!(
        encode_header_value("Hello, 世界"),
        "=?base64?SGVsbG8sIOS4lueVjA==?="
    );
    assert_eq!(encode_header_value(" padded "), "=?base64?IHBhZGRlZCA=?=");
    assert_eq!(
        encode_header_value("line1\nline2"),
        "=?base64?bGluZTEKbGluZTI=?="
    );
    assert_eq!(
        encode_header_value("=?base64?literal?="),
        "=?base64?PT9iYXNlNjQ/bGl0ZXJhbD89?="
    );
    // Interior spaces are header-safe.
    assert_eq!(encode_header_value("a b"), "a b");
}

#[test]
fn extracts_nested_property_paths() {
    let schema = json!({"type":"object","properties":{
        "region":{"type":"string","x-mcp-header":"Region"},
        "opts":{"type":"object","properties":{
            "tier":{"type":"integer","x-mcp-header":"Tier"}}},
        "plain":{"type":"string"}}});
    let params = extract(&schema).unwrap();
    assert_eq!(params.len(), 2);
    assert!(params.contains(&HeaderParam {
        name: "Region".into(),
        path: vec!["region".into()]
    }));
    assert!(params.contains(&HeaderParam {
        name: "Tier".into(),
        path: vec!["opts".into(), "tier".into()]
    }));
}

#[test]
fn invalid_annotations_are_rejected() {
    let bad = |schema: Value| extract(&schema).unwrap_err();
    // Not a token.
    bad(json!({"properties":{"a":{"type":"string","x-mcp-header":"bad name"}}}));
    bad(json!({"properties":{"a":{"type":"string","x-mcp-header":""}}}));
    bad(json!({"properties":{"a":{"type":"string","x-mcp-header":"x\ny"}}}));
    // Not unique, case-insensitively.
    bad(json!({"properties":{
        "a":{"type":"string","x-mcp-header":"Dup"},
        "b":{"type":"string","x-mcp-header":"dUP"}}}));
    // Wrong types: number is not permitted, nor object/array.
    bad(json!({"properties":{"a":{"type":"number","x-mcp-header":"N"}}}));
    bad(json!({"properties":{"a":{"type":"object","x-mcp-header":"O"}}}));
    // Not reachable through `properties` alone.
    bad(json!({"properties":{"a":{"type":"array","items":{"type":"string","x-mcp-header":"I"}}}}));
    bad(json!({"oneOf":[{"properties":{"a":{"type":"string","x-mcp-header":"A"}}}]}));
    bad(json!({"properties":{"a":{"$ref":"#/x","x-mcp-header":"R"}}}));
    // On the root there is no property to read.
    bad(json!({"type":"string","x-mcp-header":"Root"}));
    // Non-string annotation.
    bad(json!({"properties":{"a":{"type":"string","x-mcp-header":7}}}));
}

#[test]
fn schema_without_annotations_is_fine() {
    assert!(
        extract(&json!({"type":"object","properties":{"a":{"type":"string"}}}))
            .unwrap()
            .is_empty()
    );
    assert!(extract(&json!({})).unwrap().is_empty());
}

#[test]
fn headers_follow_the_spec_rules_for_values() {
    let specs = extract(&json!({"properties":{
        "region":{"type":"string","x-mcp-header":"Region"},
        "n":{"type":"integer","x-mcp-header":"N"},
        "flag":{"type":"boolean","x-mcp-header":"Flag"},
        "greeting":{"type":"string","x-mcp-header":"Greeting"},
        "nothing":{"type":"string","x-mcp-header":"Nothing"},
        "absent":{"type":"string","x-mcp-header":"Absent"}}}))
    .unwrap();
    let args = json!({"region":"us-west1","n":-7,"flag":true,
                      "greeting":"Hello, 世界","nothing":null});
    let headers: HashMap<String, String> = param_headers(&specs, Some(&args))
        .into_iter()
        .map(|(k, v)| (k.to_string(), v.to_str().unwrap().to_string()))
        .collect();
    assert_eq!(headers["mcp-param-region"], "us-west1");
    assert_eq!(headers["mcp-param-n"], "-7");
    assert_eq!(headers["mcp-param-flag"], "true");
    assert_eq!(
        headers["mcp-param-greeting"],
        "=?base64?SGVsbG8sIOS4lueVjA==?="
    );
    // null and absent values omit the header.
    assert!(!headers.contains_key("mcp-param-nothing"));
    assert!(!headers.contains_key("mcp-param-absent"));
    assert!(param_headers(&specs, None).is_empty());
}

#[test]
fn nested_values_are_read_from_the_exact_path() {
    let specs = extract(&json!({"properties":{"o":{"type":"object","properties":{
        "t":{"type":"string","x-mcp-header":"T"}}}}}))
    .unwrap();
    let args = json!({"o":{"t":"deep"},"t":"shallow"});
    let headers = param_headers(&specs, Some(&args));
    assert_eq!(headers.len(), 1);
    assert_eq!(headers[0].1.to_str().unwrap(), "deep");
}

//! `x-mcp-header` support for MCP 2026-07-28 Streamable HTTP.
//!
//! A server may annotate tool parameters in `inputSchema` with
//! `"x-mcp-header": "Name"`. The client must then mirror the argument value
//! into an `Mcp-Param-Name` header on `tools/call`, or the server rejects the
//! request with `-32020 HeaderMismatch`. Invalid annotations make the tool
//! unusable, so the client drops that tool from its catalog (spec: "MUST
//! exclude the invalid tool from the result of tools/list", with a warning).
//! stdio has no header layer and ignores the annotations.

use reqwest::header::{HeaderName, HeaderValue};
use serde_json::Value;
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, RwLock};

const KEY: &str = "x-mcp-header";

/// One mirrored parameter: header name suffix and the property path to read.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct HeaderParam {
    pub name: String,
    pub path: Vec<String>,
}

/// Per tool name, shared between the client (writes on `tools/list`) and the
/// HTTP transport (reads on `tools/call`).
pub(super) type HeaderSpecs = Arc<RwLock<HashMap<String, Vec<HeaderParam>>>>;

fn is_token(name: &str) -> bool {
    !name.is_empty()
        && name.bytes().all(|b| {
            b.is_ascii_alphanumeric()
                || matches!(
                    b,
                    b'!' | b'#'
                        | b'$'
                        | b'%'
                        | b'&'
                        | b'\''
                        | b'*'
                        | b'+'
                        | b'-'
                        | b'.'
                        | b'^'
                        | b'_'
                        | b'`'
                        | b'|'
                        | b'~'
                )
        })
}

/// Collect the annotations of one `inputSchema`, or the reason it is invalid.
pub(super) fn extract(schema: &Value) -> Result<Vec<HeaderParam>, String> {
    let mut out = Vec::new();
    let mut seen = HashSet::new();
    walk(schema, &mut Vec::new(), true, &mut out, &mut seen)?;
    Ok(out)
}

fn walk(
    node: &Value,
    path: &mut Vec<String>,
    reachable: bool,
    out: &mut Vec<HeaderParam>,
    seen: &mut HashSet<String>,
) -> Result<(), String> {
    let Value::Object(map) = node else {
        if let Value::Array(items) = node {
            for item in items {
                walk(item, path, false, out, seen)?;
            }
        }
        return Ok(());
    };
    if let Some(annotation) = map.get(KEY) {
        if !reachable || path.is_empty() {
            return Err(format!(
                "{KEY} is only allowed on properties reachable through `properties` keys"
            ));
        }
        let Some(name) = annotation.as_str() else {
            return Err(format!("{KEY} must be a string"));
        };
        if !is_token(name) {
            return Err(format!("{KEY} value {name:?} is not a valid header token"));
        }
        if !seen.insert(name.to_ascii_lowercase()) {
            return Err(format!("{KEY} value {name:?} is not unique"));
        }
        match map.get("type").and_then(Value::as_str) {
            Some("string" | "integer" | "boolean") => {}
            _ => {
                return Err(format!(
                    "{KEY} {name:?} needs type string, integer or boolean"
                ));
            }
        }
        out.push(HeaderParam {
            name: name.to_string(),
            path: path.clone(),
        });
    }
    for (key, child) in map {
        if key == KEY {
            continue;
        }
        if key == "properties" && reachable {
            if let Value::Object(props) = child {
                for (prop, schema) in props {
                    path.push(prop.clone());
                    walk(schema, path, true, out, seen)?;
                    path.pop();
                }
            }
        } else {
            // Any other keyword (items, oneOf, $ref, if/then/else, ...) makes
            // what lies below it unreachable by a `properties`-only chain.
            walk(child, path, false, out, seen)?;
        }
    }
    Ok(())
}

/// Header-safe plain ASCII is sent as is. Anything else, and any plain value
/// that looks like the sentinel, is `=?base64?...?=` (spec: Value Encoding).
pub(super) fn encode_header_value(value: &str) -> String {
    use base64::Engine as _;
    let plain = !value.is_empty()
        && value
            .bytes()
            .all(|b| (0x21..=0x7e).contains(&b) || b == b' ')
        && !value.starts_with(' ')
        && !value.ends_with(' ')
        && !(value.starts_with("=?base64?") && value.ends_with("?="));
    if plain {
        value.to_string()
    } else {
        format!(
            "=?base64?{}?=",
            base64::engine::general_purpose::STANDARD.encode(value.as_bytes())
        )
    }
}

fn scalar_text(value: &Value) -> Option<String> {
    match value {
        Value::String(s) => Some(s.clone()),
        Value::Bool(b) => Some(b.to_string()),
        Value::Number(n) if n.is_i64() || n.is_u64() => Some(n.to_string()),
        // null, absent, numbers with fractions, objects: the header is omitted.
        _ => None,
    }
}

/// `Mcp-Param-*` headers for one `tools/call` body. A missing or null value
/// omits its header.
pub(super) fn param_headers(
    specs: &[HeaderParam],
    arguments: Option<&Value>,
) -> Vec<(HeaderName, HeaderValue)> {
    let mut headers = Vec::new();
    for spec in specs {
        let mut node = arguments;
        for key in &spec.path {
            node = node.and_then(|n| n.get(key));
        }
        let Some(text) = node.and_then(scalar_text) else {
            continue;
        };
        let (Ok(name), Ok(value)) = (
            HeaderName::from_bytes(
                format!("mcp-param-{}", spec.name.to_ascii_lowercase()).as_bytes(),
            ),
            HeaderValue::from_str(&encode_header_value(&text)),
        ) else {
            continue;
        };
        headers.push((name, value));
    }
    headers
}

#[cfg(test)]
#[path = "header_params_tests.rs"]
mod header_params_tests;

//! MCP resources and prompts: list, read and get over `McpHandle::request`.
//!
//! Everything a server returns here is untrusted data. This module bounds it
//! (text per item, list length, blobs never inlined) and tolerates unknown
//! fields; callers must still present the text as data, not instructions.

use super::client::McpHandle;
use super::protocol::ServerCapabilities;
use anyhow::{Context, Result};
use async_trait::async_trait;
use serde_json::{Value, json};
use std::collections::{BTreeMap, HashSet};

/// Same page limit as `tools/list` in `McpHandle::refresh_tools`.
const MAX_LIST_PAGES: usize = 256;
/// Most list entries kept; the rest are dropped and `truncated` is set.
pub const MAX_LIST_ITEMS: usize = 2_000;
/// Most text bytes kept per resource content or prompt message.
pub const MAX_TEXT_BYTES: usize = 64 * 1024;
/// Most contents / messages kept from one read or prompt get.
pub const MAX_PARTS: usize = 64;
/// Longest name, uri, mime type or description kept in list entries.
const MAX_META_BYTES: usize = 1_024;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Listing<T> {
    pub items: Vec<T>,
    /// More entries existed than [`MAX_LIST_ITEMS`].
    pub truncated: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResourceInfo {
    pub uri: String,
    pub name: Option<String>,
    pub description: Option<String>,
    pub mime_type: Option<String>,
    pub size: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PromptArgumentInfo {
    pub name: String,
    pub description: Option<String>,
    pub required: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PromptInfo {
    pub name: String,
    pub description: Option<String>,
    pub arguments: Vec<PromptArgumentInfo>,
}

/// One entry of a `resources/read` result. A blob is described, never inlined.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResourceContent {
    pub uri: String,
    pub mime_type: Option<String>,
    pub text: Option<String>,
    /// `text` was cut at [`MAX_TEXT_BYTES`]; `text_bytes` is the original size.
    pub truncated: bool,
    pub text_bytes: usize,
    /// Size of the base64 blob payload as sent, when the entry was a blob.
    pub blob_bytes: Option<usize>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResourceRead {
    pub contents: Vec<ResourceContent>,
    /// Entries beyond [`MAX_PARTS`] were dropped.
    pub contents_truncated: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PromptMessage {
    pub role: String,
    /// Text content, or a `[kind: mime, N bytes]` description for non-text.
    pub text: String,
    pub truncated: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PromptResult {
    pub description: Option<String>,
    pub messages: Vec<PromptMessage>,
    pub messages_truncated: bool,
}

/// The one capability this module needs; `McpHandle::request` in production.
#[async_trait]
trait Rpc: Send + Sync {
    async fn rpc(&self, method: &str, params: Option<Value>) -> Result<Value>;
}

#[async_trait]
impl Rpc for McpHandle {
    async fn rpc(&self, method: &str, params: Option<Value>) -> Result<Value> {
        // `request` already turns a JSON-RPC `error` into `Err`.
        self.request(method, params)
            .await?
            .result
            .with_context(|| format!("MCP {method} response missing result"))
    }
}

fn clip(text: &str, max: usize) -> (&str, bool) {
    if text.len() <= max {
        return (text, false);
    }
    let mut end = max;
    while !text.is_char_boundary(end) {
        end -= 1;
    }
    (&text[..end], true)
}

fn meta(value: Option<&Value>) -> Option<String> {
    value
        .and_then(Value::as_str)
        .map(|s| clip(s, MAX_META_BYTES).0.to_string())
}

async fn list_paged<R: Rpc, T>(
    rpc: &R,
    method: &str,
    key: &str,
    parse: fn(&Value) -> Option<T>,
) -> Result<Listing<T>> {
    let mut params = None;
    let mut seen_cursors = HashSet::new();
    let mut items = Vec::new();
    let mut truncated = false;
    for page_index in 0..MAX_LIST_PAGES {
        let result = rpc.rpc(method, params).await?;
        let page = result
            .get(key)
            .and_then(Value::as_array)
            .with_context(|| format!("MCP {method} response missing `{key}` array"))?;
        for entry in page {
            // A malformed entry is skipped, not fatal for the whole list.
            let Some(item) = parse(entry) else { continue };
            if items.len() >= MAX_LIST_ITEMS {
                truncated = true;
                break;
            }
            items.push(item);
        }
        if truncated {
            return Ok(Listing { items, truncated });
        }
        match result.get("nextCursor").and_then(Value::as_str) {
            Some(cursor) => {
                if !seen_cursors.insert(cursor.to_string()) {
                    anyhow::bail!("MCP {method} cursor cycle detected");
                }
                if page_index + 1 == MAX_LIST_PAGES {
                    anyhow::bail!("MCP {method} exceeded the {MAX_LIST_PAGES}-page limit");
                }
                params = Some(json!({"cursor": cursor}));
            }
            None => return Ok(Listing { items, truncated }),
        }
    }
    unreachable!("the last page either completes or returns the page-limit error")
}

fn parse_resource_info(entry: &Value) -> Option<ResourceInfo> {
    Some(ResourceInfo {
        uri: meta(entry.get("uri"))?,
        name: meta(entry.get("name")),
        description: meta(entry.get("description")),
        mime_type: meta(entry.get("mimeType")),
        size: entry.get("size").and_then(Value::as_u64),
    })
}

fn parse_prompt_info(entry: &Value) -> Option<PromptInfo> {
    let arguments = entry
        .get("arguments")
        .and_then(Value::as_array)
        .map(|args| {
            args.iter()
                .filter_map(|arg| {
                    Some(PromptArgumentInfo {
                        name: meta(arg.get("name"))?,
                        description: meta(arg.get("description")),
                        required: arg
                            .get("required")
                            .and_then(Value::as_bool)
                            .unwrap_or(false),
                    })
                })
                .collect()
        })
        .unwrap_or_default();
    Some(PromptInfo {
        name: meta(entry.get("name"))?,
        description: meta(entry.get("description")),
        arguments,
    })
}

fn parse_resource_content(entry: &Value, fallback_uri: &str) -> ResourceContent {
    let uri = meta(entry.get("uri")).unwrap_or_else(|| fallback_uri.to_string());
    let mime_type = meta(entry.get("mimeType"));
    if let Some(text) = entry.get("text").and_then(Value::as_str) {
        let (kept, truncated) = clip(text, MAX_TEXT_BYTES);
        return ResourceContent {
            uri,
            mime_type,
            text: Some(kept.to_string()),
            truncated,
            text_bytes: text.len(),
            blob_bytes: None,
        };
    }
    ResourceContent {
        uri,
        mime_type,
        text: None,
        truncated: false,
        text_bytes: 0,
        blob_bytes: entry.get("blob").and_then(Value::as_str).map(str::len),
    }
}

async fn read_resource_via<R: Rpc>(rpc: &R, uri: &str) -> Result<ResourceRead> {
    let result = rpc.rpc("resources/read", Some(json!({"uri": uri}))).await?;
    let entries = result
        .get("contents")
        .and_then(Value::as_array)
        .context("MCP resources/read response missing `contents` array")?;
    Ok(ResourceRead {
        contents: entries
            .iter()
            .take(MAX_PARTS)
            .map(|entry| parse_resource_content(entry, uri))
            .collect(),
        contents_truncated: entries.len() > MAX_PARTS,
    })
}

/// Render one prompt message content block as bounded text.
fn prompt_content_text(content: &Value) -> (String, bool) {
    let describe = |kind: &str, mime: Option<&str>, data: Option<&str>| {
        format!(
            "[{kind}: {} ({} bytes)]",
            mime.unwrap_or("unknown type"),
            data.map_or(0, str::len)
        )
    };
    let text_or_blob = |resource: &Value| -> (String, bool) {
        let uri = resource.get("uri").and_then(Value::as_str).unwrap_or("");
        match resource.get("text").and_then(Value::as_str) {
            Some(text) => {
                let (kept, truncated) = clip(text, MAX_TEXT_BYTES);
                (kept.to_string(), truncated)
            }
            None => (
                format!(
                    "[resource {}: {} ({} bytes)]",
                    clip(uri, MAX_META_BYTES).0,
                    resource
                        .get("mimeType")
                        .and_then(Value::as_str)
                        .unwrap_or("unknown type"),
                    resource
                        .get("blob")
                        .and_then(Value::as_str)
                        .map_or(0, str::len)
                ),
                false,
            ),
        }
    };
    match content.get("type").and_then(Value::as_str) {
        Some("text") => {
            let text = content.get("text").and_then(Value::as_str).unwrap_or("");
            let (kept, truncated) = clip(text, MAX_TEXT_BYTES);
            (kept.to_string(), truncated)
        }
        Some(kind @ ("image" | "audio")) => (
            describe(
                kind,
                content.get("mimeType").and_then(Value::as_str),
                content.get("data").and_then(Value::as_str),
            ),
            false,
        ),
        Some("resource") => match content.get("resource") {
            Some(resource) => text_or_blob(resource),
            None => ("[resource]".to_string(), false),
        },
        // Some servers send a bare string as the content.
        _ => match content.as_str() {
            Some(text) => {
                let (kept, truncated) = clip(text, MAX_TEXT_BYTES);
                (kept.to_string(), truncated)
            }
            None => ("[unsupported content]".to_string(), false),
        },
    }
}

async fn get_prompt_via<R: Rpc>(
    rpc: &R,
    name: &str,
    arguments: &BTreeMap<String, String>,
) -> Result<PromptResult> {
    let mut params = json!({"name": name});
    if !arguments.is_empty() {
        params["arguments"] = json!(arguments);
    }
    let result = rpc.rpc("prompts/get", Some(params)).await?;
    let entries = result
        .get("messages")
        .and_then(Value::as_array)
        .context("MCP prompts/get response missing `messages` array")?;
    let messages = entries
        .iter()
        .take(MAX_PARTS)
        .map(|entry| {
            let (text, truncated) = entry
                .get("content")
                .map_or_else(|| (String::new(), false), prompt_content_text);
            PromptMessage {
                role: meta(entry.get("role")).unwrap_or_else(|| "unknown".to_string()),
                text,
                truncated,
            }
        })
        .collect();
    Ok(PromptResult {
        description: meta(result.get("description")),
        messages,
        messages_truncated: entries.len() > MAX_PARTS,
    })
}

/// Fail before any request when the server did not advertise `capability`
/// (`resources` or `prompts`) during `initialize`.
fn require_capability(
    server: &str,
    advertised: &ServerCapabilities,
    capability: &str,
    method: &str,
) -> Result<()> {
    let present = match capability {
        "resources" => advertised.resources.is_some(),
        "prompts" => advertised.prompts.is_some(),
        _ => true,
    };
    anyhow::ensure!(
        present,
        "MCP server '{server}' did not advertise the '{capability}' capability, so {method} is not available on it"
    );
    Ok(())
}

impl McpHandle {
    fn require(&self, capability: &str, method: &str) -> Result<()> {
        require_capability(self.name(), &self.capabilities(), capability, method)
    }

    /// `resources/list`, following `nextCursor` pages.
    pub async fn list_resources(&self) -> Result<Listing<ResourceInfo>> {
        self.require("resources", "resources/list")?;
        list_paged(self, "resources/list", "resources", parse_resource_info).await
    }

    /// `resources/read` for `uri`. JSON-RPC errors are returned as `Err`.
    pub async fn read_resource(&self, uri: &str) -> Result<ResourceRead> {
        self.require("resources", "resources/read")?;
        read_resource_via(self, uri).await
    }

    /// `prompts/list`, following `nextCursor` pages.
    pub async fn list_prompts(&self) -> Result<Listing<PromptInfo>> {
        self.require("prompts", "prompts/list")?;
        list_paged(self, "prompts/list", "prompts", parse_prompt_info).await
    }

    /// `prompts/get` for `name` with string `arguments`.
    pub async fn get_prompt(
        &self,
        name: &str,
        arguments: &BTreeMap<String, String>,
    ) -> Result<PromptResult> {
        self.require("prompts", "prompts/get")?;
        get_prompt_via(self, name, arguments).await
    }
}

#[cfg(test)]
#[path = "resources_tests.rs"]
mod tests;

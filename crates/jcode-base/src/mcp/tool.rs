//! MCP Tool - wraps MCP server tools for jcode's tool system

use super::manager::McpManager;
use super::protocol::{ContentBlock, McpToolDef};
use anyhow::Result;
use async_trait::async_trait;
use jcode_tool_core::{Tool, ToolContext};
use jcode_tool_types::{ToolImage, ToolOutput};
use serde_json::Value;
use std::sync::Arc;
use tokio::sync::RwLock;

const MAX_MCP_IMAGE_SIZE: usize = 20 * 1024 * 1024;
const MAX_MCP_IMAGE_BASE64_SIZE: usize = MAX_MCP_IMAGE_SIZE.div_ceil(3) * 4;

fn validated_image(data: String, mime_type: &str) -> Option<ToolImage> {
    use base64::Engine as _;

    if !matches!(
        mime_type,
        "image/png" | "image/jpeg" | "image/gif" | "image/webp"
    ) || data.is_empty()
        || data.len() > MAX_MCP_IMAGE_BASE64_SIZE
    {
        return None;
    }
    let decoded = base64::engine::general_purpose::STANDARD
        .decode(&data)
        .ok()?;
    (decoded.len() <= MAX_MCP_IMAGE_SIZE).then(|| ToolImage {
        media_type: mime_type.to_string(),
        data,
        label: None,
    })
}

fn tool_output_from_result(result: super::protocol::ToolCallResult, title: String) -> ToolOutput {
    let mut output_parts = Vec::new();
    let mut images = Vec::new();
    for block in result.content {
        match block {
            ContentBlock::Text { text } => output_parts.push(text),
            ContentBlock::Image { data, mime_type } => {
                if let Some(image) = validated_image(data, &mime_type) {
                    images.push(image);
                } else {
                    output_parts.push(format!(
                        "[Image omitted: invalid or unsupported {} payload]",
                        mime_type
                    ));
                }
            }
            ContentBlock::Resource { resource } => {
                if let Some(text) = resource.text {
                    output_parts.push(text);
                } else if let Some(blob) = resource.blob {
                    output_parts.push(format!(
                        "[Resource: {} ({} bytes)]",
                        resource.uri,
                        blob.len()
                    ));
                } else {
                    output_parts.push(format!("[Resource: {}]", resource.uri));
                }
            }
        }
    }
    let output = output_parts.join("\n");
    let output = if result.is_error {
        format!("Error: {}", output)
    } else {
        output
    };
    ToolOutput {
        output,
        title: Some(title),
        metadata: None,
        images,
    }
}

/// A tool that proxies to an MCP server
pub struct McpTool {
    server_name: String,
    tool_def: McpToolDef,
    manager: Arc<RwLock<McpManager>>,
}

impl McpTool {
    pub fn new(
        server_name: String,
        tool_def: McpToolDef,
        manager: Arc<RwLock<McpManager>>,
    ) -> Self {
        Self {
            server_name,
            tool_def,
            manager,
        }
    }
}

#[async_trait]
impl Tool for McpTool {
    fn name(&self) -> &str {
        // This will be overridden in registration with prefixed name
        &self.tool_def.name
    }

    fn description(&self) -> &str {
        self.tool_def.description.as_deref().unwrap_or("MCP tool")
    }

    fn parameters_schema(&self) -> Value {
        self.tool_def.input_schema.clone()
    }

    async fn execute(&self, input: Value, _ctx: ToolContext) -> Result<ToolOutput> {
        let mut input = if input.is_null() {
            Value::Object(serde_json::Map::new())
        } else {
            input
        };
        // `intent` is a jcode-injected display-only parameter (see
        // ensure_intent_in_schema). Strip it before forwarding unless the
        // MCP server's own schema declares an `intent` property.
        let server_declares_intent = self
            .tool_def
            .input_schema
            .get("properties")
            .and_then(|p| p.as_object())
            .is_some_and(|p| p.contains_key("intent"));
        if !server_declares_intent && let Some(object) = input.as_object_mut() {
            object.remove("intent");
        }
        let manager = self.manager.read().await;
        let result = manager
            .call_tool(&self.server_name, &self.tool_def.name, input)
            .await?;

        let title = format!("mcp:{}:{}", self.server_name, self.tool_def.name);
        Ok(tool_output_from_result(result, title))
    }
}

pub fn dispatch_name(server_name: &str, tool_name: &str) -> String {
    format!("mcp__{}__{}", server_name, tool_name).replace('-', "_")
}

/// Create tools from an MCP manager
pub async fn create_mcp_tools(manager: Arc<RwLock<McpManager>>) -> Vec<(String, Arc<dyn Tool>)> {
    let mgr = manager.read().await;
    let all_tools = mgr.all_tools().await;
    drop(mgr);

    let mut tools = Vec::new();
    for (server_name, tool_def) in all_tools {
        let prefixed_name = dispatch_name(&server_name, &tool_def.name);
        let mcp_tool = McpTool::new(server_name, tool_def, Arc::clone(&manager));
        tools.push((prefixed_name, Arc::new(mcp_tool) as Arc<dyn Tool>));
    }
    tools
}

/// Build proxy tools for a single server from cached schemas, without requiring
/// a live connection. Used to advertise a server's tools immediately at spawn
/// (the proxy connects on first call). The returned tools are functionally
/// identical to live ones; only their definitions come from the disk cache.
pub fn create_mcp_tools_from_cached(
    server_name: &str,
    tool_defs: &[McpToolDef],
    manager: Arc<RwLock<McpManager>>,
) -> Vec<(String, Arc<dyn Tool>)> {
    tool_defs
        .iter()
        .map(|tool_def| {
            let prefixed_name = dispatch_name(server_name, &tool_def.name);
            let mcp_tool = McpTool::new(
                server_name.to_string(),
                tool_def.clone(),
                Arc::clone(&manager),
            );
            (prefixed_name, Arc::new(mcp_tool) as Arc<dyn Tool>)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::{McpTool, dispatch_name, tool_output_from_result};
    use crate::mcp::manager::McpManager;
    use crate::mcp::protocol::{
        ContentBlock, McpConfig, McpServerConfig, McpToolDef, ToolCallResult,
    };
    use base64::Engine as _;
    use jcode_tool_core::{Tool, ToolContext};
    use serde_json::json;
    use std::collections::HashMap;
    use std::sync::Arc;
    use tokio::sync::RwLock;

    #[tokio::test]
    async fn stdio_mcp_image_reaches_mcp_tool_output() {
        let temp = tempfile::tempdir().expect("temp dir");
        let server = temp.path().join("image-mcp.py");
        std::fs::write(
            &server,
            r#"import json,sys
png='iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAQAAAC1HAwCAAAAC0lEQVR42mP8/x8AAwMCAO+jQioAAAAASUVORK5CYII='
for line in sys.stdin:
 r=json.loads(line); i=r.get('id'); m=r.get('method')
 if i is None: continue
 if m=='initialize': out={'protocolVersion':'2024-11-05','capabilities':{'tools':{}},'serverInfo':{'name':'test','version':'1'}}
 elif m=='tools/list': out={'tools':[{'name':'capture_screenshot','inputSchema':{'type':'object','properties':{}}}]}
 elif m=='tools/call': out={'content':[{'type':'text','text':'fixture'},{'type':'image','data':png,'mimeType':'image/png'}]}
 else: continue
 print(json.dumps({'jsonrpc':'2.0','id':i,'result':out}),flush=True)
"#,
        )
        .expect("write MCP fixture");
        let mut config = McpConfig::default();
        config.servers.insert(
            "image-test".to_string(),
            McpServerConfig {
                command: "python3".to_string(),
                args: vec![server.to_string_lossy().to_string()],
                env: HashMap::new(),
                shared: false,
                transport: None,
                url: None,
                headers: HashMap::new(),
                enabled: None,
                disabled: None,
            },
        );
        let manager = Arc::new(RwLock::new(McpManager::with_config(config)));
        let server_config = manager.read().await.config().servers["image-test"].clone();
        manager
            .read()
            .await
            .connect("image-test", &server_config)
            .await
            .expect("connect real stdio MCP fixture");
        let tool_def = McpToolDef {
            name: "capture_screenshot".to_string(),
            description: None,
            input_schema: json!({"type":"object","properties":{}}),
        };
        let tool = McpTool::new("image-test".to_string(), tool_def, Arc::clone(&manager));
        let output = tool
            .execute(
                json!({}),
                ToolContext {
                    session_id: "test".to_string(),
                    message_id: "message".to_string(),
                    tool_call_id: "call".to_string(),
                    working_dir: None,
                    stdin_request_tx: None,
                    pending_question_tx: None,
                    graceful_shutdown_signal: None,
                    execution_mode: jcode_tool_core::ToolExecutionMode::Direct,
                },
            )
            .await
            .expect("execute MCP screenshot tool");
        assert_eq!(output.output, "fixture");
        assert_eq!(output.images.len(), 1);
        assert_eq!(output.images[0].media_type, "image/png");
        assert!(!output.images[0].data.is_empty());
        manager.write().await.disconnect_all().await;
    }

    #[test]
    fn mcp_image_content_is_preserved_in_tool_output() {
        let data = base64::engine::general_purpose::STANDARD.encode(b"png bytes");
        let output = tool_output_from_result(
            ToolCallResult {
                content: vec![ContentBlock::Image {
                    data: data.clone(),
                    mime_type: "image/png".to_string(),
                }],
                is_error: false,
            },
            "mcp:desktop:screenshot".to_string(),
        );
        assert_eq!(output.images.len(), 1);
        assert_eq!(output.images[0].media_type, "image/png");
        assert_eq!(output.images[0].data, data);
    }

    #[test]
    fn invalid_or_oversized_mcp_images_are_omitted() {
        let oversized =
            base64::engine::general_purpose::STANDARD.encode(vec![0; 20 * 1024 * 1024 + 1]);
        let output = tool_output_from_result(
            ToolCallResult {
                content: vec![
                    ContentBlock::Image {
                        data: String::new(),
                        mime_type: "image/png".to_string(),
                    },
                    ContentBlock::Image {
                        data: "not-base64".to_string(),
                        mime_type: "image/png".to_string(),
                    },
                    ContentBlock::Image {
                        data: "AQID".to_string(),
                        mime_type: "text/plain".to_string(),
                    },
                    ContentBlock::Image {
                        data: oversized,
                        mime_type: "image/png".to_string(),
                    },
                ],
                is_error: false,
            },
            "mcp:desktop:screenshot".to_string(),
        );
        assert!(output.images.is_empty());
        assert_eq!(output.output.matches("[Image omitted:").count(), 4);
    }

    #[test]
    fn hyphenated_mcp_names_are_safe_for_the_standard_dispatcher() {
        assert_eq!(
            dispatch_name("context7", "resolve-library-id"),
            "mcp__context7__resolve_library_id"
        );
        assert_eq!(
            dispatch_name("hyphenated-server", "query-docs"),
            "mcp__hyphenated_server__query_docs"
        );
    }
}

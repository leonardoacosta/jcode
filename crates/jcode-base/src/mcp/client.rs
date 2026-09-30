//! MCP Client - handles communication with a single MCP server

use super::protocol::*;
use anyhow::{Context, Result};
use serde_json::Value;
use std::collections::HashMap;
use std::process::Stdio;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::{Child, Command};
use tokio::sync::{Mutex, mpsc, oneshot};

enum McpTransport {
    Stdio {
        pending: Arc<Mutex<HashMap<u64, oneshot::Sender<JsonRpcResponse>>>>,
        writer_tx: mpsc::Sender<String>,
    },
    Http {
        client: reqwest::Client,
        url: String,
        headers: HashMap<String, String>,
        session_id: Arc<Mutex<Option<String>>>,
        protocol_version: Arc<Mutex<Option<String>>>,
    },
}

/// Shared communication handle for an MCP server.
/// Multiple sessions can hold clones of this and send concurrent requests.
/// Request/response correlation by ID ensures no interference.
#[derive(Clone)]
pub struct McpHandle {
    pub(crate) name: String,
    request_id: Arc<AtomicU64>,
    transport: Arc<McpTransport>,
    server_info: Arc<std::sync::RwLock<Option<ServerInfo>>>,
    capabilities: Arc<std::sync::RwLock<ServerCapabilities>>,
    tools: Arc<std::sync::RwLock<Vec<McpToolDef>>>,
    /// Reply timeout applied to every request on this server.
    request_timeout: std::time::Duration,
}

fn parse_http_jsonrpc_response(body: &str, expected_id: u64) -> Result<JsonRpcResponse> {
    let parse = |data: &str| -> Result<Option<JsonRpcResponse>> {
        let value: Value =
            serde_json::from_str(data).context("Malformed MCP HTTP JSON-RPC response")?;
        if value.get("id").and_then(Value::as_u64) != Some(expected_id) {
            return Ok(None);
        }
        Ok(Some(
            serde_json::from_value(value).context("Invalid MCP HTTP JSON-RPC response")?,
        ))
    };
    if let Ok(Some(response)) = parse(body) {
        return Ok(response);
    }
    let mut data = String::new();
    for line in body.lines().chain(std::iter::once("")) {
        if let Some(value) = line.strip_prefix("data:") {
            if !data.is_empty() {
                data.push('\n');
            }
            data.push_str(value.strip_prefix(' ').unwrap_or(value));
        } else if line.is_empty() && !data.is_empty() {
            if let Ok(Some(response)) = parse(&data) {
                return Ok(response);
            }
            data.clear();
        }
    }
    anyhow::bail!("MCP HTTP response did not contain JSON-RPC id {expected_id}")
}

async fn read_http_jsonrpc_response(
    response: reqwest::Response,
    expected_id: u64,
) -> Result<JsonRpcResponse> {
    const MAX_SSE_EVENT_BYTES: usize = 1024 * 1024;
    const MAX_SSE_RESPONSE_BYTES: usize = 8 * 1024 * 1024;
    let is_sse = response
        .headers()
        .get(reqwest::header::CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .is_some_and(|value| {
            value
                .split(';')
                .next()
                .is_some_and(|mime| mime.trim().eq_ignore_ascii_case("text/event-stream"))
        });
    if !is_sse {
        let body = response.text().await?;
        return parse_http_jsonrpc_response(&body, expected_id);
    }
    use futures::StreamExt;
    let mut stream = response.bytes_stream();
    let mut buffer = Vec::new();
    let mut data = String::new();
    let mut total_bytes = 0usize;
    while let Some(chunk) = stream.next().await {
        let chunk = chunk?;
        total_bytes = total_bytes.saturating_add(chunk.len());
        anyhow::ensure!(
            total_bytes <= MAX_SSE_RESPONSE_BYTES,
            "MCP SSE response exceeds 8 MiB limit"
        );
        buffer.extend_from_slice(&chunk);
        anyhow::ensure!(
            buffer.len() <= MAX_SSE_EVENT_BYTES,
            "MCP SSE line exceeds 1 MiB limit"
        );
        while let Some(end) = buffer.iter().position(|byte| *byte == b'\n') {
            let mut line = buffer.drain(..=end).collect::<Vec<_>>();
            if line.last() == Some(&b'\n') {
                line.pop();
            }
            if line.last() == Some(&b'\r') {
                line.pop();
            }
            let line = String::from_utf8(line).context("Non-UTF8 MCP SSE event")?;
            if let Some(value) = line.strip_prefix("data:") {
                if !data.is_empty() {
                    data.push('\n');
                }
                data.push_str(value.strip_prefix(' ').unwrap_or(value));
                anyhow::ensure!(
                    data.len() <= MAX_SSE_EVENT_BYTES,
                    "MCP SSE event exceeds 1 MiB limit"
                );
            } else if line.is_empty() && !data.is_empty() {
                if let Ok(Some(response)) = parse_sse_response(&data, expected_id) {
                    return Ok(response);
                }
                data.clear();
            }
        }
    }
    if !data.is_empty() {
        if let Ok(Some(response)) = parse_sse_response(&data, expected_id) {
            return Ok(response);
        }
    }
    anyhow::bail!("MCP SSE stream ended without JSON-RPC id {expected_id}")
}

fn parse_sse_response(data: &str, expected_id: u64) -> Result<Option<JsonRpcResponse>> {
    let value: Value = serde_json::from_str(data).context("Malformed MCP SSE JSON-RPC response")?;
    if value.get("id").and_then(Value::as_u64) != Some(expected_id) {
        return Ok(None);
    }
    Ok(Some(
        serde_json::from_value(value).context("Invalid MCP SSE JSON-RPC response")?,
    ))
}

/// Default reply timeout when a server config does not set `timeout_secs`.
pub const DEFAULT_MCP_REQUEST_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(30);

/// Resolve the per-request reply timeout for a server config.
pub fn request_timeout_for(config: &McpServerConfig) -> std::time::Duration {
    config
        .timeout_secs
        .filter(|secs| *secs > 0)
        .map(std::time::Duration::from_secs)
        .unwrap_or(DEFAULT_MCP_REQUEST_TIMEOUT)
}

impl McpHandle {
    async fn send_notification(&self, notification: JsonRpcNotification) -> Result<()> {
        match self.transport.as_ref() {
            McpTransport::Stdio { writer_tx, .. } => {
                writer_tx
                    .send(serde_json::to_string(&notification)? + "\n")
                    .await?;
            }
            McpTransport::Http {
                client,
                url,
                headers,
                session_id,
                protocol_version,
            } => {
                let response = tokio::time::timeout(self.request_timeout, async {
                    let mut request = client
                        .post(url)
                        .json(&notification)
                        .header("accept", "application/json, text/event-stream");
                    for (key, value) in headers {
                        request = request.header(key, value);
                    }
                    if let Some(id) = session_id.lock().await.as_deref() {
                        request = request.header("mcp-session-id", id);
                    }
                    if let Some(version) = protocol_version.lock().await.as_deref() {
                        request = request.header("mcp-protocol-version", version);
                    }
                    request.send().await.map_err(anyhow::Error::from)
                })
                .await
                .context("MCP notification timeout")??;
                anyhow::ensure!(
                    response.status().is_success(),
                    "MCP HTTP notification returned {}",
                    response.status()
                );
                if let Some(id) = response
                    .headers()
                    .get("mcp-session-id")
                    .and_then(|v| v.to_str().ok())
                {
                    *session_id.lock().await = Some(id.to_owned());
                }
            }
        }
        Ok(())
    }

    /// Send a request and wait for response
    pub async fn request(&self, method: &str, params: Option<Value>) -> Result<JsonRpcResponse> {
        let id = self.request_id.fetch_add(1, Ordering::SeqCst);
        let request = JsonRpcRequest::new(id, method, params);

        let response = match self.transport.as_ref() {
            McpTransport::Stdio { pending, writer_tx } => {
                let (tx, rx) = oneshot::channel();
                pending.lock().await.insert(id, tx);
                writer_tx
                    .send(serde_json::to_string(&request)? + "\n")
                    .await
                    .context("Failed to send request")?;
                tokio::time::timeout(self.request_timeout, rx).await.with_context(|| format!("Request timeout after {}s (raise `timeout_secs` for MCP server '{}' if its tools legitimately run longer)", self.request_timeout.as_secs(), self.name))?.context("Channel closed")?
            }
            McpTransport::Http {
                client,
                url,
                headers,
                session_id,
                protocol_version,
            } => tokio::time::timeout(self.request_timeout, async {
                let mut builder = client
                    .post(url)
                    .json(&request)
                    .header("accept", "application/json, text/event-stream");
                for (key, value) in headers {
                    builder = builder.header(key, value);
                }
                if let Some(session) = session_id.lock().await.as_deref() {
                    builder = builder.header("mcp-session-id", session);
                }
                if let Some(version) = protocol_version.lock().await.as_deref() {
                    builder = builder.header("mcp-protocol-version", version);
                }
                let response = builder.send().await?;
                anyhow::ensure!(
                    response.status().is_success(),
                    "MCP HTTP request returned {}",
                    response.status()
                );
                if let Some(session) = response
                    .headers()
                    .get("mcp-session-id")
                    .and_then(|v| v.to_str().ok())
                {
                    *session_id.lock().await = Some(session.to_owned());
                }
                if method == "initialize" {
                    const HTTP_PROTOCOL_VERSION: &str = "2025-03-26";
                    let version = response
                        .headers()
                        .get("mcp-protocol-version")
                        .and_then(|value| value.to_str().ok())
                        .unwrap_or(HTTP_PROTOCOL_VERSION);
                    anyhow::ensure!(
                        version == HTTP_PROTOCOL_VERSION,
                        "MCP server selected unsupported protocol version {version}"
                    );
                }
                read_http_jsonrpc_response(response, id).await
            })
            .await
            .with_context(|| {
                format!(
                    "Request timeout after {}s for MCP server '{}'",
                    self.request_timeout.as_secs(),
                    self.name
                )
            })??,
        };

        if let Some(err) = &response.error {
            anyhow::bail!("MCP error {}: {}", err.code, err.message);
        }

        Ok(response)
    }

    /// Call a tool
    pub async fn call_tool(&self, name: &str, arguments: Value) -> Result<ToolCallResult> {
        let arguments = if arguments.is_null() {
            Value::Object(serde_json::Map::new())
        } else {
            arguments
        };
        let params = ToolCallParams {
            name: name.to_string(),
            arguments,
        };

        let response = self
            .request("tools/call", Some(serde_json::to_value(params)?))
            .await?;

        let result = response.result.context("No result from tool call")?;
        let tool_result: ToolCallResult = serde_json::from_value(result)?;

        Ok(tool_result)
    }

    /// Get the server name
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Get server info
    pub fn server_info(&self) -> Option<ServerInfo> {
        self.server_info
            .read()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .clone()
    }

    /// Get available tools
    pub fn tools(&self) -> Vec<McpToolDef> {
        self.tools
            .read()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .clone()
    }

    /// Refresh the list of available tools
    pub async fn refresh_tools(&self) -> Result<()> {
        let response = self.request("tools/list", None).await?;

        if let Some(result) = response.result {
            let tools_result: ToolsListResult = serde_json::from_value(result)?;
            *self
                .tools
                .write()
                .unwrap_or_else(|poisoned| poisoned.into_inner()) = tools_result.tools;
        }

        Ok(())
    }
}

/// MCP Client - owns the child process and provides shared handles.
/// Only one McpClient exists per MCP server process, but many McpHandle
/// clones can be distributed to different sessions.
pub struct McpClient {
    handle: McpHandle,
    child: Option<Child>,
}

impl McpClient {
    /// Connect to an MCP server, inheriting the current process working directory
    pub async fn connect(name: String, config: &McpServerConfig) -> Result<Self> {
        Self::connect_in_dir(name, config, None).await
    }

    /// Connect to an MCP server, optionally running it in `working_dir`.
    ///
    /// The working directory is only applied when it exists; otherwise the
    /// subprocess falls back to inheriting the current process cwd (issue #557).
    pub async fn connect_in_dir(
        name: String,
        config: &McpServerConfig,
        working_dir: Option<&std::path::Path>,
    ) -> Result<Self> {
        if matches!(
            config.transport.as_deref(),
            Some("http" | "streamable-http")
        ) {
            return Self::connect_http(name, config).await;
        }
        anyhow::ensure!(
            config.transport.as_deref().is_none_or(|t| t == "stdio"),
            "Unsupported MCP transport {:?}; legacy SSE is not Streamable HTTP",
            config.transport
        );
        let working_dir = working_dir.filter(|dir| dir.is_dir());
        crate::logging::info(&format!(
            "MCP: Connecting to '{}' ({} {:?}) cwd={:?}",
            name, config.command, config.args, working_dir
        ));

        // Credentials must be opted into an MCP server explicitly through its
        // config. The long-lived jcode daemon contains provider credentials in
        // its process environment, and blindly inheriting them exposes those
        // credentials to every configured MCP executable (issue #771).
        let inherited: HashMap<String, String> = std::env::vars().collect();
        let env = mcp_child_env(inherited, &config.env);

        let mut command = Command::new(&config.command);
        command
            .args(&config.args)
            .envs(&env)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        if let Some(dir) = working_dir {
            command.current_dir(dir);
        }
        let mut child = command
            .spawn()
            .with_context(|| format!("Failed to spawn MCP server: {}", config.command))?;

        let stdin = child.stdin.take().context("No stdin")?;
        let stdout = child.stdout.take().context("No stdout")?;
        let stderr = child.stderr.take().context("No stderr")?;

        // Spawn stderr reader
        let server_name = name.clone();
        tokio::spawn(async move {
            let mut reader = BufReader::new(stderr);
            let mut line = String::new();
            loop {
                line.clear();
                match reader.read_line(&mut line).await {
                    Ok(0) => break,
                    Ok(_) => {
                        let trimmed = line.trim();
                        if !trimmed.is_empty() {
                            crate::logging::warn(&format!(
                                "MCP [{}] stderr: {}",
                                server_name, trimmed
                            ));
                        }
                    }
                    Err(_) => break,
                }
            }
        });

        // Setup channels
        let pending: Arc<Mutex<HashMap<u64, oneshot::Sender<JsonRpcResponse>>>> =
            Arc::new(Mutex::new(HashMap::new()));
        let (writer_tx, mut writer_rx) = mpsc::channel::<String>(32);

        // Spawn writer task
        let mut stdin = stdin;
        tokio::spawn(async move {
            while let Some(msg) = writer_rx.recv().await {
                if stdin.write_all(msg.as_bytes()).await.is_err() {
                    break;
                }
                if stdin.flush().await.is_err() {
                    break;
                }
            }
        });

        // Spawn reader task
        let pending_clone = Arc::clone(&pending);
        let reader_name = name.clone();
        let mut reader = BufReader::new(stdout);
        tokio::spawn(async move {
            let mut line = String::new();
            loop {
                line.clear();
                match reader.read_line(&mut line).await {
                    Ok(0) => {
                        crate::logging::debug(&format!("MCP [{}]: stdout EOF", reader_name));
                        break;
                    }
                    Ok(_) => {
                        if let Ok(response) = serde_json::from_str::<JsonRpcResponse>(&line) {
                            if let Some(id) = response.id {
                                let mut pending = pending_clone.lock().await;
                                if let Some(tx) = pending.remove(&id) {
                                    let _ = tx.send(response);
                                }
                            }
                        } else {
                            let trimmed = line.trim();
                            if !trimmed.is_empty() {
                                crate::logging::debug(&format!(
                                    "MCP [{}] non-JSON output: {}",
                                    reader_name, trimmed
                                ));
                            }
                        }
                    }
                    Err(e) => {
                        crate::logging::warn(&format!("MCP [{}] read error: {}", reader_name, e));
                        break;
                    }
                }
            }
        });

        let handle = McpHandle {
            name: name.clone(),
            request_id: Arc::new(AtomicU64::new(1)),
            transport: Arc::new(McpTransport::Stdio { pending, writer_tx }),
            server_info: Arc::new(std::sync::RwLock::new(None)),
            capabilities: Arc::new(std::sync::RwLock::new(ServerCapabilities::default())),
            tools: Arc::new(std::sync::RwLock::new(Vec::new())),
            request_timeout: request_timeout_for(config),
        };

        let mut client = Self {
            handle,
            child: Some(child),
        };

        client
            .initialize()
            .await
            .with_context(|| format!("MCP server '{}' failed to initialize", name))?;

        client
            .handle
            .refresh_tools()
            .await
            .with_context(|| format!("MCP server '{}' failed to list tools", name))?;

        crate::logging::info(&format!(
            "MCP: Connected to '{}' with {} tools",
            name,
            client.handle.tools().len()
        ));

        Ok(client)
    }

    async fn connect_http(name: String, config: &McpServerConfig) -> Result<Self> {
        let url = config
            .url
            .as_deref()
            .filter(|url| !url.trim().is_empty())
            .context("HTTP MCP server requires url")?
            .to_owned();
        let handle = McpHandle {
            name,
            request_id: Arc::new(AtomicU64::new(1)),
            transport: Arc::new(McpTransport::Http {
                client: reqwest::Client::builder()
                    .timeout(request_timeout_for(config))
                    .redirect(reqwest::redirect::Policy::none())
                    .build()?,
                url,
                headers: config.headers.clone(),
                session_id: Arc::new(Mutex::new(None)),
                protocol_version: Arc::new(Mutex::new(Some("2025-03-26".to_owned()))),
            }),
            server_info: Arc::new(std::sync::RwLock::new(None)),
            capabilities: Arc::new(std::sync::RwLock::new(ServerCapabilities::default())),
            tools: Arc::new(std::sync::RwLock::new(Vec::new())),
            request_timeout: request_timeout_for(config),
        };
        let mut client = Self {
            handle,
            child: None,
        };
        client
            .initialize()
            .await
            .context("HTTP MCP server failed to initialize")?;
        client
            .handle
            .refresh_tools()
            .await
            .context("HTTP MCP server failed to list tools")?;
        Ok(client)
    }

    /// Get a shareable handle to this client
    pub fn handle(&self) -> McpHandle {
        self.handle.clone()
    }

    /// Initialize the MCP connection
    async fn initialize(&mut self) -> Result<()> {
        let params = InitializeParams {
            protocol_version: match self.handle.transport.as_ref() {
                McpTransport::Http { .. } => "2025-03-26".to_string(),
                McpTransport::Stdio { .. } => "2024-11-05".to_string(),
            },
            capabilities: ClientCapabilities::default(),
            client_info: ClientInfo {
                name: "jcode".to_string(),
                version: jcode_build_meta::pkg_version().to_string(),
            },
        };

        let response = self
            .handle
            .request("initialize", Some(serde_json::to_value(params)?))
            .await?;

        if let Some(result) = response.result {
            let init_result: InitializeResult = serde_json::from_value(result)?;
            *self
                .handle
                .server_info
                .write()
                .unwrap_or_else(|poisoned| poisoned.into_inner()) = init_result.server_info;
            *self
                .handle
                .capabilities
                .write()
                .unwrap_or_else(|poisoned| poisoned.into_inner()) = init_result.capabilities;
        }

        // Send initialized notification
        let notif = JsonRpcNotification::new("notifications/initialized", None);
        self.handle.send_notification(notif).await?;

        Ok(())
    }

    /// Check if server is still running
    pub fn is_running(&mut self) -> bool {
        match self.child.as_mut().map(Child::try_wait).transpose() {
            Ok(None) => true,
            Ok(Some(_)) => false,
            Err(_) => false,
        }
    }

    /// Shutdown the server
    pub async fn shutdown(&mut self) {
        if self.child.is_some() {
            let _ = self
                .handle
                .send_notification(JsonRpcNotification::new("shutdown", None))
                .await;
        }

        if let McpTransport::Http {
            client,
            url,
            headers,
            session_id,
            protocol_version,
        } = self.handle.transport.as_ref()
        {
            if let Some(session) = session_id.lock().await.take() {
                let mut request = client.delete(url).header("mcp-session-id", session);
                for (key, value) in headers {
                    request = request.header(key, value);
                }
                if let Some(version) = protocol_version.lock().await.as_deref() {
                    request = request.header("mcp-protocol-version", version);
                }
                let result = tokio::time::timeout(self.handle.request_timeout, async {
                    let response = request.send().await?;
                    anyhow::ensure!(
                        response.status().is_success()
                            || response.status() == reqwest::StatusCode::METHOD_NOT_ALLOWED,
                        "MCP session deletion returned {}",
                        response.status()
                    );
                    response.bytes().await?;
                    Ok::<_, anyhow::Error>(())
                })
                .await;
                let _ = result;
            }
        }

        tokio::time::sleep(std::time::Duration::from_millis(100)).await;

        if let Some(child) = self.child.as_mut() {
            let _ = child.kill().await;
        }
    }

    // === Legacy compatibility methods that delegate to handle ===

    pub fn name(&self) -> &str {
        &self.handle.name
    }

    pub fn server_info(&self) -> Option<ServerInfo> {
        self.handle.server_info()
    }

    pub fn tools(&self) -> Vec<McpToolDef> {
        self.handle.tools()
    }

    pub async fn call_tool(&self, name: &str, arguments: Value) -> Result<ToolCallResult> {
        self.handle.call_tool(name, arguments).await
    }

    pub async fn refresh_tools(&self) -> Result<()> {
        self.handle.refresh_tools().await
    }
}

/// Secrets that an MCP child must not receive merely because jcode has them.
///
/// This intentionally applies only to inherited values. A server can still be
/// given any of these names through `McpServerConfig::env`.
fn is_sensitive_inherited_env_key(key: &str) -> bool {
    let key = key.to_ascii_uppercase();
    key.ends_with("_API_KEY")
        || key.ends_with("_ACCESS_TOKEN")
        || key.ends_with("_AUTH_TOKEN")
        || matches!(
            key.as_str(),
            "AWS_ACCESS_KEY_ID"
                | "AWS_SECRET_ACCESS_KEY"
                | "AWS_SESSION_TOKEN"
                | "AZURE_CLIENT_SECRET"
                | "GOOGLE_APPLICATION_CREDENTIALS"
        )
}

fn mcp_child_env(
    mut inherited: HashMap<String, String>,
    explicit: &HashMap<String, String>,
) -> HashMap<String, String> {
    inherited.retain(|key, _| !is_sensitive_inherited_env_key(key));
    inherited.extend(explicit.clone());
    inherited
}

impl Drop for McpClient {
    fn drop(&mut self) {
        if let Some(child) = self.child.as_mut() {
            let _ = child.start_kill();
        }
    }
}

#[cfg(all(test, unix))]
mod tests {
    use super::{McpClient, is_sensitive_inherited_env_key, mcp_child_env};
    use crate::mcp::protocol::McpServerConfig;
    use std::collections::HashMap;

    #[tokio::test]
    async fn http_sse_reader_rejects_oversized_event() {
        use std::{io::Write, net::TcpListener, thread};
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();
        let server = thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            use std::io::Read;
            let mut request = [0; 4096];
            let _ = stream.read(&mut request);
            let body = format!("data: {}\n\n", "x".repeat(1024 * 1024 + 1));
            write!(stream, "HTTP/1.1 200 OK\r\nContent-Type: text/event-stream\r\nContent-Length: {}\r\nConnection: close\r\n\r\n", body.len()).unwrap();
            stream.write_all(body.as_bytes()).unwrap();
        });
        let response = reqwest::Client::new()
            .get(format!("http://{addr}"))
            .send()
            .await
            .unwrap();
        assert!(
            super::read_http_jsonrpc_response(response, 1)
                .await
                .is_err()
        );
        server.join().unwrap();
    }

    #[test]
    fn http_response_parser_correlates_json_and_multiline_sse() {
        let json = r#"{"jsonrpc":"2.0","id":5,"result":{}}"#;
        assert!(super::parse_http_jsonrpc_response(json, 4).is_err());
        assert!(super::parse_http_jsonrpc_response("not-json", 5).is_err());
        let sse = "event: message\ndata: {\"jsonrpc\":\"2.0\",\ndata: \"id\":5,\"result\":{}}\n\n";
        assert_eq!(
            super::parse_http_jsonrpc_response(sse, 5).unwrap().id,
            Some(5)
        );
    }

    #[tokio::test]
    async fn streamable_http_initializes_lists_and_propagates_session_and_headers() {
        use std::{
            io::{Read, Write},
            net::TcpListener,
            thread,
        };
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();
        let server = thread::spawn(move || {
            for (content_type, response) in [
                (
                    "application/json",
                    r#"{"jsonrpc":"2.0","id":1,"result":{"protocolVersion":"2025-03-26","capabilities":{},"serverInfo":{"name":"mock","version":"1"}}}"#,
                ),
                ("application/json", ""),
                (
                    "text/event-stream",
                    "data: {\"jsonrpc\":\"2.0\",\"id\":\ndata: 2,\"result\":{\"tools\":[]}}\n\n",
                ),
                ("delete", ""),
            ] {
                let (mut stream, _) = listener.accept().unwrap();
                let mut bytes = [0; 8192];
                let n = stream.read(&mut bytes).unwrap();
                let request = String::from_utf8_lossy(&bytes[..n]);
                assert!(request.to_ascii_lowercase().contains("x-api-key: test"));
                if content_type == "delete" {
                    assert!(request.starts_with("DELETE "));
                    assert!(request.contains("mcp-session-id: sess-1"));
                    assert!(request.contains("mcp-protocol-version: 2025-03-26"));
                    stream
                        .write_all(b"HTTP/1.1 405 Method Not Allowed\r\nContent-Length: 0\r\nConnection: close\r\n\r\n")
                        .unwrap();
                    continue;
                }
                if content_type == "text/event-stream" {
                    assert!(request.contains("mcp-session-id: sess-1"));
                    assert!(request.contains("mcp-protocol-version: 2025-03-26"));
                }
                let body = response.as_bytes();
                let notification = request.contains("notifications/initialized");
                assert_eq!(notification, response.is_empty());
                let status = if notification {
                    "202 Accepted"
                } else {
                    "200 OK"
                };
                let response_body = if notification { "".as_bytes() } else { body };
                write!(
                    stream,
                    "HTTP/1.1 {}\r\nContent-Type: {}\r\nContent-Length: {}\r\n",
                    status,
                    content_type,
                    response_body.len()
                )
                .unwrap();
                if content_type == "application/json" {
                    write!(stream, "Mcp-Session-Id: sess-1\r\n").unwrap();
                    write!(stream, "Mcp-Protocol-Version: 2025-03-26\r\n").unwrap();
                }
                write!(stream, "Connection: close\r\n\r\n").unwrap();
                stream.write_all(response_body).unwrap();
                if content_type == "text/event-stream" {
                    stream.flush().unwrap();
                    thread::sleep(std::time::Duration::from_millis(300));
                }
            }
        });
        let config = McpServerConfig {
            command: String::new(),
            args: vec![],
            env: HashMap::new(),
            shared: true,
            transport: Some("http".into()),
            url: Some(format!("http://{addr}")),
            headers: HashMap::from([("x-api-key".into(), "test".into())]),
            enabled: None,
            disabled: None,
            timeout_secs: Some(3),
        };
        let client = McpClient::connect("mock".into(), &config).await.unwrap();
        assert!(client.handle().tools().is_empty());
        let mut client = client;
        client.shutdown().await;
        server.join().unwrap();
    }

    #[test]
    fn stdio_request_timeout_secs_remains_configured() {
        let mut config = fake_server_config();
        config.timeout_secs = Some(7);
        assert_eq!(
            super::request_timeout_for(&config),
            std::time::Duration::from_secs(7)
        );
    }

    #[tokio::test]
    async fn legacy_sse_transport_is_rejected_instead_of_treated_as_http_or_stdio() {
        let mut config = fake_server_config();
        config.transport = Some("sse".into());
        let err = match McpClient::connect("legacy-sse".into(), &config).await {
            Ok(_) => panic!("legacy SSE must not connect as Streamable HTTP"),
            Err(err) => err,
        };
        assert!(
            err.to_string()
                .contains("legacy SSE is not Streamable HTTP")
        );
    }

    #[tokio::test]
    async fn streamable_http_does_not_follow_redirects() {
        use std::{
            io::{Read, Write},
            net::TcpListener,
            thread,
        };
        let target = TcpListener::bind("127.0.0.1:0").unwrap();
        let target_addr = target.local_addr().unwrap();
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();
        let server = thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            let mut request = [0; 4096];
            let _ = stream.read(&mut request);
            write!(stream, "HTTP/1.1 302 Found\r\nLocation: http://{target_addr}/steal\r\nContent-Length: 0\r\nConnection: close\r\n\r\n").unwrap();
            stream.flush().unwrap();
            assert!(target.set_nonblocking(true).is_ok());
            assert!(
                target.accept().is_err(),
                "redirect target must not receive forwarded credentials"
            );
        });
        let handle = super::McpHandle {
            name: "redirect".into(),
            request_id: std::sync::Arc::new(std::sync::atomic::AtomicU64::new(1)),
            transport: std::sync::Arc::new(super::McpTransport::Http {
                client: reqwest::Client::builder()
                    .redirect(reqwest::redirect::Policy::none())
                    .build()
                    .unwrap(),
                url: format!("http://{addr}"),
                headers: HashMap::from([("authorization".into(), "Bearer secret".into())]),
                session_id: std::sync::Arc::new(tokio::sync::Mutex::new(None)),
                protocol_version: std::sync::Arc::new(tokio::sync::Mutex::new(None)),
            }),
            server_info: std::sync::Arc::new(std::sync::RwLock::new(None)),
            capabilities: std::sync::Arc::new(std::sync::RwLock::new(
                crate::mcp::protocol::ServerCapabilities::default(),
            )),
            tools: std::sync::Arc::new(std::sync::RwLock::new(Vec::new())),
            request_timeout: std::time::Duration::from_secs(1),
        };
        let err = handle.request("initialize", None).await.unwrap_err();
        assert!(err.to_string().contains("302"));
        server.join().unwrap();
    }

    #[tokio::test]
    async fn streamable_http_rejects_error_status() {
        use std::{
            io::{Read, Write},
            net::TcpListener,
            thread,
        };
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();
        let server = thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            let mut request = [0; 4096];
            let _ = stream.read(&mut request);
            stream.write_all(b"HTTP/1.1 503 Service Unavailable\r\nContent-Length: 0\r\nConnection: close\r\n\r\n").unwrap();
        });
        let config = McpServerConfig {
            command: String::new(),
            args: vec![],
            env: HashMap::new(),
            shared: true,
            transport: Some("http".into()),
            url: Some(format!("http://{addr}")),
            headers: HashMap::new(),
            enabled: None,
            disabled: None,
            timeout_secs: Some(1),
        };
        let err = match McpClient::connect("status-error".into(), &config).await {
            Ok(_) => panic!("503 response must fail initialization"),
            Err(err) => err,
        };
        assert!(format!("{err:#}").contains("503"));
        server.join().unwrap();
    }

    #[tokio::test]
    async fn streamable_http_rejects_malformed_and_wrong_id_responses() {
        use std::{
            io::{Read, Write},
            net::TcpListener,
            thread,
        };
        for response in [
            "HTTP/1.1 200 OK\\r\\nContent-Type: application/json\\r\\nContent-Length: 7\\r\\nConnection: close\\r\\n\\r\\nnotjson",
            "HTTP/1.1 200 OK\\r\\nContent-Type: application/json\\r\\nContent-Length: 34\\r\\nConnection: close\\r\\n\\r\\n{\\\"jsonrpc\\\":\\\"2.0\\\",\\\"id\\\":99,\\\"result\\\":{}}",
        ] {
            let listener = TcpListener::bind("127.0.0.1:0").unwrap();
            let addr = listener.local_addr().unwrap();
            let response = response.to_owned();
            let server = thread::spawn(move || {
                let (mut stream, _) = listener.accept().unwrap();
                let mut request = [0; 4096];
                let _ = stream.read(&mut request);
                stream.write_all(response.as_bytes()).unwrap();
            });
            let handle = super::McpHandle {
                name: "bad-response".into(),
                request_id: std::sync::Arc::new(std::sync::atomic::AtomicU64::new(1)),
                transport: std::sync::Arc::new(super::McpTransport::Http {
                    client: reqwest::Client::new(),
                    url: format!("http://{addr}"),
                    headers: HashMap::new(),
                    session_id: std::sync::Arc::new(tokio::sync::Mutex::new(None)),
                    protocol_version: std::sync::Arc::new(tokio::sync::Mutex::new(None)),
                }),
                server_info: std::sync::Arc::new(std::sync::RwLock::new(None)),
                capabilities: std::sync::Arc::new(std::sync::RwLock::new(
                    crate::mcp::protocol::ServerCapabilities::default(),
                )),
                tools: std::sync::Arc::new(std::sync::RwLock::new(Vec::new())),
                request_timeout: std::time::Duration::from_secs(1),
            };
            assert!(handle.request("initialize", None).await.is_err());
            server.join().unwrap();
        }
    }

    #[tokio::test]
    async fn streamable_http_timeout_covers_missing_response_headers() {
        use std::{io::Read, net::TcpListener, thread, time::Duration};
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();
        let server = thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            let mut request = [0; 4096];
            let _ = stream.read(&mut request);
            thread::sleep(Duration::from_millis(400));
        });
        let handle = super::McpHandle {
            name: "header-timeout".into(),
            request_id: std::sync::Arc::new(std::sync::atomic::AtomicU64::new(1)),
            transport: std::sync::Arc::new(super::McpTransport::Http {
                client: reqwest::Client::new(),
                url: format!("http://{addr}"),
                headers: HashMap::new(),
                session_id: std::sync::Arc::new(tokio::sync::Mutex::new(None)),
                protocol_version: std::sync::Arc::new(tokio::sync::Mutex::new(None)),
            }),
            server_info: std::sync::Arc::new(std::sync::RwLock::new(None)),
            capabilities: std::sync::Arc::new(std::sync::RwLock::new(
                crate::mcp::protocol::ServerCapabilities::default(),
            )),
            tools: std::sync::Arc::new(std::sync::RwLock::new(Vec::new())),
            request_timeout: Duration::from_millis(100),
        };
        assert!(
            handle
                .request("initialize", None)
                .await
                .unwrap_err()
                .to_string()
                .contains("timeout")
        );
        server.join().unwrap();
    }

    #[tokio::test]
    async fn streamable_http_timeout_covers_response_body_wait() {
        use std::{io::Read, net::TcpListener, thread, time::Duration};
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();
        let server = thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            let mut request = [0; 4096];
            let _ = stream.read(&mut request);
            thread::sleep(Duration::from_millis(500));
        });
        let handle = super::McpHandle {
            name: "timeout".into(),
            request_id: std::sync::Arc::new(std::sync::atomic::AtomicU64::new(1)),
            transport: std::sync::Arc::new(super::McpTransport::Http {
                client: reqwest::Client::new(),
                url: format!("http://{addr}"),
                headers: HashMap::new(),
                session_id: std::sync::Arc::new(tokio::sync::Mutex::new(None)),
                protocol_version: std::sync::Arc::new(tokio::sync::Mutex::new(None)),
            }),
            server_info: std::sync::Arc::new(std::sync::RwLock::new(None)),
            capabilities: std::sync::Arc::new(std::sync::RwLock::new(
                crate::mcp::protocol::ServerCapabilities::default(),
            )),
            tools: std::sync::Arc::new(std::sync::RwLock::new(Vec::new())),
            request_timeout: Duration::from_millis(100),
        };
        assert!(
            handle
                .request("initialize", None)
                .await
                .unwrap_err()
                .to_string()
                .contains("timeout")
        );
        server.join().unwrap();
    }

    #[test]
    fn inherited_mcp_env_scrubs_provider_credentials() {
        for key in [
            "ANTHROPIC_API_KEY",
            "openai_api_key",
            "CURSOR_ACCESS_TOKEN",
            "AWS_SECRET_ACCESS_KEY",
            "AWS_SESSION_TOKEN",
            "GOOGLE_APPLICATION_CREDENTIALS",
        ] {
            assert!(is_sensitive_inherited_env_key(key), "must scrub {key}");
        }
        for key in ["PATH", "HOME", "RUST_LOG", "JCODE_OPENROUTER_API_KEY_NAME"] {
            assert!(!is_sensitive_inherited_env_key(key), "must preserve {key}");
        }
    }

    #[test]
    fn explicit_mcp_env_can_opt_a_credential_back_in() {
        let inherited = HashMap::from([
            ("PATH".to_string(), "/bin".to_string()),
            ("ANTHROPIC_API_KEY".to_string(), "daemon-secret".to_string()),
        ]);
        let explicit = HashMap::from([(
            "ANTHROPIC_API_KEY".to_string(),
            "server-specific-secret".to_string(),
        )]);

        let env = mcp_child_env(inherited, &explicit);
        assert_eq!(env.get("PATH").map(String::as_str), Some("/bin"));
        assert_eq!(
            env.get("ANTHROPIC_API_KEY").map(String::as_str),
            Some("server-specific-secret")
        );
    }

    /// A minimal fake stdio MCP server (shell script) that reports its own
    /// process cwd as the serverInfo name.
    fn fake_server_config() -> McpServerConfig {
        let script = r#"
while IFS= read -r line; do
  case "$line" in
    *'"initialize"'*)
      printf '{"jsonrpc":"2.0","id":1,"result":{"protocolVersion":"2024-11-05","capabilities":{},"serverInfo":{"name":"%s","version":"0"}}}\n' "$PWD"
      ;;
    *'"tools/list"'*)
      printf '{"jsonrpc":"2.0","id":2,"result":{"tools":[]}}\n'
      ;;
  esac
done
"#;
        McpServerConfig {
            command: "/bin/sh".to_string(),
            args: vec!["-c".to_string(), script.to_string()],
            env: Default::default(),
            shared: false,
            transport: None,
            url: None,
            headers: std::collections::HashMap::new(),
            enabled: None,
            disabled: None,
            timeout_secs: None,
        }
    }

    #[tokio::test]
    async fn connect_in_dir_sets_subprocess_cwd() {
        // Issue #557: owned MCP servers must run in the session project dir.
        let dir = tempfile::tempdir().expect("tempdir");
        let expected = dir.path().canonicalize().expect("canonicalize");

        let client = McpClient::connect_in_dir(
            "cwd-test".to_string(),
            &fake_server_config(),
            Some(dir.path()),
        )
        .await
        .expect("connect");

        let reported = client.server_info().expect("server info").name;
        assert_eq!(
            std::path::Path::new(&reported)
                .canonicalize()
                .expect("canonicalize reported"),
            expected
        );
    }

    #[tokio::test]
    async fn connect_in_dir_missing_dir_falls_back_to_inherited_cwd() {
        let client = McpClient::connect_in_dir(
            "cwd-fallback-test".to_string(),
            &fake_server_config(),
            Some(std::path::Path::new("/nonexistent/jcode-557")),
        )
        .await
        .expect("connect should fall back to inherited cwd");

        let reported = client.server_info().expect("server info").name;
        assert!(!reported.is_empty());
    }
}

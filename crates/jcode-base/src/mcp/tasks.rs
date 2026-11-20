//! MCP Tasks extension (`io.modelcontextprotocol/tasks`, spec 2026-07-28).
//!
//! A server that negotiated the extension may answer `tools/call` with a task
//! handle (`resultType: "task"`) instead of a result. The client then polls
//! `tasks/get` and may send `tasks/cancel`. This module holds the shared types
//! and the two seams the rest of jcode builds on:
//!
//! * [`TaskRemote`]: ask one server about one task.
//! * [`TaskSource`]: resolve a configured server name to a [`TaskRemote`] plus
//!   an identity string, so a saved task is never polled against a different
//!   server or authorization context than the one that created it.
//!
//! The 2025-11-25 core Tasks (`tasks/result`, `capabilities.tasks`,
//! `notifications/tasks/status`) are a different, incompatible wire contract
//! and are not implemented.

use super::protocol::ToolCallResult;
use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::Arc;

/// Extension identifier, used in client and server capabilities.
pub const TASKS_EXTENSION: &str = "io.modelcontextprotocol/tasks";

/// Remote task state as reported by the server.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RemoteStatus {
    Working,
    InputRequired,
    Completed,
    Failed,
    Cancelled,
}

impl RemoteStatus {
    pub fn parse(text: &str) -> Option<Self> {
        Some(match text {
            "working" => Self::Working,
            "input_required" => Self::InputRequired,
            "completed" => Self::Completed,
            "failed" => Self::Failed,
            "cancelled" => Self::Cancelled,
            _ => return None,
        })
    }

    pub const fn is_terminal(self) -> bool {
        matches!(self, Self::Completed | Self::Failed | Self::Cancelled)
    }

    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Working => "working",
            Self::InputRequired => "input_required",
            Self::Completed => "completed",
            Self::Failed => "failed",
            Self::Cancelled => "cancelled",
        }
    }
}

/// One snapshot of a remote task (`CreateTaskResult` or a `tasks/get` result).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RemoteTask {
    pub task_id: String,
    pub status: RemoteStatus,
    pub status_message: Option<String>,
    /// ISO 8601, as sent by the server.
    pub created_at: String,
    pub last_updated_at: String,
    /// `None` means the server keeps the task without a time limit.
    pub ttl_ms: Option<u64>,
    pub poll_interval_ms: Option<u64>,
    /// Outstanding server-to-client requests when `input_required`. Untrusted
    /// data, shown but never answered automatically.
    pub input_requests: Option<Value>,
    /// Inlined `CallToolResult` when `completed`.
    pub result: Option<Value>,
    /// Inlined JSON-RPC error when `failed`.
    pub error: Option<Value>,
}

const MAX_INLINE_BYTES: usize = 256 * 1024;

fn bounded(value: Option<&Value>) -> Option<Value> {
    let value = value?;
    // Inlined result, error and input requests are server data of unbounded
    // size. Keep what fits and say so, rather than persisting megabytes.
    if value.to_string().len() > MAX_INLINE_BYTES {
        return Some(
            serde_json::json!({"truncated": true, "reason": "inlined payload exceeds limit"}),
        );
    }
    Some(value.clone())
}

/// Parse a `CreateTaskResult` or a `tasks/get` result. Strict about the
/// fields jcode relies on, tolerant of everything else.
pub fn parse_task(value: &Value) -> Result<RemoteTask> {
    use anyhow::Context as _;
    let text = |key: &str| value.get(key).and_then(Value::as_str);
    let task_id = text("taskId").context("missing taskId")?;
    anyhow::ensure!(
        !task_id.is_empty() && task_id.len() <= 512 && !task_id.chars().any(char::is_control),
        "invalid taskId"
    );
    let status = RemoteStatus::parse(text("status").context("missing status")?)
        .context("unknown task status")?;
    let created_at = text("createdAt").context("missing createdAt")?.to_string();
    let last_updated_at = text("lastUpdatedAt")
        .context("missing lastUpdatedAt")?
        .to_string();
    let ttl_ms = match value.get("ttlMs") {
        None | Some(Value::Null) => None,
        Some(v) => Some(
            v.as_u64()
                .context("ttlMs must be a non-negative integer or null")?,
        ),
    };
    let poll_interval_ms = match value.get("pollIntervalMs") {
        None | Some(Value::Null) => None,
        Some(v) => Some(
            v.as_u64()
                .context("pollIntervalMs must be a non-negative integer")?,
        ),
    };
    Ok(RemoteTask {
        task_id: task_id.to_string(),
        status,
        status_message: text("statusMessage").map(|m| m.chars().take(1000).collect()),
        created_at,
        last_updated_at,
        ttl_ms,
        poll_interval_ms,
        input_requests: bounded(value.get("inputRequests")),
        result: bounded(value.get("result")),
        error: bounded(value.get("error")),
    })
}

/// What `tools/call` produced when task handles are accepted.
#[derive(Debug, Clone)]
pub enum CallOutcome {
    Complete(ToolCallResult),
    Task(RemoteTask),
}

/// Stable, secret-free identity of a configured server for saved tasks.
///
/// Covers everything that decides WHICH server and WHICH authorization
/// context a task id belongs to: name, transport, endpoint, launch command and
/// arguments, and the names (never the values) of env variables and headers.
/// A static credential is represented only by a one-way SHA-256 digest of its
/// value, so rotating it reads as a different authorization context while the
/// file holds nothing reusable. OAuth tokens rotate on refresh and are not
/// part of it; the OAuth endpoint is. Userinfo in a URL is dropped.
pub fn server_identity(name: &str, config: &super::protocol::McpServerConfig) -> String {
    use sha2::{Digest, Sha256};
    use std::collections::BTreeMap;
    let mut h = Sha256::new();
    let mut field = |label: &str, value: &str| {
        h.update(label.as_bytes());
        h.update([0x1f]);
        h.update((value.len() as u64).to_le_bytes());
        h.update(value.as_bytes());
        h.update([0x1e]);
    };
    field("name", name);
    field("transport", config.transport.as_deref().unwrap_or(""));
    let url = config
        .url
        .as_deref()
        .and_then(|u| reqwest::Url::parse(u).ok())
        .map(|mut u| {
            let _ = u.set_username("");
            let _ = u.set_password(None);
            u.set_fragment(None);
            u.to_string()
        })
        .unwrap_or_default();
    field("url", &url);
    field("command", &config.command);
    for arg in &config.args {
        field("arg", arg);
    }
    let env: BTreeMap<_, _> = config.env.iter().collect();
    for (key, value) in env {
        field("env", key);
        field("envdigest", &hex::encode(Sha256::digest(value.as_bytes())));
    }
    let headers: BTreeMap<_, _> = config
        .headers
        .iter()
        .map(|(k, v)| (k.to_ascii_lowercase(), v))
        .collect();
    for (key, value) in headers {
        field("header", &key);
        field(
            "headerdigest",
            &hex::encode(Sha256::digest(value.as_bytes())),
        );
    }
    hex::encode(h.finalize())
}

/// Result of asking a server for a task.
#[derive(Debug, Clone, PartialEq)]
#[expect(
    clippy::large_enum_variant,
    reason = "a poll result is short-lived and passed by value once; boxing adds churn at every call site"
)]
pub enum TaskPoll {
    Task(RemoteTask),
    /// `-32602` (unknown or expired task id). Not a task failure.
    NotFound(String),
}

/// One server, one task API. Errors are transport or protocol failures (the
/// caller keeps the last known state and retries with backoff).
#[async_trait]
pub trait TaskRemote: Send + Sync {
    async fn get_task(&self, task_id: &str) -> Result<TaskPoll>;
    /// `tasks/cancel`. Only an acknowledgement: the task may still finish.
    async fn cancel_task(&self, task_id: &str) -> Result<()>;
}

/// A server resolved for task traffic.
#[derive(Clone)]
pub struct ResolvedTaskServer {
    /// Fingerprint of the server's endpoint and authorization context, as
    /// stored with the task at creation. A mismatch means "different server".
    pub identity: String,
    pub remote: Arc<dyn TaskRemote>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResolveError {
    /// Not configured (any more) or out of workspace scope.
    Unknown,
    /// Configured but unreachable right now. Retry later.
    Unavailable(String),
}

/// Receives `notifications/tasks` as wake hints and tells the listen stream
/// which task ids to subscribe to. Implemented by the durable tracker.
///
/// A hint is never state: the implementation answers it with `tasks/get`. It
/// must not block and must coalesce repeated hints for one task.
pub trait TaskHintSink: Send + Sync {
    /// Ids of nonterminal tasks tracked for `server` (sent as `taskIds`).
    fn tracked_task_ids(&self, server: &str) -> Vec<String>;
    /// A validated `notifications/tasks` for a task id the stream subscribed to.
    fn hint(&self, server: &str, task_id: &str);
    /// Fires when the set of tracked ids changed, so listeners reopen with
    /// the new `taskIds`.
    fn changed(&self) -> Arc<tokio::sync::Notify>;
}

static TASK_HINT_SINK: std::sync::RwLock<Option<Arc<dyn TaskHintSink>>> =
    std::sync::RwLock::new(None);

/// Install (or clear) the process-wide task hint sink. Without one, task
/// notifications are not subscribed to and polling is the only path.
pub fn set_task_hint_sink(sink: Option<Arc<dyn TaskHintSink>>) {
    *TASK_HINT_SINK.write().unwrap_or_else(|p| p.into_inner()) = sink;
}

pub(super) fn task_hint_sink() -> Option<Arc<dyn TaskHintSink>> {
    TASK_HINT_SINK
        .read()
        .unwrap_or_else(|p| p.into_inner())
        .clone()
}

/// Resolves a configured server name for task polling and cancellation.
#[async_trait]
pub trait TaskSource: Send + Sync {
    async fn resolve(&self, server: &str) -> std::result::Result<ResolvedTaskServer, ResolveError>;
}

#[cfg(test)]
#[path = "tasks_tests.rs"]
mod tasks_tests;
#[cfg(test)]
#[path = "tasks_wire_tests.rs"]
mod tasks_wire_tests;

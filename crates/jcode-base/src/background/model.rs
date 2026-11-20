use crate::bus::{BackgroundTaskProgress, BackgroundTaskProgressEvent, BackgroundTaskStatus};
use anyhow::Result;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::time::Instant;
use tokio::sync::watch;
use tokio::task::JoinHandle;

/// Own an adopted task rather than detaching it when its wrapper is aborted.
/// Construct before the first await, including before spawning the wrapper, so
/// cancellation during registration or before its first poll is also safe.
pub(super) struct AbortAdoptedTask<T>(pub(super) JoinHandle<T>);

impl<T> Drop for AbortAdoptedTask<T> {
    fn drop(&mut self) {
        self.0.abort();
    }
}

/// Directory for background task output files
pub(super) fn task_dir() -> PathBuf {
    std::env::temp_dir().join("jcode-bg-tasks")
}

pub(super) const EXIT_MARKER_PREFIX: &str = "--- Command finished with exit code: ";
const MAX_EVENT_HISTORY: usize = 50;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum BackgroundTaskEventKind {
    Progress,
    Checkpoint,
    Completed,
    Failed,
    Superseded,
    Cancelled,
    DeliveryUpdated,
    Lost,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BackgroundTaskEventRecord {
    pub kind: BackgroundTaskEventKind,
    pub timestamp: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub status: Option<BackgroundTaskStatus>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub exit_code: Option<i32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub progress: Option<BackgroundTaskProgress>,
}

/// Status file format (written to disk)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskStatusFile {
    pub task_id: String,
    pub tool_name: String,
    #[serde(default)]
    pub display_name: Option<String>,
    pub session_id: String,
    pub status: BackgroundTaskStatus,
    pub exit_code: Option<i32>,
    pub error: Option<String>,
    pub started_at: String,
    pub completed_at: Option<String>,
    pub duration_secs: Option<f64>,
    #[serde(default)]
    pub pid: Option<u32>,
    /// Start time of `pid` (clock ticks since boot, Linux only) captured when
    /// the process was registered. A matching pid with a different start time
    /// is a reused pid, never the original task. `None` for files written by
    /// older builds or where the platform cannot report it; identity then
    /// falls back to plain liveness.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pid_start_time: Option<u64>,
    /// Execution generation of the owning session when the task was created
    /// (see `server::session_control`). Completion wakes are suppressed once
    /// that generation is revoked by a stop. `None` for legacy files and tasks
    /// created outside a controlled turn: those are not generation-bound.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub owner_generation: Option<u64>,
    /// PID of the process whose in-process future owns (or owned) this task.
    /// `None` for files written by older builds. Used to reconcile phantom
    /// `Running` entries after the owning server crashes or exec-reloads;
    /// files without owner metadata are deliberately never reconciled, so a
    /// task genuinely running in another live process cannot be clobbered.
    #[serde(default)]
    pub owner_pid: Option<u32>,
    /// Per-process random instance token of the owning process. Exec-based
    /// server reloads keep the PID, so PID alone cannot distinguish "this
    /// process, task still bootstrapping" from "same PID after exec, task
    /// future is gone". A fresh token per process image resolves that.
    #[serde(default)]
    pub owner_instance: Option<String>,
    #[serde(default)]
    pub detached: bool,
    #[serde(default = "default_true")]
    pub notify: bool,
    #[serde(default)]
    pub wake: bool,
    #[serde(default)]
    pub progress: Option<BackgroundTaskProgress>,
    #[serde(default)]
    pub event_history: Vec<BackgroundTaskEventRecord>,
    /// Stall watchdog window in seconds, when armed: the agent is woken after
    /// this long with no new output bytes and no progress events.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stall_wake_seconds: Option<u64>,
    /// Present when this record tracks a remote MCP task (Tasks extension,
    /// 2026-07-28). The record stays `Running` while the remote task is
    /// nonterminal; it never carries credentials, only a server name and an
    /// opaque identity fingerprint.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mcp_task: Option<McpTaskRecord>,
}

/// Local condition of an MCP task, independent of the remote status.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "state", content = "reason", rename_all = "snake_case")]
pub enum McpRemoteState {
    /// The remote task is being tracked and the server answered last time.
    Tracking,
    /// Last poll failed in transport; the last known state is kept.
    RemoteUnavailable(String),
    /// The saved server is unknown or now resolves to a different identity.
    /// Automatic polling stopped and nothing was sent to any server.
    Unresolved(String),
    /// The task TTL elapsed locally before a terminal state was retrieved.
    Expired,
    /// The server no longer knows the task id (unknown or expired remotely).
    NotFound,
}

/// Durable state of one MCP task, stored inside the background-task record.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct McpTaskRecord {
    pub server: String,
    /// Opaque fingerprint of the server endpoint and authorization context.
    pub identity: String,
    pub remote_task_id: String,
    pub tool: String,
    pub tool_call_id: String,
    pub remote_status: crate::mcp::tasks::RemoteStatus,
    #[serde(default)]
    pub status_message: Option<String>,
    pub created_at: String,
    pub last_updated_at: String,
    #[serde(default)]
    pub ttl_ms: Option<u64>,
    #[serde(default)]
    pub poll_interval_ms: Option<u64>,
    /// Local expiry: `created_at + ttl_ms`. `None` when the server set no TTL.
    #[serde(default)]
    pub expires_at: Option<String>,
    #[serde(default)]
    pub cancel_requested: bool,
    /// Set when the last `tasks/cancel` request failed or its reply was lost.
    #[serde(default)]
    pub cancel_error: Option<String>,
    /// Inlined result (bounded; a string with `result_truncated` when cut).
    #[serde(default)]
    pub result: Option<serde_json::Value>,
    #[serde(default)]
    pub result_truncated: bool,
    /// Inlined JSON-RPC error of a failed task (bounded).
    #[serde(default)]
    pub error: Option<serde_json::Value>,
    /// Outstanding input requests (bounded, untrusted, never answered).
    #[serde(default)]
    pub input_requests: Option<serde_json::Value>,
    pub remote_state: McpRemoteState,
}

/// How a finished MCP task is recorded in the background-task status.
#[derive(Debug, Clone, PartialEq)]
pub struct McpTaskFinish {
    pub status: BackgroundTaskStatus,
    pub error: Option<String>,
    /// Text for the task output file (also the notification preview).
    pub output: String,
}

/// Cap for inlined result or error text kept in a status file.
pub const MCP_TASK_RESULT_CAP: usize = 64 * 1024;
/// Cap for the stored `input_requests` summary.
pub const MCP_TASK_INPUT_CAP: usize = 8 * 1024;
/// Longest server error text copied into the task status and notifications.
pub const MCP_TASK_ERROR_CELLS: usize = 300;

/// Bound a JSON value: kept as is when its compact form fits, otherwise
/// replaced by the cut compact text. Returns `(value, truncated)`.
pub fn bound_json(value: &serde_json::Value, cap: usize) -> (serde_json::Value, bool) {
    let text = value.to_string();
    if text.len() <= cap {
        return (value.clone(), false);
    }
    (
        serde_json::Value::String(crate::util::truncate_str(&text, cap).to_string()),
        true,
    )
}

/// Single-line, control-free copy of untrusted text.
/// Unicode bidirectional override and isolate controls, which can reorder
/// displayed text.
fn is_bidi_control(c: char) -> bool {
    matches!(c, '\u{202A}'..='\u{202E}' | '\u{2066}'..='\u{2069}' | '\u{200E}' | '\u{200F}')
}

fn one_line(text: &str) -> String {
    text.chars()
        .filter(|c| !is_bidi_control(*c))
        .map(|c| if c.is_control() { ' ' } else { c })
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

/// Display width of `text` in terminal cells.
pub fn cell_width(text: &str) -> usize {
    use jcode_render_core::{ColumnWidth, WidthMeasure};
    ColumnWidth.measure(text)
}

/// Truncate to at most `max_cells` terminal cells, ending in `…` when cut.
/// Counts Unicode display cells, not chars, so UI rows never wrap.
pub fn truncate_cells(text: &str, max_cells: usize) -> String {
    use jcode_render_core::{ColumnWidth, WidthMeasure};
    let measure = ColumnWidth;
    if measure.measure(text) <= max_cells {
        return text.to_string();
    }
    if max_cells == 0 {
        return String::new();
    }
    let budget = max_cells - 1;
    let mut out = String::new();
    let mut used = 0;
    for ch in text.chars() {
        let w = measure.measure_char(ch);
        if used + w > budget {
            break;
        }
        used += w;
        out.push(ch);
    }
    out.push('…');
    out
}

/// Readable text of a `CallToolResult`, falling back to its JSON.
fn result_text(result: &serde_json::Value) -> String {
    let texts: Vec<&str> = result
        .get("content")
        .and_then(|content| content.as_array())
        .map(|items| {
            items
                .iter()
                .filter_map(|item| item.get("text").and_then(|text| text.as_str()))
                .collect()
        })
        .unwrap_or_default();
    if texts.is_empty() {
        match result {
            serde_json::Value::String(text) => text.clone(),
            other => other.to_string(),
        }
    } else {
        texts.join("\n")
    }
}

impl McpTaskRecord {
    /// The background-task outcome implied by this record, or `None` while the
    /// remote task is still being tracked. A remote terminal status always
    /// wins. Expiry and not-found end tracking as `Lost` (outcome unknown,
    /// never a success and never a failure).
    pub fn finish(&self) -> Option<McpTaskFinish> {
        use crate::mcp::tasks::RemoteStatus as S;
        match self.remote_status {
            S::Completed => Some(McpTaskFinish {
                status: BackgroundTaskStatus::Completed,
                error: None,
                output: self.result.as_ref().map(result_text).unwrap_or_default(),
            }),
            S::Failed => {
                let message = self
                    .error
                    .as_ref()
                    .and_then(|error| error.get("message"))
                    .and_then(|message| message.as_str())
                    .map(one_line)
                    .or_else(|| {
                        self.error
                            .as_ref()
                            .map(|error| one_line(&error.to_string()))
                    })
                    .map(|text| truncate_cells(&text, MCP_TASK_ERROR_CELLS))
                    .unwrap_or_else(|| "MCP task failed".to_string());
                Some(McpTaskFinish {
                    status: BackgroundTaskStatus::Failed,
                    output: format!("MCP task failed: {message}"),
                    error: Some(message),
                })
            }
            S::Cancelled => Some(McpTaskFinish {
                status: BackgroundTaskStatus::Failed,
                error: Some("Cancelled by remote".to_string()),
                output: "MCP task cancelled by the server".to_string(),
            }),
            S::Working | S::InputRequired => {
                let reason = match self.remote_state {
                    McpRemoteState::Expired => "MCP task expired before its result was retrieved",
                    McpRemoteState::NotFound => {
                        "MCP task is unknown to the server (expired or the server restarted)"
                    }
                    _ => return None,
                };
                let text = format!(
                    "{reason} (last remote status: {})",
                    self.remote_status.as_str()
                );
                Some(McpTaskFinish {
                    status: BackgroundTaskStatus::Lost,
                    error: Some(text.clone()),
                    output: text,
                })
            }
        }
    }

    /// Task kind shown in listings: `mcp:<server>:<tool>`.
    pub fn kind_label(&self) -> String {
        let clean = |text: &str| -> String {
            one_line(text)
                .chars()
                .filter(|c| *c != '`' && !is_bidi_control(*c))
                .collect()
        };
        format!("mcp:{}:{}", clean(&self.server), clean(&self.tool))
    }

    /// Compact remote-state label. Distinguishes cancellation requested from
    /// confirmed, and never claims cancellation worked when the server
    /// reported another terminal state. Single line, untrusted text bounded.
    pub fn state_label(&self) -> String {
        use crate::mcp::tasks::RemoteStatus as S;
        let status = self.remote_status.as_str();
        match &self.remote_state {
            McpRemoteState::Unresolved(reason) => {
                format!("unresolved: {}", truncate_cells(&one_line(reason), 60))
            }
            McpRemoteState::Expired => format!("expired (last {status})"),
            McpRemoteState::NotFound => format!("expired/not found (last {status})"),
            state => {
                let unavailable = match state {
                    McpRemoteState::RemoteUnavailable(reason) => {
                        format!(
                            ", remote unavailable: {}",
                            truncate_cells(&one_line(reason), 40)
                        )
                    }
                    _ => String::new(),
                };
                match (self.remote_status, self.cancel_requested) {
                    (S::Cancelled, true) => "cancelled (confirmed)".to_string(),
                    (S::Completed | S::Failed, true) => {
                        format!("{status} (cancellation requested, server finished first)")
                    }
                    (_, true) if self.cancel_error.is_some() => {
                        format!("{status}, cancellation requested (unacknowledged){unavailable}")
                    }
                    (_, true) => format!("{status}, cancellation requested{unavailable}"),
                    _ => format!("{status}{unavailable}"),
                }
            }
        }
    }

    /// Short state for table cells (at most 12 cells), e.g. `cancel-req`.
    pub fn short_state(&self) -> &'static str {
        use crate::mcp::tasks::RemoteStatus as S;
        match &self.remote_state {
            McpRemoteState::Unresolved(_) => "unresolved",
            McpRemoteState::Expired => "expired",
            McpRemoteState::NotFound => "not-found",
            McpRemoteState::RemoteUnavailable(_) => "unavailable",
            McpRemoteState::Tracking => match (self.remote_status, self.cancel_requested) {
                (S::Cancelled, _) => "cancelled",
                (S::Completed, _) => "completed",
                (S::Failed, _) => "failed",
                (_, true) => "cancel-req",
                (S::InputRequired, false) => "input-req",
                (S::Working, false) => "working",
            },
        }
    }

    /// Detail lines for task status output. Each line is single-line text.
    pub fn detail_lines(&self) -> Vec<String> {
        let mut lines = vec![
            format!("MCP task: {}", self.kind_label()),
            format!("Remote task id: {}", one_line(&self.remote_task_id)),
            format!("Remote state: {}", self.state_label()),
        ];
        if let Some(message) = self.status_message.as_deref().filter(|m| !m.is_empty()) {
            lines.push(format!(
                "Remote message: {}",
                truncate_cells(&one_line(message), 200)
            ));
        }
        if let Some(expires) = self.expires_at.as_deref() {
            lines.push(format!("Expires: {expires}"));
        }
        if let Some(error) = self.cancel_error.as_deref() {
            lines.push(format!(
                "Cancel request error: {}",
                truncate_cells(&one_line(error), 200)
            ));
        }
        if let Some(input) = self.input_requests.as_ref() {
            lines.push(format!(
                "Input required (untrusted, not answered): {}",
                truncate_cells(&one_line(&input.to_string()), 300)
            ));
        }
        if let Some(result) = self.result.as_ref() {
            let suffix = if self.result_truncated {
                " (truncated)"
            } else {
                ""
            };
            lines.push(format!(
                "Result{suffix}: {}",
                truncate_cells(&one_line(&result.to_string()), 300)
            ));
        }
        if let Some(error) = self.error.as_ref() {
            lines.push(format!(
                "Remote error: {}",
                truncate_cells(&one_line(&error.to_string()), 300)
            ));
        }
        lines
    }
}

fn default_true() -> bool {
    true
}

/// Random per-process-image instance token for `owner_instance`. Generated
/// once per process image (exec-based reloads get a fresh token even though
/// the PID is unchanged), so status files can tell "this process, task still
/// alive" apart from "same PID after exec, task future is gone".
pub fn process_instance_token() -> &'static str {
    use std::sync::OnceLock;
    static TOKEN: OnceLock<String> = OnceLock::new();
    TOKEN.get_or_init(|| uuid::Uuid::new_v4().simple().to_string())
}

pub(super) fn normalize_delivery(notify: bool, wake: bool) -> (bool, bool) {
    (notify || wake, wake)
}

pub(super) fn push_task_event(status: &mut TaskStatusFile, event: BackgroundTaskEventRecord) {
    status.event_history.push(event);
    let overflow = status.event_history.len().saturating_sub(MAX_EVENT_HISTORY);
    if overflow > 0 {
        status.event_history.drain(0..overflow);
    }
}

pub(super) fn progress_event_record(
    kind: BackgroundTaskEventKind,
    progress: BackgroundTaskProgress,
) -> BackgroundTaskEventRecord {
    BackgroundTaskEventRecord {
        kind,
        timestamp: Utc::now().to_rfc3339(),
        message: progress.message.clone(),
        status: Some(BackgroundTaskStatus::Running),
        exit_code: None,
        progress: Some(progress),
    }
}

fn terminal_event_kind(
    status: &BackgroundTaskStatus,
    error: Option<&str>,
) -> BackgroundTaskEventKind {
    match status {
        BackgroundTaskStatus::Completed => BackgroundTaskEventKind::Completed,
        BackgroundTaskStatus::Superseded => BackgroundTaskEventKind::Superseded,
        BackgroundTaskStatus::Failed
            if matches!(error, Some("Cancelled by user" | "Cancelled by remote")) =>
        {
            BackgroundTaskEventKind::Cancelled
        }
        BackgroundTaskStatus::Failed => BackgroundTaskEventKind::Failed,
        BackgroundTaskStatus::Lost => BackgroundTaskEventKind::Lost,
        BackgroundTaskStatus::Running => BackgroundTaskEventKind::Progress,
    }
}

pub(super) fn terminal_event_record(
    status: BackgroundTaskStatus,
    exit_code: Option<i32>,
    error: Option<&str>,
) -> BackgroundTaskEventRecord {
    BackgroundTaskEventRecord {
        kind: terminal_event_kind(&status, error),
        timestamp: Utc::now().to_rfc3339(),
        message: error.map(ToString::to_string),
        status: Some(status),
        exit_code,
        progress: None,
    }
}

pub(super) fn progress_wait_reason(
    event: Option<&BackgroundTaskEventRecord>,
) -> BackgroundTaskWaitReason {
    match event.map(|event| &event.kind) {
        Some(BackgroundTaskEventKind::Checkpoint) => BackgroundTaskWaitReason::Checkpoint,
        _ => BackgroundTaskWaitReason::Progress,
    }
}

// Progress-display formatting now lives in `jcode-background-types` (pure
// functions over BackgroundTaskProgress); re-export for existing callers.
pub use jcode_background_types::{
    format_progress_display, format_progress_summary, render_progress_bar,
};

pub(super) fn progress_equivalent(a: &BackgroundTaskProgress, b: &BackgroundTaskProgress) -> bool {
    a.kind == b.kind
        && a.percent == b.percent
        && a.message == b.message
        && a.current == b.current
        && a.total == b.total
        && a.unit == b.unit
        && a.eta_seconds == b.eta_seconds
        && a.source == b.source
}

#[derive(Debug, Clone, Default)]
pub struct RunningBackgroundProgress {
    pub task_id: String,
    pub tool_name: String,
    pub label: String,
    pub detail: Option<String>,
}

/// Information returned when a background task is started
#[derive(Debug, Clone, Serialize)]
pub struct BackgroundTaskInfo {
    pub task_id: String,
    pub output_file: PathBuf,
    pub status_file: PathBuf,
}

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum BackgroundTaskWaitReason {
    AlreadyFinished,
    Finished,
    Progress,
    Checkpoint,
    /// The task's stall watchdog fired while this wait was blocked.
    Stalled,
    Timeout,
}

#[derive(Debug, Clone, Serialize)]
pub struct BackgroundTaskWaitResult {
    pub reason: BackgroundTaskWaitReason,
    pub task: TaskStatusFile,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub progress_event: Option<BackgroundTaskProgressEvent>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub event_record: Option<BackgroundTaskEventRecord>,
}

#[derive(Debug, Clone, Serialize)]
pub struct BackgroundCleanupResult {
    pub matched_files: usize,
    pub removed_files: usize,
    pub skipped_running_files: usize,
}

/// Internal tracking for a running task
pub(super) struct RunningTask {
    pub(super) task_id: String,
    pub(super) tool_name: String,
    pub(super) display_name: Option<String>,
    pub(super) session_id: String,
    pub(super) status_path: PathBuf,
    pub(super) owner_generation: Option<u64>,
    pub(super) started_at: Instant,
    pub(super) started_at_rfc3339: String,
    pub(super) delivery_flags: watch::Sender<(bool, bool)>,
    pub(super) handle: JoinHandle<Result<TaskResult>>,
}

/// Result from a background task execution
pub struct TaskResult {
    pub exit_code: Option<i32>,
    pub error: Option<String>,
    pub status: Option<BackgroundTaskStatus>,
}

impl TaskResult {
    pub fn completed(exit_code: Option<i32>) -> Self {
        Self {
            exit_code,
            error: None,
            status: Some(BackgroundTaskStatus::Completed),
        }
    }

    pub fn failed(exit_code: Option<i32>, error: impl Into<String>) -> Self {
        Self {
            exit_code,
            error: Some(error.into()),
            status: Some(BackgroundTaskStatus::Failed),
        }
    }

    pub fn superseded(exit_code: Option<i32>, detail: impl Into<String>) -> Self {
        Self {
            exit_code,
            error: Some(detail.into()),
            status: Some(BackgroundTaskStatus::Superseded),
        }
    }
}

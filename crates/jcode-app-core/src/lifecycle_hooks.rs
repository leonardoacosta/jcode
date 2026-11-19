//! App-layer glue for lifecycle hooks (`[[hooks.PreToolUse]]`, `Stop`, ...).
//!
//! The runner lives in `jcode_base::hooks`. This module adds what only the
//! upper layer knows: the transcript path, and ownership of hook child
//! processes by the session's durable generation, so a stop cannot settle
//! while a hook it admitted is still alive.

use crate::hooks::{
    ChildLease, ChildOwnership, LifecycleCall, LifecycleEvent, LifecycleOutcome,
    dispatch_lifecycle_detached, lifecycle_configured, run_lifecycle,
};
use crate::server::session_control::{GenerationToken, OwnedChild, current_token};
use serde_json::Value;
use std::sync::Arc;

/// Maximum bytes of a tool response sent to a `PostToolUse` hook.
const TOOL_RESPONSE_LIMIT: usize = 64 * 1024;

/// Hook children owned by one admitted generation.
struct GenerationOwner(GenerationToken);

struct OwnedHookChild(OwnedChild);

impl ChildLease for OwnedHookChild {
    fn set_pid(&mut self, pid: u32) {
        self.0.set_pid(pid);
    }
}

impl ChildOwnership for GenerationOwner {
    fn own(&self) -> anyhow::Result<Box<dyn ChildLease>> {
        Ok(Box::new(OwnedHookChild(self.0.own_shell_child()?)))
    }
}

/// A call for `event` carrying the running generation, when there is one.
pub(crate) fn call(event: LifecycleEvent, session_id: &str, cwd: Option<&str>) -> LifecycleCall {
    let owner = current_token()
        .filter(|token| token.session_id == session_id)
        .map(|token| Arc::new(GenerationOwner(token)) as Arc<dyn ChildOwnership>);
    LifecycleCall::new(event, session_id)
        .cwd(cwd.map(str::to_owned))
        .transcript_path(
            crate::session::session_path(session_id)
                .ok()
                .map(|path| path.display().to_string()),
        )
        .owner(owner)
}

/// `SessionStart` with a Claude Code `source` (`startup`, `resume`).
pub(crate) fn fire_session_start(session_id: &str, cwd: Option<&str>, source: &str) {
    if !lifecycle_configured(LifecycleEvent::SessionStart) {
        return;
    }
    dispatch_lifecycle_detached(
        call(LifecycleEvent::SessionStart, session_id, cwd)
            .subject(source)
            .field("source", source),
    );
}

/// `PreCompact` with a Claude Code `trigger` (`auto`, `manual`).
pub(crate) fn fire_pre_compact(session_id: &str, cwd: Option<&str>, trigger: &str) {
    if !lifecycle_configured(LifecycleEvent::PreCompact) {
        return;
    }
    dispatch_lifecycle_detached(
        call(LifecycleEvent::PreCompact, session_id, cwd)
            .subject(trigger)
            .field("trigger", trigger),
    );
}

/// `PostToolUse`: runs after the tool finished, never delays the result.
pub(crate) fn fire_post_tool_use(
    session_id: &str,
    cwd: Option<&str>,
    tool_name: &str,
    tool_call_id: &str,
    tool_input: &Value,
    paths: Vec<String>,
    result: Result<&str, &str>,
) {
    if !lifecycle_configured(LifecycleEvent::PostToolUse) {
        return;
    }
    let bounded = |text: &str| {
        let mut end = text.len().min(TOOL_RESPONSE_LIMIT);
        while !text.is_char_boundary(end) {
            end -= 1;
        }
        text[..end].to_string()
    };
    let response = match result {
        Ok(output) => serde_json::json!({"success": true, "output": bounded(output)}),
        Err(error) => serde_json::json!({"success": false, "error": bounded(error)}),
    };
    dispatch_lifecycle_detached(
        call(LifecycleEvent::PostToolUse, session_id, cwd)
            .subject(tool_name)
            .paths(paths)
            .field("tool_name", tool_name)
            .field("tool_use_id", tool_call_id)
            .field("tool_input", tool_input.clone())
            .field("tool_response", response),
    );
}

/// `PreToolUse` gate. `None` when nothing is configured.
pub(crate) async fn run_pre_tool_use(
    session_id: &str,
    cwd: Option<&str>,
    tool_name: &str,
    tool_call_id: &str,
    tool_input: &Value,
    paths: Vec<String>,
) -> Option<LifecycleOutcome> {
    if !lifecycle_configured(LifecycleEvent::PreToolUse) {
        return None;
    }
    Some(
        run_lifecycle(
            call(LifecycleEvent::PreToolUse, session_id, cwd)
                .subject(tool_name)
                .paths(paths)
                .field("tool_name", tool_name)
                .field("tool_use_id", tool_call_id)
                .field("tool_input", tool_input.clone()),
        )
        .await,
    )
}

tokio::task_local! {
    /// The raw parts of a report being recorded (message, validation,
    /// follow-up), scoped by the `CommReport` handlers so the status choke
    /// point can hand `SubagentStop` hooks the structured fields.
    static REPORT_PARTS: (String, Option<String>, Option<String>);
}

/// Run `future` with the report's raw parts visible to [`fire_subagent_stop`].
pub(crate) async fn with_report_parts<F: std::future::Future>(
    message: &str,
    validation: Option<&str>,
    follow_up: Option<&str>,
    future: F,
) -> F::Output {
    REPORT_PARTS
        .scope(
            (
                message.to_string(),
                validation.map(str::to_string),
                follow_up.map(str::to_string),
            ),
            future,
        )
        .await
}

/// `SubagentStop`: a worker's completion report was recorded. Runs on the
/// coordinator side and never delays the report.
pub(crate) fn fire_subagent_stop(
    worker_session_id: &str,
    cwd: Option<&str>,
    status: &str,
    report: &str,
) {
    if !lifecycle_configured(LifecycleEvent::SubagentStop) {
        return;
    }
    let mut event = call(LifecycleEvent::SubagentStop, worker_session_id, cwd)
        .field("stop_hook_active", false)
        .field("status", status)
        .field("report", report);
    let _ = REPORT_PARTS.try_with(|(message, validation, follow_up)| {
        event = std::mem::replace(
            &mut event,
            LifecycleCall::new(LifecycleEvent::SubagentStop, worker_session_id),
        )
        .field("message", message.as_str())
        .field("validation", validation.as_deref())
        .field("follow_up", follow_up.as_deref());
    });
    dispatch_lifecycle_detached(event);
}

/// Most Stop-hook continuations one user turn may take.
pub(crate) const STOP_CONTINUATIONS_PER_TURN: u32 = 3;

/// `Stop` at natural completion. Returns the hook's reason when it asked to
/// continue; the caller must still check admission (cancel, generation).
pub(crate) async fn run_stop(
    session_id: &str,
    cwd: Option<&str>,
    final_text: &str,
    stop_hook_active: bool,
) -> Option<LifecycleOutcome> {
    if !lifecycle_configured(LifecycleEvent::Stop) {
        return None;
    }
    Some(
        run_lifecycle(
            call(LifecycleEvent::Stop, session_id, cwd)
                .field("stop_hook_active", stop_hook_active)
                .field("last_assistant_message", final_text),
        )
        .await,
    )
}

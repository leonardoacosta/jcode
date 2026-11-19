//! Routes MCP notification wakes into the scheduled queue.
//!
//! The MCP layer treats notifications as hints and calls a [`WakeHandler`]
//! with identifiers only. This router turns one into a `ScheduledItem` that the
//! existing delivery paths already handle (ambient cycle or a session). The
//! item text never carries notification params or resource content.

use crate::ambient::{Priority, ScheduleTarget, ScheduledItem, ScheduledQueue};
use crate::config::config;
use crate::logging;
use crate::mcp::notifications::{
    WakeHandler, WakeOutcome, WakeRequest, sanitize_server_name, set_wake_handler,
};
use chrono::Utc;
use std::path::PathBuf;
use std::sync::Arc;

pub(super) struct McpWakeRouter {
    /// `None` uses the real ambient queue.
    queue_path: Option<PathBuf>,
    /// Ambient-targeted wakes are dropped when ambient mode is off, since
    /// nothing would ever deliver them.
    ambient_enabled: fn() -> bool,
    /// Whether a session may take new work. A revoked session is treated as
    /// absent: the wake is reported dropped, not queued for a doomed delivery.
    session_runnable: fn(&str) -> bool,
}

fn session_is_runnable(session_id: &str) -> bool {
    crate::server::session_control::store()
        .runnable(session_id)
        .is_ok()
}

/// Install the router as the process-wide MCP wake handler.
pub(super) fn install() {
    set_wake_handler(Some(Arc::new(McpWakeRouter {
        queue_path: None,
        ambient_enabled: || config().ambient.enabled,
        session_runnable: session_is_runnable,
    })));
}

fn item_id(request: &WakeRequest) -> String {
    format!(
        "mcp_wake:{}:{}:{}",
        sanitize_server_name(&request.server),
        request.class.as_str(),
        request.session.as_deref().unwrap_or("ambient")
    )
}

pub(crate) fn build_item(request: &WakeRequest) -> ScheduledItem {
    let now = Utc::now();
    // The engine already sanitizes; re-bound here so no caller can put a
    // server-chosen string with newlines or instructions into model text.
    let server = sanitize_server_name(&request.server);
    let context = format!(
        "MCP server '{}' signalled '{}'. Its catalog was refreshed if allowed. \
         Re-check its tools, prompts or resources as needed.",
        server,
        request.class.as_str()
    );
    ScheduledItem {
        id: item_id(request),
        scheduled_for: now,
        context: context.clone(),
        priority: Priority::Normal,
        target: match &request.session {
            Some(session_id) => ScheduleTarget::Session {
                session_id: session_id.clone(),
            },
            None => ScheduleTarget::Ambient,
        },
        created_by_session: format!("mcp:{server}"),
        created_at: now,
        working_dir: None,
        task_description: Some(context),
        relevant_files: Vec::new(),
        git_branch: None,
        additional_context: None,
    }
}

impl McpWakeRouter {
    fn route(&self, request: &WakeRequest) -> WakeOutcome {
        let server = sanitize_server_name(&request.server);
        match &request.session {
            None if !(self.ambient_enabled)() => {
                logging::debug(&format!(
                    "MCP wake for '{server}' dropped: ambient mode is disabled"
                ));
                return WakeOutcome::Dropped("ambient mode is disabled");
            }
            Some(session_id) if !(self.session_runnable)(session_id) => {
                logging::warn(&format!(
                    "MCP wake for '{server}' dropped: target session admission is revoked"
                ));
                return WakeOutcome::Dropped("target session admission is revoked");
            }
            _ => {}
        }
        let item = build_item(request);
        let result = match &self.queue_path {
            Some(path) => push_unique(ScheduledQueue::load(path.clone()), item),
            None => crate::ambient::paths::queue_path()
                .and_then(|path| push_unique(ScheduledQueue::load(path), item)),
        };
        match result {
            Ok(true) => WakeOutcome::Queued,
            Ok(false) => WakeOutcome::Coalesced,
            Err(error) => {
                logging::warn(&format!("MCP wake enqueue failed: {error:#}"));
                WakeOutcome::Dropped("wake queue write failed")
            }
        }
    }
}

/// Queue the item unless an identical wake is already pending (coalescing).
fn push_unique(mut queue: ScheduledQueue, item: ScheduledItem) -> anyhow::Result<bool> {
    queue.transact(|items| {
        if items.iter().any(|existing| existing.id == item.id) {
            return Ok(false);
        }
        items.push(item);
        Ok(true)
    })
}

impl WakeHandler for McpWakeRouter {
    fn wake(&self, request: WakeRequest) -> WakeOutcome {
        self.route(&request)
    }
}

#[cfg(test)]
#[path = "runner_mcp_wake_tests.rs"]
mod tests;

use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

pub mod cron;
mod directives;
mod manager;
mod paths;
mod persistence;
mod prompt;
pub mod recurring;
pub mod runner;
pub mod scheduler;

pub use directives::{
    UserDirective, add_directive, has_pending_directives, load_directives, take_pending_directives,
};
pub use manager::AmbientManager;

/// Path of the one-shot schedule queue, for the recurring scheduler.
pub(crate) fn queue_file_path() -> anyhow::Result<std::path::PathBuf> {
    paths::queue_path()
}
pub use persistence::{AmbientLock, RecoveryReport, ScheduledQueue, Settlement};

/// Whether persisted ambient work is waiting: a pending or in-flight scheduled
/// item, or an unconsumed directive. Reads the existing durable state, so a
/// server never keeps a second ledger of its own work. State that exists but
/// cannot be read counts as work: an unreadable queue is not an empty one.
pub fn has_pending_work() -> bool {
    let Ok(queue_path) = paths::queue_path() else {
        return true;
    };
    if queue_path.exists() {
        if crate::storage::read_json::<Vec<serde_json::Value>>(&queue_path).is_err() {
            return true;
        }
        let queue = ScheduledQueue::load(queue_path);
        if !queue.is_empty() || !queue.in_flight().is_empty() {
            return true;
        }
    }
    directives::directives_unreadable_or_pending()
}
#[cfg(test)]
pub(crate) use prompt::format_duration_rough;
pub use prompt::{
    MemoryGraphHealth, RecentSessionInfo, ResourceBudget, build_ambient_system_prompt,
    format_minutes_human, format_scheduled_session_message, gather_feedback_memories,
    gather_memory_graph_health, gather_recent_sessions,
};

use crate::storage;

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

/// Context passed from the ambient runner to a visible TUI cycle.
/// Saved to `~/.jcode/ambient/visible_cycle.json`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisibleCycleContext {
    pub system_prompt: String,
    pub initial_message: String,
}

impl VisibleCycleContext {
    pub fn context_path() -> Result<PathBuf> {
        Ok(storage::jcode_dir()?
            .join("ambient")
            .join("visible_cycle.json"))
    }

    pub fn save(&self) -> Result<()> {
        let path = Self::context_path()?;
        if let Some(parent) = path.parent() {
            storage::ensure_dir(parent)?;
        }
        storage::write_json(&path, self)
    }

    pub fn load() -> Result<Self> {
        let path = Self::context_path()?;
        storage::read_json(&path)
    }

    pub fn result_path() -> Result<PathBuf> {
        Ok(storage::jcode_dir()?
            .join("ambient")
            .join("cycle_result.json"))
    }
}

/// Ambient mode status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub enum AmbientStatus {
    #[default]
    Idle,
    Running {
        detail: String,
    },
    Scheduled {
        next_wake: DateTime<Utc>,
    },
    Paused {
        reason: String,
    },
    Disabled,
}

/// Priority for scheduled items
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum Priority {
    Low,
    Normal,
    High,
}

/// Where a scheduled task should be delivered when it becomes due.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ScheduleTarget {
    /// Wake the ambient agent and hand it the queued task.
    #[default]
    Ambient,
    /// Deliver the reminder back into a specific interactive session.
    Session { session_id: String },
    /// Spawn a single new session derived from the originating session.
    Spawn { parent_session_id: String },
}

impl ScheduleTarget {
    pub fn is_direct_delivery(&self) -> bool {
        matches!(self, Self::Session { .. } | Self::Spawn { .. })
    }
}

/// A scheduled ambient task
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ScheduledItem {
    pub id: String,
    pub scheduled_for: DateTime<Utc>,
    pub context: String,
    pub priority: Priority,
    #[serde(default)]
    pub target: ScheduleTarget,
    pub created_by_session: String,
    pub created_at: DateTime<Utc>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub working_dir: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub task_description: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub relevant_files: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub git_branch: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub additional_context: Option<String>,
    /// Delivery ledger. Untouched pending items serialize exactly as before.
    #[serde(default, skip_serializing_if = "DeliveryRecord::is_untouched")]
    pub delivery: DeliveryRecord,
}

/// Where a scheduled item is in its delivery lifecycle.
///
/// Every state is stored in the one queue file. `Pending` and `Dispatched`
/// items are live; the rest are terminal history, pruned to a bound (never
/// `Uncertain`). An older binary that predates this field would treat terminal
/// items as ready, so do not downgrade with unresolved history in the queue.
/// `Delivered` means the intended target accepted the message. It is not
/// evidence that the work ran or succeeded.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum ItemState {
    #[default]
    Pending,
    /// Claimed for delivery, no acknowledgement yet.
    Dispatched,
    Delivered,
    /// Overdue beyond the grace window at startup. Never auto-delivered.
    Missed,
    /// Dispatched but never acknowledged. May have run. Needs manual retry.
    Uncertain,
    Cancelled,
}

impl ItemState {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Dispatched => "dispatched",
            Self::Delivered => "delivered",
            Self::Missed => "missed",
            Self::Uncertain => "uncertain",
            Self::Cancelled => "cancelled",
        }
    }
}

/// One delivery attempt. Attempts are append-only: a manual retry adds a new
/// one and never rewrites an earlier attempt.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DeliveryAttempt {
    pub number: u32,
    pub dispatched_at: DateTime<Utc>,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub manual_retry: bool,
    /// Set only when the target accepted the message.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub acked_at: Option<DateTime<Utc>>,
    /// Session that accepted the message (the child session for spawns).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub acked_target: Option<String>,
    /// Execution generation of the accepting session when the ack was recorded.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub generation: Option<u64>,
    /// `pid:nonce` of the process that claimed the attempt. Startup recovery
    /// leaves attempts of live processes alone; the nonce defeats pid reuse.
    #[serde(default)]
    pub owner: String,
    /// Why the attempt ended without an acknowledgement.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct DeliveryRecord {
    pub state: ItemState,
    pub attempts: Vec<DeliveryAttempt>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub resolved_at: Option<DateTime<Utc>>,
    /// How late the item was when it was marked `missed`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub missed_delay_secs: Option<i64>,
    /// A manual retry was requested; the next claim records it as such.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub retry_pending: bool,
    /// Execution generation of the creating session. A stop that revoked it
    /// before delivery suppresses the wake.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub origin_generation: Option<u64>,
}

impl DeliveryRecord {
    fn is_untouched(&self) -> bool {
        *self == Self::default()
    }
}

/// Persistent ambient state
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AmbientState {
    pub status: AmbientStatus,
    pub last_run: Option<DateTime<Utc>>,
    pub last_summary: Option<String>,
    pub last_compactions: Option<u32>,
    pub last_memories_modified: Option<u32>,
    pub total_cycles: u64,
}

/// Result from an ambient cycle
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AmbientCycleResult {
    pub summary: String,
    pub memories_modified: u32,
    pub compactions: u32,
    pub proactive_work: Option<String>,
    pub next_schedule: Option<ScheduleRequest>,
    pub started_at: DateTime<Utc>,
    pub ended_at: DateTime<Utc>,
    pub status: CycleStatus,
    /// Full conversation transcript (markdown) for email notifications
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub conversation: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CycleStatus {
    Complete,
    Interrupted,
    Incomplete,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduleRequest {
    pub wake_in_minutes: Option<u32>,
    pub wake_at: Option<DateTime<Utc>>,
    pub context: String,
    pub priority: Priority,
    #[serde(default)]
    pub target: ScheduleTarget,
    #[serde(default)]
    pub created_by_session: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub working_dir: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub task_description: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub relevant_files: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub git_branch: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub additional_context: Option<String>,
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
#[path = "ambient_tests.rs"]
mod ambient_tests;

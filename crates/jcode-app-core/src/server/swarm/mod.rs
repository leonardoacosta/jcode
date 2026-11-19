#[allow(unused_imports)]
use super::state::{MAX_EVENT_HISTORY, fanout_session_event};
#[allow(unused_imports)]
use super::{SwarmEvent, SwarmEventType, SwarmMember, SwarmState, VersionedPlan};
#[allow(unused_imports)]
use super::{persist_swarm_state_for, remove_persisted_swarm_state_for};
#[allow(unused_imports)]
use crate::agent::Agent;
#[allow(unused_imports)]
use crate::plan::{PlanItem, newly_ready_item_ids};
#[allow(unused_imports)]
use crate::protocol::{NotificationType, ServerEvent};
#[allow(unused_imports)]
use crate::session::Session;
#[allow(unused_imports)]
use anyhow::Result;
#[allow(unused_imports)]
use futures::future::try_join_all;
#[allow(unused_imports)]
use jcode_swarm_core::{completion_notification_with_lane_plan, normalize_completion_report};
#[allow(unused_imports)]
use serde::Deserialize;
#[allow(unused_imports)]
use std::collections::{HashMap, HashSet};
#[allow(unused_imports)]
use std::path::PathBuf;
#[allow(unused_imports)]
use std::sync::Arc;
#[allow(unused_imports)]
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
#[allow(unused_imports)]
use std::sync::{Mutex as StdMutex, OnceLock};
#[allow(unused_imports)]
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
#[allow(unused_imports)]
use tokio::sync::{Mutex, RwLock, broadcast};

fn status_age_secs(last_status_change: Instant) -> u64 {
    last_status_change.elapsed().as_secs()
}

/// Maximum number of live members (agents) in a single swarm. Re-exported from
/// `jcode_swarm_core` so the server, tools, and prompts all agree on the one
/// runaway-prevention cap. Swarms are root-only, one-level fan-out, and both
/// the configurable live-worker budget and this absolute cap apply.
pub(super) use jcode_swarm_core::MAX_SWARM_MEMBERS;

/// One-line preview of user or agent text for swarm status, plan progress and
/// event history.
///
/// These previews are echoed to other agents (`swarm status`, `swarm list`),
/// persisted with swarm state and rendered in every attached client, so a
/// credential at the start of a user request must not be copied into them.
pub(in crate::server) fn truncate_detail(text: &str, max_len: usize) -> String {
    // Bound the redaction scan: only the head of the text can reach the
    // preview, and prompts with pasted content can be very large.
    let scan_chars = max_len.saturating_mul(4).saturating_add(64);
    let head = match text.char_indices().nth(scan_chars) {
        Some((end, _)) => &text[..end],
        None => text,
    };
    jcode_swarm_core::truncate_detail(&crate::message::redact_preview_secrets(head), max_len)
}

/// Fire the cancel signal of every turn `session_id` is currently running.
/// Returns how many were fired.
///
/// `swarm stop` removes the worker's session entry and swarm membership. A
/// worker that was mid-turn kept its own `Arc<Agent>` and went on generating
/// and writing files for minutes, with no handle left to stop it by.
pub(in crate::server) fn cancel_active_turns(session_id: &str) -> usize {
    let signals = crate::turn_cancel_registry::active_turn_signals(session_id);
    for signal in &signals {
        signal.fire();
    }
    if !signals.is_empty() {
        crate::logging::info(&format!(
            "SWARM_STOP_CANCELLED_ACTIVE_TURNS session={session_id} fired={}",
            signals.len()
        ));
    }
    signals.len()
}

pub(in crate::server) use tree::cancel_spawned_descendant_turns;

#[path = "../swarm_event_history.rs"]
mod swarm_event_history;
use swarm_event_history::log_swarm_lifecycle;
pub(in crate::server) use swarm_event_history::{record_swarm_event, record_swarm_event_for_session};

mod tree;
#[allow(unused_imports)]
pub(in crate::server) use tree::*;
mod config;
#[allow(unused_imports)]
pub(in crate::server) use config::*;
mod lifecycle;
#[allow(unused_imports)]
pub(in crate::server) use lifecycle::*;
mod tasks;
#[allow(unused_imports)]
pub(in crate::server) use tasks::*;
mod status;
#[allow(unused_imports)]
pub(in crate::server) use status::*;
mod plan;
#[allow(unused_imports)]
pub(in crate::server) use plan::*;
/// Inline workers have no entry in the server's live-session map. Clean their
/// ephemeral environment on normal completion, errors, and future cancellation.
struct InlineWorkerEnv(String);

impl Drop for InlineWorkerEnv {
    fn drop(&mut self) {
        crate::session_env::remove_session_env(&self.0);
    }
}

mod claims;
mod run;
#[allow(unused_imports)]
pub(in crate::server) use claims::*;
pub(crate) use claims::{FileClaims, claim_files};
#[allow(unused_imports)]
pub(in crate::server) use run::*;

#[cfg(test)]
#[path = "../swarm_cascade_tests.rs"]
mod swarm_cascade_tests;

#[cfg(test)]
mod tests;

#[cfg(test)]
#[path = "../swarm_tests.rs"]
mod upstream_tests;

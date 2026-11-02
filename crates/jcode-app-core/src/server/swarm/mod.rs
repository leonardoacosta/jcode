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
use jcode_swarm_core::{
    completion_notification_message, normalize_completion_report, truncate_detail,
};
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
mod run;
mod claims;
#[allow(unused_imports)]
pub(in crate::server) use claims::*;
#[allow(unused_imports)]
pub(in crate::server) use run::*;

#[cfg(test)]
#[path = "../swarm_cascade_tests.rs"]
mod swarm_cascade_tests;

#[cfg(test)]
mod tests;

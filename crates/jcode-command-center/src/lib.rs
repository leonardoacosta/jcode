//! Backend foundation for the experimental Jcode command center.
//!
//! This crate owns the public protocol boundary and daemon-side service
//! primitives. It deliberately contains no frontend persistence and is safe to
//! wire behind a disabled-by-default loopback listener.

pub mod mx_health;
pub mod orca_operation_store;

use std::collections::{HashMap, VecDeque};
use std::fmt;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use axum::extract::{Path, Query, State};
use axum::http::{HeaderMap, HeaderValue, Method, StatusCode, header};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use chrono::{DateTime, Duration, Utc};
use jcode_task_types::{Goal, GoalMilestone, GoalStatus};
use serde::{Deserialize, Serialize};
use tokio::net::TcpListener;
use tokio::sync::oneshot;
use tokio::task::JoinHandle;
use tower_http::services::{ServeDir, ServeFile};
use tower_http::set_header::SetResponseHeaderLayer;
use uuid::Uuid;

pub const PROTOCOL_VERSION: u16 = 1;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Authority {
    Jcode,
    Orca,
    Client,
}

macro_rules! id_type {
    ($name:ident) => {
        #[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
        #[serde(transparent)]
        pub struct $name(pub String);
    };
}

id_type!(InitiativeId);
id_type!(ScheduleRefId);
id_type!(JcodeRunId);
id_type!(OrcaProjectId);
id_type!(OrcaRepositoryId);
id_type!(OrcaHostSetupId);
id_type!(OrcaHostId);
id_type!(OrcaRunId);
id_type!(OrcaTaskId);
id_type!(OrcaDispatchId);
id_type!(OrcaWorktreeId);
id_type!(OrcaTerminalId);
id_type!(OrcaRequestId);
id_type!(CorrelationId);
id_type!(StreamId);
id_type!(CommandId);
id_type!(IdempotencyKey);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Revision(pub u64);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProtocolMetadata {
    pub version: u16,
    pub generated_at: DateTime<Utc>,
}

impl Default for ProtocolMetadata {
    fn default() -> Self {
        Self {
            version: PROTOCOL_VERSION,
            generated_at: Utc::now(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FreshnessState {
    Fresh,
    Stale,
    Unavailable,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Freshness {
    pub state: FreshnessState,
    pub observed_at: DateTime<Utc>,
    pub last_success_at: Option<DateTime<Utc>>,
    pub evidence: Option<String>,
}

impl Freshness {
    pub fn fresh() -> Self {
        let now = Utc::now();
        Self {
            state: FreshnessState::Fresh,
            observed_at: now,
            last_success_at: Some(now),
            evidence: None,
        }
    }

    pub fn unavailable(evidence: impl Into<String>) -> Self {
        Self {
            state: FreshnessState::Unavailable,
            observed_at: Utc::now(),
            last_success_at: None,
            evidence: Some(evidence.into()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct AvailableActions {
    pub update_initiative: bool,
    pub checkpoint_initiative: bool,
    pub manage_blockers: bool,
    pub manage_next_actions: bool,
    pub start_initiative_run: bool,
    pub retry_linked_run: bool,
    pub cancel_linked_run: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RuntimeMutationCapabilities {
    pub start_initiative_run: bool,
    pub retry_linked_run: bool,
    pub cancel_linked_run: bool,
}

impl RuntimeMutationCapabilities {
    pub fn unavailable() -> Self {
        Self {
            start_initiative_run: false,
            retry_linked_run: false,
            cancel_linked_run: false,
        }
    }
}

impl Default for RuntimeMutationCapabilities {
    fn default() -> Self {
        Self::unavailable()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CleanupResourceState {
    VerifiedReleased,
    RecoveryRequired,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CleanupResourceProjection {
    pub resource_kind: String,
    pub resource_id: String,
    pub state: CleanupResourceState,
    pub evidence: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InitiativeProjection {
    pub id: InitiativeId,
    pub title: String,
    pub status: GoalStatus,
    pub revision: Revision,
    pub description: String,
    pub why: String,
    pub current_milestone_id: Option<String>,
    pub milestones: Vec<GoalMilestone>,
    pub success_criteria: Vec<String>,
    pub blockers: Vec<String>,
    pub next_actions: Vec<String>,
    pub progress_percent: Option<u8>,
    pub updated_at: DateTime<Utc>,
}

impl From<(Goal, Revision)> for InitiativeProjection {
    fn from((goal, revision): (Goal, Revision)) -> Self {
        Self {
            id: InitiativeId(goal.id),
            title: goal.title,
            status: goal.status,
            revision,
            description: goal.description,
            why: goal.why,
            current_milestone_id: goal.current_milestone_id,
            milestones: goal.milestones,
            success_criteria: goal.success_criteria,
            blockers: goal.blockers,
            next_actions: goal.next_steps,
            progress_percent: goal.progress_percent,
            updated_at: goal.updated_at,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LinkedScheduleProjection {
    pub id: ScheduleRefId,
    pub initiative_id: InitiativeId,
    pub cadence: String,
    pub timezone: String,
    pub next_fire_at: Option<DateTime<Utc>>,
    pub last_result: Option<String>,
    pub last_run_id: Option<JcodeRunId>,
    pub retry_count: u32,
    pub missed_wake: bool,
    pub stale_claim: bool,
    pub failure_evidence: Option<String>,
    pub freshness: Freshness,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct JcodeRunReference {
    pub id: JcodeRunId,
    pub initiative_id: InitiativeId,
    pub orca_run_id: Option<OrcaRunId>,
    #[serde(default)]
    pub orca_task_id: Option<OrcaTaskId>,
    #[serde(default)]
    pub orca_dispatch_id: Option<OrcaDispatchId>,
    #[serde(default)]
    pub retry_of_jcode_run_id: Option<JcodeRunId>,
    #[serde(default)]
    pub retry_of_dispatch_id: Option<OrcaDispatchId>,
    #[serde(default)]
    pub worktree_id: Option<OrcaWorktreeId>,
    #[serde(default)]
    pub terminal_id: Option<OrcaTerminalId>,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OrcaCanonicalPlacement {
    pub project_id: OrcaProjectId,
    pub repository_id: OrcaRepositoryId,
    pub host_setup_id: OrcaHostSetupId,
    pub host_id: OrcaHostId,
    pub worktree_id: OrcaWorktreeId,
    pub worktree_selector: String,
    pub coordinator_terminal_id: OrcaTerminalId,
    pub environment: Option<String>,
    pub launcher: OrcaWorkerLauncher,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum OrcaWorkerLauncher {
    Agent {
        agent: String,
        model: Option<String>,
        effort: Option<String>,
    },
    ExistingTerminal {
        terminal_id: OrcaTerminalId,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OrcaMutationContext {
    pub command_id: CommandId,
    pub idempotency_key: IdempotencyKey,
    pub correlation_id: CorrelationId,
    pub initiative_id: InitiativeId,
    pub jcode_attempt_id: JcodeRunId,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StartInitiativeRunRequest {
    pub context: OrcaMutationContext,
    pub objective: String,
    pub task_spec: String,
    pub placement: OrcaCanonicalPlacement,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RetryLinkedRunRequest {
    pub context: OrcaMutationContext,
    pub prior_jcode_attempt_id: JcodeRunId,
    pub orca_run_id: OrcaRunId,
    pub orca_task_id: OrcaTaskId,
    pub retry_of_dispatch_id: OrcaDispatchId,
    pub placement: OrcaCanonicalPlacement,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CancelLinkedRunRequest {
    pub context: OrcaMutationContext,
    pub target_jcode_attempt_id: JcodeRunId,
    pub orca_run_id: OrcaRunId,
    pub orca_task_id: OrcaTaskId,
    pub target_dispatch_id: OrcaDispatchId,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OrcaAttemptIdentity {
    pub run_id: OrcaRunId,
    pub task_id: OrcaTaskId,
    pub dispatch_id: OrcaDispatchId,
    pub retry_of_dispatch_id: Option<OrcaDispatchId>,
    pub worktree_id: OrcaWorktreeId,
    pub terminal_id: Option<OrcaTerminalId>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OrcaEffectReceipt {
    pub kind: String,
    #[serde(default)]
    pub role: Option<String>,
    pub action: String,
    #[serde(default)]
    pub id: Option<String>,
    #[serde(default)]
    pub surface: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OrcaMutationOutcome {
    Ready,
    Failed,
    OutcomeUnknown,
    Stopped,
    Abandoned,
    AlreadySettled,
    Rejected,
    RecoveryRequired,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OrcaLifecycleReceipt {
    pub outcome: OrcaMutationOutcome,
    pub attempt: Option<OrcaAttemptIdentity>,
    pub stage: String,
    pub failed_stage: Option<String>,
    pub last_error: Option<String>,
    pub effects: Vec<OrcaEffectReceipt>,
    pub residual_resources: Vec<OrcaEffectReceipt>,
    pub cleanup: Vec<CleanupResourceProjection>,
    pub observed_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OrcaReference {
    pub project_id: Option<OrcaProjectId>,
    #[serde(default)]
    pub runtime_id: Option<String>,
    pub run_id: Option<OrcaRunId>,
    #[serde(default)]
    pub task_ids: Vec<OrcaTaskId>,
    #[serde(default)]
    pub dispatch_ids: Vec<OrcaDispatchId>,
    #[serde(default)]
    pub worktree_ids: Vec<OrcaWorktreeId>,
    pub worker_ids: Vec<String>,
    #[serde(default)]
    pub terminal_ids: Vec<OrcaTerminalId>,
    pub gate_ids: Vec<String>,
    #[serde(default)]
    pub correlation_ids: Vec<CorrelationId>,
    #[serde(default)]
    pub idempotency_keys: Vec<IdempotencyKey>,
    pub last_observed_at: Option<DateTime<Utc>>,
    pub freshness: Freshness,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct CommandCenterSnapshot {
    pub metadata: ProtocolMetadata,
    pub revision: Revision,
    pub initiative: InitiativeProjection,
    pub schedules: Vec<LinkedScheduleProjection>,
    pub runs: Vec<JcodeRunReference>,
    pub orca: OrcaReference,
    pub freshness: Freshness,
    pub available_actions: AvailableActions,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub external_signals: Option<serde_json::Value>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct InitiativeListSnapshot {
    pub metadata: ProtocolMetadata,
    pub revision: Revision,
    pub initiatives: Vec<InitiativeProjection>,
    pub freshness: Freshness,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct BrowserProtocolMeta<'a> {
    protocol_version: &'static str,
    snapshot_revision: u64,
    stream_id: &'a str,
    sequence: u64,
}

#[derive(Serialize)]
struct BrowserConnection<'a> {
    state: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    reason: Option<&'a str>,
    #[serde(rename = "lastConnectedAt", skip_serializing_if = "Option::is_none")]
    last_connected_at: Option<DateTime<Utc>>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct BrowserMilestoneStep<'a> {
    id: &'a str,
    title: &'a str,
    status: &'a str,
}

#[derive(Serialize)]
struct BrowserMilestone<'a> {
    id: &'a str,
    title: &'a str,
    status: &'a str,
    steps: Vec<BrowserMilestoneStep<'a>>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct BrowserSchedule<'a> {
    id: &'a str,
    cadence: &'a str,
    timezone: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    next_fire: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    last_result: Option<&'a str>,
    retry_state: String,
    freshness: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    evidence: Option<&'a str>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct BrowserAvailableActions {
    checkpoint: bool,
    update_milestone: bool,
    start_run: bool,
    retry_run: bool,
    cancel_run: bool,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct BrowserInitiative<'a> {
    id: &'a str,
    title: &'a str,
    outcome: &'a str,
    status: &'static str,
    revision: u64,
    current_milestone: BrowserMilestone<'a>,
    success_criteria: &'a [String],
    blockers: &'a [String],
    next_actions: &'a [String],
    children: [(); 0],
    schedules: Vec<BrowserSchedule<'a>>,
    checkpoints: [(); 0],
    freshness: &'static str,
    updated_at: DateTime<Utc>,
    available_actions: BrowserAvailableActions,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct BrowserRun<'a> {
    id: &'a str,
    initiative_id: &'a str,
    status: &'a str,
    health: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    orca_project_id: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    orca_run_id: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    last_observed_at: Option<DateTime<Utc>>,
    workers: [(); 0],
    gates: [(); 0],
    timeline: [(); 0],
    attention: Vec<&'a str>,
    available_actions: BrowserRunActions,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct BrowserRunActions {
    start_run: bool,
    retry_run: bool,
    cancel_run: bool,
}

fn browser_freshness(freshness: &Freshness) -> &'static str {
    match freshness.state {
        FreshnessState::Fresh => "live",
        FreshnessState::Stale => "stale",
        FreshnessState::Unavailable => "unavailable",
    }
}

fn browser_status(status: GoalStatus) -> &'static str {
    status.as_str()
}

fn browser_milestone(initiative: &InitiativeProjection) -> BrowserMilestone<'_> {
    let milestone = initiative
        .current_milestone_id
        .as_deref()
        .and_then(|id| {
            initiative
                .milestones
                .iter()
                .find(|milestone| milestone.id == id)
        })
        .or_else(|| initiative.milestones.first());
    match milestone {
        Some(milestone) => BrowserMilestone {
            id: &milestone.id,
            title: &milestone.title,
            status: &milestone.status,
            steps: milestone
                .steps
                .iter()
                .map(|step| BrowserMilestoneStep {
                    id: &step.id,
                    title: &step.content,
                    status: &step.status,
                })
                .collect(),
        },
        None => BrowserMilestone {
            id: "",
            title: "No current milestone",
            status: "pending",
            steps: Vec::new(),
        },
    }
}

fn browser_initiative<'a>(
    initiative: &'a InitiativeProjection,
    schedules: &'a [LinkedScheduleProjection],
    freshness: &'a Freshness,
    actions: &'a AvailableActions,
) -> BrowserInitiative<'a> {
    BrowserInitiative {
        id: &initiative.id.0,
        title: &initiative.title,
        outcome: if initiative.why.is_empty() {
            &initiative.description
        } else {
            &initiative.why
        },
        status: browser_status(initiative.status),
        revision: initiative.revision.0,
        current_milestone: browser_milestone(initiative),
        success_criteria: &initiative.success_criteria,
        blockers: &initiative.blockers,
        next_actions: &initiative.next_actions,
        children: [],
        schedules: schedules
            .iter()
            .map(|schedule| BrowserSchedule {
                id: &schedule.id.0,
                cadence: &schedule.cadence,
                timezone: &schedule.timezone,
                next_fire: schedule.next_fire_at,
                last_result: schedule.last_result.as_deref(),
                retry_state: schedule.retry_count.to_string(),
                freshness: browser_freshness(&schedule.freshness),
                evidence: schedule.failure_evidence.as_deref(),
            })
            .collect(),
        checkpoints: [],
        freshness: browser_freshness(freshness),
        updated_at: initiative.updated_at,
        available_actions: BrowserAvailableActions {
            checkpoint: actions.checkpoint_initiative,
            update_milestone: actions.update_initiative,
            start_run: actions.start_initiative_run,
            retry_run: actions.retry_linked_run,
            cancel_run: actions.cancel_linked_run,
        },
    }
}

impl Serialize for InitiativeListSnapshot {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        #[derive(Serialize)]
        struct ListDto<'a> {
            meta: BrowserProtocolMeta<'a>,
            initiatives: Vec<BrowserInitiative<'a>>,
            connection: BrowserConnection<'a>,
        }
        let no_actions = AvailableActions::default();
        ListDto {
            meta: BrowserProtocolMeta {
                protocol_version: "command-center.v1",
                snapshot_revision: self.revision.0,
                stream_id: "",
                sequence: 0,
            },
            initiatives: self
                .initiatives
                .iter()
                .map(|initiative| browser_initiative(initiative, &[], &self.freshness, &no_actions))
                .collect(),
            connection: BrowserConnection {
                state: browser_freshness(&self.freshness),
                reason: self.freshness.evidence.as_deref(),
                last_connected_at: self.freshness.last_success_at,
            },
        }
        .serialize(serializer)
    }
}

impl Serialize for CommandCenterSnapshot {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        #[derive(Serialize)]
        #[serde(rename_all = "camelCase")]
        struct SnapshotDto<'a> {
            meta: BrowserProtocolMeta<'a>,
            initiatives: Vec<BrowserInitiative<'a>>,
            selected_initiative: BrowserInitiative<'a>,
            #[serde(skip_serializing_if = "Option::is_none")]
            selected_run: Option<BrowserRun<'a>>,
            connection: BrowserConnection<'a>,
            #[serde(skip_serializing_if = "Option::is_none")]
            external_signals: Option<&'a serde_json::Value>,
        }
        let selected_run = self.runs.first().map(|run| BrowserRun {
            id: &run.id.0,
            initiative_id: &run.initiative_id.0,
            status: &run.status,
            health: browser_freshness(&self.orca.freshness),
            orca_project_id: self.orca.project_id.as_ref().map(|id| id.0.as_str()),
            orca_run_id: run.orca_run_id.as_ref().map(|id| id.0.as_str()),
            last_observed_at: self.orca.last_observed_at,
            workers: [],
            gates: [],
            timeline: [],
            attention: self
                .orca
                .freshness
                .evidence
                .as_deref()
                .into_iter()
                .collect(),
            available_actions: BrowserRunActions {
                start_run: self.available_actions.start_initiative_run,
                retry_run: self.available_actions.retry_linked_run,
                cancel_run: self.available_actions.cancel_linked_run,
            },
        });
        SnapshotDto {
            meta: BrowserProtocolMeta {
                protocol_version: "command-center.v1",
                snapshot_revision: self.revision.0,
                stream_id: "",
                sequence: 0,
            },
            initiatives: vec![browser_initiative(
                &self.initiative,
                &self.schedules,
                &self.freshness,
                &self.available_actions,
            )],
            selected_initiative: browser_initiative(
                &self.initiative,
                &self.schedules,
                &self.freshness,
                &self.available_actions,
            ),
            selected_run,
            connection: BrowserConnection {
                state: browser_freshness(&self.freshness),
                reason: self.freshness.evidence.as_deref(),
                last_connected_at: self.freshness.last_success_at,
            },
            external_signals: self.external_signals.as_ref(),
        }
        .serialize(serializer)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuthContext {
    pub session_id: String,
    pub user_label: Option<String>,
    pub csrf_token: String,
    pub expires_at: DateTime<Utc>,
    pub allowed_initiatives: Vec<InitiativeId>,
}

impl AuthContext {
    pub fn allows(&self, id: &InitiativeId) -> bool {
        self.expires_at > Utc::now()
            && (self.allowed_initiatives.is_empty() || self.allowed_initiatives.contains(id))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BrowserSession {
    pub id: String,
    pub csrf_token: String,
    pub expires_at: DateTime<Utc>,
    pub scope: Vec<InitiativeId>,
}

#[derive(Debug)]
pub struct BrowserSessionIssuer {
    ttl: Duration,
}

impl BrowserSessionIssuer {
    pub fn new(ttl: Duration) -> Self {
        Self { ttl }
    }

    pub fn issue(&self, scope: Vec<InitiativeId>) -> BrowserSession {
        BrowserSession {
            id: Uuid::new_v4().to_string(),
            csrf_token: Uuid::new_v4().to_string(),
            expires_at: Utc::now() + self.ttl,
            scope,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CommandEnvelope {
    pub id: CommandId,
    pub idempotency_key: IdempotencyKey,
    pub correlation_id: CorrelationId,
    pub auth: AuthContext,
    pub expected_revision: Revision,
    pub payload: CommandPayload,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum CommandPayload {
    UpdateMilestone {
        initiative_id: InitiativeId,
        milestone_id: String,
        status: String,
    },
    UpdateStep {
        initiative_id: InitiativeId,
        milestone_id: String,
        step_id: String,
        status: String,
    },
    Checkpoint {
        initiative_id: InitiativeId,
        summary: String,
        blockers: Vec<String>,
        next_actions: Vec<String>,
    },
    SetBlockers {
        initiative_id: InitiativeId,
        blockers: Vec<String>,
    },
    SetNextActions {
        initiative_id: InitiativeId,
        next_actions: Vec<String>,
    },
    StartInitiativeRun {
        initiative_id: InitiativeId,
    },
    RetryLinkedRun {
        initiative_id: InitiativeId,
        run_id: JcodeRunId,
    },
    CancelLinkedRun {
        initiative_id: InitiativeId,
        run_id: JcodeRunId,
    },
    DirectOrcaMutation {
        initiative_id: InitiativeId,
        field: String,
    },
}

impl CommandPayload {
    pub fn initiative_id(&self) -> &InitiativeId {
        match self {
            Self::UpdateMilestone { initiative_id, .. }
            | Self::UpdateStep { initiative_id, .. }
            | Self::Checkpoint { initiative_id, .. }
            | Self::SetBlockers { initiative_id, .. }
            | Self::SetNextActions { initiative_id, .. }
            | Self::StartInitiativeRun { initiative_id }
            | Self::RetryLinkedRun { initiative_id, .. }
            | Self::CancelLinkedRun { initiative_id, .. }
            | Self::DirectOrcaMutation { initiative_id, .. } => initiative_id,
        }
    }

    pub fn is_runtime_command(&self) -> bool {
        matches!(
            self,
            Self::StartInitiativeRun { .. }
                | Self::RetryLinkedRun { .. }
                | Self::CancelLinkedRun { .. }
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CommandState {
    Pending,
    Completed,
    Failed,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CommandResult {
    pub command_id: CommandId,
    pub idempotency_key: IdempotencyKey,
    pub correlation_id: CorrelationId,
    pub state: CommandState,
    pub authoritative: Option<CommandResultPayload>,
    pub error: Option<CommandCenterError>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum CommandResultPayload {
    Initiative {
        initiative: InitiativeProjection,
    },
    RunAccepted {
        run: JcodeRunReference,
        #[serde(default)]
        orca_run_id: Option<OrcaRunId>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        receipt: Option<Box<OrcaLifecycleReceipt>>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RuntimeCommandExecution {
    pub state: CommandState,
    pub payload: Option<CommandResultPayload>,
    pub error: Option<CommandCenterError>,
}

impl RuntimeCommandExecution {
    pub fn failed(error: CommandCenterError) -> Self {
        Self {
            state: CommandState::Failed,
            payload: None,
            error: Some(error),
        }
    }

    pub fn from_lifecycle(run: JcodeRunReference, receipt: OrcaLifecycleReceipt) -> Self {
        let state = match receipt.outcome {
            OrcaMutationOutcome::Ready
            | OrcaMutationOutcome::OutcomeUnknown
            | OrcaMutationOutcome::RecoveryRequired => CommandState::Pending,
            OrcaMutationOutcome::Stopped
            | OrcaMutationOutcome::Abandoned
            | OrcaMutationOutcome::AlreadySettled => CommandState::Completed,
            OrcaMutationOutcome::Failed | OrcaMutationOutcome::Rejected => CommandState::Failed,
        };
        let error = match receipt.outcome {
            OrcaMutationOutcome::OutcomeUnknown => {
                Some(CommandCenterError::OrcaOperationOutcomeUnknown {
                    stage: receipt.stage.clone(),
                })
            }
            OrcaMutationOutcome::RecoveryRequired => {
                Some(CommandCenterError::OrcaOperationRecoveryRequired {
                    stage: receipt.stage.clone(),
                })
            }
            OrcaMutationOutcome::Rejected => Some(CommandCenterError::OrcaPreconditionFailed {
                reason: receipt
                    .last_error
                    .clone()
                    .unwrap_or_else(|| "Orca rejected the lifecycle mutation".to_string()),
            }),
            _ => None,
        };
        let orca_run_id = receipt
            .attempt
            .as_ref()
            .map(|attempt| attempt.run_id.clone());
        Self {
            state,
            payload: Some(CommandResultPayload::RunAccepted {
                run,
                orca_run_id,
                receipt: Some(Box::new(receipt)),
            }),
            error,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum CommandCenterError {
    Unauthorized,
    ReauthenticationRequired,
    Forbidden,
    NotFound {
        entity: String,
    },
    StaleRevision {
        expected: Revision,
        actual: Revision,
    },
    InvalidCommand {
        reason: String,
    },
    UnsupportedCapability {
        capability: String,
    },
    OrcaUnavailable,
    OrcaProfileMismatch {
        reason: String,
    },
    OrcaSchemaMismatch {
        reason: String,
    },
    OrcaIdentityUnresolved {
        reason: String,
    },
    OrcaIdentityDrift {
        reason: String,
    },
    OrcaCoordinatorUnavailable,
    OrcaPreconditionFailed {
        reason: String,
    },
    OrcaOperationOutcomeUnknown {
        stage: String,
    },
    OrcaOperationRecoveryRequired {
        stage: String,
    },
    OrcaReceiptIdentityConflict {
        field: String,
    },
    ReplayScopeMismatch,
    ReplayGap,
    CsrfRejected,
    NonLoopbackRequiresAuthenticatedRemote,
}

impl fmt::Display for CommandCenterError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}
impl std::error::Error for CommandCenterError {}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EventEnvelope {
    pub protocol_version: u16,
    pub stream_id: StreamId,
    pub sequence: u64,
    pub timestamp: DateTime<Utc>,
    pub source: EventSource,
    pub entity_refs: EntityRefs,
    pub payload: EventPayload,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EventSource {
    Jcode,
    OrcaAdapter,
    System,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct EntityRefs {
    pub initiative_id: Option<InitiativeId>,
    pub schedule_id: Option<ScheduleRefId>,
    pub jcode_run_id: Option<JcodeRunId>,
    pub orca_project_id: Option<OrcaProjectId>,
    pub orca_run_id: Option<OrcaRunId>,
    pub orca_task_id: Option<OrcaTaskId>,
    pub orca_dispatch_id: Option<OrcaDispatchId>,
    pub orca_worktree_id: Option<OrcaWorktreeId>,
    pub orca_terminal_id: Option<OrcaTerminalId>,
    pub correlation_id: Option<CorrelationId>,
    pub idempotency_key: Option<IdempotencyKey>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum EventPayload {
    InitiativeUpdated {
        initiative: InitiativeProjection,
    },
    ScheduleUpdated {
        schedule: LinkedScheduleProjection,
    },
    RunUpdated {
        run: JcodeRunReference,
    },
    OrcaObserved {
        reference: OrcaReference,
    },
    CommandUpdated {
        result: CommandResult,
    },
    Unknown {
        name: String,
        requires_snapshot: bool,
    },
}

impl<'de> Deserialize<'de> for EventPayload {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use serde::de::Error as _;

        #[derive(Deserialize)]
        #[serde(tag = "type", rename_all = "snake_case")]
        enum KnownEventPayload {
            InitiativeUpdated { initiative: InitiativeProjection },
            ScheduleUpdated { schedule: LinkedScheduleProjection },
            RunUpdated { run: JcodeRunReference },
            OrcaObserved { reference: OrcaReference },
            CommandUpdated { result: CommandResult },
        }

        let value = serde_json::Value::deserialize(deserializer)?;
        let event_type = value
            .get("type")
            .and_then(serde_json::Value::as_str)
            .ok_or_else(|| D::Error::missing_field("type"))?;

        if event_type == "unknown" {
            let name = value
                .get("name")
                .and_then(serde_json::Value::as_str)
                .ok_or_else(|| D::Error::missing_field("name"))?;
            let requires_snapshot = value
                .get("requires_snapshot")
                .and_then(serde_json::Value::as_bool)
                .ok_or_else(|| D::Error::missing_field("requires_snapshot"))?;
            return Ok(Self::Unknown {
                name: name.to_owned(),
                requires_snapshot,
            });
        }

        let known = match event_type {
            "initiative_updated" | "schedule_updated" | "run_updated" | "orca_observed"
            | "command_updated" => {
                serde_json::from_value::<KnownEventPayload>(value).map_err(D::Error::custom)?
            }
            name => {
                return Ok(Self::Unknown {
                    name: name.to_owned(),
                    requires_snapshot: true,
                });
            }
        };

        Ok(match known {
            KnownEventPayload::InitiativeUpdated { initiative } => {
                Self::InitiativeUpdated { initiative }
            }
            KnownEventPayload::ScheduleUpdated { schedule } => Self::ScheduleUpdated { schedule },
            KnownEventPayload::RunUpdated { run } => Self::RunUpdated { run },
            KnownEventPayload::OrcaObserved { reference } => Self::OrcaObserved { reference },
            KnownEventPayload::CommandUpdated { result } => Self::CommandUpdated { result },
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReplayCursor {
    pub stream_id: StreamId,
    pub sequence: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReplayScope {
    pub principal_session_id: String,
    pub initiative_id: InitiativeId,
    pub orca_run_id: Option<OrcaRunId>,
    pub authorized_until: DateTime<Utc>,
}

impl ReplayScope {
    pub fn is_valid_for(&self, auth: &AuthContext, now: DateTime<Utc>) -> bool {
        self.principal_session_id == auth.session_id
            && auth.allows(&self.initiative_id)
            && self.authorized_until > now
            && auth.expires_at > now
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReplayBatch {
    pub events: Vec<EventEnvelope>,
    pub snapshot_required: bool,
}

pub struct ReplayBuffer {
    stream_id: StreamId,
    scope: Option<ReplayScope>,
    retention: usize,
    next_sequence: u64,
    events: VecDeque<EventEnvelope>,
}

impl ReplayBuffer {
    pub fn new(stream_id: StreamId, retention: usize) -> Self {
        Self {
            stream_id,
            scope: None,
            retention,
            next_sequence: 1,
            events: VecDeque::new(),
        }
    }

    pub fn new_scoped(stream_id: StreamId, scope: ReplayScope, retention: usize) -> Self {
        Self {
            stream_id,
            scope: Some(scope),
            retention,
            next_sequence: 1,
            events: VecDeque::new(),
        }
    }

    pub fn push(
        &mut self,
        source: EventSource,
        entity_refs: EntityRefs,
        payload: EventPayload,
    ) -> EventEnvelope {
        let event = EventEnvelope {
            protocol_version: PROTOCOL_VERSION,
            stream_id: self.stream_id.clone(),
            sequence: self.next_sequence,
            timestamp: Utc::now(),
            source,
            entity_refs,
            payload,
        };
        self.next_sequence += 1;
        self.events.push_back(event.clone());
        while self.events.len() > self.retention {
            self.events.pop_front();
        }
        event
    }

    pub fn replay(&self, cursor: &ReplayCursor) -> Result<ReplayBatch, CommandCenterError> {
        if cursor.stream_id != self.stream_id {
            return Err(CommandCenterError::ReplayScopeMismatch);
        }
        if cursor.sequence >= self.next_sequence {
            return Ok(ReplayBatch {
                events: Vec::new(),
                snapshot_required: false,
            });
        }
        let first = self
            .events
            .front()
            .map(|event| event.sequence)
            .unwrap_or(self.next_sequence);
        if cursor.sequence + 1 < first {
            return Ok(ReplayBatch {
                events: Vec::new(),
                snapshot_required: true,
            });
        }
        Ok(ReplayBatch {
            events: self
                .events
                .iter()
                .filter(|event| event.sequence > cursor.sequence)
                .cloned()
                .collect(),
            snapshot_required: false,
        })
    }

    pub fn replay_authorized(
        &self,
        auth: &AuthContext,
        cursor: &ReplayCursor,
    ) -> Result<ReplayBatch, CommandCenterError> {
        if let Some(scope) = &self.scope
            && !scope.is_valid_for(auth, Utc::now())
        {
            return Err(CommandCenterError::ReplayScopeMismatch);
        }
        self.replay(cursor)
    }

    pub fn rotate(&mut self, stream_id: StreamId) {
        self.stream_id = stream_id;
        self.scope = None;
        self.next_sequence = 1;
        self.events.clear();
    }

    pub fn rotate_scoped(&mut self, stream_id: StreamId, scope: ReplayScope) {
        self.stream_id = stream_id;
        self.scope = Some(scope);
        self.next_sequence = 1;
        self.events.clear();
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandCenterConfig {
    pub enabled: bool,
    pub bind_addr: SocketAddr,
    pub allowed_origins: Vec<String>,
    pub authenticated_remote: bool,
    /// Built SolidStart output served by the daemon. API-only test hosts may omit it.
    pub asset_dir: Option<PathBuf>,
    /// Durable intake authority projected read-only into the browser UI.
    pub decision_inbox_db_path: Option<PathBuf>,
}

impl Default for CommandCenterConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            bind_addr: SocketAddr::from((Ipv4Addr::LOCALHOST, 0)),
            allowed_origins: Vec::new(),
            authenticated_remote: false,
            asset_dir: None,
            decision_inbox_db_path: None,
        }
    }
}

impl CommandCenterConfig {
    pub fn validate(&self) -> Result<(), CommandCenterError> {
        if !self.enabled {
            return Ok(());
        }
        if !self.bind_addr.ip().is_loopback()
            && (!self.authenticated_remote || self.allowed_origins.is_empty())
        {
            return Err(CommandCenterError::NonLoopbackRequiresAuthenticatedRemote);
        }
        Ok(())
    }

    pub fn origin_for_bound_addr(&self, addr: SocketAddr) -> String {
        format!("http://{}", addr)
    }
}

#[derive(Debug, Clone)]
pub struct BrowserSessionStore {
    issuer: Arc<BrowserSessionIssuer>,
    sessions: Arc<Mutex<HashMap<String, BrowserSession>>>,
}

impl BrowserSessionStore {
    pub fn new(ttl: Duration) -> Self {
        Self {
            issuer: Arc::new(BrowserSessionIssuer::new(ttl)),
            sessions: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn bootstrap(&self, scope: Vec<InitiativeId>) -> BrowserSession {
        let session = self.issuer.issue(scope);
        self.sessions
            .lock()
            .expect("session lock poisoned")
            .insert(session.id.clone(), session.clone());
        session
    }

    pub fn insert(&self, session: BrowserSession) {
        self.sessions
            .lock()
            .expect("session lock poisoned")
            .insert(session.id.clone(), session);
    }

    pub fn get(&self, id: &str) -> Result<BrowserSession, CommandCenterError> {
        let session = self
            .sessions
            .lock()
            .expect("session lock poisoned")
            .get(id)
            .cloned()
            .ok_or(CommandCenterError::Unauthorized)?;
        if session.expires_at <= Utc::now() {
            return Err(CommandCenterError::ReauthenticationRequired);
        }
        Ok(session)
    }
}

impl From<&BrowserSession> for AuthContext {
    fn from(session: &BrowserSession) -> Self {
        Self {
            session_id: session.id.clone(),
            user_label: Some("local-browser".to_string()),
            csrf_token: session.csrf_token.clone(),
            expires_at: session.expires_at,
            allowed_initiatives: session.scope.clone(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RequestGuard {
    pub origin: Option<String>,
    pub method: String,
    pub content_type: Option<String>,
    pub csrf_token: Option<String>,
}

pub fn validate_mutation_request(
    config: &CommandCenterConfig,
    session: &BrowserSession,
    request: &RequestGuard,
) -> Result<(), CommandCenterError> {
    if session.expires_at <= Utc::now() {
        return Err(CommandCenterError::ReauthenticationRequired);
    }
    let origin = request
        .origin
        .as_deref()
        .ok_or(CommandCenterError::CsrfRejected)?;
    if !config.allowed_origins.is_empty()
        && !config
            .allowed_origins
            .iter()
            .any(|allowed| allowed == origin)
    {
        return Err(CommandCenterError::CsrfRejected);
    }
    if request.method != "POST" {
        return Err(CommandCenterError::CsrfRejected);
    }
    if request.content_type.as_deref() != Some("application/json") {
        return Err(CommandCenterError::CsrfRejected);
    }
    if request.csrf_token.as_deref() != Some(session.csrf_token.as_str()) {
        return Err(CommandCenterError::CsrfRejected);
    }
    Ok(())
}

#[async_trait]
pub trait InitiativeRepository: Send + Sync {
    async fn list(&self, auth: &AuthContext) -> Result<Vec<(Goal, Revision)>, CommandCenterError>;
    async fn get(
        &self,
        auth: &AuthContext,
        id: &InitiativeId,
    ) -> Result<(Goal, Revision), CommandCenterError>;
    async fn save(
        &self,
        auth: &AuthContext,
        goal: Goal,
        expected: Revision,
    ) -> Result<(Goal, Revision), CommandCenterError>;
}

#[async_trait]
pub trait ScheduleProjectionSource: Send + Sync {
    async fn schedules_for(
        &self,
        id: &InitiativeId,
    ) -> Result<Vec<LinkedScheduleProjection>, CommandCenterError>;
}

#[async_trait]
pub trait RunProjectionSource: Send + Sync {
    async fn runs_for(
        &self,
        id: &InitiativeId,
    ) -> Result<Vec<JcodeRunReference>, CommandCenterError>;
}

#[async_trait]
pub trait ExternalSignalProjectionSource: Send + Sync {
    async fn projection(&self) -> Result<serde_json::Value, String>;
}

struct EmptyExternalSignalProjection;

#[async_trait]
impl ExternalSignalProjectionSource for EmptyExternalSignalProjection {
    async fn projection(&self) -> Result<serde_json::Value, String> {
        Err("external signal projection unavailable".to_string())
    }
}

#[async_trait]
pub trait OrcaAdapter: Send + Sync {
    async fn capabilities(&self) -> Result<RuntimeMutationCapabilities, CommandCenterError> {
        Ok(RuntimeMutationCapabilities::unavailable())
    }
    async fn observe(&self, id: &InitiativeId) -> Result<OrcaReference, CommandCenterError>;
    async fn canonical_placement(
        &self,
        id: &InitiativeId,
    ) -> Result<OrcaCanonicalPlacement, CommandCenterError>;
    async fn start_initiative_run(
        &self,
        request: StartInitiativeRunRequest,
    ) -> RuntimeCommandExecution;
    async fn retry_linked_run(&self, request: RetryLinkedRunRequest) -> RuntimeCommandExecution;
    async fn cancel_linked_run(&self, request: CancelLinkedRunRequest) -> RuntimeCommandExecution;
}

pub struct CommandCenterService<R, S, P, O> {
    initiatives: R,
    schedules: S,
    runs: P,
    orca: O,
    idempotency: Arc<Mutex<HashMap<(String, String), CommandResult>>>,
    external_signals: Arc<dyn ExternalSignalProjectionSource>,
}

impl<R, S, P, O> CommandCenterService<R, S, P, O> {
    pub fn new(initiatives: R, schedules: S, runs: P, orca: O) -> Self {
        Self {
            initiatives,
            schedules,
            runs,
            orca,
            idempotency: Arc::new(Mutex::new(HashMap::new())),
            external_signals: Arc::new(EmptyExternalSignalProjection),
        }
    }

    pub fn with_external_signal_projection(
        mut self,
        source: Arc<dyn ExternalSignalProjectionSource>,
    ) -> Self {
        self.external_signals = source;
        self
    }
}

impl<R, S, P, O> CommandCenterService<R, S, P, O>
where
    R: InitiativeRepository,
    S: ScheduleProjectionSource,
    P: RunProjectionSource,
    O: OrcaAdapter,
{
    pub async fn list_initiatives(
        &self,
        auth: &AuthContext,
    ) -> Result<InitiativeListSnapshot, CommandCenterError> {
        let initiatives = self
            .initiatives
            .list(auth)
            .await?
            .into_iter()
            .map(InitiativeProjection::from)
            .collect();
        Ok(InitiativeListSnapshot {
            metadata: ProtocolMetadata::default(),
            revision: Revision(0),
            initiatives,
            freshness: Freshness::fresh(),
        })
    }

    pub async fn snapshot(
        &self,
        auth: &AuthContext,
        id: &InitiativeId,
    ) -> Result<CommandCenterSnapshot, CommandCenterError> {
        if !auth.allows(id) {
            return Err(CommandCenterError::Forbidden);
        }
        let (goal, revision) = self.initiatives.get(auth, id).await?;
        let orca = self
            .orca
            .observe(id)
            .await
            .unwrap_or_else(|err| OrcaReference {
                project_id: None,
                runtime_id: None,
                run_id: None,
                task_ids: Vec::new(),
                dispatch_ids: Vec::new(),
                worktree_ids: Vec::new(),
                worker_ids: Vec::new(),
                terminal_ids: Vec::new(),
                gate_ids: Vec::new(),
                correlation_ids: Vec::new(),
                idempotency_keys: Vec::new(),
                last_observed_at: None,
                freshness: Freshness::unavailable(err.to_string()),
            });
        let runtime_capabilities = self.orca.capabilities().await.unwrap_or_default();
        let external_signals = self.external_signals.projection().await.ok();
        Ok(CommandCenterSnapshot {
            metadata: ProtocolMetadata::default(),
            revision,
            initiative: InitiativeProjection::from((goal, revision)),
            schedules: self.schedules.schedules_for(id).await?,
            runs: self.runs.runs_for(id).await?,
            orca,
            freshness: Freshness::fresh(),
            available_actions: AvailableActions {
                update_initiative: true,
                checkpoint_initiative: true,
                manage_blockers: true,
                manage_next_actions: true,
                start_initiative_run: runtime_capabilities.start_initiative_run,
                retry_linked_run: runtime_capabilities.retry_linked_run,
                cancel_linked_run: runtime_capabilities.cancel_linked_run,
            },
            external_signals,
        })
    }

    pub async fn execute(&self, envelope: CommandEnvelope) -> CommandResult {
        let key = (
            envelope.auth.session_id.clone(),
            envelope.idempotency_key.0.clone(),
        );
        if let Some(existing) = self
            .idempotency
            .lock()
            .expect("idempotency lock poisoned")
            .get(&key)
            .cloned()
        {
            return existing;
        }
        let result = self.execute_once(envelope).await;
        self.idempotency
            .lock()
            .expect("idempotency lock poisoned")
            .insert(key, result.clone());
        result
    }

    async fn execute_once(&self, envelope: CommandEnvelope) -> CommandResult {
        let base = |state, authoritative, error| CommandResult {
            command_id: envelope.id.clone(),
            idempotency_key: envelope.idempotency_key.clone(),
            correlation_id: envelope.correlation_id.clone(),
            state,
            authoritative,
            error,
        };
        let initiative_id = envelope.payload.initiative_id().clone();
        if !envelope.auth.allows(&initiative_id) {
            return base(
                CommandState::Failed,
                None,
                Some(CommandCenterError::Forbidden),
            );
        }
        if matches!(envelope.payload, CommandPayload::DirectOrcaMutation { .. }) {
            return base(
                CommandState::Failed,
                None,
                Some(CommandCenterError::InvalidCommand {
                    reason: "direct Orca mutations are outside Jcode authority".to_string(),
                }),
            );
        }
        if envelope.payload.is_runtime_command() {
            let execution = self.execute_runtime_command(&envelope).await;
            return base(execution.state, execution.payload, execution.error);
        }
        match self.apply_initiative_command(&envelope).await {
            Ok(initiative) => base(
                CommandState::Completed,
                Some(CommandResultPayload::Initiative { initiative }),
                None,
            ),
            Err(error) => base(CommandState::Failed, None, Some(error)),
        }
    }

    async fn execute_runtime_command(&self, envelope: &CommandEnvelope) -> RuntimeCommandExecution {
        let initiative_id = envelope.payload.initiative_id();
        let (goal, actual) = match self.initiatives.get(&envelope.auth, initiative_id).await {
            Ok(value) => value,
            Err(error) => return RuntimeCommandExecution::failed(error),
        };
        if actual != envelope.expected_revision {
            return RuntimeCommandExecution::failed(CommandCenterError::StaleRevision {
                expected: envelope.expected_revision,
                actual,
            });
        }
        let capabilities = match self.orca.capabilities().await {
            Ok(value) => value,
            Err(error) => return RuntimeCommandExecution::failed(error),
        };
        let context = |jcode_attempt_id| OrcaMutationContext {
            command_id: envelope.id.clone(),
            idempotency_key: envelope.idempotency_key.clone(),
            correlation_id: envelope.correlation_id.clone(),
            initiative_id: initiative_id.clone(),
            jcode_attempt_id,
        };
        match &envelope.payload {
            CommandPayload::StartInitiativeRun { initiative_id } => {
                if !capabilities.start_initiative_run {
                    return RuntimeCommandExecution::failed(
                        CommandCenterError::UnsupportedCapability {
                            capability: "orca.command_center.start_initiative_run".to_string(),
                        },
                    );
                }
                let placement = match self.orca.canonical_placement(initiative_id).await {
                    Ok(value) => value,
                    Err(error) => return RuntimeCommandExecution::failed(error),
                };
                self.orca
                    .start_initiative_run(StartInitiativeRunRequest {
                        context: context(JcodeRunId(envelope.id.0.clone())),
                        objective: goal.title,
                        task_spec: goal.description,
                        placement,
                    })
                    .await
            }
            CommandPayload::RetryLinkedRun {
                initiative_id,
                run_id,
            } => {
                if !capabilities.retry_linked_run {
                    return RuntimeCommandExecution::failed(
                        CommandCenterError::UnsupportedCapability {
                            capability: "orca.command_center.retry_linked_run".to_string(),
                        },
                    );
                }
                let prior = match self.runtime_run(initiative_id, run_id).await {
                    Ok(value) => value,
                    Err(error) => return RuntimeCommandExecution::failed(error),
                };
                let placement = match self.orca.canonical_placement(initiative_id).await {
                    Ok(value) => value,
                    Err(error) => return RuntimeCommandExecution::failed(error),
                };
                if prior.worktree_id.as_ref() != Some(&placement.worktree_id) {
                    return RuntimeCommandExecution::failed(
                        CommandCenterError::OrcaIdentityDrift {
                            reason: "retry placement no longer matches the prior attempt worktree"
                                .to_string(),
                        },
                    );
                }
                let Some(orca_run_id) = prior.orca_run_id else {
                    return RuntimeCommandExecution::failed(missing_attempt_identity(
                        "orca_run_id",
                    ));
                };
                let Some(orca_task_id) = prior.orca_task_id else {
                    return RuntimeCommandExecution::failed(missing_attempt_identity(
                        "orca_task_id",
                    ));
                };
                let Some(retry_of_dispatch_id) = prior.orca_dispatch_id else {
                    return RuntimeCommandExecution::failed(missing_attempt_identity(
                        "orca_dispatch_id",
                    ));
                };
                self.orca
                    .retry_linked_run(RetryLinkedRunRequest {
                        context: context(JcodeRunId(envelope.id.0.clone())),
                        prior_jcode_attempt_id: run_id.clone(),
                        orca_run_id,
                        orca_task_id,
                        retry_of_dispatch_id,
                        placement,
                    })
                    .await
            }
            CommandPayload::CancelLinkedRun {
                initiative_id,
                run_id,
            } => {
                if !capabilities.cancel_linked_run {
                    return RuntimeCommandExecution::failed(
                        CommandCenterError::UnsupportedCapability {
                            capability: "orca.command_center.cancel_linked_run".to_string(),
                        },
                    );
                }
                let target = match self.runtime_run(initiative_id, run_id).await {
                    Ok(value) => value,
                    Err(error) => return RuntimeCommandExecution::failed(error),
                };
                let Some(orca_run_id) = target.orca_run_id else {
                    return RuntimeCommandExecution::failed(missing_attempt_identity(
                        "orca_run_id",
                    ));
                };
                let Some(orca_task_id) = target.orca_task_id else {
                    return RuntimeCommandExecution::failed(missing_attempt_identity(
                        "orca_task_id",
                    ));
                };
                let Some(target_dispatch_id) = target.orca_dispatch_id else {
                    return RuntimeCommandExecution::failed(missing_attempt_identity(
                        "orca_dispatch_id",
                    ));
                };
                self.orca
                    .cancel_linked_run(CancelLinkedRunRequest {
                        context: context(run_id.clone()),
                        target_jcode_attempt_id: run_id.clone(),
                        orca_run_id,
                        orca_task_id,
                        target_dispatch_id,
                    })
                    .await
            }
            _ => RuntimeCommandExecution::failed(CommandCenterError::UnsupportedCapability {
                capability: "not_runtime".to_string(),
            }),
        }
    }

    async fn runtime_run(
        &self,
        initiative_id: &InitiativeId,
        run_id: &JcodeRunId,
    ) -> Result<JcodeRunReference, CommandCenterError> {
        self.runs
            .runs_for(initiative_id)
            .await?
            .into_iter()
            .find(|run| &run.id == run_id)
            .ok_or_else(|| CommandCenterError::NotFound {
                entity: format!("Jcode run {}", run_id.0),
            })
    }

    async fn apply_initiative_command(
        &self,
        envelope: &CommandEnvelope,
    ) -> Result<InitiativeProjection, CommandCenterError> {
        let id = envelope.payload.initiative_id();
        let (mut goal, actual) = self.initiatives.get(&envelope.auth, id).await?;
        if actual != envelope.expected_revision {
            return Err(CommandCenterError::StaleRevision {
                expected: envelope.expected_revision,
                actual,
            });
        }
        match &envelope.payload {
            CommandPayload::UpdateMilestone {
                milestone_id,
                status,
                ..
            } => update_milestone(&mut goal.milestones, milestone_id, status)?,
            CommandPayload::UpdateStep {
                milestone_id,
                step_id,
                status,
                ..
            } => update_step(&mut goal.milestones, milestone_id, step_id, status)?,
            CommandPayload::Checkpoint {
                summary,
                blockers,
                next_actions,
                ..
            } => {
                goal.updates.push(jcode_task_types::GoalUpdate {
                    at: Utc::now(),
                    summary: summary.clone(),
                });
                goal.blockers = blockers.clone();
                goal.next_steps = next_actions.clone();
            }
            CommandPayload::SetBlockers { blockers, .. } => goal.blockers = blockers.clone(),
            CommandPayload::SetNextActions { next_actions, .. } => {
                goal.next_steps = next_actions.clone()
            }
            _ => {
                return Err(CommandCenterError::UnsupportedCapability {
                    capability: "initiative_command".to_string(),
                });
            }
        }
        goal.updated_at = Utc::now();
        self.initiatives
            .save(&envelope.auth, goal, envelope.expected_revision)
            .await
            .map(InitiativeProjection::from)
    }
}

fn missing_attempt_identity(field: &str) -> CommandCenterError {
    CommandCenterError::OrcaPreconditionFailed {
        reason: format!("selected Jcode attempt has no authoritative {field}"),
    }
}

fn update_milestone(
    milestones: &mut [GoalMilestone],
    milestone_id: &str,
    status: &str,
) -> Result<(), CommandCenterError> {
    let milestone = milestones
        .iter_mut()
        .find(|milestone| milestone.id == milestone_id)
        .ok_or_else(|| CommandCenterError::NotFound {
            entity: "milestone".to_string(),
        })?;
    milestone.status = status.to_string();
    Ok(())
}

fn update_step(
    milestones: &mut [GoalMilestone],
    milestone_id: &str,
    step_id: &str,
    status: &str,
) -> Result<(), CommandCenterError> {
    let milestone = milestones
        .iter_mut()
        .find(|milestone| milestone.id == milestone_id)
        .ok_or_else(|| CommandCenterError::NotFound {
            entity: "milestone".to_string(),
        })?;
    let step = milestone
        .steps
        .iter_mut()
        .find(|step| step.id == step_id)
        .ok_or_else(|| CommandCenterError::NotFound {
            entity: "step".to_string(),
        })?;
    step.status = status.to_string();
    Ok(())
}

pub fn is_loopback_safe(addr: IpAddr) -> bool {
    addr.is_loopback()
}

#[async_trait]
pub trait CommandCenterApi: Send + Sync + 'static {
    async fn list_initiatives(
        &self,
        auth: &AuthContext,
    ) -> Result<InitiativeListSnapshot, CommandCenterError>;
    async fn snapshot(
        &self,
        auth: &AuthContext,
        id: &InitiativeId,
    ) -> Result<CommandCenterSnapshot, CommandCenterError>;
    async fn execute(&self, envelope: CommandEnvelope) -> CommandResult;
    fn replay(&self, cursor: &ReplayCursor) -> Result<ReplayBatch, CommandCenterError>;
}

pub struct CommandCenterRuntime<R, S, P, O> {
    service: CommandCenterService<R, S, P, O>,
    replay: Arc<Mutex<ReplayBuffer>>,
}

impl<R, S, P, O> CommandCenterRuntime<R, S, P, O> {
    pub fn new(service: CommandCenterService<R, S, P, O>, stream_id: StreamId) -> Self {
        Self {
            service,
            replay: Arc::new(Mutex::new(ReplayBuffer::new(stream_id, 256))),
        }
    }
}

#[async_trait]
impl<R, S, P, O> CommandCenterApi for CommandCenterRuntime<R, S, P, O>
where
    R: InitiativeRepository + 'static,
    S: ScheduleProjectionSource + 'static,
    P: RunProjectionSource + 'static,
    O: OrcaAdapter + 'static,
{
    async fn list_initiatives(
        &self,
        auth: &AuthContext,
    ) -> Result<InitiativeListSnapshot, CommandCenterError> {
        self.service.list_initiatives(auth).await
    }

    async fn snapshot(
        &self,
        auth: &AuthContext,
        id: &InitiativeId,
    ) -> Result<CommandCenterSnapshot, CommandCenterError> {
        self.service.snapshot(auth, id).await
    }

    async fn execute(&self, envelope: CommandEnvelope) -> CommandResult {
        let initiative_id = envelope.payload.initiative_id().clone();
        let result = self.service.execute(envelope).await;
        self.replay.lock().expect("replay lock poisoned").push(
            EventSource::Jcode,
            EntityRefs {
                initiative_id: Some(initiative_id),
                ..EntityRefs::default()
            },
            EventPayload::CommandUpdated {
                result: result.clone(),
            },
        );
        result
    }

    fn replay(&self, cursor: &ReplayCursor) -> Result<ReplayBatch, CommandCenterError> {
        self.replay
            .lock()
            .expect("replay lock poisoned")
            .replay(cursor)
    }
}

#[derive(Clone)]
pub struct CommandCenterHttpState {
    config: CommandCenterConfig,
    sessions: BrowserSessionStore,
    api: Arc<dyn CommandCenterApi>,
    mx_health: Option<Arc<dyn mx_health::MxHealthSource>>,
}

pub struct CommandCenterHttpHost {
    addr: SocketAddr,
    shutdown: Option<oneshot::Sender<()>>,
    task: JoinHandle<Result<(), std::io::Error>>,
}

impl CommandCenterHttpHost {
    pub fn addr(&self) -> SocketAddr {
        self.addr
    }

    pub async fn shutdown(mut self) -> Result<(), std::io::Error> {
        if let Some(tx) = self.shutdown.take() {
            let _ = tx.send(());
        }
        self.task.await.expect("command-center host task panicked")
    }
}

pub async fn spawn_command_center_http_host(
    config: CommandCenterConfig,
    sessions: BrowserSessionStore,
    api: Arc<dyn CommandCenterApi>,
) -> Result<Option<CommandCenterHttpHost>, CommandCenterError> {
    spawn_command_center_http_host_with_mx(config, sessions, api, None).await
}

pub async fn spawn_command_center_http_host_with_mx(
    mut config: CommandCenterConfig,
    sessions: BrowserSessionStore,
    api: Arc<dyn CommandCenterApi>,
    mx_health: Option<Arc<dyn mx_health::MxHealthSource>>,
) -> Result<Option<CommandCenterHttpHost>, CommandCenterError> {
    config.validate()?;
    if !config.enabled {
        return Ok(None);
    }
    let listener = TcpListener::bind(config.bind_addr).await.map_err(|err| {
        CommandCenterError::InvalidCommand {
            reason: err.to_string(),
        }
    })?;
    let addr = listener
        .local_addr()
        .map_err(|err| CommandCenterError::InvalidCommand {
            reason: err.to_string(),
        })?;
    if config.allowed_origins.is_empty() {
        config
            .allowed_origins
            .push(config.origin_for_bound_addr(addr));
    }
    let state = CommandCenterHttpState {
        config,
        sessions,
        api,
        mx_health,
    };
    let (tx, rx) = oneshot::channel();
    let app = command_center_router(state);
    let task = tokio::spawn(async move {
        axum::serve(listener, app)
            .with_graceful_shutdown(async move {
                let _ = rx.await;
            })
            .await
    });
    Ok(Some(CommandCenterHttpHost {
        addr,
        shutdown: Some(tx),
        task,
    }))
}

pub fn command_center_router(state: CommandCenterHttpState) -> Router {
    let asset_dir = state.config.asset_dir.clone();
    let router = Router::new()
        .route("/api/command-center/bootstrap", post(bootstrap_handler))
        .route("/api/command-center/initiatives", get(list_handler))
        .route(
            "/api/command-center/initiatives/{id}/snapshot",
            get(snapshot_handler),
        )
        .route("/api/command-center/commands", post(command_handler))
        .route("/api/command-center/replay", get(replay_handler))
        .route(
            "/api/command-center/decision-inbox",
            get(decision_inbox_handler),
        )
        .route("/api/command-center/mx-health", get(mx_health_handler))
        .with_state(state);
    let router = if let Some(asset_dir) = asset_dir {
        let index = asset_dir.join("index.html");
        router.fallback_service(
            ServeDir::new(asset_dir)
                .append_index_html_on_directories(true)
                .fallback(ServeFile::new(index)),
        )
    } else {
        router
    };
    router
        .layer(SetResponseHeaderLayer::if_not_present(
            header::X_FRAME_OPTIONS,
            HeaderValue::from_static("DENY"),
        ))
        .layer(SetResponseHeaderLayer::if_not_present(
            header::X_CONTENT_TYPE_OPTIONS,
            HeaderValue::from_static("nosniff"),
        ))
        .layer(SetResponseHeaderLayer::if_not_present(
            header::REFERRER_POLICY,
            HeaderValue::from_static("no-referrer"),
        ))
        .layer(SetResponseHeaderLayer::if_not_present(
            header::CONTENT_SECURITY_POLICY,
            HeaderValue::from_static(
                "default-src 'self'; script-src 'self' 'unsafe-inline'; style-src 'self' 'unsafe-inline'; connect-src 'self'; frame-ancestors 'none'",
            ),
        ))
}

fn session_from_headers(
    state: &CommandCenterHttpState,
    headers: &HeaderMap,
) -> Result<BrowserSession, CommandCenterError> {
    let value = headers
        .get(header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .ok_or(CommandCenterError::Unauthorized)?;
    state.sessions.get(value)
}

fn request_guard(headers: &HeaderMap, method: Method) -> RequestGuard {
    RequestGuard {
        origin: headers
            .get(header::ORIGIN)
            .and_then(|v| v.to_str().ok())
            .map(str::to_string),
        method: method.as_str().to_string(),
        content_type: headers
            .get(header::CONTENT_TYPE)
            .and_then(|v| v.to_str().ok())
            .map(|v| v.split(';').next().unwrap_or(v).to_string()),
        csrf_token: headers
            .get("x-csrf-token")
            .and_then(|v| v.to_str().ok())
            .map(str::to_string),
    }
}

fn error_response(error: CommandCenterError) -> Response {
    let status = match error {
        CommandCenterError::Unauthorized => StatusCode::UNAUTHORIZED,
        CommandCenterError::ReauthenticationRequired => StatusCode::UNAUTHORIZED,
        CommandCenterError::Forbidden
        | CommandCenterError::CsrfRejected
        | CommandCenterError::ReplayScopeMismatch => StatusCode::FORBIDDEN,
        CommandCenterError::NotFound { .. } => StatusCode::NOT_FOUND,
        _ => StatusCode::BAD_REQUEST,
    };
    (status, Json(error)).into_response()
}

#[derive(Debug, Deserialize)]
struct BootstrapRequest {
    scope: Option<Vec<String>>,
}

async fn bootstrap_handler(
    State(state): State<CommandCenterHttpState>,
    headers: HeaderMap,
    Json(body): Json<BootstrapRequest>,
) -> Response {
    let guard = request_guard(&headers, Method::POST);
    if guard.origin.as_ref().is_some_and(|origin| {
        !state.config.allowed_origins.is_empty() && !state.config.allowed_origins.contains(origin)
    }) {
        return error_response(CommandCenterError::CsrfRejected);
    }
    let session = state.sessions.bootstrap(
        body.scope
            .unwrap_or_default()
            .into_iter()
            .map(InitiativeId)
            .collect(),
    );
    Json(session).into_response()
}

async fn list_handler(State(state): State<CommandCenterHttpState>, headers: HeaderMap) -> Response {
    match session_from_headers(&state, &headers) {
        Ok(session) => match state
            .api
            .list_initiatives(&AuthContext::from(&session))
            .await
        {
            Ok(snapshot) => Json(snapshot).into_response(),
            Err(error) => error_response(error),
        },
        Err(error) => error_response(error),
    }
}

async fn snapshot_handler(
    State(state): State<CommandCenterHttpState>,
    Path(id): Path<String>,
    headers: HeaderMap,
) -> Response {
    match session_from_headers(&state, &headers) {
        Ok(session) => match state
            .api
            .snapshot(&AuthContext::from(&session), &InitiativeId(id))
            .await
        {
            Ok(snapshot) => Json(snapshot).into_response(),
            Err(error) => error_response(error),
        },
        Err(error) => error_response(error),
    }
}

async fn decision_inbox_handler(
    State(state): State<CommandCenterHttpState>,
    headers: HeaderMap,
) -> Response {
    if let Err(error) = session_from_headers(&state, &headers) {
        return error_response(error);
    }
    let items = match &state.config.decision_inbox_db_path {
        Some(path) if path.exists() => {
            match jcode_intake_types::SqliteIntakeStore::open(path, None)
                .and_then(|store| store.decision_inbox_items())
            {
                Ok(items) => items,
                Err(_) => {
                    return error_response(CommandCenterError::InvalidCommand {
                        reason: "decision inbox is temporarily unavailable".to_owned(),
                    });
                }
            }
        }
        _ => Vec::new(),
    };
    Json(jcode_intake_types::DecisionInboxSnapshot {
        generated_at: Utc::now(),
        items,
    })
    .into_response()
}

async fn mx_health_handler(
    State(state): State<CommandCenterHttpState>,
    headers: HeaderMap,
) -> Response {
    // Authenticate before touching the adapter. This keeps an unauthenticated
    // browser request from probing MX or learning whether it is configured.
    if let Err(error) = session_from_headers(&state, &headers) {
        return error_response(error);
    }
    let projection = match state.mx_health {
        Some(source) => source.read().await,
        None => mx_health::MxHealthProjection::unconfigured(Utc::now()),
    };
    Json(projection).into_response()
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct BrowserCommandEnvelope {
    idempotency_key: String,
    payload: CommandPayload,
}

async fn command_handler(
    State(state): State<CommandCenterHttpState>,
    headers: HeaderMap,
    Json(body): Json<BrowserCommandEnvelope>,
) -> Response {
    let session = match session_from_headers(&state, &headers) {
        Ok(s) => s,
        Err(e) => return error_response(e),
    };
    if let Err(error) = validate_mutation_request(
        &state.config,
        &session,
        &request_guard(&headers, Method::POST),
    ) {
        return error_response(error);
    }
    let expected_revision = headers
        .get("x-expected-revision")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.parse::<u64>().ok())
        .map(Revision)
        .unwrap_or(Revision(0));
    let envelope = CommandEnvelope {
        id: CommandId(Uuid::new_v4().to_string()),
        idempotency_key: IdempotencyKey(body.idempotency_key),
        correlation_id: CorrelationId(Uuid::new_v4().to_string()),
        auth: AuthContext::from(&session),
        expected_revision,
        payload: body.payload,
    };
    Json(state.api.execute(envelope).await).into_response()
}

#[derive(Debug, Deserialize)]
struct ReplayQuery {
    stream_id: String,
    sequence: u64,
}

async fn replay_handler(
    State(state): State<CommandCenterHttpState>,
    headers: HeaderMap,
    Query(query): Query<ReplayQuery>,
) -> Response {
    if let Err(error) = session_from_headers(&state, &headers) {
        return error_response(error);
    }
    match state.api.replay(&ReplayCursor {
        stream_id: StreamId(query.stream_id),
        sequence: query.sequence,
    }) {
        Ok(batch) => Json(batch).into_response(),
        Err(error) => error_response(error),
    }
}

pub fn generated_typescript_contract() -> &'static str {
    include_str!("../../../apps/command-center/src/generated/command-center-contract.ts")
}

pub fn write_typescript_contract(out_dir: &std::path::Path) -> std::io::Result<()> {
    std::fs::create_dir_all(out_dir)?;
    std::fs::write(
        out_dir.join("command-center-contract.ts"),
        generated_typescript_contract(),
    )?;
    mx_health::write_typescript_contract(out_dir)
}

#[cfg(test)]
mod tests {
    use super::*;
    use jcode_task_types::{GoalScope, GoalStep};

    #[test]
    fn protocol_ids_are_distinct_owned_types_with_scalar_json() {
        use std::any::TypeId;

        assert_ne!(TypeId::of::<InitiativeId>(), TypeId::of::<JcodeRunId>());
        assert_ne!(TypeId::of::<JcodeRunId>(), TypeId::of::<OrcaRunId>());
        assert_ne!(TypeId::of::<CommandId>(), TypeId::of::<IdempotencyKey>());
        assert_ne!(
            TypeId::of::<OrcaProjectId>(),
            TypeId::of::<OrcaRepositoryId>()
        );
        assert_ne!(TypeId::of::<OrcaHostId>(), TypeId::of::<OrcaHostSetupId>());
        assert_ne!(TypeId::of::<OrcaRequestId>(), TypeId::of::<CommandId>());

        let initiative = InitiativeId("initiative-1".into());
        assert_eq!(serde_json::to_value(&initiative).unwrap(), "initiative-1");
        assert_eq!(
            serde_json::from_value::<InitiativeId>(serde_json::json!("initiative-1")).unwrap(),
            initiative
        );
    }

    #[test]
    fn canonical_placement_and_launcher_use_typed_scalar_ids() {
        let value = serde_json::to_value(test_placement()).unwrap();

        assert_eq!(value["project_id"], "project-1");
        assert_eq!(value["repository_id"], "repository-1");
        assert_eq!(value["host_setup_id"], "host-setup-1");
        assert_eq!(value["host_id"], "host-1");
        assert_eq!(value["launcher"]["type"], "agent");
        assert_eq!(value["launcher"]["agent"], "codex");
    }

    #[test]
    fn legacy_jcode_run_reference_defaults_new_attempt_fields() {
        let run: JcodeRunReference = serde_json::from_value(serde_json::json!({
            "id": "run-legacy",
            "initiative_id": "initiative-1",
            "orca_run_id": "orca-run-1",
            "status": "ready",
            "created_at": "2026-08-17T13:00:00Z",
            "updated_at": "2026-08-17T13:00:01Z"
        }))
        .unwrap();

        assert_eq!(run.orca_task_id, None);
        assert_eq!(run.orca_dispatch_id, None);
        assert_eq!(run.retry_of_jcode_run_id, None);
        assert_eq!(run.retry_of_dispatch_id, None);
        assert_eq!(run.worktree_id, None);
        assert_eq!(run.terminal_id, None);
    }

    #[test]
    fn generated_contract_preserves_id_ownership() {
        let contract = generated_typescript_contract();

        for declaration in [
            "export type InitiativeId = Brand<string, \"InitiativeId\">;",
            "export type JcodeRunId = Brand<string, \"JcodeRunId\">;",
            "export type OrcaRunId = Brand<string, \"OrcaRunId\">;",
            "export type OrcaRepositoryId = Brand<string, \"OrcaRepositoryId\">;",
            "export type OrcaHostSetupId = Brand<string, \"OrcaHostSetupId\">;",
            "export type OrcaHostId = Brand<string, \"OrcaHostId\">;",
            "export type OrcaRequestId = Brand<string, \"OrcaRequestId\">;",
            "export type StreamId = Brand<string, \"StreamId\">;",
            "export type IdempotencyKey = Brand<string, \"IdempotencyKey\">;",
        ] {
            assert!(contract.contains(declaration), "missing {declaration}");
        }
        assert!(contract.contains("streamId: StreamId;"));
        assert!(contract.contains("initiativeId: InitiativeId;"));
        assert!(contract.contains("runId?: JcodeRunId;"));
        assert!(contract.contains("orcaRunId?: OrcaRunId;"));
        assert!(contract.contains("idempotencyKey: IdempotencyKey;"));
    }

    #[derive(Clone)]
    struct MemoryRepo(Arc<Mutex<(Goal, Revision)>>);

    #[async_trait]
    impl InitiativeRepository for MemoryRepo {
        async fn list(
            &self,
            auth: &AuthContext,
        ) -> Result<Vec<(Goal, Revision)>, CommandCenterError> {
            let (goal, revision) = self.0.lock().unwrap().clone();
            if auth.allows(&InitiativeId(goal.id.clone())) {
                Ok(vec![(goal, revision)])
            } else {
                Ok(vec![])
            }
        }
        async fn get(
            &self,
            auth: &AuthContext,
            id: &InitiativeId,
        ) -> Result<(Goal, Revision), CommandCenterError> {
            if !auth.allows(id) {
                return Err(CommandCenterError::Forbidden);
            }
            let (goal, revision) = self.0.lock().unwrap().clone();
            if goal.id == id.0 {
                Ok((goal, revision))
            } else {
                Err(CommandCenterError::NotFound {
                    entity: "initiative".into(),
                })
            }
        }
        async fn save(
            &self,
            _auth: &AuthContext,
            goal: Goal,
            expected: Revision,
        ) -> Result<(Goal, Revision), CommandCenterError> {
            let mut guard = self.0.lock().unwrap();
            if guard.1 != expected {
                return Err(CommandCenterError::StaleRevision {
                    expected,
                    actual: guard.1,
                });
            }
            guard.0 = goal;
            guard.1 = Revision(guard.1.0 + 1);
            Ok(guard.clone())
        }
    }

    struct EmptySchedules;
    #[async_trait]
    impl ScheduleProjectionSource for EmptySchedules {
        async fn schedules_for(
            &self,
            _id: &InitiativeId,
        ) -> Result<Vec<LinkedScheduleProjection>, CommandCenterError> {
            Ok(vec![])
        }
    }

    struct EmptyRuns;
    #[async_trait]
    impl RunProjectionSource for EmptyRuns {
        async fn runs_for(
            &self,
            id: &InitiativeId,
        ) -> Result<Vec<JcodeRunReference>, CommandCenterError> {
            let now = Utc::now();
            Ok(vec![JcodeRunReference {
                id: JcodeRunId("run-1".into()),
                initiative_id: id.clone(),
                orca_run_id: Some(OrcaRunId("orca-run-1".into())),
                orca_task_id: Some(OrcaTaskId("orca-task-1".into())),
                orca_dispatch_id: Some(OrcaDispatchId("orca-dispatch-1".into())),
                retry_of_jcode_run_id: None,
                retry_of_dispatch_id: None,
                worktree_id: Some(OrcaWorktreeId("worktree-1".into())),
                terminal_id: Some(OrcaTerminalId("terminal-1".into())),
                status: "ready".into(),
                created_at: now,
                updated_at: now,
            }])
        }
    }

    struct FakeOrca {
        unavailable: bool,
        calls: Arc<std::sync::atomic::AtomicUsize>,
        start_outcome: OrcaMutationOutcome,
        cancel_outcome: OrcaMutationOutcome,
    }
    #[async_trait]
    impl OrcaAdapter for FakeOrca {
        async fn capabilities(&self) -> Result<RuntimeMutationCapabilities, CommandCenterError> {
            self.calls.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            if self.unavailable {
                return Err(CommandCenterError::OrcaUnavailable);
            }
            Ok(RuntimeMutationCapabilities {
                start_initiative_run: true,
                retry_linked_run: false,
                cancel_linked_run: true,
            })
        }

        async fn observe(&self, _id: &InitiativeId) -> Result<OrcaReference, CommandCenterError> {
            if self.unavailable {
                return Err(CommandCenterError::OrcaUnavailable);
            }
            Ok(OrcaReference {
                project_id: Some(OrcaProjectId("orca-project".into())),
                runtime_id: Some("orca-runtime".into()),
                run_id: None,
                task_ids: vec![],
                dispatch_ids: vec![],
                worktree_ids: vec![],
                worker_ids: vec![],
                terminal_ids: vec![],
                gate_ids: vec![],
                correlation_ids: vec![],
                idempotency_keys: vec![],
                last_observed_at: Some(Utc::now()),
                freshness: Freshness::fresh(),
            })
        }
        async fn canonical_placement(
            &self,
            _id: &InitiativeId,
        ) -> Result<OrcaCanonicalPlacement, CommandCenterError> {
            Ok(test_placement())
        }
        async fn start_initiative_run(
            &self,
            request: StartInitiativeRunRequest,
        ) -> RuntimeCommandExecution {
            if self.unavailable {
                return RuntimeCommandExecution::failed(CommandCenterError::OrcaUnavailable);
            }
            let now = Utc::now();
            let attempt = test_attempt(None);
            RuntimeCommandExecution::from_lifecycle(
                JcodeRunReference {
                    id: request.context.jcode_attempt_id,
                    initiative_id: request.context.initiative_id,
                    orca_run_id: Some(attempt.run_id.clone()),
                    orca_task_id: Some(attempt.task_id.clone()),
                    orca_dispatch_id: Some(attempt.dispatch_id.clone()),
                    retry_of_jcode_run_id: None,
                    retry_of_dispatch_id: None,
                    worktree_id: Some(attempt.worktree_id.clone()),
                    terminal_id: attempt.terminal_id.clone(),
                    status: "accepted".into(),
                    created_at: now,
                    updated_at: now,
                },
                test_receipt(self.start_outcome.clone(), attempt),
            )
        }
        async fn retry_linked_run(
            &self,
            request: RetryLinkedRunRequest,
        ) -> RuntimeCommandExecution {
            self.start_initiative_run(StartInitiativeRunRequest {
                context: request.context,
                objective: "retry".into(),
                task_spec: "retry".into(),
                placement: request.placement,
            })
            .await
        }
        async fn cancel_linked_run(
            &self,
            request: CancelLinkedRunRequest,
        ) -> RuntimeCommandExecution {
            if self.unavailable {
                return RuntimeCommandExecution::failed(CommandCenterError::OrcaUnavailable);
            }
            let now = Utc::now();
            let attempt = test_attempt(None);
            RuntimeCommandExecution::from_lifecycle(
                JcodeRunReference {
                    id: request.target_jcode_attempt_id,
                    initiative_id: request.context.initiative_id,
                    orca_run_id: Some(request.orca_run_id),
                    orca_task_id: Some(request.orca_task_id),
                    orca_dispatch_id: Some(request.target_dispatch_id),
                    retry_of_jcode_run_id: None,
                    retry_of_dispatch_id: None,
                    worktree_id: Some(attempt.worktree_id.clone()),
                    terminal_id: attempt.terminal_id.clone(),
                    status: "stopped".into(),
                    created_at: now,
                    updated_at: now,
                },
                test_receipt(self.cancel_outcome.clone(), attempt),
            )
        }
    }

    fn test_placement() -> OrcaCanonicalPlacement {
        OrcaCanonicalPlacement {
            project_id: OrcaProjectId("project-1".into()),
            repository_id: OrcaRepositoryId("repository-1".into()),
            host_setup_id: OrcaHostSetupId("host-setup-1".into()),
            host_id: OrcaHostId("host-1".into()),
            worktree_id: OrcaWorktreeId("worktree-1".into()),
            worktree_selector: "id:worktree-1".into(),
            coordinator_terminal_id: OrcaTerminalId("coordinator-1".into()),
            environment: None,
            launcher: OrcaWorkerLauncher::Agent {
                agent: "codex".into(),
                model: None,
                effort: None,
            },
        }
    }

    fn test_attempt(retry_of_dispatch_id: Option<OrcaDispatchId>) -> OrcaAttemptIdentity {
        OrcaAttemptIdentity {
            run_id: OrcaRunId("orca-run-1".into()),
            task_id: OrcaTaskId("orca-task-1".into()),
            dispatch_id: OrcaDispatchId("orca-dispatch-1".into()),
            retry_of_dispatch_id,
            worktree_id: OrcaWorktreeId("worktree-1".into()),
            terminal_id: Some(OrcaTerminalId("terminal-1".into())),
        }
    }

    fn test_receipt(
        outcome: OrcaMutationOutcome,
        attempt: OrcaAttemptIdentity,
    ) -> OrcaLifecycleReceipt {
        OrcaLifecycleReceipt {
            outcome,
            attempt: Some(attempt),
            stage: "verified".into(),
            failed_stage: None,
            last_error: None,
            effects: vec![],
            residual_resources: vec![],
            cleanup: vec![],
            observed_at: Utc::now(),
        }
    }

    fn goal() -> Goal {
        let mut goal = Goal::new("Command Center", GoalScope::Project);
        goal.milestones = vec![GoalMilestone {
            id: "m1".into(),
            title: "M1".into(),
            status: "pending".into(),
            steps: vec![GoalStep {
                id: "s1".into(),
                content: "step".into(),
                status: "pending".into(),
            }],
        }];
        goal
    }

    fn auth() -> AuthContext {
        AuthContext {
            session_id: "session".into(),
            user_label: None,
            csrf_token: "csrf".into(),
            expires_at: Utc::now() + Duration::minutes(5),
            allowed_initiatives: vec![InitiativeId("command-center".into())],
        }
    }

    fn service(
        unavailable: bool,
    ) -> CommandCenterService<MemoryRepo, EmptySchedules, EmptyRuns, FakeOrca> {
        service_with_outcomes(
            unavailable,
            OrcaMutationOutcome::Ready,
            OrcaMutationOutcome::Stopped,
        )
        .0
    }

    fn service_with_outcomes(
        unavailable: bool,
        start_outcome: OrcaMutationOutcome,
        cancel_outcome: OrcaMutationOutcome,
    ) -> (
        CommandCenterService<MemoryRepo, EmptySchedules, EmptyRuns, FakeOrca>,
        Arc<std::sync::atomic::AtomicUsize>,
    ) {
        let calls = Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let adapter_calls = calls.clone();
        (
            CommandCenterService::new(
                MemoryRepo(Arc::new(Mutex::new((goal(), Revision(1))))),
                EmptySchedules,
                EmptyRuns,
                FakeOrca {
                    unavailable,
                    calls: adapter_calls,
                    start_outcome,
                    cancel_outcome,
                },
            ),
            calls,
        )
    }

    struct TestExternalSignalProjection;

    #[async_trait]
    impl ExternalSignalProjectionSource for TestExternalSignalProjection {
        async fn projection(&self) -> Result<serde_json::Value, String> {
            Ok(serde_json::json!({
                "readiness": {"enabled": true, "bindAddr": "127.0.0.1:39994"},
                "acceptedCount": 2,
                "processing": [{"stage": "projected", "attempts": 1}],
                "deadLetters": []
            }))
        }
    }

    struct TestMxHealthSource {
        calls: Arc<std::sync::atomic::AtomicUsize>,
        projection: mx_health::MxHealthProjection,
    }

    #[async_trait]
    impl mx_health::MxHealthSource for TestMxHealthSource {
        async fn read(&self) -> mx_health::MxHealthProjection {
            self.calls.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            self.projection.clone()
        }
    }

    #[tokio::test]
    async fn detail_snapshot_projects_redacted_external_signal_state() {
        let service =
            service(false).with_external_signal_projection(Arc::new(TestExternalSignalProjection));
        let snapshot = service
            .snapshot(&auth(), &InitiativeId("command-center".into()))
            .await
            .unwrap();
        let value = serde_json::to_value(snapshot).unwrap();
        assert_eq!(value["externalSignals"]["acceptedCount"], 2);
        assert_eq!(
            value["externalSignals"]["readiness"]["bindAddr"],
            "127.0.0.1:39994"
        );
        assert!(value["externalSignals"].get("rawJson").is_none());
    }

    fn envelope(payload: CommandPayload, rev: Revision, key: &str) -> CommandEnvelope {
        CommandEnvelope {
            id: CommandId(format!("cmd-{key}")),
            idempotency_key: IdempotencyKey(key.into()),
            correlation_id: CorrelationId("corr".into()),
            auth: auth(),
            expected_revision: rev,
            payload,
        }
    }

    #[tokio::test]
    async fn list_snapshot_serializes_to_browser_contract_shape() {
        let list = service(false).list_initiatives(&auth()).await.unwrap();
        let value = serde_json::to_value(list).unwrap();

        assert_eq!(value["meta"]["protocolVersion"], "command-center.v1");
        assert_eq!(value["meta"]["snapshotRevision"], 0);
        assert_eq!(value["connection"]["state"], "live");
        assert_eq!(value["initiatives"][0]["id"], "command-center");
        assert_eq!(value["initiatives"][0]["currentMilestone"]["id"], "m1");
        assert_eq!(
            value["initiatives"][0]["currentMilestone"]["steps"][0]["title"],
            "step"
        );
        assert!(value.get("metadata").is_none());
        assert!(value.get("revision").is_none());
    }

    #[tokio::test]
    async fn detail_snapshot_serializes_selected_browser_projections() {
        let snapshot = service(false)
            .snapshot(&auth(), &InitiativeId("command-center".into()))
            .await
            .unwrap();
        let value = serde_json::to_value(snapshot).unwrap();

        assert_eq!(value["meta"]["snapshotRevision"], 1);
        assert_eq!(value["initiatives"][0]["id"], "command-center");
        assert_eq!(
            value["selectedInitiative"]["availableActions"]["checkpoint"],
            true
        );
        assert_eq!(
            value["selectedInitiative"]["schedules"],
            serde_json::json!([])
        );
        assert_eq!(
            value["selectedInitiative"]["availableActions"]["startRun"],
            true
        );
        assert_eq!(
            value["selectedInitiative"]["availableActions"]["retryRun"],
            false
        );
        assert_eq!(
            value["selectedInitiative"]["availableActions"]["cancelRun"],
            true
        );
        assert!(value.get("initiative").is_none());
        assert!(value.get("available_actions").is_none());
    }

    #[tokio::test]
    async fn unavailable_runtime_capabilities_are_not_projected_as_actions() {
        let snapshot = service(true)
            .snapshot(&auth(), &InitiativeId("command-center".into()))
            .await
            .unwrap();

        assert!(!snapshot.available_actions.start_initiative_run);
        assert!(!snapshot.available_actions.retry_linked_run);
        assert!(!snapshot.available_actions.cancel_linked_run);
        assert_eq!(snapshot.orca.freshness.state, FreshnessState::Unavailable);
    }

    #[test]
    fn identifier_envelope_preserves_distinct_command_center_ids() {
        let refs = EntityRefs {
            initiative_id: Some(InitiativeId("jcode-initiative".into())),
            schedule_id: Some(ScheduleRefId("schedule".into())),
            jcode_run_id: Some(JcodeRunId("jcode-run".into())),
            orca_project_id: Some(OrcaProjectId("orca-project".into())),
            orca_run_id: Some(OrcaRunId("orca-run".into())),
            orca_task_id: Some(OrcaTaskId("orca-task".into())),
            orca_dispatch_id: Some(OrcaDispatchId("orca-dispatch".into())),
            orca_worktree_id: Some(OrcaWorktreeId("orca-worktree".into())),
            orca_terminal_id: Some(OrcaTerminalId("orca-terminal".into())),
            correlation_id: Some(CorrelationId("correlation".into())),
            idempotency_key: Some(IdempotencyKey("idempotency".into())),
        };

        let value = serde_json::to_value(&refs).unwrap();
        assert_eq!(value["jcode_run_id"], "jcode-run");
        assert_eq!(value["orca_project_id"], "orca-project");
        assert_eq!(value["orca_run_id"], "orca-run");
        assert_eq!(value["orca_task_id"], "orca-task");
        assert_eq!(value["orca_dispatch_id"], "orca-dispatch");
        assert_eq!(value["orca_worktree_id"], "orca-worktree");
        assert_eq!(value["orca_terminal_id"], "orca-terminal");
        assert_eq!(value["correlation_id"], "correlation");
        assert_eq!(value["idempotency_key"], "idempotency");

        let restored: EntityRefs = serde_json::from_value(value).unwrap();
        assert_eq!(restored, refs);
    }

    #[test]
    fn orca_reference_preserves_runtime_identifiers_separately() {
        let reference = OrcaReference {
            project_id: Some(OrcaProjectId("canonical-project".into())),
            runtime_id: Some("runtime-instance".into()),
            run_id: Some(OrcaRunId("run".into())),
            task_ids: vec![OrcaTaskId("task".into())],
            dispatch_ids: vec![OrcaDispatchId("dispatch".into())],
            worktree_ids: vec![OrcaWorktreeId("worktree".into())],
            worker_ids: vec!["worker".into()],
            terminal_ids: vec![OrcaTerminalId("terminal".into())],
            gate_ids: vec!["gate".into()],
            correlation_ids: vec![CorrelationId("correlation".into())],
            idempotency_keys: vec![IdempotencyKey("idempotency".into())],
            last_observed_at: None,
            freshness: Freshness::fresh(),
        };

        let value = serde_json::to_value(&reference).unwrap();
        assert_eq!(value["project_id"], "canonical-project");
        assert_eq!(value["runtime_id"], "runtime-instance");
        assert_eq!(value["task_ids"], serde_json::json!(["task"]));
        assert_eq!(value["dispatch_ids"], serde_json::json!(["dispatch"]));
        assert_eq!(value["worktree_ids"], serde_json::json!(["worktree"]));
        assert_eq!(value["terminal_ids"], serde_json::json!(["terminal"]));
        assert_eq!(value["correlation_ids"], serde_json::json!(["correlation"]));
        assert_eq!(
            value["idempotency_keys"],
            serde_json::json!(["idempotency"])
        );

        let legacy: OrcaReference = serde_json::from_value(serde_json::json!({
            "project_id": "canonical-project",
            "runtime_id": "runtime-instance",
            "run_id": null,
            "worker_ids": [],
            "terminal_ids": [],
            "gate_ids": [],
            "last_observed_at": null,
            "freshness": Freshness::fresh(),
        }))
        .unwrap();
        assert!(legacy.task_ids.is_empty());
        assert!(legacy.dispatch_ids.is_empty());
        assert!(legacy.worktree_ids.is_empty());
        assert!(legacy.correlation_ids.is_empty());
        assert!(legacy.idempotency_keys.is_empty());
    }

    #[test]
    fn default_config_is_disabled_and_loopback_safe() {
        let config = CommandCenterConfig::default();
        assert!(!config.enabled);
        assert!(config.bind_addr.ip().is_loopback());
        assert!(config.validate().is_ok());
        let remote = CommandCenterConfig {
            enabled: true,
            bind_addr: SocketAddr::from(([0, 0, 0, 0], 8080)),
            allowed_origins: vec![],
            authenticated_remote: false,
            asset_dir: None,
            decision_inbox_db_path: None,
        };
        assert_eq!(
            remote.validate(),
            Err(CommandCenterError::NonLoopbackRequiresAuthenticatedRemote)
        );
    }

    #[test]
    fn csrf_guard_requires_origin_json_post_and_token() {
        let config = CommandCenterConfig {
            enabled: true,
            bind_addr: SocketAddr::from((Ipv4Addr::LOCALHOST, 0)),
            allowed_origins: vec!["http://127.0.0.1".into()],
            authenticated_remote: false,
            asset_dir: None,
            decision_inbox_db_path: None,
        };
        let session = BrowserSessionIssuer::new(Duration::minutes(1)).issue(vec![]);
        let good = RequestGuard {
            origin: Some("http://127.0.0.1".into()),
            method: "POST".into(),
            content_type: Some("application/json".into()),
            csrf_token: Some(session.csrf_token.clone()),
        };
        assert!(validate_mutation_request(&config, &session, &good).is_ok());
        let bad = RequestGuard {
            csrf_token: None,
            ..good
        };
        assert_eq!(
            validate_mutation_request(&config, &session, &bad),
            Err(CommandCenterError::CsrfRejected)
        );
    }

    #[test]
    fn replay_rejects_scope_and_marks_gaps_snapshot_required() {
        let mut buffer = ReplayBuffer::new(StreamId("s1".into()), 2);
        buffer.push(
            EventSource::Jcode,
            EntityRefs::default(),
            EventPayload::Unknown {
                name: "future".into(),
                requires_snapshot: true,
            },
        );
        buffer.push(
            EventSource::Jcode,
            EntityRefs::default(),
            EventPayload::Unknown {
                name: "future2".into(),
                requires_snapshot: false,
            },
        );
        buffer.push(
            EventSource::Jcode,
            EntityRefs::default(),
            EventPayload::Unknown {
                name: "future3".into(),
                requires_snapshot: false,
            },
        );
        assert_eq!(
            buffer.replay(&ReplayCursor {
                stream_id: StreamId("other".into()),
                sequence: 0
            }),
            Err(CommandCenterError::ReplayScopeMismatch)
        );
        assert!(
            buffer
                .replay(&ReplayCursor {
                    stream_id: StreamId("s1".into()),
                    sequence: 0
                })
                .unwrap()
                .snapshot_required
        );
        assert_eq!(
            buffer
                .replay(&ReplayCursor {
                    stream_id: StreamId("s1".into()),
                    sequence: 2
                })
                .unwrap()
                .events
                .len(),
            1
        );
    }

    #[test]
    fn scoped_replay_rejects_authorization_and_expiry_boundaries() {
        let mut scoped = ReplayBuffer::new_scoped(
            StreamId("s1".into()),
            ReplayScope {
                principal_session_id: "session".into(),
                initiative_id: InitiativeId("command-center".into()),
                orca_run_id: Some(OrcaRunId("orca-run".into())),
                authorized_until: Utc::now() + Duration::minutes(5),
            },
            4,
        );
        scoped.push(
            EventSource::OrcaAdapter,
            EntityRefs {
                initiative_id: Some(InitiativeId("command-center".into())),
                orca_run_id: Some(OrcaRunId("orca-run".into())),
                ..EntityRefs::default()
            },
            EventPayload::Unknown {
                name: "future".into(),
                requires_snapshot: true,
            },
        );

        assert_eq!(
            scoped
                .replay_authorized(
                    &auth(),
                    &ReplayCursor {
                        stream_id: StreamId("s1".into()),
                        sequence: 0,
                    },
                )
                .unwrap()
                .events
                .len(),
            1
        );

        let mut wrong_auth = auth();
        wrong_auth.session_id = "other-session".into();
        assert_eq!(
            scoped.replay_authorized(
                &wrong_auth,
                &ReplayCursor {
                    stream_id: StreamId("s1".into()),
                    sequence: 0,
                },
            ),
            Err(CommandCenterError::ReplayScopeMismatch)
        );

        scoped.rotate_scoped(
            StreamId("s2".into()),
            ReplayScope {
                principal_session_id: "session".into(),
                initiative_id: InitiativeId("command-center".into()),
                orca_run_id: Some(OrcaRunId("orca-run".into())),
                authorized_until: Utc::now() - Duration::minutes(1),
            },
        );
        assert_eq!(
            scoped.replay_authorized(
                &auth(),
                &ReplayCursor {
                    stream_id: StreamId("s2".into()),
                    sequence: 0,
                },
            ),
            Err(CommandCenterError::ReplayScopeMismatch)
        );
    }

    #[tokio::test]
    async fn stale_revision_and_duplicate_commands_are_handled() {
        let service = service(false);
        let stale = service
            .execute(envelope(
                CommandPayload::SetBlockers {
                    initiative_id: InitiativeId("command-center".into()),
                    blockers: vec!["blocked".into()],
                },
                Revision(0),
                "k1",
            ))
            .await;
        assert!(matches!(
            stale.error,
            Some(CommandCenterError::StaleRevision { .. })
        ));
        let cmd = envelope(
            CommandPayload::UpdateStep {
                initiative_id: InitiativeId("command-center".into()),
                milestone_id: "m1".into(),
                step_id: "s1".into(),
                status: "done".into(),
            },
            Revision(1),
            "k2",
        );
        let first = service.execute(cmd.clone()).await;
        let duplicate = service.execute(cmd).await;
        assert_eq!(first, duplicate);
        assert_eq!(first.state, CommandState::Completed);
    }

    #[tokio::test]
    async fn direct_orca_mutation_is_rejected_before_adapter() {
        let service = service(false);
        let result = service
            .execute(envelope(
                CommandPayload::DirectOrcaMutation {
                    initiative_id: InitiativeId("command-center".into()),
                    field: "worker.status".into(),
                },
                Revision(1),
                "k3",
            ))
            .await;
        assert!(matches!(
            result.error,
            Some(CommandCenterError::InvalidCommand { .. })
        ));
    }

    #[tokio::test]
    async fn runtime_command_returns_pending_downstream_action() {
        let service = service(false);
        let result = service
            .execute(envelope(
                CommandPayload::StartInitiativeRun {
                    initiative_id: InitiativeId("command-center".into()),
                },
                Revision(1),
                "k4",
            ))
            .await;
        assert_eq!(result.state, CommandState::Pending);
        assert!(matches!(
            result.authoritative,
            Some(CommandResultPayload::RunAccepted { .. })
        ));
    }

    #[tokio::test]
    async fn runtime_command_rejects_stale_revision_before_adapter_invocation() {
        let (service, calls) = service_with_outcomes(
            false,
            OrcaMutationOutcome::Ready,
            OrcaMutationOutcome::Stopped,
        );
        let result = service
            .execute(envelope(
                CommandPayload::StartInitiativeRun {
                    initiative_id: InitiativeId("command-center".into()),
                },
                Revision(0),
                "stale-runtime",
            ))
            .await;

        assert_eq!(result.state, CommandState::Failed);
        assert!(matches!(
            result.error,
            Some(CommandCenterError::StaleRevision {
                expected: Revision(0),
                actual: Revision(1)
            })
        ));
        assert_eq!(calls.load(std::sync::atomic::Ordering::SeqCst), 0);
    }

    #[tokio::test]
    async fn failed_and_outcome_unknown_runtime_commands_preserve_authoritative_receipts() {
        for (outcome, expected_state) in [
            (OrcaMutationOutcome::Failed, CommandState::Failed),
            (OrcaMutationOutcome::OutcomeUnknown, CommandState::Pending),
            (OrcaMutationOutcome::RecoveryRequired, CommandState::Pending),
        ] {
            let (service, _) =
                service_with_outcomes(false, outcome.clone(), OrcaMutationOutcome::Stopped);
            let result = service
                .execute(envelope(
                    CommandPayload::StartInitiativeRun {
                        initiative_id: InitiativeId("command-center".into()),
                    },
                    Revision(1),
                    match outcome {
                        OrcaMutationOutcome::Failed => "failed-receipt",
                        _ => "unknown-receipt",
                    },
                ))
                .await;

            assert_eq!(result.state, expected_state);
            match result.authoritative {
                Some(CommandResultPayload::RunAccepted {
                    receipt: Some(receipt),
                    ..
                }) => assert_eq!(receipt.outcome, outcome),
                other => panic!("missing authoritative lifecycle receipt: {other:?}"),
            }
        }
    }

    #[tokio::test]
    async fn stopped_cancel_settlement_completes_the_command() {
        let result = service(false)
            .execute(envelope(
                CommandPayload::CancelLinkedRun {
                    initiative_id: InitiativeId("command-center".into()),
                    run_id: JcodeRunId("run-1".into()),
                },
                Revision(1),
                "cancel-stopped",
            ))
            .await;

        assert_eq!(result.state, CommandState::Completed);
        match result.authoritative {
            Some(CommandResultPayload::RunAccepted {
                receipt: Some(receipt),
                ..
            }) => assert_eq!(receipt.outcome, OrcaMutationOutcome::Stopped),
            other => panic!("missing authoritative stopped receipt: {other:?}"),
        }
    }

    #[tokio::test]
    async fn unsupported_runtime_capability_is_rejected_before_adapter_invocation() {
        let service = service(false);
        let result = service
            .execute(envelope(
                CommandPayload::RetryLinkedRun {
                    initiative_id: InitiativeId("command-center".into()),
                    run_id: JcodeRunId("run-1".into()),
                },
                Revision(1),
                "k4-retry-unsupported",
            ))
            .await;

        assert_eq!(result.state, CommandState::Failed);
        assert_eq!(result.authoritative, None);
        assert_eq!(
            result.error,
            Some(CommandCenterError::UnsupportedCapability {
                capability: "orca.command_center.retry_linked_run".to_string(),
            })
        );
    }

    #[tokio::test]
    async fn unavailable_runtime_command_fails_closed_without_initiative_fallback() {
        let service = service(true);
        let result = service
            .execute(envelope(
                CommandPayload::StartInitiativeRun {
                    initiative_id: InitiativeId("command-center".into()),
                },
                Revision(1),
                "k5",
            ))
            .await;

        assert_eq!(result.state, CommandState::Failed);
        assert_eq!(result.authoritative, None);
        assert_eq!(result.error, Some(CommandCenterError::OrcaUnavailable));
    }

    #[tokio::test]
    async fn snapshot_degrades_when_orca_unavailable() {
        let service = service(true);
        let snapshot = service
            .snapshot(&auth(), &InitiativeId("command-center".into()))
            .await
            .unwrap();
        assert_eq!(snapshot.orca.freshness.state, FreshnessState::Unavailable);
        assert_eq!(snapshot.initiative.title, "Command Center");
    }

    fn runtime() -> Arc<dyn CommandCenterApi> {
        Arc::new(CommandCenterRuntime::new(
            service(false),
            StreamId("test-stream".into()),
        ))
    }

    async fn spawn_test_host(ttl: Duration) -> CommandCenterHttpHost {
        spawn_test_host_with_inbox(ttl, None).await
    }

    async fn spawn_test_host_with_inbox(
        ttl: Duration,
        decision_inbox_db_path: Option<PathBuf>,
    ) -> CommandCenterHttpHost {
        spawn_command_center_http_host(
            CommandCenterConfig {
                enabled: true,
                bind_addr: SocketAddr::from((Ipv4Addr::LOCALHOST, 0)),
                allowed_origins: Vec::new(),
                authenticated_remote: false,
                asset_dir: None,
                decision_inbox_db_path,
            },
            BrowserSessionStore::new(ttl),
            runtime(),
        )
        .await
        .unwrap()
        .unwrap()
    }

    #[tokio::test]
    async fn authenticated_decision_inbox_projects_durable_provider_items() {
        use jcode_intake_types::{Envelope, SqliteIntakeStore};

        let directory = tempfile::tempdir().unwrap();
        let database = directory.path().join("decision-inbox.sqlite");
        let mut store = SqliteIntakeStore::open(&database, None).unwrap();
        store
            .receive(
                Envelope {
                    adapter: "slack".to_owned(),
                    sender_identity: "sl:U123".to_owned(),
                    conversation: "sl:D123".to_owned(),
                    content: Some("implement command center inbox".to_owned()),
                    attachments: Vec::new(),
                    received_at: Utc::now(),
                },
                serde_json::json!({"envelope_id": "env-1"}),
                Some("sl:U123".to_owned()),
            )
            .unwrap();
        drop(store);

        let host = spawn_test_host_with_inbox(Duration::minutes(1), Some(database)).await;
        let client = reqwest::Client::new();
        let unauthenticated = client
            .get(format!(
                "http://{}/api/command-center/decision-inbox",
                host.addr()
            ))
            .send()
            .await
            .unwrap();
        assert_eq!(unauthenticated.status(), StatusCode::UNAUTHORIZED);

        let session = bootstrap(&client, &host).await;
        let snapshot: serde_json::Value = client
            .get(format!(
                "http://{}/api/command-center/decision-inbox",
                host.addr()
            ))
            .bearer_auth(&session.id)
            .send()
            .await
            .unwrap()
            .error_for_status()
            .unwrap()
            .json()
            .await
            .unwrap();
        assert_eq!(snapshot["items"][0]["source"]["adapter"], "slack");
        assert_eq!(snapshot["items"][0]["category"], "work_request");
        assert_eq!(snapshot["items"][0]["status"], "awaiting_approval");
        assert_eq!(snapshot["items"][0]["raw_payload_retained"], true);
        host.shutdown().await.unwrap();
    }

    #[tokio::test]
    async fn authenticated_mx_health_projects_only_safe_data_and_authenticates_first() {
        let calls = Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let projection = mx_health::MxHealthProjection {
            provenance: mx_health::MxHealthProvenance::default(),
            adapter_state: mx_health::MxAdapterState::Live,
            failure_category: None,
            fetched_at: Utc::now(),
            health: Some(mx_health::MxHealthSnapshot {
                version: mx_health::MX_HEALTH_VERSION.to_owned(),
                generated_at: Utc::now(),
                overall: mx_health::MxOverallStatus::Degraded,
                redacted: true,
                checks: vec![mx_health::MxHealthCheck {
                    id: "persistence".to_owned(),
                    layer: "persistence".to_owned(),
                    status: mx_health::MxCheckStatus::Down,
                    reason_code: "persistence_unavailable".to_owned(),
                    summary: "Persistence is unavailable".to_owned(),
                    depends_on: Vec::new(),
                }],
            }),
            stale: None,
        };
        let source = Arc::new(TestMxHealthSource {
            calls: calls.clone(),
            projection,
        });
        let sessions = BrowserSessionStore::new(Duration::minutes(1));
        let host = spawn_command_center_http_host_with_mx(
            CommandCenterConfig {
                enabled: true,
                bind_addr: SocketAddr::from((Ipv4Addr::LOCALHOST, 0)),
                allowed_origins: Vec::new(),
                authenticated_remote: false,
                asset_dir: None,
                decision_inbox_db_path: None,
            },
            sessions.clone(),
            runtime(),
            Some(source),
        )
        .await
        .unwrap()
        .unwrap();
        let client = reqwest::Client::new();
        let url = format!("http://{}/api/command-center/mx-health", host.addr());

        let unauthorized = client.get(&url).send().await.unwrap();
        assert_eq!(unauthorized.status(), StatusCode::UNAUTHORIZED);
        assert_eq!(calls.load(std::sync::atomic::Ordering::SeqCst), 0);

        sessions.insert(BrowserSession {
            id: "expired-mx-session".to_owned(),
            csrf_token: "csrf".to_owned(),
            expires_at: Utc::now() - Duration::minutes(1),
            scope: Vec::new(),
        });
        let expired = client
            .get(&url)
            .bearer_auth("expired-mx-session")
            .send()
            .await
            .unwrap();
        assert_eq!(expired.status(), StatusCode::UNAUTHORIZED);
        assert_eq!(calls.load(std::sync::atomic::Ordering::SeqCst), 0);

        let session = bootstrap(&client, &host).await;
        let body: serde_json::Value = client
            .get(&url)
            .bearer_auth(&session.id)
            .send()
            .await
            .unwrap()
            .error_for_status()
            .unwrap()
            .json()
            .await
            .unwrap();
        assert_eq!(body["adapterState"], "live");
        assert_eq!(body["health"]["overall"], "degraded");
        assert_eq!(body["health"]["checks"][0]["status"], "down");
        assert!(body.get("token").is_none());
        assert!(body.to_string().find("Authorization").is_none());
        assert_eq!(calls.load(std::sync::atomic::Ordering::SeqCst), 1);
        host.shutdown().await.unwrap();
    }

    #[tokio::test]
    async fn http_host_serves_static_spa_routes_from_managed_asset_dir() {
        let asset_dir =
            std::env::temp_dir().join(format!("jcode-command-center-assets-{}", Uuid::new_v4()));
        std::fs::create_dir_all(&asset_dir).unwrap();
        std::fs::write(
            asset_dir.join("index.html"),
            "<main>managed command center</main>",
        )
        .unwrap();
        let host = spawn_command_center_http_host(
            CommandCenterConfig {
                enabled: true,
                bind_addr: SocketAddr::from((Ipv4Addr::LOCALHOST, 0)),
                allowed_origins: Vec::new(),
                authenticated_remote: false,
                asset_dir: Some(asset_dir.clone()),
                decision_inbox_db_path: None,
            },
            BrowserSessionStore::new(Duration::minutes(1)),
            runtime(),
        )
        .await
        .unwrap()
        .unwrap();

        let response = reqwest::get(format!(
            "http://{}/initiatives/example/runs/run-1",
            host.addr()
        ))
        .await
        .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        assert!(
            response
                .text()
                .await
                .unwrap()
                .contains("managed command center")
        );

        host.shutdown().await.unwrap();
        std::fs::remove_dir_all(asset_dir).unwrap();
    }

    async fn bootstrap(client: &reqwest::Client, host: &CommandCenterHttpHost) -> BrowserSession {
        client
            .post(format!(
                "http://{}/api/command-center/bootstrap",
                host.addr()
            ))
            .json(&serde_json::json!({ "scope": ["command-center"] }))
            .send()
            .await
            .unwrap()
            .error_for_status()
            .unwrap()
            .json::<BrowserSession>()
            .await
            .unwrap()
    }

    #[tokio::test]
    async fn http_host_rejects_non_loopback_without_remote_auth() {
        let result = spawn_command_center_http_host(
            CommandCenterConfig {
                enabled: true,
                bind_addr: SocketAddr::from(([0, 0, 0, 0], 0)),
                allowed_origins: Vec::new(),
                authenticated_remote: false,
                asset_dir: None,
                decision_inbox_db_path: None,
            },
            BrowserSessionStore::new(Duration::minutes(5)),
            runtime(),
        )
        .await;

        assert_eq!(
            result.err(),
            Some(CommandCenterError::NonLoopbackRequiresAuthenticatedRemote)
        );
    }

    #[tokio::test]
    async fn http_auth_csrf_origin_cursor_and_shutdown_are_enforced() {
        let host = spawn_test_host(Duration::minutes(5)).await;
        let base = format!("http://{}", host.addr());
        let origin = base.clone();
        let client = reqwest::Client::new();

        let unauthenticated = client
            .get(format!("{base}/api/command-center/initiatives"))
            .send()
            .await
            .unwrap();
        assert_eq!(unauthenticated.status(), StatusCode::UNAUTHORIZED);

        let session = bootstrap(&client, &host).await;

        let listed = client
            .get(format!("{base}/api/command-center/initiatives"))
            .bearer_auth(&session.id)
            .send()
            .await
            .unwrap()
            .error_for_status()
            .unwrap()
            .json::<serde_json::Value>()
            .await
            .unwrap();
        assert_eq!(listed["initiatives"].as_array().unwrap().len(), 1);
        assert_eq!(listed["initiatives"][0]["id"], "command-center");

        let missing_csrf = client
            .post(format!("{base}/api/command-center/commands"))
            .bearer_auth(&session.id)
            .header(header::ORIGIN, origin.as_str())
            .json(&serde_json::json!({
                "idempotencyKey": "http-k1",
                "payload": {
                    "type": "set_blockers",
                    "initiative_id": "command-center",
                    "blockers": ["blocked"]
                }
            }))
            .send()
            .await
            .unwrap();
        assert_eq!(missing_csrf.status(), StatusCode::FORBIDDEN);

        let bad_origin = client
            .post(format!("{base}/api/command-center/commands"))
            .bearer_auth(&session.id)
            .header(header::ORIGIN, "http://evil.example")
            .header("x-csrf-token", &session.csrf_token)
            .json(&serde_json::json!({
                "idempotencyKey": "http-k2",
                "payload": {
                    "type": "set_blockers",
                    "initiative_id": "command-center",
                    "blockers": ["blocked"]
                }
            }))
            .send()
            .await
            .unwrap();
        assert_eq!(bad_origin.status(), StatusCode::FORBIDDEN);

        let accepted = client
            .post(format!("{base}/api/command-center/commands"))
            .bearer_auth(&session.id)
            .header(header::ORIGIN, origin.as_str())
            .header("x-csrf-token", &session.csrf_token)
            .header("x-expected-revision", "1")
            .json(&serde_json::json!({
                "idempotencyKey": "http-k3",
                "payload": {
                    "type": "set_next_actions",
                    "initiative_id": "command-center",
                    "next_actions": ["ship"]
                }
            }))
            .send()
            .await
            .unwrap()
            .error_for_status()
            .unwrap()
            .json::<CommandResult>()
            .await
            .unwrap();
        assert_eq!(accepted.state, CommandState::Completed);

        let wrong_cursor = client
            .get(format!(
                "{base}/api/command-center/replay?stream_id=other&sequence=0"
            ))
            .bearer_auth(&session.id)
            .send()
            .await
            .unwrap();
        assert_eq!(wrong_cursor.status(), StatusCode::FORBIDDEN);

        host.shutdown().await.unwrap();
        let after_shutdown = client
            .get(format!("{base}/api/command-center/initiatives"))
            .bearer_auth(&session.id)
            .send()
            .await;
        assert!(after_shutdown.is_err());
    }

    #[tokio::test]
    async fn http_sessions_expire() {
        let host = spawn_test_host(Duration::milliseconds(-1)).await;
        let client = reqwest::Client::new();
        let session = bootstrap(&client, &host).await;
        let expired = client
            .get(format!(
                "http://{}/api/command-center/initiatives",
                host.addr()
            ))
            .bearer_auth(&session.id)
            .send()
            .await
            .unwrap();

        assert_eq!(expired.status(), StatusCode::UNAUTHORIZED);
        host.shutdown().await.unwrap();
    }

    #[test]
    fn dto_serialization_keeps_unknown_event_progress() {
        let event = EventEnvelope {
            protocol_version: PROTOCOL_VERSION,
            stream_id: StreamId("s".into()),
            sequence: 7,
            timestamp: Utc::now(),
            source: EventSource::System,
            entity_refs: EntityRefs::default(),
            payload: EventPayload::Unknown {
                name: "newer_event".into(),
                requires_snapshot: true,
            },
        };
        let json = serde_json::to_string(&event).unwrap();
        let decoded: EventEnvelope = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded.sequence, 7);
        assert!(matches!(
            decoded.payload,
            EventPayload::Unknown {
                requires_snapshot: true,
                ..
            }
        ));
    }

    #[test]
    fn future_event_variant_deserializes_as_snapshot_requiring_unknown() {
        let event: EventEnvelope = serde_json::from_value(serde_json::json!({
            "protocol_version": PROTOCOL_VERSION,
            "stream_id": "s",
            "sequence": 8,
            "timestamp": "2026-08-11T06:00:00Z",
            "source": "system",
            "entity_refs": {},
            "payload": {
                "type": "initiative_replanned",
                "plan_revision": 4,
                "details": { "reason": "future protocol field" }
            }
        }))
        .unwrap();

        assert_eq!(event.sequence, 8);
        assert_eq!(
            event.payload,
            EventPayload::Unknown {
                name: "initiative_replanned".into(),
                requires_snapshot: true,
            }
        );
    }

    #[test]
    fn explicit_unknown_payload_accepts_additive_fields() {
        let payload: EventPayload = serde_json::from_value(serde_json::json!({
            "type": "unknown",
            "name": "adapter_extension",
            "requires_snapshot": false,
            "raw_payload": { "adapter_version": 2 },
            "future_metadata": ["compatible"]
        }))
        .unwrap();

        assert_eq!(
            payload,
            EventPayload::Unknown {
                name: "adapter_extension".into(),
                requires_snapshot: false,
            }
        );
    }

    #[test]
    fn malformed_known_event_is_not_downgraded_to_unknown() {
        let error = serde_json::from_value::<EventPayload>(serde_json::json!({
            "type": "run_updated",
            "unexpected": true
        }))
        .unwrap_err();

        assert!(error.to_string().contains("run"));
    }

    #[test]
    fn generated_contract_matches_unknown_event_wire_shape() {
        assert!(generated_typescript_contract().contains(
            "{ type: \"unknown\"; name: string; requires_snapshot: boolean } & Record<string, unknown>"
        ));
    }
}

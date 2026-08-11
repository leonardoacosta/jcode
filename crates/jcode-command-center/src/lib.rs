//! Backend foundation for the experimental Jcode command center.
//!
//! This crate owns the public protocol boundary and daemon-side service
//! primitives. It deliberately contains no frontend persistence and is safe to
//! wire behind a disabled-by-default loopback listener.

use std::collections::{HashMap, VecDeque};
use std::fmt;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
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
        pub struct $name(pub String);
    };
}

id_type!(InitiativeId);
id_type!(ScheduleRefId);
id_type!(JcodeRunId);
id_type!(OrcaProjectId);
id_type!(OrcaRunId);
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
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OrcaReference {
    pub project_id: Option<OrcaProjectId>,
    pub run_id: Option<OrcaRunId>,
    pub worker_ids: Vec<String>,
    pub terminal_ids: Vec<String>,
    pub gate_ids: Vec<String>,
    pub last_observed_at: Option<DateTime<Utc>>,
    pub freshness: Freshness,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CommandCenterSnapshot {
    pub metadata: ProtocolMetadata,
    pub revision: Revision,
    pub initiative: InitiativeProjection,
    pub schedules: Vec<LinkedScheduleProjection>,
    pub runs: Vec<JcodeRunReference>,
    pub orca: OrcaReference,
    pub freshness: Freshness,
    pub available_actions: AvailableActions,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InitiativeListSnapshot {
    pub metadata: ProtocolMetadata,
    pub revision: Revision,
    pub initiatives: Vec<InitiativeProjection>,
    pub freshness: Freshness,
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
    pub correlation_id: String,
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
    pub correlation_id: String,
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
        orca_run_id: Option<OrcaRunId>,
    },
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
pub struct EntityRefs {
    pub initiative_id: Option<InitiativeId>,
    pub schedule_id: Option<ScheduleRefId>,
    pub jcode_run_id: Option<JcodeRunId>,
    pub orca_run_id: Option<OrcaRunId>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReplayCursor {
    pub stream_id: StreamId,
    pub sequence: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReplayBatch {
    pub events: Vec<EventEnvelope>,
    pub snapshot_required: bool,
}

pub struct ReplayBuffer {
    stream_id: StreamId,
    retention: usize,
    next_sequence: u64,
    events: VecDeque<EventEnvelope>,
}

impl ReplayBuffer {
    pub fn new(stream_id: StreamId, retention: usize) -> Self {
        Self {
            stream_id,
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

    pub fn rotate(&mut self, stream_id: StreamId) {
        self.stream_id = stream_id;
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
}

impl Default for CommandCenterConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            bind_addr: SocketAddr::from((Ipv4Addr::LOCALHOST, 0)),
            allowed_origins: Vec::new(),
            authenticated_remote: false,
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
pub trait OrcaAdapter: Send + Sync {
    async fn observe(&self, id: &InitiativeId) -> Result<OrcaReference, CommandCenterError>;
    async fn start_initiative_run(
        &self,
        id: &InitiativeId,
        key: &IdempotencyKey,
    ) -> Result<(JcodeRunReference, OrcaRunId), CommandCenterError>;
    async fn retry_linked_run(
        &self,
        id: &InitiativeId,
        run_id: &JcodeRunId,
        key: &IdempotencyKey,
    ) -> Result<(JcodeRunReference, OrcaRunId), CommandCenterError>;
    async fn cancel_linked_run(
        &self,
        id: &InitiativeId,
        run_id: &JcodeRunId,
        key: &IdempotencyKey,
    ) -> Result<JcodeRunReference, CommandCenterError>;
}

pub struct CommandCenterService<R, S, P, O> {
    initiatives: R,
    schedules: S,
    runs: P,
    orca: O,
    idempotency: Arc<Mutex<HashMap<(String, String), CommandResult>>>,
}

impl<R, S, P, O> CommandCenterService<R, S, P, O> {
    pub fn new(initiatives: R, schedules: S, runs: P, orca: O) -> Self {
        Self {
            initiatives,
            schedules,
            runs,
            orca,
            idempotency: Arc::new(Mutex::new(HashMap::new())),
        }
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
        Ok(CommandCenterSnapshot {
            metadata: ProtocolMetadata::default(),
            revision,
            initiative: InitiativeProjection::from((goal, revision)),
            schedules: self.schedules.schedules_for(id).await?,
            runs: self.runs.runs_for(id).await?,
            orca: self
                .orca
                .observe(id)
                .await
                .unwrap_or_else(|err| OrcaReference {
                    project_id: None,
                    run_id: None,
                    worker_ids: Vec::new(),
                    terminal_ids: Vec::new(),
                    gate_ids: Vec::new(),
                    last_observed_at: None,
                    freshness: Freshness::unavailable(err.to_string()),
                }),
            freshness: Freshness::fresh(),
            available_actions: AvailableActions {
                update_initiative: true,
                checkpoint_initiative: true,
                manage_blockers: true,
                manage_next_actions: true,
                start_initiative_run: true,
                retry_linked_run: true,
                cancel_linked_run: true,
            },
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
        if let Ok(payload) = self.execute_runtime_command(&envelope).await {
            return base(CommandState::Pending, Some(payload), None);
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

    async fn execute_runtime_command(
        &self,
        envelope: &CommandEnvelope,
    ) -> Result<CommandResultPayload, CommandCenterError> {
        match &envelope.payload {
            CommandPayload::StartInitiativeRun { initiative_id } => self
                .orca
                .start_initiative_run(initiative_id, &envelope.idempotency_key)
                .await
                .map(|(run, orca_run_id)| CommandResultPayload::RunAccepted {
                    run,
                    orca_run_id: Some(orca_run_id),
                }),
            CommandPayload::RetryLinkedRun {
                initiative_id,
                run_id,
            } => self
                .orca
                .retry_linked_run(initiative_id, run_id, &envelope.idempotency_key)
                .await
                .map(|(run, orca_run_id)| CommandResultPayload::RunAccepted {
                    run,
                    orca_run_id: Some(orca_run_id),
                }),
            CommandPayload::CancelLinkedRun {
                initiative_id,
                run_id,
            } => self
                .orca
                .cancel_linked_run(initiative_id, run_id, &envelope.idempotency_key)
                .await
                .map(|run| CommandResultPayload::RunAccepted {
                    run,
                    orca_run_id: None,
                }),
            _ => Err(CommandCenterError::UnsupportedCapability {
                capability: "not_runtime".to_string(),
            }),
        }
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
    mut config: CommandCenterConfig,
    sessions: BrowserSessionStore,
    api: Arc<dyn CommandCenterApi>,
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
    Router::new()
        .route("/api/command-center/bootstrap", post(bootstrap_handler))
        .route("/api/command-center/initiatives", get(list_handler))
        .route(
            "/api/command-center/initiatives/{id}/snapshot",
            get(snapshot_handler),
        )
        .route("/api/command-center/commands", post(command_handler))
        .route("/api/command-center/replay", get(replay_handler))
        .with_state(state)
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
            HeaderValue::from_static("default-src 'self'; frame-ancestors 'none'"),
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
        correlation_id: Uuid::new_v4().to_string(),
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
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use jcode_task_types::{GoalScope, GoalStep};

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
            _id: &InitiativeId,
        ) -> Result<Vec<JcodeRunReference>, CommandCenterError> {
            Ok(vec![])
        }
    }

    struct FakeOrca {
        unavailable: bool,
    }
    #[async_trait]
    impl OrcaAdapter for FakeOrca {
        async fn observe(&self, _id: &InitiativeId) -> Result<OrcaReference, CommandCenterError> {
            if self.unavailable {
                return Err(CommandCenterError::OrcaUnavailable);
            }
            Ok(OrcaReference {
                project_id: Some(OrcaProjectId("orca-project".into())),
                run_id: None,
                worker_ids: vec![],
                terminal_ids: vec![],
                gate_ids: vec![],
                last_observed_at: Some(Utc::now()),
                freshness: Freshness::fresh(),
            })
        }
        async fn start_initiative_run(
            &self,
            id: &InitiativeId,
            _key: &IdempotencyKey,
        ) -> Result<(JcodeRunReference, OrcaRunId), CommandCenterError> {
            let now = Utc::now();
            Ok((
                JcodeRunReference {
                    id: JcodeRunId("run-1".into()),
                    initiative_id: id.clone(),
                    orca_run_id: Some(OrcaRunId("orca-run-1".into())),
                    status: "accepted".into(),
                    created_at: now,
                    updated_at: now,
                },
                OrcaRunId("orca-run-1".into()),
            ))
        }
        async fn retry_linked_run(
            &self,
            id: &InitiativeId,
            _run_id: &JcodeRunId,
            key: &IdempotencyKey,
        ) -> Result<(JcodeRunReference, OrcaRunId), CommandCenterError> {
            self.start_initiative_run(id, key).await
        }
        async fn cancel_linked_run(
            &self,
            id: &InitiativeId,
            run_id: &JcodeRunId,
            _key: &IdempotencyKey,
        ) -> Result<JcodeRunReference, CommandCenterError> {
            let now = Utc::now();
            Ok(JcodeRunReference {
                id: run_id.clone(),
                initiative_id: id.clone(),
                orca_run_id: Some(OrcaRunId("orca-run-1".into())),
                status: "cancel_accepted".into(),
                created_at: now,
                updated_at: now,
            })
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
        CommandCenterService::new(
            MemoryRepo(Arc::new(Mutex::new((goal(), Revision(1))))),
            EmptySchedules,
            EmptyRuns,
            FakeOrca { unavailable },
        )
    }

    fn envelope(payload: CommandPayload, rev: Revision, key: &str) -> CommandEnvelope {
        CommandEnvelope {
            id: CommandId(format!("cmd-{key}")),
            idempotency_key: IdempotencyKey(key.into()),
            correlation_id: "corr".into(),
            auth: auth(),
            expected_revision: rev,
            payload,
        }
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
        spawn_command_center_http_host(
            CommandCenterConfig {
                enabled: true,
                bind_addr: SocketAddr::from((Ipv4Addr::LOCALHOST, 0)),
                allowed_origins: Vec::new(),
                authenticated_remote: false,
            },
            BrowserSessionStore::new(ttl),
            runtime(),
        )
        .await
        .unwrap()
        .unwrap()
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
            .json::<InitiativeListSnapshot>()
            .await
            .unwrap();
        assert_eq!(listed.initiatives.len(), 1);

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
}

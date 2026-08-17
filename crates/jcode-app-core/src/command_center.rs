use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use chrono::Utc;
use jcode_command_center::{
    AuthContext, CancelLinkedRunRequest, CommandCenterError, Freshness, InitiativeId,
    InitiativeRepository, JcodeRunReference, LinkedScheduleProjection, OrcaAdapter,
    OrcaCanonicalPlacement, OrcaProjectId, OrcaReference, OrcaTerminalId, OrcaWorkerLauncher,
    RetryLinkedRunRequest, Revision, RunProjectionSource, RuntimeCommandExecution,
    RuntimeMutationCapabilities, ScheduleProjectionSource, ScheduleRefId,
    StartInitiativeRunRequest,
};
use jcode_task_types::Goal;
use serde::{Deserialize, Serialize};
use tokio::process::Command;

use crate::ambient::AmbientManager;
use crate::command_center_orca::OrcaCompatibilityProfile;
use orca_lifecycle::{OrcaCoordinatorBinding, OrcaLifecycleAdapter, OrcaLifecycleConfig};

#[path = "command_center/orca_lifecycle.rs"]
mod orca_lifecycle;

#[cfg(test)]
#[path = "command_center/orca_lifecycle_tests.rs"]
mod orca_lifecycle_tests;

#[cfg(test)]
#[path = "command_center/orca_lifecycle_acceptance_tests.rs"]
mod orca_lifecycle_acceptance_tests;

pub(super) async fn spawn_managed_http_host(runtime: &crate::server::runtime::ServerRuntime) {
    let enabled = std::env::var("JCODE_COMMAND_CENTER_ENABLED")
        .ok()
        .is_some_and(|value| matches!(value.as_str(), "1" | "true" | "yes"));
    if !enabled {
        return;
    }

    let bind_addr = std::env::var("JCODE_COMMAND_CENTER_BIND_ADDR")
        .unwrap_or_else(|_| "127.0.0.1:0".to_string())
        .parse()
        .unwrap_or_else(|error| {
            crate::logging::warn(&format!(
                "Invalid JCODE_COMMAND_CENTER_BIND_ADDR, using loopback ephemeral port: {error}"
            ));
            "127.0.0.1:0".parse().expect("valid loopback address")
        });
    let allowed_origins = std::env::var("JCODE_COMMAND_CENTER_ALLOWED_ORIGINS")
        .ok()
        .map(|value| {
            value
                .split(',')
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(str::to_string)
                .collect()
        })
        .unwrap_or_default();
    let authenticated_remote = std::env::var("JCODE_COMMAND_CENTER_AUTHENTICATED_REMOTE")
        .ok()
        .is_some_and(|value| matches!(value.as_str(), "1" | "true" | "yes"));
    let asset_dir = std::env::var_os("JCODE_COMMAND_CENTER_ASSET_DIR").map(PathBuf::from);
    let decision_inbox_db_path = std::env::var_os("JCODE_DECISION_INBOX_DB")
        .map(PathBuf::from)
        .or_else(|| {
            crate::storage::jcode_dir()
                .ok()
                .map(|home| home.join("intake").join("decision-inbox.sqlite"))
        });
    let config = jcode_command_center::CommandCenterConfig {
        enabled: true,
        bind_addr,
        allowed_origins,
        authenticated_remote,
        asset_dir,
        decision_inbox_db_path,
    };
    let working_dir = std::env::current_dir().ok();
    let orca = OrcaCliAdapter::for_working_dir(working_dir.clone());
    if let Err(error) = orca.reconcile_pending_operations().await {
        crate::logging::warn(&format!(
            "Command Center Orca reconciliation failed during startup: {error}"
        ));
    }
    let service = service_for_working_dir_and_orca(working_dir, orca);
    let api = Arc::new(jcode_command_center::CommandCenterRuntime::new(
        service,
        jcode_command_center::StreamId(format!("daemon-{}", uuid::Uuid::new_v4())),
    ));
    let sessions = jcode_command_center::BrowserSessionStore::new(chrono::Duration::minutes(15));
    let host =
        match jcode_command_center::spawn_command_center_http_host(config, sessions, api).await {
            Ok(Some(host)) => host,
            Ok(None) => return,
            Err(error) => {
                crate::logging::error(&format!("Command Center failed to start: {error}"));
                return;
            }
        };
    let addr = host.addr();
    crate::logging::info(&format!(
        "Command Center listening on http://{addr} (managed, browser sessions expire after 15m)"
    ));
    let spawned = runtime
        .spawn_cancellable_background_task(move |cancellation| async move {
            cancellation.cancelled().await;
            if let Err(error) = host.shutdown().await {
                crate::logging::warn(&format!("Command Center shutdown failed: {error}"));
            }
        })
        .await;
    if !spawned {
        crate::logging::warn("Command Center lifecycle task was rejected during shutdown");
    }
}

/// Concrete Command Center adapters backed by the durable stores Jcode already owns.
///
/// This module intentionally does not start or wire the HTTP server. It only exposes
/// adapter implementations that the server lifecycle can opt into later.
pub fn service_for_working_dir(
    working_dir: Option<PathBuf>,
) -> jcode_command_center::CommandCenterService<
    GoalInitiativeRepository,
    AmbientScheduleProjectionSource,
    SessionRunProjectionSource,
    OrcaCliAdapter,
> {
    let orca = OrcaCliAdapter::for_working_dir(working_dir.clone());
    service_for_working_dir_and_orca(working_dir, orca)
}

fn service_for_working_dir_and_orca(
    working_dir: Option<PathBuf>,
    orca: OrcaCliAdapter,
) -> jcode_command_center::CommandCenterService<
    GoalInitiativeRepository,
    AmbientScheduleProjectionSource,
    SessionRunProjectionSource,
    OrcaCliAdapter,
> {
    jcode_command_center::CommandCenterService::new(
        GoalInitiativeRepository::new(working_dir.clone()),
        AmbientScheduleProjectionSource::new(),
        SessionRunProjectionSource::new(),
        orca,
    )
    .with_external_signal_projection(Arc::new(ExternalSignalProjectionSource))
}

struct ExternalSignalProjectionSource;

#[async_trait]
impl jcode_command_center::ExternalSignalProjectionSource for ExternalSignalProjectionSource {
    async fn projection(&self) -> Result<serde_json::Value, String> {
        let config = crate::external_signal::ExternalSignalConfig::from_env()
            .map_err(|error| error.to_string())?;
        let path = crate::storage::jcode_dir()
            .map_err(|error| error.to_string())?
            .join("external-signals")
            .join("state.json");
        let projection = crate::external_signal::command_center_projection(&path, &config)
            .map_err(|error| error.to_string())?;
        serde_json::to_value(projection).map_err(|error| error.to_string())
    }
}

#[derive(Debug, Clone)]
pub struct GoalInitiativeRepository {
    working_dir: Option<PathBuf>,
}

impl GoalInitiativeRepository {
    pub fn new(working_dir: Option<PathBuf>) -> Self {
        Self { working_dir }
    }

    fn working_dir(&self) -> Option<&Path> {
        self.working_dir.as_deref()
    }
}

#[async_trait]
impl InitiativeRepository for GoalInitiativeRepository {
    async fn list(&self, auth: &AuthContext) -> Result<Vec<(Goal, Revision)>, CommandCenterError> {
        let mut goals =
            crate::goal::list_relevant_goals(self.working_dir()).map_err(storage_error)?;
        goals.retain(|goal| auth.allows(&InitiativeId(goal.id.clone())));
        Ok(goals.into_iter().map(goal_with_revision).collect())
    }

    async fn get(
        &self,
        auth: &AuthContext,
        id: &InitiativeId,
    ) -> Result<(Goal, Revision), CommandCenterError> {
        if !auth.allows(id) {
            return Err(CommandCenterError::Forbidden);
        }
        let goal = crate::goal::load_goal(&id.0, None, self.working_dir())
            .map_err(storage_error)?
            .ok_or_else(|| CommandCenterError::NotFound {
                entity: format!("initiative:{}", id.0),
            })?;
        Ok(goal_with_revision(goal))
    }

    async fn save(
        &self,
        auth: &AuthContext,
        goal: Goal,
        expected: Revision,
    ) -> Result<(Goal, Revision), CommandCenterError> {
        let id = InitiativeId(goal.id.clone());
        if !auth.allows(&id) {
            return Err(CommandCenterError::Forbidden);
        }
        let current = crate::goal::load_goal(&goal.id, Some(goal.scope), self.working_dir())
            .map_err(storage_error)?
            .ok_or_else(|| CommandCenterError::NotFound {
                entity: format!("initiative:{}", goal.id),
            })?;
        let actual = revision_for_goal(&current);
        if actual != expected {
            return Err(CommandCenterError::StaleRevision { expected, actual });
        }

        let updated = crate::goal::update_goal(
            &goal.id,
            Some(goal.scope),
            self.working_dir(),
            crate::goal::GoalUpdateInput {
                title: Some(goal.title),
                description: Some(goal.description),
                why: Some(goal.why),
                status: Some(goal.status),
                success_criteria: Some(goal.success_criteria),
                milestones: Some(goal.milestones),
                next_steps: Some(goal.next_steps),
                blockers: Some(goal.blockers),
                current_milestone_id: Some(goal.current_milestone_id),
                progress_percent: Some(goal.progress_percent),
                updates: Some(goal.updates),
                checkpoint_summary: None,
            },
        )
        .map_err(storage_error)?
        .ok_or_else(|| CommandCenterError::NotFound {
            entity: format!("initiative:{}", id.0),
        })?;
        Ok(goal_with_revision(updated))
    }
}

#[derive(Debug, Clone, Default)]
pub struct AmbientScheduleProjectionSource;

impl AmbientScheduleProjectionSource {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl ScheduleProjectionSource for AmbientScheduleProjectionSource {
    async fn schedules_for(
        &self,
        id: &InitiativeId,
    ) -> Result<Vec<LinkedScheduleProjection>, CommandCenterError> {
        let manager = AmbientManager::new().map_err(storage_error)?;
        let projections = manager
            .queue()
            .items()
            .iter()
            .filter(|item| schedule_mentions_initiative(item, id))
            .map(|item| LinkedScheduleProjection {
                id: ScheduleRefId(item.id.clone()),
                initiative_id: id.clone(),
                cadence: "one_shot".to_string(),
                timezone: "UTC".to_string(),
                next_fire_at: Some(item.scheduled_for),
                last_result: None,
                last_run_id: None,
                retry_count: 0,
                missed_wake: item.scheduled_for < Utc::now(),
                stale_claim: false,
                failure_evidence: None,
                freshness: Freshness::fresh(),
            })
            .collect();
        Ok(projections)
    }
}

#[derive(Debug, Clone, Default)]
pub struct SessionRunProjectionSource {
    store: RunRecordStore,
}

impl SessionRunProjectionSource {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_store_path(path: PathBuf) -> Self {
        Self {
            store: RunRecordStore::new(path),
        }
    }
}

#[async_trait]
impl RunProjectionSource for SessionRunProjectionSource {
    async fn runs_for(
        &self,
        id: &InitiativeId,
    ) -> Result<Vec<JcodeRunReference>, CommandCenterError> {
        Ok(self
            .store
            .load()
            .map_err(storage_error)?
            .into_iter()
            .filter(|run| run.initiative_id == *id)
            .collect())
    }
}

#[derive(Debug, Clone)]
pub struct OrcaCliAdapter {
    command: String,
    working_dir: Option<PathBuf>,
    runner: Arc<dyn OrcaCommandRunner>,
    lifecycle: Option<Arc<OrcaLifecycleAdapter>>,
}

impl Default for OrcaCliAdapter {
    fn default() -> Self {
        Self::new(std::env::var("JCODE_ORCA_CLI").unwrap_or_else(|_| "orca".to_string()))
    }
}

impl OrcaCliAdapter {
    pub fn new(command: impl Into<String>) -> Self {
        Self::with_working_dir(command, std::env::current_dir().ok())
    }

    pub fn for_working_dir(working_dir: Option<PathBuf>) -> Self {
        Self::with_working_dir(
            std::env::var("JCODE_ORCA_CLI").unwrap_or_else(|_| "orca".to_string()),
            working_dir,
        )
    }

    fn with_working_dir(command: impl Into<String>, working_dir: Option<PathBuf>) -> Self {
        let command = command.into();
        let runner: Arc<dyn OrcaCommandRunner> = Arc::new(ProcessOrcaCommandRunner);
        Self::with_components(command, working_dir, runner)
    }

    fn with_components(
        command: String,
        working_dir: Option<PathBuf>,
        runner: Arc<dyn OrcaCommandRunner>,
    ) -> Self {
        let coordinator = std::env::var("JCODE_COMMAND_CENTER_ORCA_COORDINATOR_TERMINAL")
            .ok()
            .filter(|value| !value.trim().is_empty());
        let agent = std::env::var("JCODE_COMMAND_CENTER_ORCA_AGENT")
            .ok()
            .filter(|value| !value.trim().is_empty());
        let store = crate::storage::jcode_dir()
            .ok()
            .and_then(|home| {
                jcode_command_center::orca_operation_store::SqliteOrcaOperationStore::open(
                    home.join("command-center").join("orca-operations.sqlite"),
                )
                .ok()
            })
            .map(Arc::new);
        let lifecycle = match (coordinator, agent, store) {
            (Some(terminal), Some(agent), Some(store)) => {
                Some(Arc::new(OrcaLifecycleAdapter::new(OrcaLifecycleConfig {
                    command: command.clone(),
                    working_dir: working_dir.clone(),
                    runner: Arc::clone(&runner),
                    store,
                    coordinator: OrcaCoordinatorBinding {
                        terminal: OrcaTerminalId(terminal),
                    },
                    launcher: OrcaWorkerLauncher::Agent {
                        agent,
                        model: None,
                        effort: None,
                    },
                    timeout: Duration::from_secs(60),
                })))
            }
            _ => None,
        };
        Self {
            command,
            working_dir,
            runner,
            lifecycle,
        }
    }

    async fn reconcile_pending_operations(&self) -> Result<(), CommandCenterError> {
        if let Some(lifecycle) = &self.lifecycle {
            lifecycle.reconcile_pending_operations().await?;
        }
        Ok(())
    }

    #[cfg(test)]
    fn with_runner(
        command: impl Into<String>,
        working_dir: Option<PathBuf>,
        runner: Arc<dyn OrcaCommandRunner>,
    ) -> Self {
        Self::with_components(command.into(), working_dir, runner)
    }

    async fn status(&self) -> Result<OrcaStatusResponse, CommandCenterError> {
        let output = self
            .runner
            .run(
                &self.command,
                &["status".to_string(), "--json".to_string()],
                self.working_dir.as_deref(),
                Duration::from_secs(10),
            )
            .await
            .map_err(|_| CommandCenterError::OrcaUnavailable)?;
        if output.exit_code != Some(0) {
            return Err(CommandCenterError::OrcaUnavailable);
        }
        serde_json::from_slice(&output.stdout).map_err(|_| CommandCenterError::OrcaUnavailable)
    }

    async fn validate_compatibility_profile(&self) -> Result<(), CommandCenterError> {
        let status = self
            .runner
            .run(
                &self.command,
                &["status".to_string(), "--json".to_string()],
                self.working_dir.as_deref(),
                Duration::from_secs(10),
            )
            .await
            .map_err(|_| incompatible_orca_profile())?;
        let registry = self
            .runner
            .run(
                &self.command,
                &["agent-context".to_string(), "--json".to_string()],
                self.working_dir.as_deref(),
                Duration::from_secs(10),
            )
            .await
            .map_err(|_| incompatible_orca_profile())?;
        let status: serde_json::Value =
            serde_json::from_slice(&status.stdout).map_err(|_| incompatible_orca_profile())?;
        let registry: serde_json::Value =
            serde_json::from_slice(&registry.stdout).map_err(|_| incompatible_orca_profile())?;
        OrcaCompatibilityProfile::pinned()
            .and_then(|profile| profile.validate_discovery_values(&status, &registry))
            .map_err(|_| incompatible_orca_profile())
    }

    async fn canonical_project_id(&self) -> Result<OrcaProjectId, CommandCenterError> {
        let working_dir = self
            .working_dir
            .as_deref()
            .ok_or_else(unresolved_orca_identity)?
            .canonicalize()
            .map_err(|_| unresolved_orca_identity())?;
        let output = self
            .runner
            .run(
                &self.command,
                &["repo".to_string(), "list".to_string(), "--json".to_string()],
                self.working_dir.as_deref(),
                Duration::from_secs(10),
            )
            .await
            .map_err(|_| CommandCenterError::OrcaUnavailable)?;
        let response: OrcaRepoListResponse = serde_json::from_slice(&output.stdout)
            .map_err(|_| CommandCenterError::OrcaUnavailable)?;
        if !response.ok {
            return Err(CommandCenterError::OrcaUnavailable);
        }

        let mut matches = response
            .result
            .repos
            .into_iter()
            .filter_map(|repo| {
                let path = Path::new(&repo.path).canonicalize().ok()?;
                working_dir.starts_with(&path).then_some((path, repo.id))
            })
            .collect::<Vec<_>>();
        matches.sort_by_key(|(path, _)| std::cmp::Reverse(path.components().count()));
        let (matched_path, matched_id) = matches.first().ok_or_else(unresolved_orca_identity)?;
        let matched_depth = matched_path.components().count();
        if matched_id.trim().is_empty()
            || matches
                .get(1)
                .is_some_and(|(path, _)| path.components().count() == matched_depth)
        {
            return Err(unresolved_orca_identity());
        }
        Ok(OrcaProjectId(matched_id.clone()))
    }
}

#[async_trait]
impl OrcaAdapter for OrcaCliAdapter {
    async fn capabilities(&self) -> Result<RuntimeMutationCapabilities, CommandCenterError> {
        // Lifecycle mutations remain unadvertised until task 4.5e runs the live
        // acceptance gate. Capability observation is deliberately side-effect free.
        Ok(RuntimeMutationCapabilities::unavailable())
    }

    async fn observe(&self, _id: &InitiativeId) -> Result<OrcaReference, CommandCenterError> {
        let status = self.status().await?;
        if !status.ok || !status.result.runtime.reachable {
            return Err(CommandCenterError::OrcaUnavailable);
        }
        let runtime_id = status.result.runtime.runtime_id;
        let project_id = self.canonical_project_id().await?;
        Ok(OrcaReference {
            project_id: Some(project_id),
            runtime_id,
            run_id: None,
            task_ids: Vec::new(),
            dispatch_ids: Vec::new(),
            worktree_ids: Vec::new(),
            worker_ids: Vec::new(),
            terminal_ids: Vec::new(),
            gate_ids: status.result.runtime.capabilities,
            correlation_ids: Vec::new(),
            idempotency_keys: Vec::new(),
            last_observed_at: Some(Utc::now()),
            freshness: Freshness::fresh(),
        })
    }

    async fn canonical_placement(
        &self,
        _id: &InitiativeId,
    ) -> Result<OrcaCanonicalPlacement, CommandCenterError> {
        let lifecycle = self
            .lifecycle
            .as_ref()
            .ok_or(CommandCenterError::OrcaCoordinatorUnavailable)?;
        lifecycle.canonical_placement().await
    }

    async fn start_initiative_run(
        &self,
        request: StartInitiativeRunRequest,
    ) -> RuntimeCommandExecution {
        match self.lifecycle.as_ref() {
            Some(lifecycle) => lifecycle.start(request).await,
            None => RuntimeCommandExecution::failed(CommandCenterError::OrcaCoordinatorUnavailable),
        }
    }

    async fn retry_linked_run(&self, request: RetryLinkedRunRequest) -> RuntimeCommandExecution {
        match self.lifecycle.as_ref() {
            Some(lifecycle) => lifecycle.retry(request).await,
            None => RuntimeCommandExecution::failed(CommandCenterError::OrcaCoordinatorUnavailable),
        }
    }

    async fn cancel_linked_run(&self, request: CancelLinkedRunRequest) -> RuntimeCommandExecution {
        match self.lifecycle.as_ref() {
            Some(lifecycle) => lifecycle.cancel(request).await,
            None => RuntimeCommandExecution::failed(CommandCenterError::OrcaCoordinatorUnavailable),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OrcaCommandOutput {
    pub exit_code: Option<i32>,
    pub stdout: Vec<u8>,
    pub stderr: Vec<u8>,
}

#[derive(Debug)]
pub enum OrcaProcessError {
    Spawn(std::io::Error),
    Timeout,
    Transport(std::io::Error),
}

impl std::fmt::Display for OrcaProcessError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Spawn(error) => write!(formatter, "failed to spawn Orca: {error}"),
            Self::Timeout => formatter.write_str("Orca command timed out"),
            Self::Transport(error) => write!(formatter, "Orca transport failed: {error}"),
        }
    }
}

impl std::error::Error for OrcaProcessError {}

#[async_trait]
pub trait OrcaCommandRunner: Send + Sync + std::fmt::Debug {
    async fn run(
        &self,
        command: &str,
        args: &[String],
        current_dir: Option<&Path>,
        timeout: Duration,
    ) -> Result<OrcaCommandOutput, OrcaProcessError>;
}

#[derive(Debug)]
struct ProcessOrcaCommandRunner;

#[async_trait]
impl OrcaCommandRunner for ProcessOrcaCommandRunner {
    async fn run(
        &self,
        command: &str,
        args: &[String],
        current_dir: Option<&Path>,
        timeout: Duration,
    ) -> Result<OrcaCommandOutput, OrcaProcessError> {
        let mut process = Command::new(command);
        process
            .args(args)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .kill_on_drop(true);
        if let Some(current_dir) = current_dir {
            process.current_dir(current_dir);
        }
        let child = process.spawn().map_err(OrcaProcessError::Spawn)?;
        let output = tokio::time::timeout(timeout, child.wait_with_output())
            .await
            .map_err(|_| OrcaProcessError::Timeout)?
            .map_err(OrcaProcessError::Transport)?;
        Ok(OrcaCommandOutput {
            exit_code: output.status.code(),
            stdout: output.stdout,
            stderr: output.stderr,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct OrcaStatusResponse {
    #[serde(default)]
    ok: bool,
    #[serde(default)]
    result: OrcaStatusResult,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct OrcaStatusResult {
    #[serde(default)]
    runtime: OrcaRuntimeStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct OrcaRuntimeStatus {
    #[serde(default)]
    reachable: bool,
    #[serde(default, alias = "runtimeId")]
    runtime_id: Option<String>,
    #[serde(default)]
    capabilities: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct OrcaRepoListResponse {
    #[serde(default)]
    ok: bool,
    #[serde(default)]
    result: OrcaRepoListResult,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct OrcaRepoListResult {
    #[serde(default)]
    repos: Vec<OrcaRepo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct OrcaRepo {
    id: String,
    path: PathBuf,
}

#[derive(Debug, Clone)]
struct RunRecordStore {
    path: PathBuf,
}

impl Default for RunRecordStore {
    fn default() -> Self {
        let path = crate::storage::jcode_dir()
            .unwrap_or_else(|_| PathBuf::from("."))
            .join("command-center")
            .join("runs.json");
        Self::new(path)
    }
}

impl RunRecordStore {
    fn new(path: PathBuf) -> Self {
        Self { path }
    }

    fn load(&self) -> anyhow::Result<Vec<JcodeRunReference>> {
        if !self.path.exists() {
            return Ok(Vec::new());
        }
        crate::storage::read_json(&self.path)
    }
}

fn goal_with_revision(goal: Goal) -> (Goal, Revision) {
    let revision = revision_for_goal(&goal);
    (goal, revision)
}

fn revision_for_goal(goal: &Goal) -> Revision {
    Revision(goal.updated_at.timestamp_millis().max(0) as u64)
}

fn storage_error(error: impl std::fmt::Display) -> CommandCenterError {
    CommandCenterError::InvalidCommand {
        reason: error.to_string(),
    }
}

fn unresolved_orca_identity() -> CommandCenterError {
    CommandCenterError::InvalidCommand {
        reason: "unresolved canonical Orca repository identity".to_string(),
    }
}

fn incompatible_orca_profile() -> CommandCenterError {
    CommandCenterError::UnsupportedCapability {
        capability: "orca.command_center.compatibility_profile.1.4.176".to_string(),
    }
}

fn schedule_mentions_initiative(item: &crate::ambient::ScheduledItem, id: &InitiativeId) -> bool {
    let needle = id.0.as_str();
    item.context.contains(needle)
        || item
            .task_description
            .as_deref()
            .is_some_and(|value| value.contains(needle))
        || item
            .additional_context
            .as_deref()
            .is_some_and(|value| value.contains(needle))
}

#[cfg(test)]
mod tests {
    use super::*;
    use jcode_command_center::AuthContext;
    use jcode_task_types::GoalScope;
    use serde_json::json;
    use std::collections::VecDeque;
    use std::sync::Mutex;

    fn with_home<T>(f: impl FnOnce(PathBuf, PathBuf) -> T) -> T {
        let _guard = crate::storage::lock_test_env();
        let temp = tempfile::tempdir().expect("tempdir");
        let project = temp.path().join("repo");
        std::fs::create_dir_all(&project).expect("project dir");
        let prev_home = std::env::var_os("JCODE_HOME");
        crate::env::set_var("JCODE_HOME", temp.path());
        let result = f(temp.path().to_path_buf(), project);
        if let Some(prev_home) = prev_home {
            crate::env::set_var("JCODE_HOME", prev_home);
        } else {
            crate::env::remove_var("JCODE_HOME");
        }
        result
    }

    #[test]
    fn goal_repository_lists_gets_and_saves_with_revision_check() {
        with_home(|_, project| {
            tokio::runtime::Runtime::new()
                .unwrap()
                .block_on(async move {
                    let goal = crate::goal::create_goal(
                        crate::goal::GoalCreateInput {
                            title: "Ship Command Center".to_string(),
                            scope: GoalScope::Project,
                            next_steps: vec!["wire adapters".to_string()],
                            ..crate::goal::GoalCreateInput::default()
                        },
                        Some(&project),
                    )
                    .expect("create goal");
                    let repo = GoalInitiativeRepository::new(Some(project.clone()));
                    let auth = AuthContext {
                        session_id: "test-session".to_string(),
                        user_label: None,
                        csrf_token: "csrf".to_string(),
                        expires_at: Utc::now() + chrono::Duration::minutes(5),
                        allowed_initiatives: Vec::new(),
                    };

                    let listed = repo.list(&auth).await.expect("list");
                    assert_eq!(listed.len(), 1);
                    let (mut loaded, revision) = repo
                        .get(&auth, &InitiativeId(goal.id.clone()))
                        .await
                        .expect("get");
                    loaded.next_steps.push("run tests".to_string());
                    let (saved, saved_revision) =
                        repo.save(&auth, loaded, revision).await.expect("save");
                    assert!(saved.next_steps.iter().any(|step| step == "run tests"));
                    assert_ne!(saved_revision, Revision(0));

                    let stale = repo.save(&auth, saved, revision).await.expect_err("stale");
                    assert!(matches!(stale, CommandCenterError::StaleRevision { .. }));
                })
        });
    }

    #[test]
    fn goal_repository_persists_checkpoint_history_appended_by_the_service() {
        with_home(|_, project| {
            tokio::runtime::Runtime::new()
                .unwrap()
                .block_on(async move {
                    let goal = crate::goal::create_goal(
                        crate::goal::GoalCreateInput {
                            title: "Ship Command Center".to_string(),
                            scope: GoalScope::Project,
                            ..crate::goal::GoalCreateInput::default()
                        },
                        Some(&project),
                    )
                    .expect("create goal");
                    let repo = GoalInitiativeRepository::new(Some(project.clone()));
                    let auth = AuthContext {
                        session_id: "test-session".to_string(),
                        user_label: None,
                        csrf_token: "csrf".to_string(),
                        expires_at: Utc::now() + chrono::Duration::minutes(5),
                        allowed_initiatives: Vec::new(),
                    };

                    let (mut loaded, revision) = repo
                        .get(&auth, &InitiativeId(goal.id.clone()))
                        .await
                        .expect("get");
                    let before = loaded.updates.len();
                    // Mirrors what CommandCenterService does for CommandPayload::Checkpoint.
                    loaded.updates.push(jcode_task_types::GoalUpdate {
                        at: Utc::now(),
                        summary: "browser checkpoint".to_string(),
                    });

                    let (saved, _) = repo.save(&auth, loaded, revision).await.expect("save");
                    assert_eq!(
                        saved.updates.len(),
                        before + 1,
                        "checkpoint must survive the save round trip"
                    );
                    assert_eq!(
                        saved.updates.last().map(|u| u.summary.as_str()),
                        Some("browser checkpoint")
                    );

                    // The checkpoint must be durable on disk, not just in the response.
                    let (reloaded, _) = repo
                        .get(&auth, &InitiativeId(goal.id.clone()))
                        .await
                        .expect("reload");
                    assert_eq!(
                        reloaded.updates.last().map(|u| u.summary.as_str()),
                        Some("browser checkpoint"),
                        "checkpoint must be persisted to the goal store"
                    );
                })
        });
    }

    #[test]
    fn ambient_schedule_source_projects_matching_queued_items() {
        with_home(|_, _project| {
            tokio::runtime::Runtime::new()
                .unwrap()
                .block_on(async move {
                    let mut manager = AmbientManager::new().expect("manager");
                    let id = manager
                        .schedule(crate::ambient::ScheduleRequest {
                            wake_in_minutes: Some(10),
                            wake_at: None,
                            context: "Follow up initiative alpha-goal".to_string(),
                            priority: crate::ambient::Priority::Normal,
                            target: crate::ambient::ScheduleTarget::Ambient,
                            created_by_session: "test".to_string(),
                            working_dir: None,
                            task_description: Some("alpha-goal".to_string()),
                            relevant_files: Vec::new(),
                            git_branch: None,
                            additional_context: None,
                        })
                        .expect("schedule");

                    let source = AmbientScheduleProjectionSource::new();
                    let schedules = source
                        .schedules_for(&InitiativeId("alpha-goal".to_string()))
                        .await
                        .expect("schedules");
                    assert_eq!(schedules.len(), 1);
                    assert_eq!(schedules[0].id, ScheduleRefId(id));
                })
        });
    }

    #[derive(Debug)]
    struct ExpectedOrcaCall {
        args: Vec<String>,
        output: Vec<u8>,
    }

    #[derive(Debug)]
    struct ScriptedRunner(Mutex<VecDeque<ExpectedOrcaCall>>);

    impl ScriptedRunner {
        fn new(calls: Vec<ExpectedOrcaCall>) -> Self {
            Self(Mutex::new(calls.into()))
        }
    }

    #[async_trait]
    impl OrcaCommandRunner for ScriptedRunner {
        async fn run(
            &self,
            _command: &str,
            args: &[String],
            _current_dir: Option<&Path>,
            _timeout: Duration,
        ) -> Result<OrcaCommandOutput, OrcaProcessError> {
            let call = self
                .0
                .lock()
                .expect("runner lock")
                .pop_front()
                .expect("unexpected Orca command");
            assert_eq!(args, call.args);
            Ok(OrcaCommandOutput {
                exit_code: Some(0),
                stdout: call.output,
                stderr: Vec::new(),
            })
        }
    }

    fn expected_call(args: &[&str], value: serde_json::Value) -> ExpectedOrcaCall {
        ExpectedOrcaCall {
            args: args.iter().map(|value| (*value).to_string()).collect(),
            output: serde_json::to_vec(&value).expect("serialize Orca response"),
        }
    }

    #[test]
    fn orca_cli_observes_status_and_rejects_unsupported_run_commands() {
        with_home(|_, project| {
            tokio::runtime::Runtime::new()
                .unwrap()
                .block_on(async move {
                    let nested_working_dir = project.join("source/jcode");
                    std::fs::create_dir_all(&nested_working_dir).expect("nested working dir");
                    let adapter = OrcaCliAdapter::with_runner(
                        "orca",
                        Some(nested_working_dir),
                        Arc::new(ScriptedRunner::new(vec![
                            expected_call(
                                &["status", "--json"],
                                json!({
                                    "ok": true,
                                    "result": {
                                        "runtime": {
                                            "reachable": true,
                                            "runtimeId": "runtime-1",
                                            "capabilities": ["orchestration.contract.v1"]
                                        }
                                    }
                                }),
                            ),
                            expected_call(
                                &["repo", "list", "--json"],
                                json!({
                                    "ok": true,
                                    "result": {
                                        "repos": [{
                                            "id": "repo-1",
                                            "path": project
                                        }]
                                    },
                                    "_meta": { "runtimeId": "runtime-1" }
                                }),
                            ),
                        ])),
                    );
                    let observed = adapter
                        .observe(&InitiativeId("alpha-goal".to_string()))
                        .await
                        .expect("observe status");
                    assert_eq!(
                        observed.project_id,
                        Some(OrcaProjectId("repo-1".to_string()))
                    );
                    assert_eq!(observed.runtime_id.as_deref(), Some("runtime-1"));
                    assert_eq!(
                        observed.gate_ids,
                        vec!["orchestration.contract.v1".to_string()]
                    );

                    assert!(!adapter.capabilities().await.unwrap().start_initiative_run);
                })
        });
    }

    #[test]
    fn orca_cli_leaves_canonical_identity_unresolved_for_zero_or_multiple_path_matches() {
        with_home(|_, project| {
            tokio::runtime::Runtime::new()
                .unwrap()
                .block_on(async move {
                    for repos in [
                        json!([]),
                        json!([
                            { "id": "repo-1", "path": project },
                            { "id": "repo-2", "path": project }
                        ]),
                    ] {
                        let adapter = OrcaCliAdapter::with_runner(
                            "orca",
                            Some(project.clone()),
                            Arc::new(ScriptedRunner::new(vec![
                                expected_call(
                                    &["status", "--json"],
                                    json!({
                                        "ok": true,
                                        "result": {
                                            "runtime": {
                                                "reachable": true,
                                                "runtimeId": "runtime-1"
                                            }
                                        }
                                    }),
                                ),
                                expected_call(
                                    &["repo", "list", "--json"],
                                    json!({
                                        "ok": true,
                                        "result": { "repos": repos }
                                    }),
                                ),
                            ])),
                        );

                        let error = adapter
                            .observe(&InitiativeId("alpha-goal".to_string()))
                            .await
                            .expect_err("identity must fail closed");
                        assert!(matches!(error, CommandCenterError::InvalidCommand { .. }));
                    }
                })
        });
    }

    #[test]
    #[ignore = "requires a live Orca runtime and registered repository path"]
    fn orca_cli_resolves_live_repository_without_using_runtime_identity() {
        let repo_path = std::env::var_os("JCODE_TEST_ORCA_REPO_PATH")
            .map(PathBuf::from)
            .expect("JCODE_TEST_ORCA_REPO_PATH must name an Orca-registered repository");
        tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(async move {
                let observed = OrcaCliAdapter::for_working_dir(Some(repo_path))
                    .observe(&InitiativeId("live-orca-identity-check".to_string()))
                    .await
                    .expect("resolve live Orca repository identity");
                let project_id = observed.project_id.expect("canonical project identity").0;
                let runtime_id = observed.runtime_id.expect("runtime metadata");
                assert_ne!(project_id, runtime_id);
                assert!(!project_id.trim().is_empty());
            });
    }

    #[test]
    fn orca_1_4_176_compatibility_fixtures_are_pinned_and_self_consistent() {
        let profile = OrcaCompatibilityProfile::pinned().expect("load pinned Orca profile");

        assert_eq!(profile.orca_version(), "1.4.176");
        profile
            .validate_pinned_fixtures()
            .expect("pinned fixtures must describe the complete compatibility profile");

        for command in [
            "orchestration run-create",
            "orchestration task-create",
            "orchestration worker-start",
            "orchestration worker-stop",
            "orchestration worker-abandon",
            "orchestration worker-release",
        ] {
            assert!(
                profile.has_required_command(command),
                "missing pinned command {command}"
            );
        }
        for fixture in [
            "worker-start.ready",
            "worker-start.failed",
            "worker-start.outcome-unknown",
            "worker-stop.stopped",
            "worker-stop.unknown",
            "worker-abandon.abandoned",
            "worker-release.released",
            "worker-release.pending",
            "worker-release.unknown",
            "error.typed-rejection",
        ] {
            profile
                .response_fixture(fixture)
                .unwrap_or_else(|| panic!("missing pinned response fixture {fixture}"));
        }
    }

    #[test]
    fn orca_compatibility_profile_rejects_version_registry_and_json_shape_drift() {
        let profile = OrcaCompatibilityProfile::pinned().expect("load pinned Orca profile");
        let status = profile
            .response_fixture("status.ready")
            .expect("status fixture")
            .clone();
        let registry = profile.command_registry_fixture().clone();

        profile
            .validate_discovery_values(&status, &registry)
            .expect("pinned discovery fixtures validate");

        let mut wrong_version = status.clone();
        wrong_version["result"]["runtime"]["appVersion"] = json!("1.4.177");
        assert!(
            profile
                .validate_discovery_values(&wrong_version, &registry)
                .is_err(),
            "an unpinned Orca version must fail closed"
        );

        let mut wrong_registry = registry.clone();
        let worker_start = wrong_registry["commands"]
            .as_array_mut()
            .expect("registry commands")
            .iter_mut()
            .find(|command| command["command"] == "orchestration worker-start")
            .expect("worker-start command");
        worker_start["flags"]
            .as_array_mut()
            .expect("worker-start flags")
            .retain(|flag| flag != "retry-of");
        assert!(
            profile
                .validate_discovery_values(&status, &wrong_registry)
                .is_err(),
            "command registry drift must fail closed"
        );

        let mut wrong_shape = profile
            .response_fixture("worker-start.ready")
            .expect("worker-start fixture")
            .clone();
        wrong_shape["result"]
            .as_object_mut()
            .expect("worker-start result")
            .remove("dispatchId");
        assert!(
            profile
                .validate_response_value("worker-start.ready", &wrong_shape)
                .is_err(),
            "JSON response shape drift must fail closed"
        );
    }

    #[test]
    fn orca_cli_validates_profile_read_only_and_keeps_mutations_unavailable() {
        tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(async move {
                let profile = OrcaCompatibilityProfile::pinned().expect("load pinned profile");
                let status = profile
                    .response_fixture("status.ready")
                    .expect("status fixture")
                    .clone();
                let registry = profile.command_registry_fixture().clone();
                let adapter = OrcaCliAdapter::with_runner(
                    "orca",
                    None,
                    Arc::new(ScriptedRunner::new(vec![
                        expected_call(&["status", "--json"], status.clone()),
                        expected_call(&["agent-context", "--json"], registry.clone()),
                        expected_call(&["status", "--json"], status),
                        expected_call(&["agent-context", "--json"], registry),
                    ])),
                );

                adapter
                    .validate_compatibility_profile()
                    .await
                    .expect("pinned profile validates");
                let capabilities = adapter.capabilities().await.expect("capabilities");
                assert!(!capabilities.start_initiative_run);
                assert!(!capabilities.retry_linked_run);
                assert!(!capabilities.cancel_linked_run);
            });
    }

    #[test]
    #[ignore = "requires the installed Orca 1.4.176 CLI and a reachable runtime"]
    fn orca_cli_validates_live_1_4_176_compatibility_profile() {
        let command = std::env::var("JCODE_TEST_ORCA_1_4_176_CLI")
            .expect("JCODE_TEST_ORCA_1_4_176_CLI must name the Orca 1.4.176 executable");
        tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(async move {
                OrcaCliAdapter::new(command)
                    .validate_compatibility_profile()
                    .await
                    .expect("live Orca 1.4.176 profile must match pinned fixtures");
            });
    }
}

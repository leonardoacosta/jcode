use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::sync::Arc;

use async_trait::async_trait;
use chrono::Utc;
use jcode_command_center::{
    AuthContext, CommandCenterError, Freshness, IdempotencyKey, InitiativeId, InitiativeRepository,
    JcodeRunId, JcodeRunReference, LinkedScheduleProjection, OrcaAdapter, OrcaProjectId,
    OrcaReference, OrcaRunId, Revision, RunProjectionSource, ScheduleProjectionSource,
    ScheduleRefId,
};
use jcode_task_types::Goal;
use serde::{Deserialize, Serialize};
use tokio::process::Command;

use crate::ambient::AmbientManager;

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
    let config = jcode_command_center::CommandCenterConfig {
        enabled: true,
        bind_addr,
        allowed_origins,
        authenticated_remote,
        asset_dir,
    };
    let service = service_for_working_dir(std::env::current_dir().ok());
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
    jcode_command_center::CommandCenterService::new(
        GoalInitiativeRepository::new(working_dir.clone()),
        AmbientScheduleProjectionSource::new(),
        SessionRunProjectionSource::new(),
        OrcaCliAdapter::for_working_dir(working_dir),
    )
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
        Self {
            command: command.into(),
            working_dir,
            runner: Arc::new(ProcessOrcaCommandRunner),
        }
    }

    #[cfg(test)]
    fn with_runner(
        command: impl Into<String>,
        working_dir: Option<PathBuf>,
        runner: Arc<dyn OrcaCommandRunner>,
    ) -> Self {
        Self {
            command: command.into(),
            working_dir,
            runner,
        }
    }

    async fn status(&self) -> Result<OrcaStatusResponse, CommandCenterError> {
        let output = self
            .runner
            .run(&self.command, &["status".to_string(), "--json".to_string()])
            .await?;
        serde_json::from_slice(&output).map_err(|_| CommandCenterError::OrcaUnavailable)
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
            )
            .await?;
        let response: OrcaRepoListResponse =
            serde_json::from_slice(&output).map_err(|_| CommandCenterError::OrcaUnavailable)?;
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

    fn unsupported_runtime_command(&self, capability: &str) -> CommandCenterError {
        CommandCenterError::UnsupportedCapability {
            capability: capability.to_string(),
        }
    }
}

#[async_trait]
impl OrcaAdapter for OrcaCliAdapter {
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
            worker_ids: Vec::new(),
            terminal_ids: Vec::new(),
            gate_ids: status.result.runtime.capabilities,
            last_observed_at: Some(Utc::now()),
            freshness: Freshness::fresh(),
        })
    }

    async fn start_initiative_run(
        &self,
        _id: &InitiativeId,
        _key: &IdempotencyKey,
    ) -> Result<(JcodeRunReference, OrcaRunId), CommandCenterError> {
        Err(self.unsupported_runtime_command("orca.command_center.start_initiative_run"))
    }

    async fn retry_linked_run(
        &self,
        _id: &InitiativeId,
        _run_id: &JcodeRunId,
        _key: &IdempotencyKey,
    ) -> Result<(JcodeRunReference, OrcaRunId), CommandCenterError> {
        Err(self.unsupported_runtime_command("orca.command_center.retry_linked_run"))
    }

    async fn cancel_linked_run(
        &self,
        _id: &InitiativeId,
        _run_id: &JcodeRunId,
        _key: &IdempotencyKey,
    ) -> Result<JcodeRunReference, CommandCenterError> {
        Err(self.unsupported_runtime_command("orca.command_center.cancel_linked_run"))
    }
}

#[async_trait]
pub trait OrcaCommandRunner: Send + Sync + std::fmt::Debug {
    async fn run(&self, command: &str, args: &[String]) -> Result<Vec<u8>, CommandCenterError>;
}

#[derive(Debug)]
struct ProcessOrcaCommandRunner;

#[async_trait]
impl OrcaCommandRunner for ProcessOrcaCommandRunner {
    async fn run(&self, command: &str, args: &[String]) -> Result<Vec<u8>, CommandCenterError> {
        let output = Command::new(command)
            .args(args)
            .stdin(Stdio::null())
            .stderr(Stdio::null())
            .output()
            .await
            .map_err(|_| CommandCenterError::OrcaUnavailable)?;
        if !output.status.success() {
            return Err(CommandCenterError::OrcaUnavailable);
        }
        Ok(output.stdout)
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
        ) -> Result<Vec<u8>, CommandCenterError> {
            let call = self
                .0
                .lock()
                .expect("runner lock")
                .pop_front()
                .expect("unexpected Orca command");
            assert_eq!(args, call.args);
            Ok(call.output)
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

                    let err = adapter
                        .start_initiative_run(
                            &InitiativeId("alpha-goal".to_string()),
                            &IdempotencyKey("key-1".to_string()),
                        )
                        .await
                        .expect_err("start unsupported");
                    assert!(matches!(
                        err,
                        CommandCenterError::UnsupportedCapability { .. }
                    ));
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
}

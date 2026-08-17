use std::collections::BTreeSet;
use std::fmt;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use chrono::Utc;
use jcode_command_center::orca_operation_store::{
    BeginOrcaOperation, NewOrcaOperation, OrcaOperationRecord, OrcaOperationState,
    OrcaOperationStoreError, OrcaOperationUpdate, OrcaPartialEffect, OrcaReceiptRecord,
    OrcaRecoveryObligation, OrcaRecoveryState, OrcaRequestRecord, OrcaTypedOperationRequest,
    SqliteOrcaOperationStore,
};
use jcode_command_center::{
    CancelLinkedRunRequest, CleanupResourceProjection, CleanupResourceState, CommandCenterError,
    CommandId, JcodeRunReference, OrcaAttemptIdentity, OrcaCanonicalPlacement, OrcaDispatchId,
    OrcaEffectReceipt, OrcaHostId, OrcaHostSetupId, OrcaLifecycleReceipt, OrcaMutationOutcome,
    OrcaProjectId, OrcaRepositoryId, OrcaRequestId, OrcaRunId, OrcaTaskId, OrcaTerminalId,
    OrcaWorkerLauncher, OrcaWorktreeId, RetryLinkedRunRequest, RuntimeCommandExecution,
    StartInitiativeRunRequest,
};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, json};
use tokio::sync::Mutex;
use uuid::Uuid;

use super::{OrcaCommandOutput, OrcaCommandRunner, OrcaProcessError};

const REQUEST_NAMESPACE: Uuid = Uuid::from_u128(0xd0f9_53c7_1bdd_5d2e_8cf1_5c74aa26_c36a);
const TASK_SPEC_SCHEMA: &str = "jcode.command-center.task.v1";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum OrcaMutationStage {
    RunCreate,
    RunBind,
    TaskCreate,
    WorkerStart,
    WorkerRetry,
    WorkerStop,
    WorkerAbandon,
    WorkerRelease,
}

impl OrcaMutationStage {
    pub(super) fn as_str(self) -> &'static str {
        match self {
            Self::RunCreate => "run_create",
            Self::RunBind => "run_bind",
            Self::TaskCreate => "task_create",
            Self::WorkerStart => "worker_start",
            Self::WorkerRetry => "worker_retry",
            Self::WorkerStop => "worker_stop",
            Self::WorkerAbandon => "worker_abandon",
            Self::WorkerRelease => "worker_release",
        }
    }
}

pub(super) fn orca_request_id(command_id: &CommandId, stage: OrcaMutationStage) -> OrcaRequestId {
    OrcaRequestId(
        Uuid::new_v5(
            &REQUEST_NAMESPACE,
            format!("{}:{}", command_id.0, stage.as_str()).as_bytes(),
        )
        .to_string(),
    )
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct OrcaCoordinatorBinding {
    pub terminal: OrcaTerminalId,
}

pub(super) struct OrcaLifecycleConfig {
    pub command: String,
    pub working_dir: Option<PathBuf>,
    pub runner: Arc<dyn OrcaCommandRunner>,
    pub store: Arc<SqliteOrcaOperationStore>,
    pub coordinator: OrcaCoordinatorBinding,
    pub launcher: OrcaWorkerLauncher,
    pub timeout: Duration,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(super) struct ReconciliationSummary {
    pub examined: usize,
    pub ready: usize,
    pub failed: usize,
    pub outcome_unknown: usize,
    pub recovery_required: usize,
    pub completed: usize,
}

fn operation_is_terminal(state: OrcaOperationState) -> bool {
    matches!(
        state,
        OrcaOperationState::Ready
            | OrcaOperationState::Rejected
            | OrcaOperationState::Failed
            | OrcaOperationState::Completed
    )
}

fn summarize_reconciled_state(summary: &mut ReconciliationSummary, state: OrcaOperationState) {
    match state {
        OrcaOperationState::Ready => summary.ready += 1,
        OrcaOperationState::Failed | OrcaOperationState::Rejected => summary.failed += 1,
        OrcaOperationState::OutcomeUnknown => summary.outcome_unknown += 1,
        OrcaOperationState::Completed => summary.completed += 1,
        OrcaOperationState::Recorded
        | OrcaOperationState::InProgress
        | OrcaOperationState::RecoveryRequired => summary.recovery_required += 1,
    }
}

#[derive(Clone)]
pub(super) struct OrcaLifecycleAdapter {
    command: String,
    working_dir: Option<PathBuf>,
    runner: Arc<dyn OrcaCommandRunner>,
    store: Arc<SqliteOrcaOperationStore>,
    coordinator: OrcaCoordinatorBinding,
    launcher: OrcaWorkerLauncher,
    timeout: Duration,
    binding: Arc<Mutex<()>>,
    startup_reconciled: Arc<AtomicBool>,
}

impl fmt::Debug for OrcaLifecycleAdapter {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("OrcaLifecycleAdapter")
            .field("command", &self.command)
            .field("working_dir", &self.working_dir)
            .field("coordinator", &self.coordinator)
            .field("launcher", &self.launcher)
            .field("timeout", &self.timeout)
            .finish_non_exhaustive()
    }
}

impl OrcaLifecycleAdapter {
    pub(super) fn new(config: OrcaLifecycleConfig) -> Self {
        Self {
            command: config.command,
            working_dir: config.working_dir,
            runner: config.runner,
            store: config.store,
            coordinator: config.coordinator,
            launcher: config.launcher,
            timeout: config.timeout,
            binding: Arc::new(Mutex::new(())),
            startup_reconciled: Arc::new(AtomicBool::new(false)),
        }
    }

    pub(super) fn runtime_store_is_ready(&self) -> bool {
        self.startup_reconciled.load(Ordering::Acquire)
            && self
                .store
                .has_unresolved_or_recoverable_operations()
                .is_ok_and(|unresolved| !unresolved)
    }

    pub(super) async fn canonical_placement(
        &self,
    ) -> Result<OrcaCanonicalPlacement, CommandCenterError> {
        let working_dir = canonical_working_dir(self.working_dir.as_deref())?;
        let worktree = self.read_result(&["worktree", "current", "--json"]).await?;
        let worktree = required_object(&worktree, "worktree")?;
        let worktree_id = required_string(worktree, "id")?;
        let repository_id = required_string_alias(worktree, &["repoId", "repo_id"])?;
        let host_id = required_string_alias(worktree, &["hostId", "host_id"])?;
        let worktree_path = canonical_path(required_string(worktree, "path")?)?;
        if !working_dir.starts_with(&worktree_path) {
            return Err(identity_unresolved(
                "working directory is outside Orca's current worktree",
            ));
        }

        let repos = self.read_result(&["repo", "list", "--json"]).await?;
        let repos = required_array(&repos, "repos")?;
        let matching_repos = repos
            .iter()
            .filter_map(Value::as_object)
            .filter(|repo| string_alias(repo, &["id"]) == Some(repository_id))
            .filter(|repo| {
                string_alias(repo, &["path"])
                    .and_then(|path| canonical_path(path).ok())
                    .is_some_and(|path| working_dir.starts_with(path))
            })
            .count();
        if matching_repos != 1 {
            return Err(identity_unresolved(
                "Orca repository identity is missing or ambiguous",
            ));
        }

        let setups = self.read_result(&["project", "setups", "--json"]).await?;
        let setups = required_array(&setups, "setups")?;
        let matching_setups = setups
            .iter()
            .filter_map(Value::as_object)
            .filter(|setup| {
                string_alias(setup, &["repoId", "repo_id"]) == Some(repository_id)
                    && string_alias(setup, &["hostId", "host_id"]) == Some(host_id)
                    && string_alias(setup, &["setupState", "setup_state"]) == Some("ready")
            })
            .collect::<Vec<_>>();
        if matching_setups.len() != 1 {
            return Err(identity_unresolved(
                "ready Orca project setup is missing or ambiguous",
            ));
        }
        let setup = matching_setups[0];
        let setup_id = required_string(setup, "id")?;
        let project_id = required_string_alias(setup, &["projectId", "project_id"])?;

        let projects = self.read_result(&["project", "list", "--json"]).await?;
        let projects = required_array(&projects, "projects")?;
        let matching_projects = projects
            .iter()
            .filter_map(Value::as_object)
            .filter(|project| string_alias(project, &["id"]) == Some(project_id))
            .filter(|project| {
                array_alias(project, &["sourceRepoIds", "source_repo_ids"])
                    .is_some_and(|ids| ids.iter().any(|id| id.as_str() == Some(repository_id)))
            })
            .count();
        if matching_projects != 1 {
            return Err(identity_unresolved(
                "Orca project identity is missing or ambiguous",
            ));
        }

        let terminal = self
            .read_result(&[
                "terminal",
                "show",
                "--terminal",
                &self.coordinator.terminal.0,
                "--json",
            ])
            .await
            .map_err(|_| CommandCenterError::OrcaCoordinatorUnavailable)?;
        let terminal = required_object(&terminal, "terminal")
            .map_err(|_| CommandCenterError::OrcaCoordinatorUnavailable)?;
        let observed_terminal = string_alias(terminal, &["handle", "id"])
            .ok_or(CommandCenterError::OrcaCoordinatorUnavailable)?;
        if observed_terminal != self.coordinator.terminal.0 {
            return Err(CommandCenterError::OrcaCoordinatorUnavailable);
        }

        let placement = OrcaCanonicalPlacement {
            project_id: OrcaProjectId(project_id.to_string()),
            repository_id: OrcaRepositoryId(repository_id.to_string()),
            host_setup_id: OrcaHostSetupId(setup_id.to_string()),
            host_id: OrcaHostId(host_id.to_string()),
            worktree_id: OrcaWorktreeId(worktree_id.to_string()),
            worktree_selector: format!("id:{worktree_id}"),
            coordinator_terminal_id: self.coordinator.terminal.clone(),
            environment: None,
            launcher: self.launcher.clone(),
        };
        validate_supported_placement(&placement)?;
        Ok(placement)
    }

    pub(super) async fn start(
        &self,
        request: StartInitiativeRunRequest,
    ) -> RuntimeCommandExecution {
        let _guard = self.binding.lock().await;
        let record = match self.begin(
            scope("start", &request.context.initiative_id.0),
            OrcaTypedOperationRequest::StartInitiativeRun(request.clone()),
        ) {
            Ok(BeginOrcaOperation::Existing(record)) if operation_is_terminal(record.state) => {
                return execution_from_record(&record);
            }
            Ok(BeginOrcaOperation::Existing(record) | BeginOrcaOperation::Created(record)) => {
                record
            }
            Err(error) => return RuntimeCommandExecution::failed(error),
        };
        let live = match self.canonical_placement().await {
            Ok(placement) => placement,
            Err(error) => {
                return self.mark_recovery(record, OrcaMutationStage::RunCreate, error);
            }
        };
        if let Err(error) = compare_placement(&request.placement, &live) {
            return self.mark_recovery(record, OrcaMutationStage::RunCreate, error);
        }
        let record = match self.persist_placement(&record, &request.placement) {
            Ok(record) => record,
            Err(error) => return RuntimeCommandExecution::failed(error),
        };
        self.resume_start(record, request).await
    }

    async fn resume_start(
        &self,
        mut record: OrcaOperationRecord,
        request: StartInitiativeRunRequest,
    ) -> RuntimeCommandExecution {
        let objective = format!(
            "[jcode-cc:{}] {}",
            request.context.command_id.0, request.objective
        );
        if record.orca_run_id.is_none()
            && record
                .requests
                .iter()
                .any(|request| request.stage == OrcaMutationStage::RunCreate.as_str())
        {
            match self.reconcile_run_marker(&objective).await {
                Ok(Some((run_id, evidence))) => {
                    record = match self.append_receipt(
                        &record,
                        OrcaMutationStage::RunCreate,
                        "reconciled",
                        evidence,
                        OrcaOperationUpdate {
                            state: Some(OrcaOperationState::InProgress),
                            orca_run_id: Some(run_id),
                            ..update_now()
                        },
                    ) {
                        Ok(record) => record,
                        Err(error) => return RuntimeCommandExecution::failed(error),
                    };
                }
                Ok(None) if record.state == OrcaOperationState::OutcomeUnknown => {
                    return execution_from_record(&record);
                }
                Ok(None) => {}
                Err(error) => {
                    return self.mark_recovery(record, OrcaMutationStage::RunCreate, error);
                }
            }
        }
        if record.orca_run_id.is_none() {
            let request_id =
                orca_request_id(&request.context.command_id, OrcaMutationStage::RunCreate);
            let args = strings(&[
                "orchestration",
                "run-create",
                "--objective",
                &objective,
                "--from",
                &self.coordinator.terminal.0,
                "--retry-request",
                &request_id.0,
                "--json",
            ]);
            let output = match self
                .persist_and_invoke(
                    &record,
                    OrcaMutationStage::RunCreate,
                    request_id.clone(),
                    args,
                    json!({"objective": objective, "terminal": self.coordinator.terminal}),
                )
                .await
            {
                Ok(output) => output,
                Err(error) => {
                    return self.handle_invocation_error(
                        record,
                        OrcaMutationStage::RunCreate,
                        error,
                    );
                }
            };
            let parsed = match parse_run_create(&output, Some(&request_id)) {
                Ok(parsed) => parsed,
                Err(error) => {
                    return self.handle_invocation_error(
                        record,
                        OrcaMutationStage::RunCreate,
                        error,
                    );
                }
            };
            record = match self.append_receipt(
                &record,
                OrcaMutationStage::RunCreate,
                "created",
                parsed.raw,
                OrcaOperationUpdate {
                    state: Some(OrcaOperationState::InProgress),
                    orca_run_id: Some(parsed.run_id),
                    ..update_now()
                },
            ) {
                Ok(record) => record,
                Err(error) => return RuntimeCommandExecution::failed(error),
            };
        }
        let run_id = record.orca_run_id.clone().expect("run persisted");
        if let Err(error) = self.ensure_run_binding(&record, &run_id).await {
            return self.mark_recovery(record, OrcaMutationStage::RunBind, error);
        }

        if record.orca_task_id.is_none() {
            let display_name = format!("jcode-cc:{}:initial", request.context.command_id.0);
            if record
                .requests
                .iter()
                .any(|request| request.stage == OrcaMutationStage::TaskCreate.as_str())
            {
                match self.reconcile_task_marker(&run_id, &display_name).await {
                    Ok(Some((task_id, evidence))) => {
                        record = match self.append_receipt(
                            &record,
                            OrcaMutationStage::TaskCreate,
                            "reconciled",
                            evidence,
                            OrcaOperationUpdate {
                                state: Some(OrcaOperationState::InProgress),
                                orca_task_id: Some(task_id),
                                ..update_now()
                            },
                        ) {
                            Ok(record) => record,
                            Err(error) => return RuntimeCommandExecution::failed(error),
                        };
                    }
                    Ok(None) if record.state == OrcaOperationState::OutcomeUnknown => {
                        return execution_from_record(&record);
                    }
                    Ok(None) => {}
                    Err(error) => {
                        return self.mark_recovery(record, OrcaMutationStage::TaskCreate, error);
                    }
                }
            }
        }
        if record.orca_task_id.is_none() {
            let request_id =
                orca_request_id(&request.context.command_id, OrcaMutationStage::TaskCreate);
            let spec = deterministic_task_spec(&request);
            let display_name = format!("jcode-cc:{}:initial", request.context.command_id.0);
            let args = vec![
                "orchestration".into(),
                "task-create".into(),
                "--spec".into(),
                spec.to_string(),
                "--task-title".into(),
                request.objective.clone(),
                "--display-name".into(),
                display_name.clone(),
                "--run".into(),
                run_id.0.clone(),
                "--from".into(),
                self.coordinator.terminal.0.clone(),
                "--retry-request".into(),
                request_id.0.clone(),
                "--json".into(),
            ];
            let output = match self
                .persist_and_invoke(
                    &record,
                    OrcaMutationStage::TaskCreate,
                    request_id.clone(),
                    args,
                    json!({"spec": spec, "displayName": display_name, "runId": run_id}),
                )
                .await
            {
                Ok(output) => output,
                Err(error) => {
                    return self.handle_invocation_error(
                        record,
                        OrcaMutationStage::TaskCreate,
                        error,
                    );
                }
            };
            let parsed = match parse_task_create(&output, &run_id, Some(&request_id)) {
                Ok(parsed) => parsed,
                Err(error) => {
                    return self.handle_invocation_error(
                        record,
                        OrcaMutationStage::TaskCreate,
                        error,
                    );
                }
            };
            record = match self.append_receipt(
                &record,
                OrcaMutationStage::TaskCreate,
                "created",
                parsed.raw,
                OrcaOperationUpdate {
                    state: Some(OrcaOperationState::InProgress),
                    orca_task_id: Some(parsed.task_id),
                    ..update_now()
                },
            ) {
                Ok(record) => record,
                Err(error) => return RuntimeCommandExecution::failed(error),
            };
        }

        let live = match self.canonical_placement().await {
            Ok(placement) => placement,
            Err(error) => return self.mark_recovery(record, OrcaMutationStage::WorkerStart, error),
        };
        if let Err(error) = compare_placement(&request.placement, &live) {
            return self.mark_recovery(record, OrcaMutationStage::WorkerStart, error);
        }
        if let Err(error) = self.ensure_run_binding(&record, &run_id).await {
            return self.mark_recovery(record, OrcaMutationStage::RunBind, error);
        }

        let task_id = record.orca_task_id.clone().expect("task persisted");
        if record.orca_dispatch_id.is_none() {
            let result = self
                .start_worker(
                    &record,
                    &run_id,
                    &task_id,
                    &request.placement,
                    None,
                    OrcaMutationStage::WorkerStart,
                )
                .await;
            match result {
                Ok(updated) => record = updated,
                Err(WorkerMutationFailure::Unavailable) => {
                    return self.mark_unavailable(record, OrcaMutationStage::WorkerStart);
                }
                Err(WorkerMutationFailure::Execution(execution)) => return execution,
            }
        }
        self.observe_attempt(record, None, OrcaMutationStage::WorkerStart)
            .await
    }

    pub(super) async fn retry(&self, request: RetryLinkedRunRequest) -> RuntimeCommandExecution {
        let _guard = self.binding.lock().await;
        let record = match self.begin(
            scope("retry", &request.context.initiative_id.0),
            OrcaTypedOperationRequest::RetryLinkedRun(request.clone()),
        ) {
            Ok(BeginOrcaOperation::Existing(record)) if operation_is_terminal(record.state) => {
                return execution_from_record(&record);
            }
            Ok(BeginOrcaOperation::Existing(record) | BeginOrcaOperation::Created(record)) => {
                record
            }
            Err(error) => return RuntimeCommandExecution::failed(error),
        };
        let live = match self.canonical_placement().await {
            Ok(placement) => placement,
            Err(error) => {
                return self.mark_recovery(record, OrcaMutationStage::WorkerRetry, error);
            }
        };
        if let Err(error) = compare_placement(&request.placement, &live) {
            return self.mark_recovery(record, OrcaMutationStage::WorkerRetry, error);
        }
        let mut record = match self.persist_placement(&record, &request.placement) {
            Ok(record) => record,
            Err(error) => return RuntimeCommandExecution::failed(error),
        };
        record = match self.store.update(
            &record.command_id,
            OrcaOperationUpdate {
                orca_run_id: Some(request.orca_run_id.clone()),
                orca_task_id: Some(request.orca_task_id.clone()),
                ..update_now()
            },
        ) {
            Ok(record) => record,
            Err(error) => return RuntimeCommandExecution::failed(store_error(error)),
        };
        if let Some(dispatch_id) = record.orca_dispatch_id.clone() {
            if dispatch_id == request.retry_of_dispatch_id {
                return self.mark_recovery(
                    record,
                    OrcaMutationStage::WorkerRetry,
                    schema_mismatch("retry operation stored the prior Dispatch as its replacement"),
                );
            }
            return self
                .observe_attempt(
                    record,
                    Some(request.retry_of_dispatch_id),
                    OrcaMutationStage::WorkerRetry,
                )
                .await;
        }

        let prior = match self.worker_show(&request.retry_of_dispatch_id).await {
            Ok(observation) => observation,
            Err(error) => return self.reject(record, "retry_preflight", error.to_string()),
        };
        if prior.run_id != request.orca_run_id
            || prior.task_id != request.orca_task_id
            || prior.dispatch_id != request.retry_of_dispatch_id
        {
            return self.reject(
                record,
                "retry_preflight",
                "prior Dispatch identity mismatch".into(),
            );
        }
        if !matches!(prior.state.as_str(), "failed" | "stopped" | "abandoned") {
            return self.reject(
                record,
                "retry_preflight",
                format!("prior Dispatch state {} is not retryable", prior.state),
            );
        }
        let task_status = match self
            .task_status(&request.orca_run_id, &request.orca_task_id)
            .await
        {
            Ok(status) => status,
            Err(error) => return self.reject(record, "retry_preflight", error.to_string()),
        };
        if !matches!(task_status.as_str(), "failed" | "blocked") {
            return self.reject(
                record,
                "retry_preflight",
                format!("Task state {task_status} is not retryable"),
            );
        }
        match self
            .latest_dispatch(&request.orca_run_id, &request.orca_task_id)
            .await
        {
            Ok(Some(latest)) if latest == request.retry_of_dispatch_id => {}
            Ok(_) => {
                return self.reject(
                    record,
                    "retry_preflight",
                    "selected Dispatch is not the Task's latest Dispatch".into(),
                );
            }
            Err(error) => return self.reject(record, "retry_preflight", error.to_string()),
        }
        if let Err(error) = self.ensure_run_binding(&record, &request.orca_run_id).await {
            return self.mark_recovery(record, OrcaMutationStage::RunBind, error);
        }
        match self
            .start_worker(
                &record,
                &request.orca_run_id,
                &request.orca_task_id,
                &request.placement,
                Some(&request.retry_of_dispatch_id),
                OrcaMutationStage::WorkerRetry,
            )
            .await
        {
            Ok(updated) => record = updated,
            Err(WorkerMutationFailure::Unavailable) => {
                return self.mark_unavailable(record, OrcaMutationStage::WorkerRetry);
            }
            Err(WorkerMutationFailure::Execution(execution)) => return execution,
        }
        if record.orca_dispatch_id.as_ref() == Some(&request.retry_of_dispatch_id) {
            return self.mark_recovery(
                record,
                OrcaMutationStage::WorkerRetry,
                schema_mismatch("retry returned the prior Dispatch ID"),
            );
        }
        self.observe_attempt(
            record,
            Some(request.retry_of_dispatch_id),
            OrcaMutationStage::WorkerRetry,
        )
        .await
    }

    pub(super) async fn cancel(&self, request: CancelLinkedRunRequest) -> RuntimeCommandExecution {
        let record = match self.begin(
            scope("cancel", &request.context.initiative_id.0),
            OrcaTypedOperationRequest::CancelLinkedRun(request.clone()),
        ) {
            Ok(BeginOrcaOperation::Existing(record)) if operation_is_terminal(record.state) => {
                return execution_from_record(&record);
            }
            Ok(BeginOrcaOperation::Existing(record) | BeginOrcaOperation::Created(record)) => {
                record
            }
            Err(error) => return RuntimeCommandExecution::failed(error),
        };
        let mut record = match self.store.update(
            &record.command_id,
            OrcaOperationUpdate {
                orca_run_id: Some(request.orca_run_id.clone()),
                orca_task_id: Some(request.orca_task_id.clone()),
                orca_dispatch_id: Some(request.target_dispatch_id.clone()),
                ..update_now()
            },
        ) {
            Ok(record) => record,
            Err(error) => return RuntimeCommandExecution::failed(store_error(error)),
        };
        let before = match self.worker_show(&request.target_dispatch_id).await {
            Ok(observation) => observation,
            Err(error) => return self.mark_recovery(record, OrcaMutationStage::WorkerStop, error),
        };
        if before.run_id != request.orca_run_id
            || before.task_id != request.orca_task_id
            || before.dispatch_id != request.target_dispatch_id
        {
            return self.mark_recovery(
                record,
                OrcaMutationStage::WorkerStop,
                schema_mismatch("cancel preflight identity mismatch"),
            );
        }
        let stop_requested = record
            .requests
            .iter()
            .any(|request| request.stage == OrcaMutationStage::WorkerStop.as_str());
        if stop_requested && before.state == "stopped" {
            return self.release_stopped(record, request, before).await;
        }
        if stop_requested && before.state == "stop_unknown" {
            return self.abandon(record, request, before).await;
        }
        match before.state.as_str() {
            "succeeded" => {
                return self.reject(
                    record,
                    "cancel_preflight",
                    "Dispatch already succeeded; cancellation is stale".into(),
                );
            }
            "failed" | "stopped" | "abandoned" => {
                let outcome = if before.state == "abandoned" {
                    OrcaMutationOutcome::Abandoned
                } else {
                    OrcaMutationOutcome::AlreadySettled
                };
                record = match self.store.update(
                    &record.command_id,
                    OrcaOperationUpdate {
                        state: Some(OrcaOperationState::Completed),
                        orca_terminal_id: before.terminal_id.clone(),
                        ..update_now()
                    },
                ) {
                    Ok(record) => record,
                    Err(error) => return RuntimeCommandExecution::failed(store_error(error)),
                };
                return lifecycle_execution(
                    &record,
                    outcome,
                    format!("already_{}", before.state),
                    None,
                    before.effects,
                    before.residual_resources,
                    Vec::new(),
                    None,
                );
            }
            "starting" | "stopping" | "stop_unknown" => {
                return self.mark_unknown(
                    record,
                    OrcaMutationStage::WorkerStop,
                    CommandCenterError::OrcaOperationOutcomeUnknown {
                        stage: "cancel_preflight".into(),
                    },
                );
            }
            "ready" | "start_unknown" => {}
            other => {
                return self.mark_recovery(
                    record,
                    OrcaMutationStage::WorkerStop,
                    schema_mismatch(format!("unsupported worker state {other}")),
                );
            }
        }

        let stop_request =
            orca_request_id(&request.context.command_id, OrcaMutationStage::WorkerStop);
        let stop_args = strings(&[
            "orchestration",
            "worker-stop",
            "--dispatch",
            &request.target_dispatch_id.0,
            "--retry-request",
            &stop_request.0,
            "--json",
        ]);
        let output = match self
            .persist_and_invoke(
                &record,
                OrcaMutationStage::WorkerStop,
                stop_request,
                stop_args,
                json!({"dispatchId": request.target_dispatch_id}),
            )
            .await
        {
            Ok(output) => output,
            Err(CommandCenterError::OrcaUnavailable) => {
                return RuntimeCommandExecution::failed(CommandCenterError::OrcaUnavailable);
            }
            Err(error) => return self.mark_unknown(record, OrcaMutationStage::WorkerStop, error),
        };
        let stop = match parse_worker_stop(&output, &request.target_dispatch_id) {
            Ok(stop) => stop,
            Err(error) => return self.mark_unknown(record, OrcaMutationStage::WorkerStop, error),
        };
        record = match self.append_receipt(
            &record,
            OrcaMutationStage::WorkerStop,
            &stop.state,
            stop.raw,
            OrcaOperationUpdate {
                state: Some(if stop.state == "stop_unknown" {
                    OrcaOperationState::OutcomeUnknown
                } else {
                    OrcaOperationState::InProgress
                }),
                ..update_now()
            },
        ) {
            Ok(record) => record,
            Err(error) => return RuntimeCommandExecution::failed(error),
        };

        let after = match self.worker_show(&request.target_dispatch_id).await {
            Ok(observation) => observation,
            Err(error) => return self.mark_unknown(record, OrcaMutationStage::WorkerStop, error),
        };
        if stop.state == "stopped" && after.state == "stopped" {
            return self.release_stopped(record, request, after).await;
        }
        if stop.state != "stop_unknown" && after.state != "stop_unknown" {
            return self.mark_recovery(
                record,
                OrcaMutationStage::WorkerStop,
                schema_mismatch("worker-stop settlement was not confirmed by worker-show"),
            );
        }
        self.abandon(record, request, after).await
    }

    async fn release_stopped(
        &self,
        mut record: OrcaOperationRecord,
        request: CancelLinkedRunRequest,
        observation: WorkerObservation,
    ) -> RuntimeCommandExecution {
        let release_request = orca_request_id(
            &request.context.command_id,
            OrcaMutationStage::WorkerRelease,
        );
        let args = strings(&[
            "orchestration",
            "worker-release",
            "--dispatch",
            &request.target_dispatch_id.0,
            "--retry-request",
            &release_request.0,
            "--json",
        ]);
        let output = match self
            .persist_and_invoke(
                &record,
                OrcaMutationStage::WorkerRelease,
                release_request,
                args,
                json!({"dispatchId": request.target_dispatch_id}),
            )
            .await
        {
            Ok(output) => output,
            Err(error) => {
                let recovery =
                    observation
                        .terminal_id
                        .as_ref()
                        .map(|terminal| OrcaRecoveryObligation {
                            id: format!("{}:terminal-release", record.command_id.0),
                            resource_kind: "terminal".into(),
                            resource_id: Some(terminal.0.clone()),
                            action: "retry_worker_release_with_same_request".into(),
                            state: OrcaRecoveryState::OutcomeUnknown,
                            evidence: Some(json!({"error": error.to_string()})),
                            updated_at: Utc::now(),
                        });
                record = match self.append_receipt(
                    &record,
                    OrcaMutationStage::WorkerRelease,
                    "release_unknown",
                    json!({"error": error.to_string()}),
                    OrcaOperationUpdate {
                        state: Some(OrcaOperationState::Completed),
                        orca_terminal_id: observation.terminal_id.clone(),
                        recovery: recovery.into_iter().collect(),
                        ..update_now()
                    },
                ) {
                    Ok(record) => record,
                    Err(store_error) => return RuntimeCommandExecution::failed(store_error),
                };
                let cleanup = terminal_cleanup(
                    observation.terminal_id.as_ref(),
                    CleanupResourceState::RecoveryRequired,
                    error.to_string(),
                );
                return lifecycle_execution(
                    &record,
                    OrcaMutationOutcome::Stopped,
                    "worker_release_unknown".into(),
                    Some(error.to_string()),
                    observation.effects,
                    observation.residual_resources,
                    cleanup,
                    None,
                );
            }
        };
        let release = match parse_worker_release(&output, &request.target_dispatch_id) {
            Ok(release) => release,
            Err(error) => {
                return self.mark_recovery(record, OrcaMutationStage::WorkerRelease, error);
            }
        };
        let terminal_id = observation.terminal_id.clone();
        let (cleanup_state, operation_state, recovery) = match release.state.as_str() {
            "released" | "already_released" => (
                CleanupResourceState::VerifiedReleased,
                OrcaOperationState::Completed,
                Vec::new(),
            ),
            "retained" | "release_pending" | "release_unknown" => {
                let obligation = terminal_id.as_ref().map(|terminal| OrcaRecoveryObligation {
                    id: format!("{}:terminal-release", record.command_id.0),
                    resource_kind: "terminal".into(),
                    resource_id: Some(terminal.0.clone()),
                    action: "retry_worker_release_with_same_request".into(),
                    state: OrcaRecoveryState::Pending,
                    evidence: Some(release.raw.clone()),
                    updated_at: Utc::now(),
                });
                (
                    CleanupResourceState::RecoveryRequired,
                    OrcaOperationState::Completed,
                    obligation.into_iter().collect(),
                )
            }
            _ => unreachable!("validated release state"),
        };
        record = match self.append_receipt(
            &record,
            OrcaMutationStage::WorkerRelease,
            &release.state,
            release.raw,
            OrcaOperationUpdate {
                state: Some(operation_state),
                orca_terminal_id: terminal_id.clone(),
                recovery,
                ..update_now()
            },
        ) {
            Ok(record) => record,
            Err(error) => return RuntimeCommandExecution::failed(error),
        };
        lifecycle_execution(
            &record,
            OrcaMutationOutcome::Stopped,
            "worker_release".into(),
            release.last_error,
            observation.effects,
            observation.residual_resources,
            terminal_cleanup(
                terminal_id.as_ref(),
                cleanup_state,
                format!("worker-release returned {}", release.state),
            ),
            None,
        )
    }

    async fn abandon(
        &self,
        mut record: OrcaOperationRecord,
        request: CancelLinkedRunRequest,
        observation: WorkerObservation,
    ) -> RuntimeCommandExecution {
        let abandon_request = orca_request_id(
            &request.context.command_id,
            OrcaMutationStage::WorkerAbandon,
        );
        let args = strings(&[
            "orchestration",
            "worker-abandon",
            "--dispatch",
            &request.target_dispatch_id.0,
            "--retry-request",
            &abandon_request.0,
            "--json",
        ]);
        let output = match self
            .persist_and_invoke(
                &record,
                OrcaMutationStage::WorkerAbandon,
                abandon_request,
                args,
                json!({"dispatchId": request.target_dispatch_id}),
            )
            .await
        {
            Ok(output) => output,
            Err(error) => {
                return self.mark_unknown(record, OrcaMutationStage::WorkerAbandon, error);
            }
        };
        let abandon = match parse_worker_abandon(&output, &request.target_dispatch_id) {
            Ok(abandon) => abandon,
            Err(error) => {
                return self.mark_recovery(record, OrcaMutationStage::WorkerAbandon, error);
            }
        };
        let after = match self.worker_show(&request.target_dispatch_id).await {
            Ok(observation) if observation.state == "abandoned" => observation,
            Ok(_) => {
                return self.mark_recovery(
                    record,
                    OrcaMutationStage::WorkerAbandon,
                    schema_mismatch("worker-show did not confirm abandoned"),
                );
            }
            Err(error) => {
                return self.mark_recovery(record, OrcaMutationStage::WorkerAbandon, error);
            }
        };
        let mut recovery = Vec::new();
        if let Some(terminal) = after
            .terminal_id
            .as_ref()
            .or(observation.terminal_id.as_ref())
        {
            recovery.push(OrcaRecoveryObligation {
                id: format!("{}:abandoned-terminal", record.command_id.0),
                resource_kind: "terminal".into(),
                resource_id: Some(terminal.0.clone()),
                action: "inspect_possibly_live_terminal".into(),
                state: OrcaRecoveryState::Pending,
                evidence: Some(abandon.raw.clone()),
                updated_at: Utc::now(),
            });
        }
        if let Some(worktree) = record.orca_worktree_id.as_ref() {
            recovery.push(OrcaRecoveryObligation {
                id: format!("{}:retained-worktree", record.command_id.0),
                resource_kind: "worktree".into(),
                resource_id: Some(worktree.0.clone()),
                action: "intentionally_retained".into(),
                state: OrcaRecoveryState::Pending,
                evidence: Some(abandon.raw.clone()),
                updated_at: Utc::now(),
            });
        }
        record = match self.append_receipt(
            &record,
            OrcaMutationStage::WorkerAbandon,
            "abandoned",
            abandon.raw,
            OrcaOperationUpdate {
                state: Some(OrcaOperationState::Completed),
                orca_terminal_id: after.terminal_id.clone(),
                recovery,
                ..update_now()
            },
        ) {
            Ok(record) => record,
            Err(error) => return RuntimeCommandExecution::failed(error),
        };
        let mut cleanup = terminal_cleanup(
            after.terminal_id.as_ref(),
            CleanupResourceState::RecoveryRequired,
            "worker was abandoned; process termination was not proven".into(),
        );
        if let Some(worktree) = record.orca_worktree_id.as_ref() {
            cleanup.push(CleanupResourceProjection {
                resource_kind: "worktree".into(),
                resource_id: worktree.0.clone(),
                state: CleanupResourceState::RecoveryRequired,
                evidence: "worktree intentionally retained after abandon".into(),
            });
        }
        lifecycle_execution(
            &record,
            OrcaMutationOutcome::Abandoned,
            "worker_abandon".into(),
            None,
            after.effects,
            abandon.residual_resources,
            cleanup,
            None,
        )
    }

    pub(super) async fn reconcile_pending_operations(
        &self,
    ) -> Result<ReconciliationSummary, CommandCenterError> {
        self.startup_reconciled.store(false, Ordering::Release);
        let result = self.reconcile_pending_operations_inner().await;
        if result.is_ok() {
            self.startup_reconciled.store(true, Ordering::Release);
        }
        result
    }

    async fn reconcile_pending_operations_inner(
        &self,
    ) -> Result<ReconciliationSummary, CommandCenterError> {
        let records = self.store.list_recoverable().map_err(store_error)?;
        let mut summary = ReconciliationSummary::default();
        for record in records {
            summary.examined += 1;
            let request = match serde_json::from_value::<OrcaTypedOperationRequest>(
                record.command_payload.clone(),
            ) {
                Ok(request) => request,
                Err(error) => {
                    self.store
                        .update(
                            &record.command_id,
                            OrcaOperationUpdate {
                                state: Some(OrcaOperationState::RecoveryRequired),
                                ..update_now()
                            },
                        )
                        .map_err(store_error)?;
                    crate::logging::warn(&format!(
                        "Cannot decode durable Orca operation {} during reconciliation: {error}",
                        record.command_id.0
                    ));
                    summary.recovery_required += 1;
                    continue;
                }
            };
            match request {
                OrcaTypedOperationRequest::StartInitiativeRun(request) => {
                    if record.orca_dispatch_id.is_some() {
                        let _ = self
                            .observe_attempt(record.clone(), None, OrcaMutationStage::WorkerStart)
                            .await;
                    } else {
                        let _ = self.start(request).await;
                    }
                }
                OrcaTypedOperationRequest::RetryLinkedRun(request) => {
                    if record.orca_dispatch_id.is_some() {
                        let _ = self
                            .observe_attempt(
                                record.clone(),
                                Some(request.retry_of_dispatch_id.clone()),
                                OrcaMutationStage::WorkerRetry,
                            )
                            .await;
                    } else {
                        let _ = self.retry(request).await;
                    }
                }
                OrcaTypedOperationRequest::CancelLinkedRun(request) => {
                    let _ = self.cancel(request).await;
                }
            }
            let Some(reconciled) = self
                .store
                .get_by_command(&record.command_id)
                .map_err(store_error)?
            else {
                summary.recovery_required += 1;
                continue;
            };
            summarize_reconciled_state(&mut summary, reconciled.state);
        }
        Ok(summary)
    }

    fn begin(
        &self,
        idempotency_scope: String,
        request: OrcaTypedOperationRequest,
    ) -> Result<BeginOrcaOperation, CommandCenterError> {
        let operation =
            NewOrcaOperation::from_typed_request(idempotency_scope, request, Utc::now())
                .map_err(store_error)?;
        self.store.begin(operation).map_err(store_error)
    }

    fn persist_placement(
        &self,
        record: &OrcaOperationRecord,
        placement: &OrcaCanonicalPlacement,
    ) -> Result<OrcaOperationRecord, CommandCenterError> {
        self.store
            .update(
                &record.command_id,
                OrcaOperationUpdate {
                    orca_project_id: Some(placement.project_id.clone()),
                    orca_repository_id: Some(placement.repository_id.clone()),
                    orca_host_setup_id: Some(placement.host_setup_id.clone()),
                    orca_host_id: Some(placement.host_id.clone()),
                    orca_worktree_id: Some(placement.worktree_id.clone()),
                    ..update_now()
                },
            )
            .map_err(store_error)
    }

    async fn ensure_run_binding(
        &self,
        record: &OrcaOperationRecord,
        run_id: &OrcaRunId,
    ) -> Result<(), CommandCenterError> {
        let current = self
            .read_result(&[
                "orchestration",
                "run-current",
                "--from",
                &self.coordinator.terminal.0,
                "--json",
            ])
            .await?;
        let current_id = nested_string(&current, &["run", "id"])
            .or_else(|| string_from_value(&current, &["runId", "id"]));
        if current_id.as_deref() == Some(run_id.0.as_str()) {
            return Ok(());
        }
        let request_id = orca_request_id(&record.command_id, OrcaMutationStage::RunBind);
        let args = strings(&[
            "orchestration",
            "run-use",
            "--id",
            &run_id.0,
            "--from",
            &self.coordinator.terminal.0,
            "--retry-request",
            &request_id.0,
            "--json",
        ]);
        let output = self
            .persist_and_invoke(
                record,
                OrcaMutationStage::RunBind,
                request_id.clone(),
                args,
                json!({"runId": run_id, "terminal": self.coordinator.terminal}),
            )
            .await?;
        let raw = parse_bound_run(&output, run_id, Some(&request_id))?;
        self.append_receipt(
            record,
            OrcaMutationStage::RunBind,
            "bound",
            raw,
            OrcaOperationUpdate {
                state: Some(OrcaOperationState::InProgress),
                ..update_now()
            },
        )?;
        Ok(())
    }

    async fn start_worker(
        &self,
        record: &OrcaOperationRecord,
        run_id: &OrcaRunId,
        task_id: &OrcaTaskId,
        placement: &OrcaCanonicalPlacement,
        retry_of: Option<&OrcaDispatchId>,
        stage: OrcaMutationStage,
    ) -> Result<OrcaOperationRecord, WorkerMutationFailure> {
        validate_supported_placement(placement).map_err(|error| {
            WorkerMutationFailure::Execution(RuntimeCommandExecution::failed(error))
        })?;
        let request_id = orca_request_id(&record.command_id, stage);
        let mut args = strings(&[
            "orchestration",
            "worker-start",
            "--task",
            &task_id.0,
            "--run",
            &run_id.0,
            "--from",
            &self.coordinator.terminal.0,
            "--worktree",
            &placement.worktree_selector,
        ]);
        append_launcher_args(&mut args, &placement.launcher).map_err(|error| {
            WorkerMutationFailure::Execution(RuntimeCommandExecution::failed(error))
        })?;
        if let Some(retry_of) = retry_of {
            args.extend(["--retry-of".into(), retry_of.0.clone()]);
        }
        args.extend([
            "--retry-request".into(),
            request_id.0.clone(),
            "--json".into(),
        ]);
        let output = match self
            .persist_and_invoke(
                record,
                stage,
                request_id.clone(),
                args,
                json!({
                    "runId": run_id,
                    "taskId": task_id,
                    "worktree": placement.worktree_selector,
                    "launcher": placement.launcher,
                    "retryOf": retry_of,
                }),
            )
            .await
        {
            Ok(output) => output,
            Err(CommandCenterError::OrcaUnavailable) => {
                return Err(WorkerMutationFailure::Unavailable);
            }
            Err(error) => {
                return Err(WorkerMutationFailure::Execution(self.mark_unknown(
                    record.clone(),
                    stage,
                    error,
                )));
            }
        };
        let parsed = match parse_worker_start(&output, run_id, task_id, Some(&request_id)) {
            Ok(parsed) => parsed,
            Err(error) => {
                return Err(WorkerMutationFailure::Execution(
                    self.handle_invocation_error(record.clone(), stage, error),
                ));
            }
        };
        let operation_state = match parsed.state.as_str() {
            "ready" => OrcaOperationState::InProgress,
            "failed" => OrcaOperationState::Failed,
            "outcome_unknown" => OrcaOperationState::OutcomeUnknown,
            _ => unreachable!("validated worker state"),
        };
        let partial_effects = parsed
            .effects
            .iter()
            .chain(parsed.residual_resources.iter())
            .enumerate()
            .map(|(index, effect)| OrcaPartialEffect {
                id: format!("{}:{}:effect:{index}", record.command_id.0, stage.as_str()),
                stage: stage.as_str().into(),
                resource_kind: effect.kind.clone(),
                resource_id: effect.id.clone(),
                evidence: serde_json::to_value(effect).unwrap_or(Value::Null),
                observed_at: Utc::now(),
            })
            .collect();
        let recovery = if parsed.state == "outcome_unknown" {
            parsed
                .residual_resources
                .iter()
                .enumerate()
                .map(|(index, effect)| OrcaRecoveryObligation {
                    id: format!(
                        "{}:{}:recovery:{index}",
                        record.command_id.0,
                        stage.as_str()
                    ),
                    resource_kind: effect.kind.clone(),
                    resource_id: effect.id.clone(),
                    action: "inspect_before_retry".into(),
                    state: OrcaRecoveryState::OutcomeUnknown,
                    evidence: Some(serde_json::to_value(effect).unwrap_or(Value::Null)),
                    updated_at: Utc::now(),
                })
                .collect()
        } else {
            Vec::new()
        };
        self.append_receipt(
            record,
            stage,
            &parsed.state,
            parsed.raw,
            OrcaOperationUpdate {
                state: Some(operation_state),
                orca_dispatch_id: Some(parsed.dispatch_id),
                partial_effects,
                recovery,
                ..update_now()
            },
        )
        .map_err(|error| WorkerMutationFailure::Execution(RuntimeCommandExecution::failed(error)))
    }

    async fn observe_attempt(
        &self,
        mut record: OrcaOperationRecord,
        retry_of: Option<OrcaDispatchId>,
        stage: OrcaMutationStage,
    ) -> RuntimeCommandExecution {
        let dispatch_id = record.orca_dispatch_id.clone().expect("dispatch persisted");
        let observation = match self.worker_show(&dispatch_id).await {
            Ok(observation) => observation,
            Err(error) => return self.mark_unknown(record, stage, error),
        };
        if record.orca_run_id.as_ref() != Some(&observation.run_id)
            || record.orca_task_id.as_ref() != Some(&observation.task_id)
            || record.orca_dispatch_id.as_ref() != Some(&observation.dispatch_id)
            || record.orca_worktree_id.as_ref() != Some(&observation.worktree_id)
        {
            return self.mark_recovery(
                record,
                stage,
                schema_mismatch("worker-show identity or placement mismatch"),
            );
        }
        if observation.state == "ready" {
            let expected = typed_request(&record).and_then(|request| match request {
                OrcaTypedOperationRequest::StartInitiativeRun(request) => Some(request.placement),
                OrcaTypedOperationRequest::RetryLinkedRun(request) => Some(request.placement),
                OrcaTypedOperationRequest::CancelLinkedRun(_) => None,
            });
            if let Some(expected) = expected {
                match expected.launcher {
                    OrcaWorkerLauncher::Agent {
                        agent,
                        model,
                        effort,
                    } => {
                        if observation.terminal_id.is_none()
                            || observation.agent.as_deref() != Some(agent.as_str())
                            || observation.model != model
                            || observation.effort != effort
                        {
                            return self.mark_recovery(
                                record,
                                stage,
                                schema_mismatch(
                                    "worker-show did not confirm the exact agent launch placement",
                                ),
                            );
                        }
                    }
                    OrcaWorkerLauncher::ExistingTerminal { terminal_id } => {
                        if observation.terminal_id.as_ref() != Some(&terminal_id) {
                            return self.mark_recovery(
                                record,
                                stage,
                                schema_mismatch(
                                    "worker-show did not confirm the exact terminal placement",
                                ),
                            );
                        }
                    }
                }
            }
        }
        let (outcome, state) = match observation.state.as_str() {
            "ready" => (OrcaMutationOutcome::Ready, OrcaOperationState::Ready),
            "failed" => (OrcaMutationOutcome::Failed, OrcaOperationState::Failed),
            "starting" | "start_unknown" => (
                OrcaMutationOutcome::OutcomeUnknown,
                OrcaOperationState::OutcomeUnknown,
            ),
            other => {
                return self.mark_recovery(
                    record,
                    stage,
                    schema_mismatch(format!("unexpected worker state after start: {other}")),
                );
            }
        };
        record = match self.store.update(
            &record.command_id,
            OrcaOperationUpdate {
                state: Some(state),
                orca_terminal_id: observation.terminal_id.clone(),
                ..update_now()
            },
        ) {
            Ok(record) => record,
            Err(error) => return RuntimeCommandExecution::failed(store_error(error)),
        };
        lifecycle_execution(
            &record,
            outcome,
            observation.stage,
            observation.last_error,
            observation.effects,
            observation.residual_resources,
            Vec::new(),
            retry_of,
        )
    }

    async fn reconcile_run_marker(
        &self,
        objective: &str,
    ) -> Result<Option<(OrcaRunId, Value)>, CommandCenterError> {
        let result = self
            .read_result(&["orchestration", "run-list", "--json"])
            .await?;
        let runs = required_array(&result, "runs")?;
        let matching = runs
            .iter()
            .filter_map(Value::as_object)
            .filter(|run| string_alias(run, &["objective"]) == Some(objective))
            .collect::<Vec<_>>();
        match matching.as_slice() {
            [] => Ok(None),
            [run] => {
                let run_id = OrcaRunId(required_string(run, "id")?.to_string());
                Ok(Some((run_id, json!({"reconciledRun": run}))))
            }
            _ => Err(CommandCenterError::OrcaOperationRecoveryRequired {
                stage: "run_create_marker_ambiguous".into(),
            }),
        }
    }

    async fn reconcile_task_marker(
        &self,
        run_id: &OrcaRunId,
        display_name: &str,
    ) -> Result<Option<(OrcaTaskId, Value)>, CommandCenterError> {
        let result = self
            .read_result(&["orchestration", "task-list", "--run", &run_id.0, "--json"])
            .await?;
        let tasks = required_array(&result, "tasks")?;
        let matching = tasks
            .iter()
            .filter_map(Value::as_object)
            .filter(|task| {
                string_alias(task, &["run_id", "runId"]) == Some(run_id.0.as_str())
                    && string_alias(task, &["display_name", "displayName"]) == Some(display_name)
            })
            .collect::<Vec<_>>();
        match matching.as_slice() {
            [] => Ok(None),
            [task] => {
                let task_id = OrcaTaskId(required_string(task, "id")?.to_string());
                Ok(Some((task_id, json!({"reconciledTask": task}))))
            }
            _ => Err(CommandCenterError::OrcaOperationRecoveryRequired {
                stage: "task_create_marker_ambiguous".into(),
            }),
        }
    }

    async fn task_status(
        &self,
        run_id: &OrcaRunId,
        task_id: &OrcaTaskId,
    ) -> Result<String, CommandCenterError> {
        let result = self
            .read_result(&["orchestration", "task-list", "--run", &run_id.0, "--json"])
            .await?;
        let tasks = required_array(&result, "tasks")?;
        let matching = tasks
            .iter()
            .filter_map(Value::as_object)
            .filter(|task| string_alias(task, &["id"]) == Some(task_id.0.as_str()))
            .collect::<Vec<_>>();
        if matching.len() != 1 {
            return Err(CommandCenterError::OrcaPreconditionFailed {
                reason: "Task identity is missing or ambiguous".into(),
            });
        }
        Ok(required_string(matching[0], "status")?.to_string())
    }

    async fn latest_dispatch(
        &self,
        run_id: &OrcaRunId,
        task_id: &OrcaTaskId,
    ) -> Result<Option<OrcaDispatchId>, CommandCenterError> {
        let result = self
            .read_result(&["orchestration", "worker-list", "--run", &run_id.0, "--json"])
            .await?;
        let workers = required_array(&result, "workers")?;
        Ok(workers
            .iter()
            .filter_map(Value::as_object)
            .filter(|worker| {
                string_alias(worker, &["taskId", "task_id"]) == Some(task_id.0.as_str())
            })
            .filter_map(|worker| string_alias(worker, &["dispatchId", "dispatch_id"]))
            .last()
            .map(|id| OrcaDispatchId(id.to_string())))
    }

    async fn worker_show(
        &self,
        dispatch_id: &OrcaDispatchId,
    ) -> Result<WorkerObservation, CommandCenterError> {
        let result = self
            .read_result(&[
                "orchestration",
                "worker-show",
                "--dispatch",
                &dispatch_id.0,
                "--json",
            ])
            .await?;
        validate_keys(
            object(&result)?,
            &[
                "dispatch",
                "worker",
                "terminal",
                "observation",
                "terminalResource",
            ],
            "worker-show result",
        )?;
        let dispatch = required_object(&result, "dispatch")?;
        let worker = required_object(&result, "worker")?;
        let state = required_string(worker, "state")?.to_string();
        validate_worker_state(&state)?;
        let observed_dispatch = required_string_alias(worker, &["dispatch_id", "dispatchId"])?;
        if observed_dispatch != dispatch_id.0 {
            return Err(schema_mismatch("worker-show Dispatch ID mismatch"));
        }
        let effects = effects_from_alias(worker, &["effects"])?;
        let residual_resources =
            effects_from_alias(worker, &["residualResources", "residual_resources"])?;
        let start_options = worker
            .get("startOptions")
            .or_else(|| worker.get("start_options"))
            .and_then(Value::as_object);
        let effective_launch = start_options
            .and_then(|options| options.get("launch"))
            .and_then(Value::as_object)
            .and_then(|launch| launch.get("effective"))
            .and_then(Value::as_object);
        let launch_value = |key: &str| {
            start_options
                .and_then(|options| options.get(key))
                .and_then(Value::as_str)
                .or_else(|| effective_launch.and_then(|launch| string_alias(launch, &[key])))
                .map(str::to_string)
        };
        Ok(WorkerObservation {
            dispatch_id: OrcaDispatchId(required_string(dispatch, "id")?.to_string()),
            run_id: OrcaRunId(required_string_alias(dispatch, &["run_id", "runId"])?.to_string()),
            task_id: OrcaTaskId(
                required_string_alias(dispatch, &["task_id", "taskId"])?.to_string(),
            ),
            state,
            stage: string_alias(worker, &["stage"])
                .unwrap_or("observed")
                .to_string(),
            worktree_id: OrcaWorktreeId(
                required_string_alias(worker, &["worktree_id", "worktreeId"])?.to_string(),
            ),
            terminal_id: string_alias(worker, &["agent_terminal_handle", "agentTerminalHandle"])
                .filter(|value| !value.is_empty())
                .map(|value| OrcaTerminalId(value.to_string())),
            agent: launch_value("agent"),
            model: launch_value("model"),
            effort: launch_value("effort"),
            last_error: string_alias(worker, &["last_error", "lastError"]).map(str::to_string),
            effects,
            residual_resources,
        })
    }

    async fn read_result(&self, args: &[&str]) -> Result<Value, CommandCenterError> {
        let args = strings(args);
        let output = self
            .runner
            .run(
                &self.command,
                &args,
                self.working_dir.as_deref(),
                self.timeout,
            )
            .await
            .map_err(|_| CommandCenterError::OrcaUnavailable)?;
        let envelope = parse_envelope(&output.stdout)?;
        if output.exit_code != Some(0) || !envelope.ok {
            return Err(map_envelope_error(&envelope));
        }
        envelope
            .result
            .ok_or_else(|| schema_mismatch("Orca response omitted result"))
    }

    async fn persist_and_invoke(
        &self,
        record: &OrcaOperationRecord,
        stage: OrcaMutationStage,
        request_id: OrcaRequestId,
        args: Vec<String>,
        payload: Value,
    ) -> Result<OrcaCommandOutput, CommandCenterError> {
        // Always inspect the current durable record. A lifecycle can append a request or receipt
        // and continue with an older in-memory snapshot, especially across run rebinding. The
        // store remains the authority for deciding whether this is a replay or a new invocation.
        let durable_record = self
            .store
            .get_by_command(&record.command_id)
            .map_err(store_error)?
            .ok_or_else(|| CommandCenterError::NotFound {
                entity: format!("Orca operation {}", record.command_id.0),
            })?;
        if let Some(existing) = durable_record
            .requests
            .iter()
            .find(|request| request.stage == stage.as_str())
        {
            if existing.orca_request_id.as_ref() != Some(&request_id)
                || existing.arguments != args
                || existing.payload != payload
            {
                return Err(CommandCenterError::OrcaReceiptIdentityConflict {
                    field: format!("{} request", stage.as_str()),
                });
            }
        } else {
            self.store
                .append_request(
                    &record.command_id,
                    OrcaRequestRecord {
                        id: format!("{}:{}", record.command_id.0, stage.as_str()),
                        stage: stage.as_str().into(),
                        orca_request_id: Some(request_id),
                        command: self.command.clone(),
                        arguments: args.clone(),
                        payload,
                        recorded_at: Utc::now(),
                    },
                    OrcaOperationUpdate {
                        state: Some(OrcaOperationState::InProgress),
                        ..update_now()
                    },
                )
                .map_err(store_error)?;
        }
        self.runner
            .run(
                &self.command,
                &args,
                self.working_dir.as_deref(),
                self.timeout,
            )
            .await
            .map_err(|error| match error {
                OrcaProcessError::Spawn(_) => CommandCenterError::OrcaUnavailable,
                OrcaProcessError::Timeout | OrcaProcessError::Transport(_) => {
                    CommandCenterError::OrcaOperationOutcomeUnknown {
                        stage: stage.as_str().into(),
                    }
                }
            })
    }

    fn append_receipt(
        &self,
        record: &OrcaOperationRecord,
        stage: OrcaMutationStage,
        status: &str,
        payload: Value,
        update: OrcaOperationUpdate,
    ) -> Result<OrcaOperationRecord, CommandCenterError> {
        self.store
            .append_receipt(
                &record.command_id,
                OrcaReceiptRecord {
                    id: format!("{}:{}:receipt", record.command_id.0, stage.as_str()),
                    request_id: format!("{}:{}", record.command_id.0, stage.as_str()),
                    status: status.into(),
                    payload,
                    observed_at: Utc::now(),
                },
                update,
            )
            .map_err(store_error)
    }

    fn mark_unknown(
        &self,
        record: OrcaOperationRecord,
        stage: OrcaMutationStage,
        error: CommandCenterError,
    ) -> RuntimeCommandExecution {
        let updated = self
            .store
            .update(
                &record.command_id,
                OrcaOperationUpdate {
                    state: Some(OrcaOperationState::OutcomeUnknown),
                    ..update_now()
                },
            )
            .unwrap_or(record);
        lifecycle_execution(
            &updated,
            OrcaMutationOutcome::OutcomeUnknown,
            stage.as_str().into(),
            Some(error.to_string()),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            None,
        )
    }

    fn mark_unavailable(
        &self,
        record: OrcaOperationRecord,
        stage: OrcaMutationStage,
    ) -> RuntimeCommandExecution {
        let updated = self
            .store
            .update(
                &record.command_id,
                OrcaOperationUpdate {
                    state: Some(OrcaOperationState::Failed),
                    ..update_now()
                },
            )
            .unwrap_or(record);
        let mut execution = lifecycle_execution(
            &updated,
            OrcaMutationOutcome::Failed,
            stage.as_str().into(),
            Some(CommandCenterError::OrcaUnavailable.to_string()),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            None,
        );
        execution.error = Some(CommandCenterError::OrcaUnavailable);
        execution
    }

    fn handle_invocation_error(
        &self,
        record: OrcaOperationRecord,
        stage: OrcaMutationStage,
        error: CommandCenterError,
    ) -> RuntimeCommandExecution {
        match error {
            CommandCenterError::OrcaUnavailable => self.mark_unavailable(record, stage),
            CommandCenterError::OrcaOperationOutcomeUnknown { .. } => {
                self.mark_unknown(record, stage, error)
            }
            CommandCenterError::OrcaSchemaMismatch { .. }
            | CommandCenterError::OrcaProfileMismatch { .. }
            | CommandCenterError::OrcaReceiptIdentityConflict { .. }
            | CommandCenterError::OrcaOperationRecoveryRequired { .. } => {
                self.mark_recovery(record, stage, error)
            }
            _ => self.mark_recovery(record, stage, error),
        }
    }

    fn mark_recovery(
        &self,
        record: OrcaOperationRecord,
        stage: OrcaMutationStage,
        error: CommandCenterError,
    ) -> RuntimeCommandExecution {
        let updated = self
            .store
            .update(
                &record.command_id,
                OrcaOperationUpdate {
                    state: Some(OrcaOperationState::RecoveryRequired),
                    ..update_now()
                },
            )
            .unwrap_or(record);
        lifecycle_execution(
            &updated,
            OrcaMutationOutcome::RecoveryRequired,
            stage.as_str().into(),
            Some(error.to_string()),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            None,
        )
    }

    fn reject(
        &self,
        record: OrcaOperationRecord,
        stage: &str,
        reason: String,
    ) -> RuntimeCommandExecution {
        let updated = self
            .store
            .update(
                &record.command_id,
                OrcaOperationUpdate {
                    state: Some(OrcaOperationState::Rejected),
                    ..update_now()
                },
            )
            .unwrap_or(record);
        lifecycle_execution(
            &updated,
            OrcaMutationOutcome::Rejected,
            stage.into(),
            Some(reason),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            None,
        )
    }
}

#[derive(Debug)]
enum WorkerMutationFailure {
    Unavailable,
    Execution(RuntimeCommandExecution),
}

#[derive(Debug)]
struct ParsedRunCreate {
    run_id: OrcaRunId,
    raw: Value,
}

#[derive(Debug)]
struct ParsedTaskCreate {
    task_id: OrcaTaskId,
    raw: Value,
}

#[derive(Debug)]
struct ParsedWorkerStart {
    dispatch_id: OrcaDispatchId,
    state: String,
    effects: Vec<OrcaEffectReceipt>,
    residual_resources: Vec<OrcaEffectReceipt>,
    raw: Value,
}

#[derive(Debug)]
struct ParsedWorkerStop {
    state: String,
    raw: Value,
}

#[derive(Debug)]
struct ParsedWorkerAbandon {
    residual_resources: Vec<OrcaEffectReceipt>,
    raw: Value,
}

#[derive(Debug)]
struct ParsedWorkerRelease {
    state: String,
    last_error: Option<String>,
    raw: Value,
}

#[derive(Debug)]
struct WorkerObservation {
    dispatch_id: OrcaDispatchId,
    run_id: OrcaRunId,
    task_id: OrcaTaskId,
    state: String,
    stage: String,
    worktree_id: OrcaWorktreeId,
    terminal_id: Option<OrcaTerminalId>,
    agent: Option<String>,
    model: Option<String>,
    effort: Option<String>,
    last_error: Option<String>,
    effects: Vec<OrcaEffectReceipt>,
    residual_resources: Vec<OrcaEffectReceipt>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Envelope {
    id: String,
    ok: bool,
    result: Option<Value>,
    error: Option<EnvelopeError>,
    #[serde(rename = "_meta")]
    meta: EnvelopeMeta,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct EnvelopeMeta {
    #[serde(rename = "runtimeId")]
    runtime_id: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct EnvelopeError {
    code: String,
    message: String,
    #[serde(default)]
    data: Option<Value>,
}

fn parse_run_create(
    output: &OrcaCommandOutput,
    expected_request: Option<&OrcaRequestId>,
) -> Result<ParsedRunCreate, CommandCenterError> {
    let envelope = parse_mutation_envelope(output)?;
    require_success_exit(output, &envelope, "run-create")?;
    let result = envelope
        .result
        .ok_or_else(|| schema_mismatch("run-create omitted result"))?;
    validate_keys(
        object(&result)?,
        &["run", "binding", "mutation"],
        "run-create result",
    )?;
    validate_mutation_request(&result, expected_request)?;
    let run = required_object(&result, "run")?;
    let run_id = required_string(run, "id")?;
    if run_id.is_empty() {
        return Err(schema_mismatch("run-create returned an empty Run ID"));
    }
    Ok(ParsedRunCreate {
        run_id: OrcaRunId(run_id.to_string()),
        raw: result,
    })
}

fn parse_bound_run(
    output: &OrcaCommandOutput,
    expected: &OrcaRunId,
    expected_request: Option<&OrcaRequestId>,
) -> Result<Value, CommandCenterError> {
    let parsed = parse_run_create(output, expected_request)?;
    if parsed.run_id != *expected {
        return Err(schema_mismatch("run-use returned a different Run ID"));
    }
    Ok(parsed.raw)
}

fn parse_task_create(
    output: &OrcaCommandOutput,
    expected_run: &OrcaRunId,
    expected_request: Option<&OrcaRequestId>,
) -> Result<ParsedTaskCreate, CommandCenterError> {
    let envelope = parse_mutation_envelope(output)?;
    require_success_exit(output, &envelope, "task-create")?;
    let result = envelope
        .result
        .ok_or_else(|| schema_mismatch("task-create omitted result"))?;
    validate_keys(
        object(&result)?,
        &["task", "mutation"],
        "task-create result",
    )?;
    validate_mutation_request(&result, expected_request)?;
    let task = required_object(&result, "task")?;
    if let Some(run_id) = string_alias(task, &["run_id", "runId"])
        && run_id != expected_run.0
    {
        return Err(schema_mismatch("task-create returned a different Run ID"));
    }
    Ok(ParsedTaskCreate {
        task_id: OrcaTaskId(required_string(task, "id")?.to_string()),
        raw: result,
    })
}

fn parse_worker_start(
    output: &OrcaCommandOutput,
    expected_run: &OrcaRunId,
    expected_task: &OrcaTaskId,
    expected_request: Option<&OrcaRequestId>,
) -> Result<ParsedWorkerStart, CommandCenterError> {
    let envelope = parse_mutation_envelope(output)?;
    if !envelope.ok {
        let error = envelope
            .error
            .as_ref()
            .ok_or_else(|| schema_mismatch("failed worker-start omitted error"))?;
        if error.code == "operation_unknown" {
            let data = error.data.as_ref().and_then(Value::as_object);
            let dispatch = data
                .and_then(|data| string_alias(data, &["dispatchId", "dispatch_id"]))
                .ok_or_else(|| schema_mismatch("operation_unknown omitted Dispatch ID"))?;
            if output.exit_code == Some(0) {
                return Err(schema_mismatch("operation_unknown exited successfully"));
            }
            let effects = data
                .map(|data| effects_from_alias(data, &["effects"]))
                .transpose()?
                .unwrap_or_default();
            let residual_resources = data
                .map(|data| effects_from_alias(data, &["residualResources", "residual_resources"]))
                .transpose()?
                .unwrap_or_default();
            return Ok(ParsedWorkerStart {
                dispatch_id: OrcaDispatchId(dispatch.to_string()),
                state: "outcome_unknown".into(),
                effects,
                residual_resources,
                raw: serde_json::to_value(&envelope).unwrap_or(Value::Null),
            });
        }
        return Err(map_envelope_error(&envelope));
    }
    let result = envelope
        .result
        .ok_or_else(|| schema_mismatch("worker-start omitted result"))?;
    validate_keys(
        object(&result)?,
        &[
            "runId",
            "taskId",
            "dispatchId",
            "state",
            "stage",
            "failedStage",
            "lastError",
            "setup",
            "launch",
            "timeoutMs",
            "effects",
            "residualResources",
            "nextCommands",
            "warning",
            "mutation",
        ],
        "worker-start result",
    )?;
    validate_mutation_request(&result, expected_request)?;
    if string_from_value(&result, &["runId"]).as_deref() != Some(expected_run.0.as_str())
        || string_from_value(&result, &["taskId"]).as_deref() != Some(expected_task.0.as_str())
    {
        return Err(schema_mismatch(
            "worker-start Run or Task identity mismatch",
        ));
    }
    let state = string_from_value(&result, &["state"])
        .ok_or_else(|| schema_mismatch("worker-start omitted state"))?;
    if !matches!(state.as_str(), "ready" | "failed" | "outcome_unknown") {
        return Err(schema_mismatch(format!(
            "unknown worker-start state {state}"
        )));
    }
    let should_succeed = state == "ready";
    if (output.exit_code == Some(0)) != should_succeed {
        return Err(schema_mismatch(
            "worker-start exit status disagrees with receipt state",
        ));
    }
    let effects = effects_from_alias(object(&result)?, &["effects"])?;
    let residual_resources = effects_from_alias(object(&result)?, &["residualResources"])?;
    Ok(ParsedWorkerStart {
        dispatch_id: OrcaDispatchId(
            string_from_value(&result, &["dispatchId"])
                .ok_or_else(|| schema_mismatch("worker-start omitted Dispatch ID"))?,
        ),
        state,
        effects,
        residual_resources,
        raw: result,
    })
}

fn parse_worker_stop(
    output: &OrcaCommandOutput,
    expected: &OrcaDispatchId,
) -> Result<ParsedWorkerStop, CommandCenterError> {
    let envelope = parse_mutation_envelope(output)?;
    if !envelope.ok {
        return Err(map_envelope_error(&envelope));
    }
    let result = envelope
        .result
        .ok_or_else(|| schema_mismatch("worker-stop omitted result"))?;
    validate_keys(
        object(&result)?,
        &[
            "dispatchId",
            "state",
            "alreadySettled",
            "processAction",
            "close",
            "lastError",
            "mutation",
        ],
        "worker-stop result",
    )?;
    require_dispatch(&result, expected, "worker-stop")?;
    let state = string_from_value(&result, &["state"])
        .ok_or_else(|| schema_mismatch("worker-stop omitted state"))?;
    if !matches!(
        state.as_str(),
        "stopped" | "stop_unknown" | "succeeded" | "failed" | "abandoned"
    ) {
        return Err(schema_mismatch(format!(
            "unknown worker-stop state {state}"
        )));
    }
    let should_succeed = state != "stop_unknown";
    if (output.exit_code == Some(0)) != should_succeed {
        return Err(schema_mismatch(
            "worker-stop exit status disagrees with receipt state",
        ));
    }
    Ok(ParsedWorkerStop { state, raw: result })
}

fn parse_worker_abandon(
    output: &OrcaCommandOutput,
    expected: &OrcaDispatchId,
) -> Result<ParsedWorkerAbandon, CommandCenterError> {
    let envelope = parse_mutation_envelope(output)?;
    require_success_exit(output, &envelope, "worker-abandon")?;
    let result = envelope
        .result
        .ok_or_else(|| schema_mismatch("worker-abandon omitted result"))?;
    validate_keys(
        object(&result)?,
        &[
            "dispatchId",
            "state",
            "alreadySettled",
            "stale",
            "processAction",
            "warning",
            "residualResources",
            "mutation",
        ],
        "worker-abandon result",
    )?;
    require_dispatch(&result, expected, "worker-abandon")?;
    if string_from_value(&result, &["state"]).as_deref() != Some("abandoned")
        || result.get("stale").and_then(Value::as_bool) != Some(false)
    {
        return Err(schema_mismatch(
            "worker-abandon did not settle the exact live Dispatch",
        ));
    }
    Ok(ParsedWorkerAbandon {
        residual_resources: effects_from_alias(object(&result)?, &["residualResources"])?,
        raw: result,
    })
}

fn parse_worker_release(
    output: &OrcaCommandOutput,
    expected: &OrcaDispatchId,
) -> Result<ParsedWorkerRelease, CommandCenterError> {
    let envelope = parse_mutation_envelope(output)?;
    if !envelope.ok {
        return Err(map_envelope_error(&envelope));
    }
    let result = envelope
        .result
        .ok_or_else(|| schema_mismatch("worker-release omitted result"))?;
    validate_keys(
        object(&result)?,
        &[
            "dispatchId",
            "state",
            "reason",
            "processAction",
            "archive",
            "lastError",
            "recovery",
            "mutation",
        ],
        "worker-release result",
    )?;
    require_dispatch(&result, expected, "worker-release")?;
    let state = string_from_value(&result, &["state"])
        .ok_or_else(|| schema_mismatch("worker-release omitted state"))?;
    if !matches!(
        state.as_str(),
        "released" | "already_released" | "retained" | "release_pending" | "release_unknown"
    ) {
        return Err(schema_mismatch(format!(
            "unknown worker-release state {state}"
        )));
    }
    let should_succeed = state != "release_unknown";
    if (output.exit_code == Some(0)) != should_succeed {
        return Err(schema_mismatch(
            "worker-release exit status disagrees with receipt state",
        ));
    }
    Ok(ParsedWorkerRelease {
        last_error: string_from_value(&result, &["lastError"]),
        state,
        raw: result,
    })
}

fn parse_mutation_envelope(output: &OrcaCommandOutput) -> Result<Envelope, CommandCenterError> {
    if output.stdout.is_empty() {
        return Err(CommandCenterError::OrcaOperationOutcomeUnknown {
            stage: "missing_stdout".into(),
        });
    }
    parse_envelope(&output.stdout).map_err(|_| CommandCenterError::OrcaOperationOutcomeUnknown {
        stage: "invalid_json".into(),
    })
}

fn parse_envelope(bytes: &[u8]) -> Result<Envelope, CommandCenterError> {
    let envelope: Envelope = serde_json::from_slice(bytes)
        .map_err(|error| schema_mismatch(format!("invalid Orca JSON envelope: {error}")))?;
    if envelope.id.trim().is_empty() {
        return Err(schema_mismatch("Orca response ID is empty"));
    }
    let _runtime_id = envelope.meta.runtime_id.as_deref();
    if envelope.ok == envelope.error.is_some() {
        return Err(schema_mismatch("Orca envelope ok/error fields disagree"));
    }
    Ok(envelope)
}

fn require_success_exit(
    output: &OrcaCommandOutput,
    envelope: &Envelope,
    command: &str,
) -> Result<(), CommandCenterError> {
    if !envelope.ok {
        return Err(map_envelope_error(envelope));
    }
    if output.exit_code != Some(0) {
        return Err(schema_mismatch(format!(
            "{command} successful receipt exited nonzero"
        )));
    }
    Ok(())
}

fn map_envelope_error(envelope: &Envelope) -> CommandCenterError {
    let Some(error) = envelope.error.as_ref() else {
        return schema_mismatch("failed Orca envelope omitted error");
    };
    match error.code.as_str() {
        "request_mismatch" => CommandCenterError::OrcaReceiptIdentityConflict {
            field: "Orca request method or parameter hash".into(),
        },
        "dispatch_not_found" => CommandCenterError::OrcaOperationRecoveryRequired {
            stage: "dispatch_not_found".into(),
        },
        "dispatch_inactive" | "task_not_startable" => CommandCenterError::OrcaPreconditionFailed {
            reason: error.message.clone(),
        },
        "operation_unknown" => CommandCenterError::OrcaOperationOutcomeUnknown {
            stage: "operation_unknown".into(),
        },
        "runtime_unavailable" => CommandCenterError::OrcaOperationOutcomeUnknown {
            stage: "runtime_unavailable_after_invoke".into(),
        },
        code => schema_mismatch(format!("unknown Orca error code {code}")),
    }
}

fn lifecycle_execution(
    record: &OrcaOperationRecord,
    outcome: OrcaMutationOutcome,
    stage: String,
    last_error: Option<String>,
    effects: Vec<OrcaEffectReceipt>,
    residual_resources: Vec<OrcaEffectReceipt>,
    cleanup: Vec<CleanupResourceProjection>,
    retry_of: Option<OrcaDispatchId>,
) -> RuntimeCommandExecution {
    let typed = typed_request(record);
    let run = run_reference(record, typed.as_ref(), &outcome, retry_of.clone());
    let attempt = match (
        record.orca_run_id.clone(),
        record.orca_task_id.clone(),
        record.orca_dispatch_id.clone(),
        record.orca_worktree_id.clone(),
    ) {
        (Some(run_id), Some(task_id), Some(dispatch_id), Some(worktree_id)) => {
            Some(OrcaAttemptIdentity {
                run_id,
                task_id,
                dispatch_id,
                retry_of_dispatch_id: retry_of,
                worktree_id,
                terminal_id: record.orca_terminal_id.clone(),
            })
        }
        _ => None,
    };
    RuntimeCommandExecution::from_lifecycle(
        run,
        OrcaLifecycleReceipt {
            outcome,
            attempt,
            stage,
            failed_stage: None,
            last_error,
            effects,
            residual_resources,
            cleanup,
            observed_at: Utc::now(),
        },
    )
}

fn execution_from_record(record: &OrcaOperationRecord) -> RuntimeCommandExecution {
    let mut stage = "stored_operation".to_string();
    let mut last_error = None;
    let mut residual_resources = Vec::new();
    let mut cleanup = recovery_cleanup(record);
    let outcome = match record.state {
        OrcaOperationState::Recorded
        | OrcaOperationState::InProgress
        | OrcaOperationState::OutcomeUnknown => OrcaMutationOutcome::OutcomeUnknown,
        OrcaOperationState::Ready => OrcaMutationOutcome::Ready,
        OrcaOperationState::Rejected => OrcaMutationOutcome::Rejected,
        OrcaOperationState::Failed => OrcaMutationOutcome::Failed,
        OrcaOperationState::RecoveryRequired => OrcaMutationOutcome::RecoveryRequired,
        OrcaOperationState::Completed => {
            if let Some(receipt) = record.receipts.iter().rev().find(|receipt| {
                record
                    .requests
                    .iter()
                    .find(|request| request.id == receipt.request_id)
                    .is_some_and(|request| {
                        matches!(request.stage.as_str(), "worker_release" | "worker_abandon")
                    })
            }) {
                let receipt_stage = record
                    .requests
                    .iter()
                    .find(|request| request.id == receipt.request_id)
                    .map(|request| request.stage.as_str())
                    .expect("terminal receipt must reference a persisted request");
                stage = receipt_stage.to_string();
                last_error = string_from_value(&receipt.payload, &["lastError"]);
                match receipt_stage {
                    "worker_release" => {
                        let cleanup_state =
                            if matches!(receipt.status.as_str(), "released" | "already_released") {
                                CleanupResourceState::VerifiedReleased
                            } else {
                                CleanupResourceState::RecoveryRequired
                            };
                        cleanup = terminal_cleanup(
                            record.orca_terminal_id.as_ref(),
                            cleanup_state,
                            format!("worker-release returned {}", receipt.status),
                        );
                        OrcaMutationOutcome::Stopped
                    }
                    "worker_abandon" => {
                        residual_resources = receipt
                            .payload
                            .as_object()
                            .and_then(|payload| {
                                effects_from_alias(payload, &["residualResources"]).ok()
                            })
                            .unwrap_or_default();
                        OrcaMutationOutcome::Abandoned
                    }
                    _ => unreachable!("filtered terminal cancellation receipt"),
                }
            } else {
                OrcaMutationOutcome::AlreadySettled
            }
        }
    };
    let retry_of = match typed_request(record) {
        Some(OrcaTypedOperationRequest::RetryLinkedRun(request)) => {
            Some(request.retry_of_dispatch_id)
        }
        _ => None,
    };
    lifecycle_execution(
        record,
        outcome,
        stage,
        last_error,
        Vec::new(),
        residual_resources,
        cleanup,
        retry_of,
    )
}

fn run_reference(
    record: &OrcaOperationRecord,
    typed: Option<&OrcaTypedOperationRequest>,
    outcome: &OrcaMutationOutcome,
    retry_of: Option<OrcaDispatchId>,
) -> JcodeRunReference {
    let (run_id, retry_of_jcode) = match typed {
        Some(OrcaTypedOperationRequest::StartInitiativeRun(request)) => {
            (request.context.jcode_attempt_id.clone(), None)
        }
        Some(OrcaTypedOperationRequest::RetryLinkedRun(request)) => (
            request.context.jcode_attempt_id.clone(),
            Some(request.prior_jcode_attempt_id.clone()),
        ),
        Some(OrcaTypedOperationRequest::CancelLinkedRun(request)) => {
            (request.target_jcode_attempt_id.clone(), None)
        }
        None => (
            record
                .jcode_run_id
                .clone()
                .unwrap_or_else(|| jcode_command_center::JcodeRunId(record.command_id.0.clone())),
            None,
        ),
    };
    JcodeRunReference {
        id: run_id,
        initiative_id: record.initiative_id.clone(),
        orca_run_id: record.orca_run_id.clone(),
        orca_task_id: record.orca_task_id.clone(),
        orca_dispatch_id: record.orca_dispatch_id.clone(),
        retry_of_jcode_run_id: retry_of_jcode,
        retry_of_dispatch_id: retry_of,
        worktree_id: record.orca_worktree_id.clone(),
        terminal_id: record.orca_terminal_id.clone(),
        status: match outcome {
            OrcaMutationOutcome::Ready => "running",
            OrcaMutationOutcome::Failed => "failed",
            OrcaMutationOutcome::Stopped => "cancelled",
            OrcaMutationOutcome::Abandoned => "abandoned",
            OrcaMutationOutcome::AlreadySettled => "settled",
            OrcaMutationOutcome::Rejected => "rejected",
            OrcaMutationOutcome::OutcomeUnknown => "outcome_unknown",
            OrcaMutationOutcome::RecoveryRequired => "recovery_required",
        }
        .into(),
        created_at: record.created_at,
        updated_at: record.updated_at,
    }
}

fn typed_request(record: &OrcaOperationRecord) -> Option<OrcaTypedOperationRequest> {
    serde_json::from_value(record.command_payload.clone()).ok()
}

fn deterministic_task_spec(request: &StartInitiativeRunRequest) -> Value {
    json!({
        "schema": TASK_SPEC_SCHEMA,
        "commandId": request.context.command_id.0,
        "correlationId": request.context.correlation_id.0,
        "initiativeId": request.context.initiative_id.0,
        "jcodeAttemptId": request.context.jcode_attempt_id.0,
        "objective": request.objective,
        "successCriteria": [],
    })
}

fn append_launcher_args(
    args: &mut Vec<String>,
    launcher: &OrcaWorkerLauncher,
) -> Result<(), CommandCenterError> {
    match launcher {
        OrcaWorkerLauncher::Agent {
            agent,
            model,
            effort,
        } if !agent.trim().is_empty() => {
            args.extend(["--agent".into(), agent.clone()]);
            if let Some(model) = model {
                args.extend(["--model".into(), model.clone()]);
            }
            if let Some(effort) = effort {
                args.extend(["--effort".into(), effort.clone()]);
            }
            Ok(())
        }
        OrcaWorkerLauncher::Agent { .. } => Err(CommandCenterError::OrcaCoordinatorUnavailable),
        OrcaWorkerLauncher::ExistingTerminal { .. } => {
            Err(CommandCenterError::OrcaPreconditionFailed {
                reason: "the pinned profile supports only configured agent launch".into(),
            })
        }
    }
}

fn validate_supported_placement(
    placement: &OrcaCanonicalPlacement,
) -> Result<(), CommandCenterError> {
    if placement.worktree_selector != format!("id:{}", placement.worktree_id.0)
        || placement.worktree_selector == "current"
    {
        return Err(CommandCenterError::OrcaPreconditionFailed {
            reason: "Orca placement must select one exact existing worktree ID".into(),
        });
    }
    if placement.coordinator_terminal_id.0.trim().is_empty() {
        return Err(CommandCenterError::OrcaCoordinatorUnavailable);
    }
    let mut args = Vec::new();
    append_launcher_args(&mut args, &placement.launcher)
}

fn compare_placement(
    expected: &OrcaCanonicalPlacement,
    observed: &OrcaCanonicalPlacement,
) -> Result<(), CommandCenterError> {
    if expected != observed {
        return Err(CommandCenterError::OrcaIdentityDrift {
            reason: "stored Orca placement differs from the current canonical placement".into(),
        });
    }
    Ok(())
}

fn terminal_cleanup(
    terminal: Option<&OrcaTerminalId>,
    state: CleanupResourceState,
    evidence: String,
) -> Vec<CleanupResourceProjection> {
    terminal
        .map(|terminal| {
            vec![CleanupResourceProjection {
                resource_kind: "terminal".into(),
                resource_id: terminal.0.clone(),
                state,
                evidence,
            }]
        })
        .unwrap_or_default()
}

fn recovery_cleanup(record: &OrcaOperationRecord) -> Vec<CleanupResourceProjection> {
    record
        .recovery
        .iter()
        .filter_map(|obligation| {
            obligation
                .resource_id
                .as_ref()
                .map(|resource_id| CleanupResourceProjection {
                    resource_kind: obligation.resource_kind.clone(),
                    resource_id: resource_id.clone(),
                    state: if obligation.state == OrcaRecoveryState::Verified {
                        CleanupResourceState::VerifiedReleased
                    } else {
                        CleanupResourceState::RecoveryRequired
                    },
                    evidence: obligation.action.clone(),
                })
        })
        .collect()
}

fn effects_from_alias(
    object: &Map<String, Value>,
    aliases: &[&str],
) -> Result<Vec<OrcaEffectReceipt>, CommandCenterError> {
    let Some(value) = aliases.iter().find_map(|key| object.get(*key)) else {
        return Ok(Vec::new());
    };
    if let Some(text) = value.as_str() {
        return serde_json::from_str(text)
            .map_err(|error| schema_mismatch(format!("invalid effect list: {error}")));
    }
    serde_json::from_value(value.clone())
        .map_err(|error| schema_mismatch(format!("invalid effect list: {error}")))
}

fn require_dispatch(
    result: &Value,
    expected: &OrcaDispatchId,
    command: &str,
) -> Result<(), CommandCenterError> {
    if string_from_value(result, &["dispatchId"]).as_deref() != Some(expected.0.as_str()) {
        return Err(schema_mismatch(format!(
            "{command} returned a different Dispatch ID"
        )));
    }
    Ok(())
}

fn validate_mutation_request(
    result: &Value,
    expected: Option<&OrcaRequestId>,
) -> Result<(), CommandCenterError> {
    let Some(expected) = expected else {
        return Ok(());
    };
    let Some(mutation) = result.get("mutation") else {
        // The pinned 1.4.176 fixture corpus predates mutation metadata on a few
        // success fixtures. When the runtime supplies it, it is authoritative.
        return Ok(());
    };
    let mutation = mutation
        .as_object()
        .ok_or_else(|| schema_mismatch("mutation metadata is not an object"))?;
    validate_keys(mutation, &["requestId", "replayed"], "mutation metadata")?;
    if required_string(mutation, "requestId")? != expected.0 {
        return Err(CommandCenterError::OrcaReceiptIdentityConflict {
            field: "Orca mutation request_id".into(),
        });
    }
    if !mutation.get("replayed").is_some_and(Value::is_boolean) {
        return Err(schema_mismatch("mutation metadata omitted replayed"));
    }
    Ok(())
}

fn validate_worker_state(state: &str) -> Result<(), CommandCenterError> {
    if matches!(
        state,
        "starting"
            | "ready"
            | "start_unknown"
            | "succeeded"
            | "failed"
            | "stopping"
            | "stop_unknown"
            | "stopped"
            | "abandoned"
    ) {
        Ok(())
    } else {
        Err(schema_mismatch(format!("unknown worker state {state}")))
    }
}

fn validate_keys(
    object: &Map<String, Value>,
    allowed: &[&str],
    context: &str,
) -> Result<(), CommandCenterError> {
    let allowed = allowed.iter().copied().collect::<BTreeSet<_>>();
    if let Some(unknown) = object.keys().find(|key| !allowed.contains(key.as_str())) {
        return Err(schema_mismatch(format!(
            "{context} contains unknown field {unknown}"
        )));
    }
    Ok(())
}

fn object(value: &Value) -> Result<&Map<String, Value>, CommandCenterError> {
    value
        .as_object()
        .ok_or_else(|| schema_mismatch("expected a JSON object"))
}

fn required_object<'a>(
    value: &'a Value,
    key: &str,
) -> Result<&'a Map<String, Value>, CommandCenterError> {
    value
        .get(key)
        .and_then(Value::as_object)
        .ok_or_else(|| schema_mismatch(format!("missing object field {key}")))
}

fn required_array<'a>(value: &'a Value, key: &str) -> Result<&'a [Value], CommandCenterError> {
    value
        .get(key)
        .and_then(Value::as_array)
        .map(Vec::as_slice)
        .ok_or_else(|| schema_mismatch(format!("missing array field {key}")))
}

fn required_string<'a>(
    object: &'a Map<String, Value>,
    key: &str,
) -> Result<&'a str, CommandCenterError> {
    object
        .get(key)
        .and_then(Value::as_str)
        .ok_or_else(|| schema_mismatch(format!("missing string field {key}")))
}

fn required_string_alias<'a>(
    object: &'a Map<String, Value>,
    aliases: &[&str],
) -> Result<&'a str, CommandCenterError> {
    string_alias(object, aliases)
        .ok_or_else(|| schema_mismatch(format!("missing string field {}", aliases.join("/"))))
}

fn string_alias<'a>(object: &'a Map<String, Value>, aliases: &[&str]) -> Option<&'a str> {
    aliases
        .iter()
        .find_map(|key| object.get(*key).and_then(Value::as_str))
}

fn array_alias<'a>(object: &'a Map<String, Value>, aliases: &[&str]) -> Option<&'a [Value]> {
    aliases.iter().find_map(|key| {
        object
            .get(*key)
            .and_then(Value::as_array)
            .map(Vec::as_slice)
    })
}

fn nested_string(value: &Value, path: &[&str]) -> Option<String> {
    let mut current = value;
    for key in path {
        current = current.get(*key)?;
    }
    current.as_str().map(str::to_string)
}

fn string_from_value(value: &Value, keys: &[&str]) -> Option<String> {
    let object = value.as_object()?;
    string_alias(object, keys).map(str::to_string)
}

fn canonical_working_dir(path: Option<&Path>) -> Result<PathBuf, CommandCenterError> {
    path.ok_or_else(|| identity_unresolved("working directory is unavailable"))?
        .canonicalize()
        .map_err(|_| identity_unresolved("working directory cannot be canonicalized"))
}

fn canonical_path(path: &str) -> Result<PathBuf, CommandCenterError> {
    Path::new(path)
        .canonicalize()
        .map_err(|_| identity_unresolved("Orca returned a path that cannot be canonicalized"))
}

fn strings(values: &[&str]) -> Vec<String> {
    values.iter().map(|value| (*value).to_string()).collect()
}

fn scope(kind: &str, initiative: &str) -> String {
    format!("command-center:{kind}:{initiative}")
}

fn update_now() -> OrcaOperationUpdate {
    OrcaOperationUpdate {
        updated_at: Utc::now(),
        ..Default::default()
    }
}

fn store_error(error: OrcaOperationStoreError) -> CommandCenterError {
    match error {
        OrcaOperationStoreError::IdentityConflict { field } => {
            CommandCenterError::OrcaReceiptIdentityConflict {
                field: field.to_string(),
            }
        }
        other => CommandCenterError::OrcaOperationRecoveryRequired {
            stage: other.to_string(),
        },
    }
}

fn schema_mismatch(reason: impl Into<String>) -> CommandCenterError {
    CommandCenterError::OrcaSchemaMismatch {
        reason: reason.into(),
    }
}

fn identity_unresolved(reason: impl Into<String>) -> CommandCenterError {
    CommandCenterError::OrcaIdentityUnresolved {
        reason: reason.into(),
    }
}

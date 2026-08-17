use std::collections::VecDeque;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use async_trait::async_trait;
use jcode_command_center::orca_operation_store::SqliteOrcaOperationStore;
use jcode_command_center::{
    CancelLinkedRunRequest, CommandId, CorrelationId, IdempotencyKey, InitiativeId, JcodeRunId,
    OrcaCanonicalPlacement, OrcaDispatchId, OrcaHostId, OrcaHostSetupId, OrcaMutationContext,
    OrcaMutationOutcome, OrcaProjectId, OrcaRepositoryId, OrcaRunId, OrcaTaskId, OrcaTerminalId,
    OrcaWorkerLauncher, OrcaWorktreeId, RetryLinkedRunRequest, StartInitiativeRunRequest,
};
use serde_json::{Value, json};

use super::orca_lifecycle::{
    OrcaCoordinatorBinding, OrcaLifecycleAdapter, OrcaLifecycleConfig, OrcaMutationStage,
    ReconciliationSummary, orca_request_id,
};
use super::{OrcaCommandOutput, OrcaCommandRunner, OrcaProcessError};

#[derive(Debug)]
struct ScriptedCall {
    args: Vec<String>,
    exit_code: i32,
    response: Value,
    persisted_stage: Option<&'static str>,
}

struct ScriptedRunner {
    calls: Mutex<VecDeque<ScriptedCall>>,
    store: Arc<SqliteOrcaOperationStore>,
    command_id: CommandId,
    expected_dir: PathBuf,
}

impl std::fmt::Debug for ScriptedRunner {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("ScriptedRunner")
            .finish_non_exhaustive()
    }
}

#[async_trait]
impl OrcaCommandRunner for ScriptedRunner {
    async fn run(
        &self,
        _command: &str,
        args: &[String],
        current_dir: Option<&Path>,
        _timeout: Duration,
    ) -> Result<OrcaCommandOutput, OrcaProcessError> {
        assert_eq!(current_dir, Some(self.expected_dir.as_path()));
        let call = self
            .calls
            .lock()
            .unwrap()
            .pop_front()
            .expect("unexpected Orca call");
        assert_eq!(args, call.args);
        if let Some(stage) = call.persisted_stage {
            let operation = self
                .store
                .get_by_command(&self.command_id)
                .unwrap()
                .expect("operation persisted before runner invocation");
            let request = operation
                .requests
                .iter()
                .find(|request| request.stage == stage)
                .expect("stage request persisted before runner invocation");
            let stable = request
                .orca_request_id
                .as_ref()
                .expect("stable Orca request ID");
            assert!(
                args.windows(2)
                    .any(|pair| { pair[0] == "--retry-request" && pair[1] == stable.0 })
            );
        }
        Ok(OrcaCommandOutput {
            exit_code: Some(call.exit_code),
            stdout: serde_json::to_vec(&call.response).unwrap(),
            stderr: Vec::new(),
        })
    }
}

fn envelope(result: Value) -> Value {
    json!({
        "id": "fixture-response",
        "ok": true,
        "result": result,
        "_meta": { "runtimeId": "runtime-fixture" }
    })
}

fn mutation_call(
    args: &[String],
    exit_code: i32,
    result: Value,
    stage: &'static str,
) -> ScriptedCall {
    ScriptedCall {
        args: args.to_vec(),
        exit_code,
        response: envelope(result),
        persisted_stage: Some(stage),
    }
}

fn read_call(args: &[&str], result: Value) -> ScriptedCall {
    ScriptedCall {
        args: args.iter().map(|value| (*value).to_string()).collect(),
        exit_code: 0,
        response: envelope(result),
        persisted_stage: None,
    }
}

fn context(command: &str, attempt: &str) -> OrcaMutationContext {
    OrcaMutationContext {
        command_id: CommandId(command.to_string()),
        idempotency_key: IdempotencyKey(format!("key-{command}")),
        correlation_id: CorrelationId(format!("correlation-{command}")),
        initiative_id: InitiativeId("initiative-1".to_string()),
        jcode_attempt_id: JcodeRunId(attempt.to_string()),
    }
}

fn placement() -> OrcaCanonicalPlacement {
    OrcaCanonicalPlacement {
        project_id: OrcaProjectId("project-1".to_string()),
        repository_id: OrcaRepositoryId("repo-1".to_string()),
        host_setup_id: OrcaHostSetupId("setup-1".to_string()),
        host_id: OrcaHostId("host-1".to_string()),
        worktree_id: OrcaWorktreeId("worktree-1".to_string()),
        worktree_selector: "id:worktree-1".to_string(),
        coordinator_terminal_id: OrcaTerminalId("terminal-coordinator".to_string()),
        environment: None,
        launcher: OrcaWorkerLauncher::Agent {
            agent: "codex".to_string(),
            model: None,
            effort: None,
        },
    }
}

fn identity_calls() -> Vec<ScriptedCall> {
    vec![
        read_call(
            &["worktree", "current", "--json"],
            json!({"worktree": {"id":"worktree-1","repoId":"repo-1","hostId":"host-1","path":"WORKING_DIR"}}),
        ),
        read_call(
            &["repo", "list", "--json"],
            json!({"repos":[{"id":"repo-1","path":"WORKING_DIR"}]}),
        ),
        read_call(
            &["project", "setups", "--json"],
            json!({"setups":[{"id":"setup-1","projectId":"project-1","repoId":"repo-1","hostId":"host-1","setupState":"ready"}]}),
        ),
        read_call(
            &["project", "list", "--json"],
            json!({"projects":[{"id":"project-1","sourceRepoIds":["repo-1"]}]}),
        ),
        read_call(
            &[
                "terminal",
                "show",
                "--terminal",
                "terminal-coordinator",
                "--json",
            ],
            json!({"terminal":{"handle":"terminal-coordinator"}}),
        ),
    ]
}

fn replace_working_dir(calls: &mut [ScriptedCall], path: &Path) {
    let rendered = path.to_string_lossy();
    for call in calls {
        let text = serde_json::to_string(&call.response)
            .unwrap()
            .replace("WORKING_DIR", &rendered);
        call.response = serde_json::from_str(&text).unwrap();
    }
}

fn adapter_with_calls(
    command_id: &str,
    calls: Vec<ScriptedCall>,
) -> (
    OrcaLifecycleAdapter,
    Arc<SqliteOrcaOperationStore>,
    tempfile::TempDir,
) {
    let directory = tempfile::tempdir().unwrap();
    let working_dir = directory.path().canonicalize().unwrap();
    let store = Arc::new(SqliteOrcaOperationStore::open_in_memory().unwrap());
    let mut calls = calls;
    replace_working_dir(&mut calls, &working_dir);
    let runner = Arc::new(ScriptedRunner {
        calls: Mutex::new(calls.into()),
        store: Arc::clone(&store),
        command_id: CommandId(command_id.to_string()),
        expected_dir: working_dir.clone(),
    });
    let adapter = OrcaLifecycleAdapter::new(OrcaLifecycleConfig {
        command: "orca".to_string(),
        working_dir: Some(working_dir),
        runner,
        store: Arc::clone(&store),
        coordinator: OrcaCoordinatorBinding {
            terminal: OrcaTerminalId("terminal-coordinator".to_string()),
        },
        launcher: OrcaWorkerLauncher::Agent {
            agent: "codex".to_string(),
            model: None,
            effort: None,
        },
        timeout: Duration::from_secs(5),
    });
    (adapter, store, directory)
}

#[test]
fn request_ids_are_deterministic_per_command_and_stage() {
    let command = CommandId("command-1".to_string());
    assert_eq!(
        orca_request_id(&command, OrcaMutationStage::WorkerStart),
        orca_request_id(&command, OrcaMutationStage::WorkerStart)
    );
    assert_ne!(
        orca_request_id(&command, OrcaMutationStage::WorkerStart),
        orca_request_id(&command, OrcaMutationStage::WorkerStop)
    );
}

#[test]
fn clean_start_persists_every_mutation_before_invocation_and_uses_exact_placement() {
    tokio::runtime::Runtime::new().unwrap().block_on(async {
        let command_id = "command-start";
        let run_request = orca_request_id(&CommandId(command_id.into()), OrcaMutationStage::RunCreate);
        let task_request = orca_request_id(&CommandId(command_id.into()), OrcaMutationStage::TaskCreate);
        let worker_request = orca_request_id(&CommandId(command_id.into()), OrcaMutationStage::WorkerStart);
        let mut calls = identity_calls();
        calls.extend([
            mutation_call(
                &[
                    "orchestration".into(), "run-create".into(), "--objective".into(),
                    "[jcode-cc:command-start] Ship lifecycle".into(), "--from".into(),
                    "terminal-coordinator".into(), "--retry-request".into(), run_request.0,
                    "--json".into(),
                ],
                0,
                json!({"run":{"id":"run-1","objective":"[jcode-cc:command-start] Ship lifecycle"},"binding":{"consumerGeneration":1}}),
                "run_create",
            ),
            read_call(&["orchestration","run-current","--from","terminal-coordinator","--json"], json!({"run":{"id":"run-1"}})),
            mutation_call(
                &[
                    "orchestration".into(), "task-create".into(), "--spec".into(),
                    json!({"schema":"jcode.command-center.task.v1","commandId":"command-start","correlationId":"correlation-command-start","initiativeId":"initiative-1","jcodeAttemptId":"attempt-1","objective":"Ship lifecycle","successCriteria":[]}).to_string(),
                    "--task-title".into(), "Ship lifecycle".into(), "--display-name".into(),
                    "jcode-cc:command-start:initial".into(), "--run".into(), "run-1".into(),
                    "--from".into(), "terminal-coordinator".into(), "--retry-request".into(),
                    task_request.0, "--json".into(),
                ],
                0,
                json!({"task":{"id":"task-1","run_id":"run-1","display_name":"jcode-cc:command-start:initial","status":"pending"}}),
                "task_create",
            ),
        ]);
        calls.extend(identity_calls());
        calls.extend([
            read_call(&["orchestration","run-current","--from","terminal-coordinator","--json"], json!({"run":{"id":"run-1"}})),
            mutation_call(
                &[
                    "orchestration".into(), "worker-start".into(), "--task".into(), "task-1".into(),
                    "--run".into(), "run-1".into(), "--from".into(), "terminal-coordinator".into(),
                    "--worktree".into(), "id:worktree-1".into(), "--agent".into(), "codex".into(),
                    "--retry-request".into(), worker_request.0, "--json".into(),
                ],
                0,
                json!({"runId":"run-1","taskId":"task-1","dispatchId":"dispatch-1","state":"ready","stage":"ready","setup":{},"launch":{},"effects":[{"kind":"terminal","role":"agent","action":"created","id":"terminal-worker","surface":"background"}],"residualResources":[]}),
                "worker_start",
            ),
            read_call(&["orchestration","worker-show","--dispatch","dispatch-1","--json"], json!({
                "dispatch":{"id":"dispatch-1","run_id":"run-1","task_id":"task-1"},
                "worker":{"dispatch_id":"dispatch-1","runtime_epoch":"runtime-fixture","state":"ready","stage":"ready","worktree_id":"worktree-1","agent_terminal_handle":"terminal-worker","effects":[],"residualResources":[],"startOptions":{"resolvedWorktreeId":"worktree-1","agent":"codex"}},
                "terminal":null,"observation":{"status":"missing","exactWorker":true},"terminalResource":null
            })),
        ]);
        let (adapter, store, _directory) = adapter_with_calls(command_id, calls);
        let result = adapter.start(StartInitiativeRunRequest {
            context: context(command_id, "attempt-1"),
            objective: "Ship lifecycle".to_string(),
            task_spec: "{}".to_string(),
            placement: placement(),
        }).await;
        assert_eq!(result.state, jcode_command_center::CommandState::Pending);
        let record = store.get_by_command(&CommandId(command_id.into())).unwrap().unwrap();
        assert_eq!(record.requests.len(), 3);
        assert_eq!(record.orca_dispatch_id, Some(OrcaDispatchId("dispatch-1".into())));
    });
}

#[test]
fn worker_start_failed_json_from_nonzero_exit_is_authoritative_not_unavailable() {
    tokio::runtime::Runtime::new().unwrap().block_on(async {
        let command_id = "command-failed";
        let mut calls = identity_calls();
        let run_request = orca_request_id(&CommandId(command_id.into()), OrcaMutationStage::RunCreate);
        let task_request = orca_request_id(&CommandId(command_id.into()), OrcaMutationStage::TaskCreate);
        let worker_request = orca_request_id(&CommandId(command_id.into()), OrcaMutationStage::WorkerStart);
        calls.extend([
            mutation_call(&["orchestration".into(),"run-create".into(),"--objective".into(),"[jcode-cc:command-failed] Fail safely".into(),"--from".into(),"terminal-coordinator".into(),"--retry-request".into(),run_request.0,"--json".into()],0,json!({"run":{"id":"run-1","objective":"[jcode-cc:command-failed] Fail safely"},"binding":{"consumerGeneration":1}}),"run_create"),
            read_call(&["orchestration","run-current","--from","terminal-coordinator","--json"],json!({"run":{"id":"run-1"}})),
            mutation_call(&["orchestration".into(),"task-create".into(),"--spec".into(),json!({"schema":"jcode.command-center.task.v1","commandId":command_id,"correlationId":"correlation-command-failed","initiativeId":"initiative-1","jcodeAttemptId":"attempt-failed","objective":"Fail safely","successCriteria":[]}).to_string(),"--task-title".into(),"Fail safely".into(),"--display-name".into(),"jcode-cc:command-failed:initial".into(),"--run".into(),"run-1".into(),"--from".into(),"terminal-coordinator".into(),"--retry-request".into(),task_request.0,"--json".into()],0,json!({"task":{"id":"task-1","run_id":"run-1","display_name":"jcode-cc:command-failed:initial","status":"pending"}}),"task_create"),
        ]);
        calls.extend(identity_calls());
        calls.extend([
            read_call(&["orchestration","run-current","--from","terminal-coordinator","--json"],json!({"run":{"id":"run-1"}})),
            mutation_call(&["orchestration".into(),"worker-start".into(),"--task".into(),"task-1".into(),"--run".into(),"run-1".into(),"--from".into(),"terminal-coordinator".into(),"--worktree".into(),"id:worktree-1".into(),"--agent".into(),"codex".into(),"--retry-request".into(),worker_request.0,"--json".into()],7,json!({"runId":"run-1","taskId":"task-1","dispatchId":"dispatch-failed","state":"failed","stage":"failed","failedStage":"terminal_create","lastError":"fixture failure","setup":{},"launch":{},"effects":[],"residualResources":[]}),"worker_start"),
            read_call(&["orchestration","worker-show","--dispatch","dispatch-failed","--json"],json!({"dispatch":{"id":"dispatch-failed","run_id":"run-1","task_id":"task-1"},"worker":{"dispatch_id":"dispatch-failed","runtime_epoch":"runtime-fixture","state":"failed","stage":"failed","worktree_id":"worktree-1","agent_terminal_handle":null,"effects":[],"residualResources":[],"startOptions":{"resolvedWorktreeId":"worktree-1","agent":"codex"}},"terminal":null,"observation":{"status":"missing","exactWorker":true},"terminalResource":null})),
        ]);
        let (adapter, _, _directory) = adapter_with_calls(command_id, calls);
        let execution = adapter.start(StartInitiativeRunRequest { context: context(command_id,"attempt-failed"), objective:"Fail safely".into(), task_spec:"{}".into(), placement:placement() }).await;
        assert_eq!(execution.state, jcode_command_center::CommandState::Failed);
        let receipt = match execution.payload.unwrap() { jcode_command_center::CommandResultPayload::RunAccepted { receipt: Some(receipt), .. } => receipt, _ => panic!("receipt") };
        assert_eq!(receipt.outcome, OrcaMutationOutcome::Failed);
        assert_eq!(receipt.attempt.unwrap().dispatch_id, OrcaDispatchId("dispatch-failed".into()));
    });
}

#[test]
fn retry_targets_the_exact_prior_dispatch_and_creates_a_distinct_dispatch() {
    tokio::runtime::Runtime::new().unwrap().block_on(async {
        let command_id = "command-retry";
        let retry_request = orca_request_id(&CommandId(command_id.into()), OrcaMutationStage::WorkerRetry);
        let mut calls = identity_calls();
        calls.extend(vec![
            read_call(&["orchestration","worker-show","--dispatch","dispatch-old","--json"],json!({"dispatch":{"id":"dispatch-old","run_id":"run-1","task_id":"task-1"},"worker":{"dispatch_id":"dispatch-old","runtime_epoch":"runtime-fixture","state":"failed","stage":"failed","worktree_id":"worktree-1","agent_terminal_handle":"terminal-old","effects":[],"residualResources":[],"startOptions":{"resolvedWorktreeId":"worktree-1","agent":"codex"}},"terminal":null,"observation":{"status":"missing","exactWorker":true},"terminalResource":null})),
            read_call(&["orchestration","task-list","--run","run-1","--json"],json!({"runId":"run-1","tasks":[{"id":"task-1","status":"failed"}]})),
            read_call(&["orchestration","worker-list","--run","run-1","--json"],json!({"workers":[{"dispatchId":"dispatch-old","taskId":"task-1","runId":"run-1","workerState":"failed"}]})),
            read_call(&["orchestration","run-current","--from","terminal-coordinator","--json"],json!({"run":{"id":"run-1"}})),
            mutation_call(&["orchestration".into(),"worker-start".into(),"--task".into(),"task-1".into(),"--run".into(),"run-1".into(),"--from".into(),"terminal-coordinator".into(),"--worktree".into(),"id:worktree-1".into(),"--agent".into(),"codex".into(),"--retry-of".into(),"dispatch-old".into(),"--retry-request".into(),retry_request.0,"--json".into()],0,json!({"runId":"run-1","taskId":"task-1","dispatchId":"dispatch-new","state":"ready","stage":"ready","setup":{},"launch":{},"effects":[],"residualResources":[]}),"worker_retry"),
            read_call(&["orchestration","worker-show","--dispatch","dispatch-new","--json"],json!({"dispatch":{"id":"dispatch-new","run_id":"run-1","task_id":"task-1"},"worker":{"dispatch_id":"dispatch-new","runtime_epoch":"runtime-fixture","state":"ready","stage":"ready","worktree_id":"worktree-1","agent_terminal_handle":"terminal-new","effects":[],"residualResources":[],"startOptions":{"resolvedWorktreeId":"worktree-1","agent":"codex"}},"terminal":null,"observation":{"status":"missing","exactWorker":true},"terminalResource":null})),
        ]);
        let (adapter, store, _directory) = adapter_with_calls(command_id, calls);
        let execution = adapter.retry(RetryLinkedRunRequest { context:context(command_id,"attempt-new"), prior_jcode_attempt_id:JcodeRunId("attempt-old".into()), orca_run_id:OrcaRunId("run-1".into()), orca_task_id:OrcaTaskId("task-1".into()), retry_of_dispatch_id:OrcaDispatchId("dispatch-old".into()), placement:placement() }).await;
        assert_eq!(execution.state, jcode_command_center::CommandState::Pending);
        let record = store.get_by_command(&CommandId(command_id.into())).unwrap().unwrap();
        assert_eq!(record.orca_dispatch_id, Some(OrcaDispatchId("dispatch-new".into())));
    });
}

#[test]
fn retry_rebinds_the_coordinator_with_a_persisted_stable_request() {
    tokio::runtime::Runtime::new().unwrap().block_on(async {
        let command_id = "command-rebind";
        let bind_request =
            orca_request_id(&CommandId(command_id.into()), OrcaMutationStage::RunBind);
        let retry_request =
            orca_request_id(&CommandId(command_id.into()), OrcaMutationStage::WorkerRetry);
        let mut calls = identity_calls();
        calls.extend(vec![
            read_call(&["orchestration","worker-show","--dispatch","dispatch-old","--json"],json!({"dispatch":{"id":"dispatch-old","run_id":"run-1","task_id":"task-1"},"worker":{"dispatch_id":"dispatch-old","runtime_epoch":"runtime-fixture","state":"failed","stage":"failed","worktree_id":"worktree-1","agent_terminal_handle":"terminal-old","effects":[],"residualResources":[],"startOptions":{"resolvedWorktreeId":"worktree-1","agent":"codex"}},"terminal":null,"observation":{"status":"missing","exactWorker":true},"terminalResource":null})),
            read_call(&["orchestration","task-list","--run","run-1","--json"],json!({"runId":"run-1","tasks":[{"id":"task-1","status":"failed"}]})),
            read_call(&["orchestration","worker-list","--run","run-1","--json"],json!({"workers":[{"dispatchId":"dispatch-old","taskId":"task-1","runId":"run-1","workerState":"failed"}]})),
            read_call(&["orchestration","run-current","--from","terminal-coordinator","--json"],json!({"run":{"id":"run-other"}})),
            mutation_call(&["orchestration".into(),"run-use".into(),"--id".into(),"run-1".into(),"--from".into(),"terminal-coordinator".into(),"--retry-request".into(),bind_request.0,"--json".into()],0,json!({"run":{"id":"run-1"},"binding":{"consumerGeneration":2}}),"run_bind"),
            mutation_call(&["orchestration".into(),"worker-start".into(),"--task".into(),"task-1".into(),"--run".into(),"run-1".into(),"--from".into(),"terminal-coordinator".into(),"--worktree".into(),"id:worktree-1".into(),"--agent".into(),"codex".into(),"--retry-of".into(),"dispatch-old".into(),"--retry-request".into(),retry_request.0,"--json".into()],0,json!({"runId":"run-1","taskId":"task-1","dispatchId":"dispatch-new","state":"ready","stage":"ready","setup":{},"launch":{},"effects":[],"residualResources":[]}),"worker_retry"),
            read_call(&["orchestration","worker-show","--dispatch","dispatch-new","--json"],json!({"dispatch":{"id":"dispatch-new","run_id":"run-1","task_id":"task-1"},"worker":{"dispatch_id":"dispatch-new","runtime_epoch":"runtime-fixture","state":"ready","stage":"ready","worktree_id":"worktree-1","agent_terminal_handle":"terminal-new","effects":[],"residualResources":[],"startOptions":{"resolvedWorktreeId":"worktree-1","agent":"codex"}},"terminal":null,"observation":{"status":"missing","exactWorker":true},"terminalResource":null})),
        ]);
        let (adapter, store, _directory) = adapter_with_calls(command_id, calls);
        let execution = adapter.retry(RetryLinkedRunRequest { context:context(command_id,"attempt-new"), prior_jcode_attempt_id:JcodeRunId("attempt-old".into()), orca_run_id:OrcaRunId("run-1".into()), orca_task_id:OrcaTaskId("task-1".into()), retry_of_dispatch_id:OrcaDispatchId("dispatch-old".into()), placement:placement() }).await;
        assert_eq!(execution.state, jcode_command_center::CommandState::Pending);
        let record = store.get_by_command(&CommandId(command_id.into())).unwrap().unwrap();
        assert!(record.requests.iter().any(|request| request.stage == "run_bind"));
        assert_eq!(record.orca_dispatch_id, Some(OrcaDispatchId("dispatch-new".into())));
    });
}

#[test]
fn cancel_stops_exact_dispatch_then_releases_only_its_terminal() {
    tokio::runtime::Runtime::new().unwrap().block_on(async {
        let command_id = "command-cancel";
        let stop_request = orca_request_id(&CommandId(command_id.into()), OrcaMutationStage::WorkerStop);
        let release_request = orca_request_id(&CommandId(command_id.into()), OrcaMutationStage::WorkerRelease);
        let calls = vec![
            read_call(&["orchestration","worker-show","--dispatch","dispatch-1","--json"],json!({"dispatch":{"id":"dispatch-1","run_id":"run-1","task_id":"task-1"},"worker":{"dispatch_id":"dispatch-1","runtime_epoch":"runtime-fixture","state":"ready","stage":"ready","worktree_id":"worktree-1","agent_terminal_handle":"terminal-worker","effects":[],"residualResources":[],"startOptions":{"resolvedWorktreeId":"worktree-1","agent":"codex"}},"terminal":null,"observation":{"status":"missing","exactWorker":true},"terminalResource":null})),
            mutation_call(&["orchestration".into(),"worker-stop".into(),"--dispatch".into(),"dispatch-1".into(),"--retry-request".into(),stop_request.0,"--json".into()],0,json!({"dispatchId":"dispatch-1","state":"stopped","alreadySettled":false,"processAction":"closed_agent_terminal","close":{"handle":"terminal-worker","closed":true}}),"worker_stop"),
            read_call(&["orchestration","worker-show","--dispatch","dispatch-1","--json"],json!({"dispatch":{"id":"dispatch-1","run_id":"run-1","task_id":"task-1"},"worker":{"dispatch_id":"dispatch-1","runtime_epoch":"runtime-fixture","state":"stopped","stage":"stopped","worktree_id":"worktree-1","agent_terminal_handle":"terminal-worker","effects":[],"residualResources":[],"startOptions":{"resolvedWorktreeId":"worktree-1","agent":"codex"}},"terminal":null,"observation":{"status":"missing","exactWorker":true},"terminalResource":null})),
            mutation_call(&["orchestration".into(),"worker-release".into(),"--dispatch".into(),"dispatch-1".into(),"--retry-request".into(),release_request.0,"--json".into()],0,json!({"dispatchId":"dispatch-1","state":"released","processAction":"closed_agent_terminal","archive":{"source":"terminal","status":"captured"}}),"worker_release"),
        ];
        let (adapter, _, _) = adapter_with_calls(command_id, calls);
        let execution = adapter.cancel(CancelLinkedRunRequest { context:context(command_id,"cancel-attempt"), target_jcode_attempt_id:JcodeRunId("attempt-1".into()), orca_run_id:OrcaRunId("run-1".into()), orca_task_id:OrcaTaskId("task-1".into()), target_dispatch_id:OrcaDispatchId("dispatch-1".into()) }).await;
        assert_eq!(execution.state, jcode_command_center::CommandState::Completed);
        let receipt = match execution.payload.unwrap() { jcode_command_center::CommandResultPayload::RunAccepted { receipt:Some(receipt), .. } => receipt, _ => panic!("receipt") };
        assert_eq!(receipt.outcome, OrcaMutationOutcome::Stopped);
        assert_eq!(receipt.cleanup.len(), 1);
        assert_eq!(receipt.cleanup[0].resource_kind, "terminal");
        assert_eq!(receipt.cleanup[0].resource_id, "terminal-worker");
    });
}

#[test]
fn stop_unknown_is_abandoned_with_a_separate_receipt_and_never_released() {
    tokio::runtime::Runtime::new().unwrap().block_on(async {
        let command_id = "command-abandon";
        let stop_request =
            orca_request_id(&CommandId(command_id.into()), OrcaMutationStage::WorkerStop);
        let abandon_request =
            orca_request_id(&CommandId(command_id.into()), OrcaMutationStage::WorkerAbandon);
        let calls = vec![
            read_call(&["orchestration","worker-show","--dispatch","dispatch-1","--json"],json!({"dispatch":{"id":"dispatch-1","run_id":"run-1","task_id":"task-1"},"worker":{"dispatch_id":"dispatch-1","runtime_epoch":"runtime-fixture","state":"ready","stage":"ready","worktree_id":"worktree-1","agent_terminal_handle":"terminal-worker","effects":[],"residualResources":[],"startOptions":{"resolvedWorktreeId":"worktree-1","agent":"codex"}},"terminal":null,"observation":{"status":"missing","exactWorker":true},"terminalResource":null})),
            mutation_call(&["orchestration".into(),"worker-stop".into(),"--dispatch".into(),"dispatch-1".into(),"--retry-request".into(),stop_request.0,"--json".into()],7,json!({"dispatchId":"dispatch-1","state":"stop_unknown","alreadySettled":false,"processAction":"close_unconfirmed","lastError":"fixture timeout"}),"worker_stop"),
            read_call(&["orchestration","worker-show","--dispatch","dispatch-1","--json"],json!({"dispatch":{"id":"dispatch-1","run_id":"run-1","task_id":"task-1"},"worker":{"dispatch_id":"dispatch-1","runtime_epoch":"runtime-fixture","state":"stop_unknown","stage":"stop_unknown","worktree_id":"worktree-1","agent_terminal_handle":"terminal-worker","effects":[],"residualResources":[],"startOptions":{"resolvedWorktreeId":"worktree-1","agent":"codex"}},"terminal":null,"observation":{"status":"missing","exactWorker":true},"terminalResource":null})),
            mutation_call(&["orchestration".into(),"worker-abandon".into(),"--dispatch".into(),"dispatch-1".into(),"--retry-request".into(),abandon_request.0,"--json".into()],0,json!({"dispatchId":"dispatch-1","state":"abandoned","alreadySettled":false,"stale":false,"processAction":"not_proven_stopped","residualResources":[{"kind":"terminal","role":"agent","action":"retained","id":"terminal-worker","surface":"background"}]}),"worker_abandon"),
            read_call(&["orchestration","worker-show","--dispatch","dispatch-1","--json"],json!({"dispatch":{"id":"dispatch-1","run_id":"run-1","task_id":"task-1"},"worker":{"dispatch_id":"dispatch-1","runtime_epoch":"runtime-fixture","state":"abandoned","stage":"abandoned","worktree_id":"worktree-1","agent_terminal_handle":"terminal-worker","effects":[],"residualResources":[],"startOptions":{"resolvedWorktreeId":"worktree-1","agent":"codex"}},"terminal":null,"observation":{"status":"missing","exactWorker":true},"terminalResource":null})),
        ];
        let (adapter, store, _) = adapter_with_calls(command_id, calls);
        let execution = adapter.cancel(CancelLinkedRunRequest { context:context(command_id,"cancel-attempt"), target_jcode_attempt_id:JcodeRunId("attempt-1".into()), orca_run_id:OrcaRunId("run-1".into()), orca_task_id:OrcaTaskId("task-1".into()), target_dispatch_id:OrcaDispatchId("dispatch-1".into()) }).await;
        assert_eq!(execution.state, jcode_command_center::CommandState::Completed);
        let receipt = match execution.payload.unwrap() { jcode_command_center::CommandResultPayload::RunAccepted { receipt:Some(receipt), .. } => receipt, _ => panic!("receipt") };
        assert_eq!(receipt.outcome, OrcaMutationOutcome::Abandoned);
        let record = store.get_by_command(&CommandId(command_id.into())).unwrap().unwrap();
        assert!(record.receipts.iter().any(|receipt| receipt.status == "stop_unknown"));
        assert!(record.receipts.iter().any(|receipt| receipt.status == "abandoned"));
        assert!(!record.requests.iter().any(|request| request.stage == "worker_release"));
    });
}

#[test]
fn reconciliation_observes_known_dispatch_without_starting_another_worker() {
    tokio::runtime::Runtime::new().unwrap().block_on(async {
        let command_id = "command-reconcile";
        let store = Arc::new(SqliteOrcaOperationStore::open_in_memory().unwrap());
        let request = StartInitiativeRunRequest { context:context(command_id,"attempt-1"), objective:"Recover".into(), task_spec:"{}".into(), placement:placement() };
        let operation = jcode_command_center::orca_operation_store::NewOrcaOperation::from_typed_request(
            "start:initiative-1",
            jcode_command_center::orca_operation_store::OrcaTypedOperationRequest::StartInitiativeRun(request.clone()),
            chrono::Utc::now(),
        ).unwrap();
        store.begin(operation).unwrap();
        store.update(&request.context.command_id, jcode_command_center::orca_operation_store::OrcaOperationUpdate {
            state:Some(jcode_command_center::orca_operation_store::OrcaOperationState::OutcomeUnknown),
            orca_run_id:Some(OrcaRunId("run-1".into())), orca_task_id:Some(OrcaTaskId("task-1".into())), orca_dispatch_id:Some(OrcaDispatchId("dispatch-1".into())),
            orca_project_id:Some(placement().project_id), orca_repository_id:Some(placement().repository_id), orca_host_setup_id:Some(placement().host_setup_id), orca_host_id:Some(placement().host_id), orca_worktree_id:Some(placement().worktree_id),
            updated_at:chrono::Utc::now(), ..Default::default()
        }).unwrap();
        let directory = tempfile::tempdir().unwrap();
        let working_dir = directory.path().canonicalize().unwrap();
        let runner = Arc::new(ScriptedRunner { calls:Mutex::new(vec![read_call(&["orchestration","worker-show","--dispatch","dispatch-1","--json"],json!({"dispatch":{"id":"dispatch-1","run_id":"run-1","task_id":"task-1"},"worker":{"dispatch_id":"dispatch-1","runtime_epoch":"runtime-fixture","state":"ready","stage":"ready","worktree_id":"worktree-1","agent_terminal_handle":"terminal-worker","effects":[],"residualResources":[],"startOptions":{"resolvedWorktreeId":"worktree-1","agent":"codex"}},"terminal":null,"observation":{"status":"missing","exactWorker":true},"terminalResource":null}))].into()), store:Arc::clone(&store), command_id:CommandId(command_id.into()), expected_dir:working_dir.clone() });
        let adapter = OrcaLifecycleAdapter::new(OrcaLifecycleConfig { command:"orca".into(), working_dir:Some(working_dir), runner, store:Arc::clone(&store), coordinator:OrcaCoordinatorBinding { terminal:OrcaTerminalId("terminal-coordinator".into()) }, launcher:placement().launcher, timeout:Duration::from_secs(5) });
        let ReconciliationSummary { examined, ready, .. } = adapter.reconcile_pending_operations().await.unwrap();
        assert_eq!((examined, ready), (1,1));
    });
}

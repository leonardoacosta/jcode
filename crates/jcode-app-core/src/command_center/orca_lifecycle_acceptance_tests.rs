//! Process-boundary acceptance for the Orca lifecycle adapter.
//!
//! The suite compiles a repository-owned fake Orca executable and gives every
//! scenario its own worktree directory and SQLite store. No installed Orca
//! binary, daemon, terminal, worktree, or other shared resource is touched.

use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::{Arc, OnceLock};
use std::time::Duration;

use jcode_command_center::orca_operation_store::{
    OrcaOperationRecord, OrcaOperationState, SqliteOrcaOperationStore,
};
use jcode_command_center::{
    CancelLinkedRunRequest, CleanupResourceState, CommandId, CommandResultPayload, CommandState,
    CorrelationId, IdempotencyKey, InitiativeId, JcodeRunId, OrcaAdapter, OrcaCanonicalPlacement,
    OrcaDispatchId, OrcaHostId, OrcaHostSetupId, OrcaLifecycleReceipt, OrcaMutationContext,
    OrcaMutationOutcome, OrcaProjectId, OrcaRepositoryId, OrcaRunId, OrcaTaskId, OrcaTerminalId,
    OrcaWorkerLauncher, OrcaWorktreeId, RetryLinkedRunRequest, RuntimeCommandExecution,
    StartInitiativeRunRequest,
};

use super::orca_lifecycle::{OrcaCoordinatorBinding, OrcaLifecycleAdapter, OrcaLifecycleConfig};
use super::{OrcaCliAdapter, OrcaCommandRunner, ProcessOrcaCommandRunner};

const RUN_ID: &str = "run-1";
const TASK_ID: &str = "task-1";
const PRIOR_DISPATCH_ID: &str = "dispatch-prior";
const CANCEL_DISPATCH_ID: &str = "dispatch-cancel";

struct CompiledFakeOrca {
    _directory: tempfile::TempDir,
    path: PathBuf,
}

struct AcceptanceHarness {
    _directory: tempfile::TempDir,
    working_dir: PathBuf,
    store_path: PathBuf,
}

impl AcceptanceHarness {
    fn new(scenario: &str) -> Self {
        let directory = tempfile::tempdir().expect("acceptance temp directory");
        let working_dir = directory.path().join("worktree");
        std::fs::create_dir(&working_dir).expect("create fake worktree");
        std::fs::write(working_dir.join(".fake-orca-scenario"), scenario)
            .expect("write fake Orca scenario");
        let store_path = directory.path().join("orca-operations.sqlite");
        Self {
            _directory: directory,
            working_dir,
            store_path,
        }
    }

    fn open_adapter(&self) -> (OrcaCliAdapter, Arc<SqliteOrcaOperationStore>) {
        let store = Arc::new(
            SqliteOrcaOperationStore::open(&self.store_path).expect("open acceptance store"),
        );
        let runner: Arc<dyn OrcaCommandRunner> = Arc::new(ProcessOrcaCommandRunner);
        let command = fake_orca_path().to_string_lossy().into_owned();
        let lifecycle = Arc::new(OrcaLifecycleAdapter::new(OrcaLifecycleConfig {
            command: command.clone(),
            working_dir: Some(self.working_dir.clone()),
            runner: Arc::clone(&runner),
            store: Arc::clone(&store),
            coordinator: OrcaCoordinatorBinding {
                terminal: OrcaTerminalId("terminal-coordinator".into()),
            },
            launcher: OrcaWorkerLauncher::Agent {
                agent: "codex".into(),
                model: None,
                effort: None,
            },
            timeout: Duration::from_secs(5),
        }));
        (
            OrcaCliAdapter {
                command,
                working_dir: Some(self.working_dir.clone()),
                runner,
                lifecycle: Some(lifecycle),
            },
            store,
        )
    }

    fn log(&self) -> Vec<String> {
        std::fs::read_to_string(self.working_dir.join(".fake-orca-log"))
            .unwrap_or_default()
            .lines()
            .map(str::to_owned)
            .collect()
    }
}

fn fake_orca_path() -> &'static Path {
    static FAKE: OnceLock<CompiledFakeOrca> = OnceLock::new();
    &FAKE
        .get_or_init(|| {
            let directory = tempfile::tempdir().expect("fake Orca build directory");
            let source = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("tests/fixtures/orca-acceptance/fake_orca.rs");
            let path = directory
                .path()
                .join(format!("orca-fake{}", std::env::consts::EXE_SUFFIX));
            let rustc = std::env::var_os("RUSTC").unwrap_or_else(|| "rustc".into());
            let output = Command::new(rustc)
                .args(["--edition=2024", "-o"])
                .arg(&path)
                .arg(&source)
                .output()
                .expect("compile repository-owned fake Orca executable");
            assert!(
                output.status.success(),
                "fake Orca compilation failed:\n{}",
                String::from_utf8_lossy(&output.stderr)
            );
            CompiledFakeOrca {
                _directory: directory,
                path,
            }
        })
        .path
}

fn context(command: &str, attempt: &str) -> OrcaMutationContext {
    OrcaMutationContext {
        command_id: CommandId(command.into()),
        idempotency_key: IdempotencyKey(format!("key-{command}")),
        correlation_id: CorrelationId(format!("correlation-{command}")),
        initiative_id: InitiativeId("initiative-acceptance".into()),
        jcode_attempt_id: JcodeRunId(attempt.into()),
    }
}

fn placement() -> OrcaCanonicalPlacement {
    OrcaCanonicalPlacement {
        project_id: OrcaProjectId("project-1".into()),
        repository_id: OrcaRepositoryId("repo-1".into()),
        host_setup_id: OrcaHostSetupId("setup-1".into()),
        host_id: OrcaHostId("host-1".into()),
        worktree_id: OrcaWorktreeId("worktree-1".into()),
        worktree_selector: "id:worktree-1".into(),
        coordinator_terminal_id: OrcaTerminalId("terminal-coordinator".into()),
        environment: None,
        launcher: OrcaWorkerLauncher::Agent {
            agent: "codex".into(),
            model: None,
            effort: None,
        },
    }
}

fn start_request(command: &str) -> StartInitiativeRunRequest {
    StartInitiativeRunRequest {
        context: context(command, &format!("attempt-{command}")),
        objective: "Exercise isolated lifecycle acceptance".into(),
        task_spec: "{}".into(),
        placement: placement(),
    }
}

fn retry_request(command: &str) -> RetryLinkedRunRequest {
    RetryLinkedRunRequest {
        context: context(command, &format!("attempt-{command}")),
        prior_jcode_attempt_id: JcodeRunId("attempt-prior".into()),
        orca_run_id: OrcaRunId(RUN_ID.into()),
        orca_task_id: OrcaTaskId(TASK_ID.into()),
        retry_of_dispatch_id: OrcaDispatchId(PRIOR_DISPATCH_ID.into()),
        placement: placement(),
    }
}

fn cancel_request(command: &str) -> CancelLinkedRunRequest {
    CancelLinkedRunRequest {
        context: context(command, &format!("attempt-{command}")),
        target_jcode_attempt_id: JcodeRunId("attempt-target".into()),
        orca_run_id: OrcaRunId(RUN_ID.into()),
        orca_task_id: OrcaTaskId(TASK_ID.into()),
        target_dispatch_id: OrcaDispatchId(CANCEL_DISPATCH_ID.into()),
    }
}

fn receipt(execution: &RuntimeCommandExecution) -> &OrcaLifecycleReceipt {
    match execution.payload.as_ref() {
        Some(CommandResultPayload::RunAccepted {
            receipt: Some(receipt),
            ..
        }) => receipt,
        other => panic!("expected lifecycle receipt, got {other:?}"),
    }
}

fn mutation_count(log: &[String], command: &str) -> usize {
    log.iter()
        .filter(|line| line.starts_with(&format!("orchestration\t{command}\t")))
        .count()
}

fn assert_persisted_request_ids_were_invoked(record: &OrcaOperationRecord, log: &[String]) {
    for request in &record.requests {
        let request_id = request
            .orca_request_id
            .as_ref()
            .expect("mutation request has a stable Orca request ID");
        assert!(
            log.iter()
                .any(|line| { line.contains(&format!("\t--retry-request\t{}\t", request_id.0)) }),
            "persisted request {} was not observed at the process boundary",
            request_id.0
        );
    }
}

#[tokio::test]
async fn process_acceptance_does_not_enable_production_mutation_capabilities() {
    let harness = AcceptanceHarness::new("ready");
    let (adapter, _store) = harness.open_adapter();

    let capabilities = OrcaAdapter::capabilities(&adapter)
        .await
        .expect("read mutation capabilities");
    assert!(!capabilities.start_initiative_run);
    assert!(!capabilities.retry_linked_run);
    assert!(!capabilities.cancel_linked_run);
    assert!(harness.log().is_empty());
}

#[tokio::test]
async fn process_acceptance_ready_and_duplicate_start_are_isolated_and_idempotent() {
    let harness = AcceptanceHarness::new("ready");
    let (adapter, store) = harness.open_adapter();
    let request = start_request("accept-ready");

    let first = OrcaAdapter::start_initiative_run(&adapter, request.clone()).await;
    assert_eq!(first.state, CommandState::Pending);
    assert_eq!(receipt(&first).outcome, OrcaMutationOutcome::Ready);
    let record = store
        .get_by_command(&request.context.command_id)
        .unwrap()
        .unwrap();
    assert_eq!(record.state, OrcaOperationState::Ready);
    assert_eq!(record.requests.len(), 3);
    assert!(
        record
            .requests
            .iter()
            .all(|request| request.orca_request_id.is_some())
    );
    let before_duplicate = harness.log();
    assert_persisted_request_ids_were_invoked(&record, &before_duplicate);

    let duplicate = OrcaAdapter::start_initiative_run(&adapter, request).await;
    assert_eq!(receipt(&duplicate).outcome, OrcaMutationOutcome::Ready);
    assert_eq!(harness.log(), before_duplicate);
    assert_eq!(mutation_count(&before_duplicate, "worker-start"), 1);
}

#[tokio::test]
async fn process_acceptance_concurrent_duplicate_starts_create_one_dispatch() {
    let harness = AcceptanceHarness::new("ready");
    let (adapter, store) = harness.open_adapter();
    let adapter = Arc::new(adapter);
    let request = start_request("accept-concurrent-duplicate");

    let (first, second) = tokio::join!(
        OrcaAdapter::start_initiative_run(adapter.as_ref(), request.clone()),
        OrcaAdapter::start_initiative_run(adapter.as_ref(), request.clone()),
    );
    assert_eq!(receipt(&first).outcome, OrcaMutationOutcome::Ready);
    assert_eq!(receipt(&second).outcome, OrcaMutationOutcome::Ready);
    assert_eq!(store.len().unwrap(), 1);
    assert_eq!(mutation_count(&harness.log(), "worker-start"), 1);
}

#[tokio::test]
async fn process_acceptance_preserves_failed_and_outcome_unknown_receipts() {
    for (scenario, command, expected, state) in [
        (
            "failed",
            "accept-failed",
            OrcaMutationOutcome::Failed,
            CommandState::Failed,
        ),
        (
            "outcome_unknown",
            "accept-unknown",
            OrcaMutationOutcome::OutcomeUnknown,
            CommandState::Pending,
        ),
    ] {
        let harness = AcceptanceHarness::new(scenario);
        let (adapter, store) = harness.open_adapter();
        let request = start_request(command);
        let result = OrcaAdapter::start_initiative_run(&adapter, request.clone()).await;
        assert_eq!(result.state, state, "scenario {scenario}");
        assert_eq!(receipt(&result).outcome, expected, "scenario {scenario}");
        let record = store
            .get_by_command(&request.context.command_id)
            .unwrap()
            .unwrap();
        assert_eq!(record.receipts.last().unwrap().status, scenario);
        assert_eq!(mutation_count(&harness.log(), "worker-start"), 1);
    }
}

#[tokio::test]
async fn process_acceptance_rejects_non_retryable_dispatch_without_mutation() {
    let harness = AcceptanceHarness::new("rejected");
    let (adapter, _store) = harness.open_adapter();
    let request = retry_request("accept-rejected");

    let result = OrcaAdapter::retry_linked_run(&adapter, request.clone()).await;
    assert_eq!(result.state, CommandState::Failed);
    assert_eq!(receipt(&result).outcome, OrcaMutationOutcome::Rejected);
    let before_duplicate = harness.log();
    let duplicate = OrcaAdapter::retry_linked_run(&adapter, request).await;
    assert_eq!(receipt(&duplicate).outcome, OrcaMutationOutcome::Rejected);
    assert_eq!(harness.log(), before_duplicate);
    assert_eq!(mutation_count(&before_duplicate, "worker-start"), 0);
}

#[tokio::test]
async fn process_acceptance_retry_targets_exact_dispatch_and_creates_replacement() {
    let harness = AcceptanceHarness::new("retry");
    let (adapter, store) = harness.open_adapter();
    let request = retry_request("accept-retry");

    let result = OrcaAdapter::retry_linked_run(&adapter, request.clone()).await;
    assert_eq!(receipt(&result).outcome, OrcaMutationOutcome::Ready);
    let attempt = receipt(&result).attempt.as_ref().unwrap();
    assert_eq!(attempt.dispatch_id, OrcaDispatchId("dispatch-retry".into()));
    assert_eq!(
        attempt.retry_of_dispatch_id,
        Some(OrcaDispatchId(PRIOR_DISPATCH_ID.into()))
    );
    let record = store
        .get_by_command(&request.context.command_id)
        .unwrap()
        .unwrap();
    assert_eq!(
        record.orca_dispatch_id,
        Some(OrcaDispatchId("dispatch-retry".into()))
    );
    let log = harness.log();
    assert_persisted_request_ids_were_invoked(&record, &log);
    assert!(log.iter().any(|line| {
        line.contains("orchestration\tworker-start\t")
            && line.contains("\t--retry-of\tdispatch-prior\t")
    }));
    assert!(
        log.iter()
            .any(|line| { line.contains("orchestration\trun-use\t--id\trun-1\t") })
    );
}

#[tokio::test]
async fn process_acceptance_stop_releases_exact_terminal_and_duplicate_cancel_is_noop() {
    let harness = AcceptanceHarness::new("stop_released");
    let (adapter, store) = harness.open_adapter();
    let request = cancel_request("accept-stop");

    let result = OrcaAdapter::cancel_linked_run(&adapter, request.clone()).await;
    assert_eq!(result.state, CommandState::Completed);
    assert_eq!(receipt(&result).outcome, OrcaMutationOutcome::Stopped);
    assert_eq!(receipt(&result).stage, "worker_release");
    assert!(
        receipt(&result)
            .cleanup
            .iter()
            .all(|item| item.state == CleanupResourceState::VerifiedReleased)
    );
    let before_duplicate = harness.log();
    let record = store
        .get_by_command(&request.context.command_id)
        .unwrap()
        .unwrap();
    assert_persisted_request_ids_were_invoked(&record, &before_duplicate);
    let duplicate = OrcaAdapter::cancel_linked_run(&adapter, request).await;
    assert_eq!(receipt(&duplicate).outcome, OrcaMutationOutcome::Stopped);
    assert_eq!(harness.log(), before_duplicate);
    assert_eq!(mutation_count(&before_duplicate, "worker-stop"), 1);
    assert_eq!(mutation_count(&before_duplicate, "worker-release"), 1);
    assert_eq!(mutation_count(&before_duplicate, "worker-abandon"), 0);
    assert!(before_duplicate.iter().any(|line| {
        line.contains("orchestration\tworker-stop\t--dispatch\tdispatch-cancel\t")
    }));
    assert!(before_duplicate.iter().any(|line| {
        line.contains("orchestration\tworker-release\t--dispatch\tdispatch-cancel\t")
    }));
}

#[tokio::test]
async fn process_acceptance_stop_unknown_abandons_without_release() {
    let harness = AcceptanceHarness::new("abandon");
    let (adapter, store) = harness.open_adapter();
    let request = cancel_request("accept-abandon");

    let result = OrcaAdapter::cancel_linked_run(&adapter, request.clone()).await;
    assert_eq!(result.state, CommandState::Completed);
    assert_eq!(receipt(&result).outcome, OrcaMutationOutcome::Abandoned);
    assert_eq!(receipt(&result).stage, "worker_abandon");
    assert!(
        receipt(&result)
            .cleanup
            .iter()
            .any(|item| item.state == CleanupResourceState::RecoveryRequired)
    );
    let record = store
        .get_by_command(&request.context.command_id)
        .unwrap()
        .unwrap();
    assert!(
        record
            .recovery
            .iter()
            .any(|item| item.resource_kind == "terminal")
    );
    let log = harness.log();
    assert_persisted_request_ids_were_invoked(&record, &log);
    assert_eq!(mutation_count(&log, "worker-stop"), 1);
    assert_eq!(mutation_count(&log, "worker-abandon"), 1);
    assert_eq!(mutation_count(&log, "worker-release"), 0);
    assert!(log.iter().any(|line| {
        line.contains("orchestration\tworker-abandon\t--dispatch\tdispatch-cancel\t")
    }));
}

#[tokio::test]
async fn process_acceptance_release_pending_and_unknown_preserve_recovery_obligations() {
    for (scenario, command, expected_status) in [
        (
            "release_pending",
            "accept-release-pending",
            "release_pending",
        ),
        (
            "release_unknown",
            "accept-release-unknown",
            "release_unknown",
        ),
    ] {
        let harness = AcceptanceHarness::new(scenario);
        let (adapter, store) = harness.open_adapter();
        let request = cancel_request(command);
        let result = OrcaAdapter::cancel_linked_run(&adapter, request.clone()).await;
        assert_eq!(result.state, CommandState::Completed, "scenario {scenario}");
        assert_eq!(
            receipt(&result).outcome,
            OrcaMutationOutcome::Stopped,
            "scenario {scenario}"
        );
        assert!(
            receipt(&result)
                .cleanup
                .iter()
                .any(|item| item.state == CleanupResourceState::RecoveryRequired)
        );
        let record = store
            .get_by_command(&request.context.command_id)
            .unwrap()
            .unwrap();
        assert_eq!(record.state, OrcaOperationState::Completed);
        assert_eq!(record.receipts.last().unwrap().status, expected_status);
        assert!(
            record
                .recovery
                .iter()
                .any(|item| item.action == "retry_worker_release_with_same_request")
        );
        let log = harness.log();
        assert_persisted_request_ids_were_invoked(&record, &log);
        assert_eq!(mutation_count(&log, "worker-release"), 1);
        assert!(log.iter().any(|line| {
            line.contains("orchestration\tworker-release\t--dispatch\tdispatch-cancel\t")
        }));
        assert!(!log.iter().any(|line| line.starts_with("terminal\tclose\t")));
    }
}

#[tokio::test]
async fn process_acceptance_daemon_restart_reconciles_without_second_start() {
    let harness = AcceptanceHarness::new("restart");
    let request = start_request("accept-restart");
    let (adapter, store) = harness.open_adapter();
    let first = OrcaAdapter::start_initiative_run(&adapter, request.clone()).await;
    assert_eq!(receipt(&first).outcome, OrcaMutationOutcome::OutcomeUnknown);
    assert_eq!(
        store
            .get_by_command(&request.context.command_id)
            .unwrap()
            .unwrap()
            .state,
        OrcaOperationState::OutcomeUnknown
    );
    drop(adapter);
    drop(store);

    let (restarted, reopened_store) = harness.open_adapter();
    restarted
        .reconcile_pending_operations()
        .await
        .expect("restart reconciliation");
    assert_eq!(
        reopened_store
            .get_by_command(&request.context.command_id)
            .unwrap()
            .unwrap()
            .state,
        OrcaOperationState::Ready
    );
    let log = harness.log();
    assert_eq!(mutation_count(&log, "worker-start"), 1);
    assert_eq!(mutation_count(&log, "worker-show"), 2);
}

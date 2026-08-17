use std::collections::VecDeque;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use async_trait::async_trait;
use chrono::Utc;
use jcode_command_center::OrcaAdapter;
use jcode_command_center::orca_operation_store::{
    NewOrcaOperation, OrcaOperationKind, SqliteOrcaOperationStore,
};
use jcode_command_center::{
    CommandId, CorrelationId, IdempotencyKey, InitiativeId, JcodeRunId, OrcaTerminalId,
    OrcaWorkerLauncher, RuntimeMutationCapabilities,
};
use serde_json::{Value, json};

use crate::command_center_orca::OrcaCompatibilityProfile;

use super::orca_lifecycle::{OrcaCoordinatorBinding, OrcaLifecycleAdapter, OrcaLifecycleConfig};
use super::{OrcaCliAdapter, OrcaCommandOutput, OrcaCommandRunner, OrcaProcessError};

#[derive(Debug)]
struct ScriptedCall {
    args: Vec<String>,
    output: Result<OrcaCommandOutput, OrcaProcessError>,
}

#[derive(Debug)]
struct ScriptedRunner {
    calls: Mutex<VecDeque<ScriptedCall>>,
    working_dir: PathBuf,
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
        assert_eq!(current_dir, Some(self.working_dir.as_path()));
        let call = self
            .calls
            .lock()
            .expect("scripted runner lock")
            .pop_front()
            .expect("unexpected Orca readiness command");
        assert_eq!(args, call.args);
        call.output
    }
}

fn success(args: &[&str], response: Value) -> ScriptedCall {
    ScriptedCall {
        args: args.iter().map(|value| (*value).to_string()).collect(),
        output: Ok(OrcaCommandOutput {
            exit_code: Some(0),
            stdout: serde_json::to_vec(&response).expect("serialize scripted response"),
            stderr: Vec::new(),
        }),
    }
}

fn failure(args: &[&str], error: OrcaProcessError) -> ScriptedCall {
    ScriptedCall {
        args: args.iter().map(|value| (*value).to_string()).collect(),
        output: Err(error),
    }
}

fn envelope(result: Value) -> Value {
    json!({
        "id": "readiness-response",
        "ok": true,
        "result": result,
        "_meta": { "runtimeId": "runtime-readiness" }
    })
}

fn ready_calls(working_dir: &Path) -> Vec<ScriptedCall> {
    let profile = OrcaCompatibilityProfile::pinned().expect("load pinned Orca profile");
    vec![
        success(
            &["status", "--json"],
            profile
                .response_fixture("status.ready")
                .expect("status fixture")
                .clone(),
        ),
        success(
            &["agent-context", "--json"],
            profile.command_registry_fixture().clone(),
        ),
        success(
            &["worktree", "current", "--json"],
            envelope(json!({
                "worktree": {
                    "id": "worktree-1",
                    "repoId": "repo-1",
                    "hostId": "host-1",
                    "path": working_dir,
                }
            })),
        ),
        success(
            &["repo", "list", "--json"],
            envelope(json!({
                "repos": [{"id": "repo-1", "path": working_dir}]
            })),
        ),
        success(
            &["project", "setups", "--json"],
            envelope(json!({
                "setups": [{
                    "id": "setup-1",
                    "projectId": "project-1",
                    "repoId": "repo-1",
                    "hostId": "host-1",
                    "setupState": "ready",
                }]
            })),
        ),
        success(
            &["project", "list", "--json"],
            envelope(json!({
                "projects": [{"id": "project-1", "sourceRepoIds": ["repo-1"]}]
            })),
        ),
        success(
            &[
                "terminal",
                "show",
                "--terminal",
                "terminal-coordinator",
                "--json",
            ],
            envelope(json!({
                "terminal": {"handle": "terminal-coordinator"}
            })),
        ),
    ]
}

fn adapter_with_calls(
    working_dir: PathBuf,
    calls: Vec<ScriptedCall>,
) -> (OrcaCliAdapter, Arc<SqliteOrcaOperationStore>) {
    let store = Arc::new(SqliteOrcaOperationStore::open_in_memory().expect("operation store"));
    let runner: Arc<dyn OrcaCommandRunner> = Arc::new(ScriptedRunner {
        calls: Mutex::new(calls.into()),
        working_dir: working_dir.clone(),
    });
    let lifecycle = Arc::new(OrcaLifecycleAdapter::new(OrcaLifecycleConfig {
        command: "orca".into(),
        working_dir: Some(working_dir.clone()),
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
        timeout: Duration::from_secs(1),
    }));
    (
        OrcaCliAdapter {
            command: "orca".into(),
            working_dir: Some(working_dir),
            runner,
            lifecycle: Some(lifecycle),
        },
        store,
    )
}

fn assert_unavailable(capabilities: RuntimeMutationCapabilities) {
    assert!(!capabilities.start_initiative_run);
    assert!(!capabilities.retry_linked_run);
    assert!(!capabilities.cancel_linked_run);
}

async fn reconciled_capabilities(
    adapter: &OrcaCliAdapter,
) -> Result<RuntimeMutationCapabilities, jcode_command_center::CommandCenterError> {
    adapter
        .reconcile_pending_operations()
        .await
        .expect("startup reconciliation succeeds");
    OrcaAdapter::capabilities(adapter).await
}

#[tokio::test]
async fn capabilities_are_ready_only_after_reconciliation_and_read_only_runtime_validation() {
    let directory = tempfile::tempdir().expect("readiness directory");
    let working_dir = directory
        .path()
        .canonicalize()
        .expect("canonical directory");
    let (adapter, _store) = adapter_with_calls(working_dir.clone(), ready_calls(&working_dir));

    adapter
        .reconcile_pending_operations()
        .await
        .expect("empty startup reconciliation succeeds");
    let capabilities = OrcaAdapter::capabilities(&adapter)
        .await
        .expect("read capabilities");

    assert!(capabilities.start_initiative_run);
    assert!(capabilities.retry_linked_run);
    assert!(capabilities.cancel_linked_run);
}

#[tokio::test]
async fn capabilities_fail_closed_without_lifecycle_configuration() {
    let directory = tempfile::tempdir().expect("readiness directory");
    let working_dir = directory
        .path()
        .canonicalize()
        .expect("canonical directory");
    let runner: Arc<dyn OrcaCommandRunner> = Arc::new(ScriptedRunner {
        calls: Mutex::new(VecDeque::new()),
        working_dir: working_dir.clone(),
    });
    let adapter = OrcaCliAdapter {
        command: "orca".into(),
        working_dir: Some(working_dir),
        runner,
        lifecycle: None,
    };

    assert_unavailable(OrcaAdapter::capabilities(&adapter).await.unwrap());
}

#[tokio::test]
async fn capabilities_fail_closed_before_startup_reconciliation() {
    let directory = tempfile::tempdir().expect("readiness directory");
    let working_dir = directory
        .path()
        .canonicalize()
        .expect("canonical directory");
    let (adapter, _store) = adapter_with_calls(working_dir.clone(), ready_calls(&working_dir));

    assert_unavailable(OrcaAdapter::capabilities(&adapter).await.unwrap());
}

#[tokio::test]
async fn capabilities_fail_closed_for_status_or_registry_profile_drift() {
    for drifted_call in [0, 1] {
        let directory = tempfile::tempdir().expect("readiness directory");
        let working_dir = directory
            .path()
            .canonicalize()
            .expect("canonical directory");
        let mut calls = ready_calls(&working_dir);
        if drifted_call == 0 {
            calls[0] = success(
                &["status", "--json"],
                json!({"unexpectedStatusSchema": true}),
            );
        } else {
            let profile = OrcaCompatibilityProfile::pinned().expect("pinned profile");
            let mut registry = profile.command_registry_fixture().clone();
            registry["futureField"] = json!(true);
            calls[1] = success(&["agent-context", "--json"], registry);
        }
        let (adapter, _store) = adapter_with_calls(working_dir, calls);

        assert_unavailable(reconciled_capabilities(&adapter).await.unwrap());
    }
}

#[tokio::test]
async fn capabilities_fail_closed_on_runtime_probe_error_or_nonzero_exit() {
    for nonzero in [false, true] {
        let directory = tempfile::tempdir().expect("readiness directory");
        let working_dir = directory
            .path()
            .canonicalize()
            .expect("canonical directory");
        let mut calls = ready_calls(&working_dir);
        calls[0] = if nonzero {
            ScriptedCall {
                args: vec!["status".into(), "--json".into()],
                output: Ok(OrcaCommandOutput {
                    exit_code: Some(1),
                    stdout: serde_json::to_vec(
                        OrcaCompatibilityProfile::pinned()
                            .unwrap()
                            .response_fixture("status.ready")
                            .unwrap(),
                    )
                    .unwrap(),
                    stderr: b"status failed".to_vec(),
                }),
            }
        } else {
            failure(&["status", "--json"], OrcaProcessError::Timeout)
        };
        if !nonzero {
            calls.remove(1);
        }
        let (adapter, _store) = adapter_with_calls(working_dir, calls);

        assert_unavailable(reconciled_capabilities(&adapter).await.unwrap());
    }
}

#[tokio::test]
async fn capabilities_fail_closed_when_canonical_placement_does_not_resolve() {
    let directory = tempfile::tempdir().expect("readiness directory");
    let working_dir = directory
        .path()
        .canonicalize()
        .expect("canonical directory");
    let mut calls = ready_calls(&working_dir);
    calls[2] = success(
        &["worktree", "current", "--json"],
        envelope(json!({"worktree": {"path": working_dir}})),
    );
    calls.truncate(3);
    let (adapter, _store) = adapter_with_calls(working_dir, calls);

    assert_unavailable(reconciled_capabilities(&adapter).await.unwrap());
}

#[tokio::test]
async fn capabilities_fail_closed_when_coordinator_terminal_is_not_verified() {
    let directory = tempfile::tempdir().expect("readiness directory");
    let working_dir = directory
        .path()
        .canonicalize()
        .expect("canonical directory");
    let mut calls = ready_calls(&working_dir);
    calls[6] = success(
        &[
            "terminal",
            "show",
            "--terminal",
            "terminal-coordinator",
            "--json",
        ],
        envelope(json!({"terminal": {"handle": "terminal-other"}})),
    );
    let (adapter, _store) = adapter_with_calls(working_dir, calls);

    assert_unavailable(reconciled_capabilities(&adapter).await.unwrap());
}

#[tokio::test]
async fn capabilities_fail_closed_while_the_store_has_unresolved_operations() {
    let directory = tempfile::tempdir().expect("readiness directory");
    let working_dir = directory
        .path()
        .canonicalize()
        .expect("canonical directory");
    let (adapter, store) = adapter_with_calls(working_dir.clone(), ready_calls(&working_dir));
    adapter
        .reconcile_pending_operations()
        .await
        .expect("initial reconciliation succeeds");
    store
        .begin(NewOrcaOperation {
            command_id: CommandId("readiness-pending".into()),
            idempotency_scope: "readiness".into(),
            idempotency_key: IdempotencyKey("readiness-pending".into()),
            correlation_id: CorrelationId("readiness-correlation".into()),
            initiative_id: InitiativeId("readiness-initiative".into()),
            jcode_run_id: Some(JcodeRunId("readiness-run".into())),
            kind: OrcaOperationKind::StartInitiativeRun,
            command_payload: json!({}),
            created_at: Utc::now(),
        })
        .expect("seed unresolved operation");

    assert_unavailable(OrcaAdapter::capabilities(&adapter).await.unwrap());
}

use chrono::{TimeZone, Utc};
use jcode_command_center::orca_operation_store::{
    BeginOrcaOperation, NewOrcaOperation, OrcaOperationKind, OrcaOperationState,
    OrcaOperationStoreError, OrcaOperationUpdate, OrcaPartialEffect, OrcaReceiptRecord,
    OrcaRecoveryObligation, OrcaRecoveryState, OrcaRequestRecord, SqliteOrcaOperationStore,
};
use jcode_command_center::{
    CommandId, IdempotencyKey, InitiativeId, JcodeRunId, OrcaDispatchId, OrcaProjectId, OrcaRunId,
    OrcaTaskId, OrcaTerminalId, OrcaWorktreeId,
};
use serde_json::json;
use tempfile::tempdir;

fn at(second: u32) -> chrono::DateTime<Utc> {
    Utc.with_ymd_and_hms(2026, 8, 17, 13, 0, second).unwrap()
}

fn operation(command_id: &str, key: &str) -> NewOrcaOperation {
    NewOrcaOperation {
        command_id: CommandId(command_id.into()),
        idempotency_scope: "browser-session-1".into(),
        idempotency_key: IdempotencyKey(key.into()),
        correlation_id: "corr-1".into(),
        initiative_id: InitiativeId("initiative-1".into()),
        jcode_run_id: Some(JcodeRunId("jcode-run-1".into())),
        kind: OrcaOperationKind::StartInitiativeRun,
        command_payload: json!({"type": "start_initiative_run", "initiative_id": "initiative-1"}),
        created_at: at(0),
    }
}

#[test]
fn complete_operation_evidence_survives_close_and_reopen() {
    let directory = tempdir().unwrap();
    let path = directory.path().join("orca-operations.sqlite");
    let store = SqliteOrcaOperationStore::open(&path).unwrap();
    assert!(matches!(
        store.begin(operation("command-1", "key-1")).unwrap(),
        BeginOrcaOperation::Created(_)
    ));

    let updated = store
        .update(
            &CommandId("command-1".into()),
            OrcaOperationUpdate {
                state: Some(OrcaOperationState::RecoveryRequired),
                orca_run_id: Some(OrcaRunId("run-1".into())),
                orca_task_id: Some(OrcaTaskId("task-1".into())),
                orca_dispatch_id: Some(OrcaDispatchId("dispatch-1".into())),
                orca_project_id: Some(OrcaProjectId("project-1".into())),
                orca_repository_id: Some("repository-1".into()),
                orca_host_setup_id: Some("host-setup-1".into()),
                orca_worktree_id: Some(OrcaWorktreeId("worktree-1".into())),
                orca_terminal_id: Some(OrcaTerminalId("terminal-1".into())),
                requests: vec![OrcaRequestRecord {
                    id: "request-step-1".into(),
                    stage: "worker_start".into(),
                    orca_request_id: Some("orca-request-1".into()),
                    command: "worker-start".into(),
                    arguments: vec!["--task".into(), "task-1".into()],
                    payload: json!({"placement": {"worktreeId": "worktree-1"}}),
                    recorded_at: at(1),
                }],
                receipts: vec![OrcaReceiptRecord {
                    id: "receipt-1".into(),
                    request_id: "request-step-1".into(),
                    status: "outcome_unknown".into(),
                    payload: json!({"ok": false, "error": {"kind": "outcome_unknown"}}),
                    observed_at: at(2),
                }],
                partial_effects: vec![OrcaPartialEffect {
                    id: "effect-1".into(),
                    stage: "worker_start".into(),
                    resource_kind: "terminal".into(),
                    resource_id: Some("terminal-1".into()),
                    evidence: json!({"created": true, "released": false}),
                    observed_at: at(3),
                }],
                recovery: vec![OrcaRecoveryObligation {
                    id: "recovery-1".into(),
                    resource_kind: "terminal".into(),
                    resource_id: Some("terminal-1".into()),
                    action: "worker-show then release".into(),
                    state: OrcaRecoveryState::Pending,
                    evidence: Some(json!({"reason": "start outcome unknown"})),
                    updated_at: at(4),
                }],
                updated_at: at(5),
                ..Default::default()
            },
        )
        .unwrap();
    assert_eq!(updated.requests.len(), 1);
    drop(store);

    let reopened = SqliteOrcaOperationStore::open(&path).unwrap();
    let recovered = reopened
        .get_by_command(&CommandId("command-1".into()))
        .unwrap()
        .unwrap();
    assert_eq!(recovered, updated);
    assert_eq!(recovered.orca_run_id, Some(OrcaRunId("run-1".into())));
    assert_eq!(
        recovered.orca_repository_id.as_deref(),
        Some("repository-1")
    );
    assert_eq!(
        recovered.orca_host_setup_id.as_deref(),
        Some("host-setup-1")
    );
    assert_eq!(recovered.receipts[0].status, "outcome_unknown");
    assert_eq!(
        recovered.partial_effects[0].resource_id.as_deref(),
        Some("terminal-1")
    );
    assert_eq!(recovered.recovery[0].state, OrcaRecoveryState::Pending);
    assert_eq!(reopened.recovery_candidates().unwrap(), vec![recovered]);
}

#[test]
fn duplicate_idempotency_identity_returns_the_original_operation() {
    let store = SqliteOrcaOperationStore::open_in_memory().unwrap();
    assert!(store.is_empty().unwrap());
    let created = match store.begin(operation("command-1", "key-1")).unwrap() {
        BeginOrcaOperation::Created(record) => record,
        BeginOrcaOperation::Existing(_) => panic!("first operation must be created"),
    };

    let duplicate = match store.begin(operation("command-2", "key-1")).unwrap() {
        BeginOrcaOperation::Existing(record) => record,
        BeginOrcaOperation::Created(_) => {
            panic!("duplicate idempotency key created a second operation")
        }
    };

    assert_eq!(duplicate.command_id, created.command_id);
    assert!(!store.is_empty().unwrap());
    assert_eq!(store.len().unwrap(), 1);
    assert_eq!(
        store
            .get_by_idempotency("browser-session-1", &IdempotencyKey("key-1".into()))
            .unwrap(),
        Some(created)
    );
}

#[test]
fn reusing_an_idempotency_key_for_a_different_intent_fails_closed() {
    let store = SqliteOrcaOperationStore::open_in_memory().unwrap();
    store.begin(operation("command-1", "key-1")).unwrap();
    let mut conflicting = operation("command-2", "key-1");
    conflicting.kind = OrcaOperationKind::CancelLinkedRun;
    conflicting.command_payload = json!({"type": "cancel_linked_run", "run_id": "jcode-run-1"});

    let error = store.begin(conflicting).unwrap_err();
    assert!(matches!(
        error,
        OrcaOperationStoreError::IdentityConflict { .. }
    ));
    assert_eq!(store.len().unwrap(), 1);
}

#[test]
fn replayed_updates_are_idempotent_and_cannot_rebind_canonical_ids() {
    let store = SqliteOrcaOperationStore::open_in_memory().unwrap();
    store.begin(operation("command-1", "key-1")).unwrap();
    let update = OrcaOperationUpdate {
        orca_dispatch_id: Some(OrcaDispatchId("dispatch-1".into())),
        requests: vec![OrcaRequestRecord {
            id: "request-step-1".into(),
            stage: "worker_start".into(),
            orca_request_id: Some("orca-request-1".into()),
            command: "worker-start".into(),
            arguments: vec![],
            payload: json!({}),
            recorded_at: at(1),
        }],
        updated_at: at(2),
        ..Default::default()
    };
    store
        .update(&CommandId("command-1".into()), update.clone())
        .unwrap();
    let replayed = store
        .update(&CommandId("command-1".into()), update)
        .unwrap();
    assert_eq!(replayed.requests.len(), 1);

    let error = store
        .update(
            &CommandId("command-1".into()),
            OrcaOperationUpdate {
                orca_dispatch_id: Some(OrcaDispatchId("dispatch-2".into())),
                updated_at: at(3),
                ..Default::default()
            },
        )
        .unwrap_err();
    assert!(matches!(
        error,
        OrcaOperationStoreError::IdentityConflict { .. }
    ));
    assert_eq!(
        store
            .get_by_command(&CommandId("command-1".into()))
            .unwrap()
            .unwrap()
            .orca_dispatch_id,
        Some(OrcaDispatchId("dispatch-1".into()))
    );
}

#[test]
fn unknown_schema_versions_fail_closed_instead_of_being_downgraded() {
    let directory = tempdir().unwrap();
    let path = directory.path().join("future.sqlite");
    let connection = rusqlite::Connection::open(&path).unwrap();
    connection.pragma_update(None, "user_version", 99).unwrap();
    drop(connection);

    let error = match SqliteOrcaOperationStore::open(&path) {
        Ok(_) => panic!("future schema must be rejected"),
        Err(error) => error,
    };
    assert!(matches!(error, OrcaOperationStoreError::InvalidData(_)));
}

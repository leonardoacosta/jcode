//! Durable crash-recovery authority for composed Orca lifecycle mutations.
//!
//! A Jcode command is inserted before the first Orca process invocation. Later
//! calls append request and receipt evidence while preserving canonical IDs.
//! The `(idempotency_scope, idempotency_key)` uniqueness constraint prevents a
//! daemon restart or browser retry from creating a second composed operation.

use std::fmt;
use std::path::Path;
use std::sync::Mutex;

use chrono::{DateTime, Utc};
use rusqlite::{Connection, OptionalExtension, Transaction, params};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

use crate::{
    CommandId, IdempotencyKey, InitiativeId, JcodeRunId, OrcaDispatchId, OrcaProjectId, OrcaRunId,
    OrcaTaskId, OrcaTerminalId, OrcaWorktreeId,
};

const SCHEMA_VERSION: i64 = 1;

#[derive(Debug)]
pub enum OrcaOperationStoreError {
    Database(rusqlite::Error),
    Serialization(serde_json::Error),
    InvalidData(String),
    IdentityConflict { field: &'static str },
    NotFound { command_id: CommandId },
    LockPoisoned,
}

impl fmt::Display for OrcaOperationStoreError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Database(_) => formatter.write_str("Orca operation database operation failed"),
            Self::Serialization(_) => {
                formatter.write_str("Orca operation evidence serialization failed")
            }
            Self::InvalidData(message) => {
                write!(formatter, "invalid Orca operation data: {message}")
            }
            Self::IdentityConflict { field } => {
                write!(formatter, "Orca operation identity conflict for {field}")
            }
            Self::NotFound { command_id } => {
                write!(
                    formatter,
                    "Orca operation not found for command {}",
                    command_id.0
                )
            }
            Self::LockPoisoned => formatter.write_str("Orca operation store lock poisoned"),
        }
    }
}

impl std::error::Error for OrcaOperationStoreError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Database(error) => Some(error),
            Self::Serialization(error) => Some(error),
            _ => None,
        }
    }
}

impl From<rusqlite::Error> for OrcaOperationStoreError {
    fn from(value: rusqlite::Error) -> Self {
        Self::Database(value)
    }
}

impl From<serde_json::Error> for OrcaOperationStoreError {
    fn from(value: serde_json::Error) -> Self {
        Self::Serialization(value)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OrcaOperationKind {
    StartInitiativeRun,
    RetryLinkedRun,
    CancelLinkedRun,
}

impl OrcaOperationKind {
    fn as_str(self) -> &'static str {
        match self {
            Self::StartInitiativeRun => "start_initiative_run",
            Self::RetryLinkedRun => "retry_linked_run",
            Self::CancelLinkedRun => "cancel_linked_run",
        }
    }

    fn parse(value: &str) -> Result<Self, OrcaOperationStoreError> {
        match value {
            "start_initiative_run" => Ok(Self::StartInitiativeRun),
            "retry_linked_run" => Ok(Self::RetryLinkedRun),
            "cancel_linked_run" => Ok(Self::CancelLinkedRun),
            _ => Err(OrcaOperationStoreError::InvalidData(format!(
                "unknown operation kind {value:?}"
            ))),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OrcaOperationState {
    Recorded,
    InProgress,
    Ready,
    Rejected,
    Failed,
    OutcomeUnknown,
    RecoveryRequired,
    Completed,
}

impl OrcaOperationState {
    fn as_str(self) -> &'static str {
        match self {
            Self::Recorded => "recorded",
            Self::InProgress => "in_progress",
            Self::Ready => "ready",
            Self::Rejected => "rejected",
            Self::Failed => "failed",
            Self::OutcomeUnknown => "outcome_unknown",
            Self::RecoveryRequired => "recovery_required",
            Self::Completed => "completed",
        }
    }

    fn parse(value: &str) -> Result<Self, OrcaOperationStoreError> {
        match value {
            "recorded" => Ok(Self::Recorded),
            "in_progress" => Ok(Self::InProgress),
            "ready" => Ok(Self::Ready),
            "rejected" => Ok(Self::Rejected),
            "failed" => Ok(Self::Failed),
            "outcome_unknown" => Ok(Self::OutcomeUnknown),
            "recovery_required" => Ok(Self::RecoveryRequired),
            "completed" => Ok(Self::Completed),
            _ => Err(OrcaOperationStoreError::InvalidData(format!(
                "unknown operation state {value:?}"
            ))),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OrcaRecoveryState {
    Pending,
    OutcomeUnknown,
    Failed,
    Verified,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OrcaRequestRecord {
    /// Jcode-owned stable step identity, recorded before process invocation.
    pub id: String,
    pub stage: String,
    /// Orca request identity when the CLI exposes one.
    pub orca_request_id: Option<String>,
    pub command: String,
    pub arguments: Vec<String>,
    pub payload: serde_json::Value,
    pub recorded_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OrcaReceiptRecord {
    pub id: String,
    pub request_id: String,
    pub status: String,
    pub payload: serde_json::Value,
    pub observed_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OrcaPartialEffect {
    pub id: String,
    pub stage: String,
    pub resource_kind: String,
    pub resource_id: Option<String>,
    pub evidence: serde_json::Value,
    pub observed_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OrcaRecoveryObligation {
    pub id: String,
    pub resource_kind: String,
    pub resource_id: Option<String>,
    pub action: String,
    pub state: OrcaRecoveryState,
    pub evidence: Option<serde_json::Value>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NewOrcaOperation {
    pub command_id: CommandId,
    pub idempotency_scope: String,
    pub idempotency_key: IdempotencyKey,
    pub correlation_id: String,
    pub initiative_id: InitiativeId,
    pub jcode_run_id: Option<JcodeRunId>,
    pub kind: OrcaOperationKind,
    pub command_payload: serde_json::Value,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OrcaOperationRecord {
    pub command_id: CommandId,
    pub idempotency_scope: String,
    pub idempotency_key: IdempotencyKey,
    pub correlation_id: String,
    pub initiative_id: InitiativeId,
    pub jcode_run_id: Option<JcodeRunId>,
    pub kind: OrcaOperationKind,
    pub state: OrcaOperationState,
    pub command_payload: serde_json::Value,
    pub requests: Vec<OrcaRequestRecord>,
    pub orca_run_id: Option<OrcaRunId>,
    pub orca_task_id: Option<OrcaTaskId>,
    pub orca_dispatch_id: Option<OrcaDispatchId>,
    pub orca_project_id: Option<OrcaProjectId>,
    pub orca_repository_id: Option<String>,
    pub orca_host_setup_id: Option<String>,
    pub orca_worktree_id: Option<OrcaWorktreeId>,
    pub orca_terminal_id: Option<OrcaTerminalId>,
    pub receipts: Vec<OrcaReceiptRecord>,
    pub partial_effects: Vec<OrcaPartialEffect>,
    pub recovery: Vec<OrcaRecoveryObligation>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<NewOrcaOperation> for OrcaOperationRecord {
    fn from(value: NewOrcaOperation) -> Self {
        Self {
            command_id: value.command_id,
            idempotency_scope: value.idempotency_scope,
            idempotency_key: value.idempotency_key,
            correlation_id: value.correlation_id,
            initiative_id: value.initiative_id,
            jcode_run_id: value.jcode_run_id,
            kind: value.kind,
            state: OrcaOperationState::Recorded,
            command_payload: value.command_payload,
            requests: Vec::new(),
            orca_run_id: None,
            orca_task_id: None,
            orca_dispatch_id: None,
            orca_project_id: None,
            orca_repository_id: None,
            orca_host_setup_id: None,
            orca_worktree_id: None,
            orca_terminal_id: None,
            receipts: Vec::new(),
            partial_effects: Vec::new(),
            recovery: Vec::new(),
            created_at: value.created_at,
            updated_at: value.created_at,
        }
    }
}

impl OrcaOperationRecord {
    fn same_intent(&self, candidate: &NewOrcaOperation) -> bool {
        self.idempotency_scope == candidate.idempotency_scope
            && self.idempotency_key == candidate.idempotency_key
            && self.initiative_id == candidate.initiative_id
            && self.jcode_run_id == candidate.jcode_run_id
            && self.kind == candidate.kind
            && self.command_payload == candidate.command_payload
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BeginOrcaOperation {
    Created(OrcaOperationRecord),
    Existing(OrcaOperationRecord),
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct OrcaOperationUpdate {
    pub state: Option<OrcaOperationState>,
    pub jcode_run_id: Option<JcodeRunId>,
    pub orca_run_id: Option<OrcaRunId>,
    pub orca_task_id: Option<OrcaTaskId>,
    pub orca_dispatch_id: Option<OrcaDispatchId>,
    pub orca_project_id: Option<OrcaProjectId>,
    pub orca_repository_id: Option<String>,
    pub orca_host_setup_id: Option<String>,
    pub orca_worktree_id: Option<OrcaWorktreeId>,
    pub orca_terminal_id: Option<OrcaTerminalId>,
    pub requests: Vec<OrcaRequestRecord>,
    pub receipts: Vec<OrcaReceiptRecord>,
    pub partial_effects: Vec<OrcaPartialEffect>,
    pub recovery: Vec<OrcaRecoveryObligation>,
    pub updated_at: DateTime<Utc>,
}

/// SQLite-backed operation authority shared by the Command Center service and
/// its Orca adapter. Every public mutation is serialized through one connection
/// and every progressive update is committed atomically.
pub struct SqliteOrcaOperationStore {
    connection: Mutex<Connection>,
}

impl SqliteOrcaOperationStore {
    pub fn open(path: impl AsRef<Path>) -> Result<Self, OrcaOperationStoreError> {
        if let Some(parent) = path
            .as_ref()
            .parent()
            .filter(|parent| !parent.as_os_str().is_empty())
        {
            std::fs::create_dir_all(parent).map_err(|error| {
                OrcaOperationStoreError::InvalidData(format!(
                    "could not create operation store directory: {error}"
                ))
            })?;
        }
        Self::from_connection(Connection::open(path)?)
    }

    pub fn open_in_memory() -> Result<Self, OrcaOperationStoreError> {
        Self::from_connection(Connection::open_in_memory()?)
    }

    fn from_connection(connection: Connection) -> Result<Self, OrcaOperationStoreError> {
        connection.pragma_update(None, "foreign_keys", true)?;
        connection.pragma_update(None, "journal_mode", "WAL")?;
        connection.busy_timeout(std::time::Duration::from_secs(5))?;
        let schema_version =
            connection.pragma_query_value(None, "user_version", |row| row.get::<_, i64>(0))?;
        match schema_version {
            0 => {
                connection.execute_batch(SCHEMA)?;
                connection.pragma_update(None, "user_version", SCHEMA_VERSION)?;
            }
            SCHEMA_VERSION => connection.execute_batch(SCHEMA)?,
            other => {
                return Err(OrcaOperationStoreError::InvalidData(format!(
                    "unsupported schema version {other}; expected {SCHEMA_VERSION}"
                )));
            }
        }
        Ok(Self {
            connection: Mutex::new(connection),
        })
    }

    pub fn begin(
        &self,
        operation: NewOrcaOperation,
    ) -> Result<BeginOrcaOperation, OrcaOperationStoreError> {
        let mut connection = self.lock()?;
        let transaction = connection.transaction()?;
        if let Some(existing) = get_by_idempotency_in(
            &transaction,
            &operation.idempotency_scope,
            &operation.idempotency_key,
        )? {
            if existing.same_intent(&operation) {
                return Ok(BeginOrcaOperation::Existing(existing));
            }
            return Err(OrcaOperationStoreError::IdentityConflict {
                field: "idempotency identity",
            });
        }
        if get_by_command_in(&transaction, &operation.command_id)?.is_some() {
            return Err(OrcaOperationStoreError::IdentityConflict {
                field: "command_id",
            });
        }

        let record = OrcaOperationRecord::from(operation);
        insert_record(&transaction, &record)?;
        transaction.commit()?;
        Ok(BeginOrcaOperation::Created(record))
    }

    pub fn update(
        &self,
        command_id: &CommandId,
        update: OrcaOperationUpdate,
    ) -> Result<OrcaOperationRecord, OrcaOperationStoreError> {
        let mut connection = self.lock()?;
        let transaction = connection.transaction()?;
        let mut record = get_by_command_in(&transaction, command_id)?.ok_or_else(|| {
            OrcaOperationStoreError::NotFound {
                command_id: command_id.clone(),
            }
        })?;

        set_once(
            &mut record.jcode_run_id,
            update.jcode_run_id,
            "jcode_run_id",
        )?;
        set_once(&mut record.orca_run_id, update.orca_run_id, "orca_run_id")?;
        set_once(
            &mut record.orca_task_id,
            update.orca_task_id,
            "orca_task_id",
        )?;
        set_once(
            &mut record.orca_dispatch_id,
            update.orca_dispatch_id,
            "orca_dispatch_id",
        )?;
        set_once(
            &mut record.orca_project_id,
            update.orca_project_id,
            "orca_project_id",
        )?;
        set_once(
            &mut record.orca_repository_id,
            update.orca_repository_id,
            "orca_repository_id",
        )?;
        set_once(
            &mut record.orca_host_setup_id,
            update.orca_host_setup_id,
            "orca_host_setup_id",
        )?;
        set_once(
            &mut record.orca_worktree_id,
            update.orca_worktree_id,
            "orca_worktree_id",
        )?;
        set_once(
            &mut record.orca_terminal_id,
            update.orca_terminal_id,
            "orca_terminal_id",
        )?;
        append_unique(
            &mut record.requests,
            update.requests,
            |item| &item.id,
            "request",
        )?;
        append_unique(
            &mut record.receipts,
            update.receipts,
            |item| &item.id,
            "receipt",
        )?;
        append_unique(
            &mut record.partial_effects,
            update.partial_effects,
            |item| &item.id,
            "partial_effect",
        )?;
        merge_recovery(&mut record.recovery, update.recovery)?;
        if let Some(state) = update.state {
            record.state = state;
        }
        if update.updated_at < record.updated_at {
            return Err(OrcaOperationStoreError::IdentityConflict {
                field: "updated_at",
            });
        }
        record.updated_at = update.updated_at;
        update_record(&transaction, &record)?;
        transaction.commit()?;
        Ok(record)
    }

    pub fn get_by_command(
        &self,
        command_id: &CommandId,
    ) -> Result<Option<OrcaOperationRecord>, OrcaOperationStoreError> {
        get_by_command_in(&*self.lock()?, command_id)
    }

    pub fn get_by_idempotency(
        &self,
        scope: &str,
        key: &IdempotencyKey,
    ) -> Result<Option<OrcaOperationRecord>, OrcaOperationStoreError> {
        get_by_idempotency_in(&*self.lock()?, scope, key)
    }

    pub fn len(&self) -> Result<usize, OrcaOperationStoreError> {
        let connection = self.lock()?;
        let count = connection.query_row("SELECT COUNT(*) FROM orca_operations", [], |row| {
            row.get::<_, i64>(0)
        })?;
        usize::try_from(count).map_err(|_| {
            OrcaOperationStoreError::InvalidData("negative operation count".to_string())
        })
    }

    pub fn is_empty(&self) -> Result<bool, OrcaOperationStoreError> {
        Ok(self.len()? == 0)
    }

    /// Operations that must be reconciled before a mutation can be replayed.
    /// Ready operations remain candidates because Orca runtime state can change
    /// while Jcode is offline. Terminal operations are returned only while they
    /// retain an unresolved resource-level recovery obligation.
    pub fn recovery_candidates(&self) -> Result<Vec<OrcaOperationRecord>, OrcaOperationStoreError> {
        let connection = self.lock()?;
        let mut statement =
            connection.prepare(&format!("{SELECT_COLUMNS} ORDER BY created_at, command_id"))?;
        let rows = statement.query_map([], StoredOperationRow::from_row)?;
        let mut candidates = Vec::new();
        for row in rows {
            let record = row?.decode()?;
            let nonterminal = !matches!(
                record.state,
                OrcaOperationState::Rejected
                    | OrcaOperationState::Failed
                    | OrcaOperationState::Completed
            );
            let unresolved_recovery = record
                .recovery
                .iter()
                .any(|obligation| obligation.state != OrcaRecoveryState::Verified);
            if nonterminal || unresolved_recovery {
                candidates.push(record);
            }
        }
        Ok(candidates)
    }

    fn lock(&self) -> Result<std::sync::MutexGuard<'_, Connection>, OrcaOperationStoreError> {
        self.connection
            .lock()
            .map_err(|_| OrcaOperationStoreError::LockPoisoned)
    }
}

fn set_once<T: PartialEq>(
    target: &mut Option<T>,
    value: Option<T>,
    field: &'static str,
) -> Result<(), OrcaOperationStoreError> {
    let Some(value) = value else {
        return Ok(());
    };
    match target {
        Some(existing) if existing != &value => {
            Err(OrcaOperationStoreError::IdentityConflict { field })
        }
        Some(_) => Ok(()),
        None => {
            *target = Some(value);
            Ok(())
        }
    }
}

fn append_unique<T: Clone + PartialEq>(
    target: &mut Vec<T>,
    incoming: Vec<T>,
    id: impl Fn(&T) -> &str,
    field: &'static str,
) -> Result<(), OrcaOperationStoreError> {
    for item in incoming {
        if let Some(existing) = target.iter().find(|existing| id(existing) == id(&item)) {
            if existing != &item {
                return Err(OrcaOperationStoreError::IdentityConflict { field });
            }
        } else {
            target.push(item);
        }
    }
    Ok(())
}

fn merge_recovery(
    target: &mut Vec<OrcaRecoveryObligation>,
    incoming: Vec<OrcaRecoveryObligation>,
) -> Result<(), OrcaOperationStoreError> {
    for item in incoming {
        if let Some(existing) = target.iter_mut().find(|existing| existing.id == item.id) {
            if existing.resource_kind != item.resource_kind
                || existing.resource_id != item.resource_id
                || existing.action != item.action
            {
                return Err(OrcaOperationStoreError::IdentityConflict { field: "recovery" });
            }
            if item.updated_at < existing.updated_at {
                return Err(OrcaOperationStoreError::IdentityConflict {
                    field: "recovery.updated_at",
                });
            }
            *existing = item;
        } else {
            target.push(item);
        }
    }
    Ok(())
}

trait QueryConnection {
    fn query_operation<P: rusqlite::Params>(
        &self,
        sql: &str,
        params: P,
    ) -> Result<Option<StoredOperationRow>, rusqlite::Error>;
}

impl QueryConnection for Connection {
    fn query_operation<P: rusqlite::Params>(
        &self,
        sql: &str,
        params: P,
    ) -> Result<Option<StoredOperationRow>, rusqlite::Error> {
        self.query_row(sql, params, StoredOperationRow::from_row)
            .optional()
    }
}

impl QueryConnection for Transaction<'_> {
    fn query_operation<P: rusqlite::Params>(
        &self,
        sql: &str,
        params: P,
    ) -> Result<Option<StoredOperationRow>, rusqlite::Error> {
        self.query_row(sql, params, StoredOperationRow::from_row)
            .optional()
    }
}

fn get_by_command_in(
    connection: &impl QueryConnection,
    command_id: &CommandId,
) -> Result<Option<OrcaOperationRecord>, OrcaOperationStoreError> {
    connection
        .query_operation(
            &format!("{SELECT_COLUMNS} WHERE command_id = ?1"),
            params![command_id.0],
        )?
        .map(StoredOperationRow::decode)
        .transpose()
}

fn get_by_idempotency_in(
    connection: &impl QueryConnection,
    scope: &str,
    key: &IdempotencyKey,
) -> Result<Option<OrcaOperationRecord>, OrcaOperationStoreError> {
    connection
        .query_operation(
            &format!("{SELECT_COLUMNS} WHERE idempotency_scope = ?1 AND idempotency_key = ?2"),
            params![scope, key.0],
        )?
        .map(StoredOperationRow::decode)
        .transpose()
}

fn insert_record(
    transaction: &Transaction<'_>,
    record: &OrcaOperationRecord,
) -> Result<(), OrcaOperationStoreError> {
    transaction.execute(
        r#"INSERT INTO orca_operations (
            command_id, idempotency_scope, idempotency_key, correlation_id, initiative_id,
            jcode_run_id, operation_kind, operation_state, command_payload_json,
            requests_json, orca_run_id, orca_task_id, orca_dispatch_id, orca_project_id,
            orca_repository_id, orca_host_setup_id, orca_worktree_id, orca_terminal_id,
            receipts_json, partial_effects_json, recovery_json, created_at, updated_at
        ) VALUES (
            ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15,
            ?16, ?17, ?18, ?19, ?20, ?21, ?22, ?23
        )"#,
        record_params(record)?,
    )?;
    Ok(())
}

fn update_record(
    transaction: &Transaction<'_>,
    record: &OrcaOperationRecord,
) -> Result<(), OrcaOperationStoreError> {
    transaction.execute(
        r#"UPDATE orca_operations SET
            idempotency_scope = ?2, idempotency_key = ?3, correlation_id = ?4,
            initiative_id = ?5, jcode_run_id = ?6, operation_kind = ?7,
            operation_state = ?8, command_payload_json = ?9, requests_json = ?10,
            orca_run_id = ?11, orca_task_id = ?12, orca_dispatch_id = ?13,
            orca_project_id = ?14, orca_repository_id = ?15, orca_host_setup_id = ?16,
            orca_worktree_id = ?17, orca_terminal_id = ?18, receipts_json = ?19,
            partial_effects_json = ?20, recovery_json = ?21, created_at = ?22,
            updated_at = ?23
        WHERE command_id = ?1"#,
        record_params(record)?,
    )?;
    Ok(())
}

fn record_params(
    record: &OrcaOperationRecord,
) -> Result<[rusqlite::types::Value; 23], OrcaOperationStoreError> {
    use rusqlite::types::Value;
    let optional = |value: Option<&str>| match value {
        Some(value) => Value::Text(value.to_string()),
        None => Value::Null,
    };
    Ok([
        Value::Text(record.command_id.0.clone()),
        Value::Text(record.idempotency_scope.clone()),
        Value::Text(record.idempotency_key.0.clone()),
        Value::Text(record.correlation_id.clone()),
        Value::Text(record.initiative_id.0.clone()),
        optional(record.jcode_run_id.as_ref().map(|id| id.0.as_str())),
        Value::Text(record.kind.as_str().to_string()),
        Value::Text(record.state.as_str().to_string()),
        Value::Text(serde_json::to_string(&record.command_payload)?),
        Value::Text(serde_json::to_string(&record.requests)?),
        optional(record.orca_run_id.as_ref().map(|id| id.0.as_str())),
        optional(record.orca_task_id.as_ref().map(|id| id.0.as_str())),
        optional(record.orca_dispatch_id.as_ref().map(|id| id.0.as_str())),
        optional(record.orca_project_id.as_ref().map(|id| id.0.as_str())),
        optional(record.orca_repository_id.as_deref()),
        optional(record.orca_host_setup_id.as_deref()),
        optional(record.orca_worktree_id.as_ref().map(|id| id.0.as_str())),
        optional(record.orca_terminal_id.as_ref().map(|id| id.0.as_str())),
        Value::Text(serde_json::to_string(&record.receipts)?),
        Value::Text(serde_json::to_string(&record.partial_effects)?),
        Value::Text(serde_json::to_string(&record.recovery)?),
        Value::Text(record.created_at.to_rfc3339()),
        Value::Text(record.updated_at.to_rfc3339()),
    ])
}

struct StoredOperationRow {
    command_id: String,
    idempotency_scope: String,
    idempotency_key: String,
    correlation_id: String,
    initiative_id: String,
    jcode_run_id: Option<String>,
    operation_kind: String,
    operation_state: String,
    command_payload_json: String,
    requests_json: String,
    orca_run_id: Option<String>,
    orca_task_id: Option<String>,
    orca_dispatch_id: Option<String>,
    orca_project_id: Option<String>,
    orca_repository_id: Option<String>,
    orca_host_setup_id: Option<String>,
    orca_worktree_id: Option<String>,
    orca_terminal_id: Option<String>,
    receipts_json: String,
    partial_effects_json: String,
    recovery_json: String,
    created_at: String,
    updated_at: String,
}

impl StoredOperationRow {
    fn from_row(row: &rusqlite::Row<'_>) -> Result<Self, rusqlite::Error> {
        Ok(Self {
            command_id: row.get(0)?,
            idempotency_scope: row.get(1)?,
            idempotency_key: row.get(2)?,
            correlation_id: row.get(3)?,
            initiative_id: row.get(4)?,
            jcode_run_id: row.get(5)?,
            operation_kind: row.get(6)?,
            operation_state: row.get(7)?,
            command_payload_json: row.get(8)?,
            requests_json: row.get(9)?,
            orca_run_id: row.get(10)?,
            orca_task_id: row.get(11)?,
            orca_dispatch_id: row.get(12)?,
            orca_project_id: row.get(13)?,
            orca_repository_id: row.get(14)?,
            orca_host_setup_id: row.get(15)?,
            orca_worktree_id: row.get(16)?,
            orca_terminal_id: row.get(17)?,
            receipts_json: row.get(18)?,
            partial_effects_json: row.get(19)?,
            recovery_json: row.get(20)?,
            created_at: row.get(21)?,
            updated_at: row.get(22)?,
        })
    }

    fn decode(self) -> Result<OrcaOperationRecord, OrcaOperationStoreError> {
        Ok(OrcaOperationRecord {
            command_id: CommandId(self.command_id),
            idempotency_scope: self.idempotency_scope,
            idempotency_key: IdempotencyKey(self.idempotency_key),
            correlation_id: self.correlation_id,
            initiative_id: InitiativeId(self.initiative_id),
            jcode_run_id: self.jcode_run_id.map(JcodeRunId),
            kind: OrcaOperationKind::parse(&self.operation_kind)?,
            state: OrcaOperationState::parse(&self.operation_state)?,
            command_payload: decode_json(&self.command_payload_json)?,
            requests: decode_json(&self.requests_json)?,
            orca_run_id: self.orca_run_id.map(OrcaRunId),
            orca_task_id: self.orca_task_id.map(OrcaTaskId),
            orca_dispatch_id: self.orca_dispatch_id.map(OrcaDispatchId),
            orca_project_id: self.orca_project_id.map(OrcaProjectId),
            orca_repository_id: self.orca_repository_id,
            orca_host_setup_id: self.orca_host_setup_id,
            orca_worktree_id: self.orca_worktree_id.map(OrcaWorktreeId),
            orca_terminal_id: self.orca_terminal_id.map(OrcaTerminalId),
            receipts: decode_json(&self.receipts_json)?,
            partial_effects: decode_json(&self.partial_effects_json)?,
            recovery: decode_json(&self.recovery_json)?,
            created_at: decode_time(&self.created_at)?,
            updated_at: decode_time(&self.updated_at)?,
        })
    }
}

fn decode_json<T: DeserializeOwned>(value: &str) -> Result<T, OrcaOperationStoreError> {
    Ok(serde_json::from_str(value)?)
}

fn decode_time(value: &str) -> Result<DateTime<Utc>, OrcaOperationStoreError> {
    DateTime::parse_from_rfc3339(value)
        .map(|value| value.with_timezone(&Utc))
        .map_err(|error| {
            OrcaOperationStoreError::InvalidData(format!("invalid timestamp: {error}"))
        })
}

const SELECT_COLUMNS: &str = r#"SELECT
    command_id, idempotency_scope, idempotency_key, correlation_id, initiative_id,
    jcode_run_id, operation_kind, operation_state, command_payload_json, requests_json,
    orca_run_id, orca_task_id, orca_dispatch_id, orca_project_id, orca_repository_id,
    orca_host_setup_id, orca_worktree_id, orca_terminal_id, receipts_json,
    partial_effects_json, recovery_json, created_at, updated_at
FROM orca_operations"#;

const SCHEMA: &str = r#"
CREATE TABLE IF NOT EXISTS orca_operations (
    command_id TEXT PRIMARY KEY,
    idempotency_scope TEXT NOT NULL,
    idempotency_key TEXT NOT NULL,
    correlation_id TEXT NOT NULL,
    initiative_id TEXT NOT NULL,
    jcode_run_id TEXT,
    operation_kind TEXT NOT NULL,
    operation_state TEXT NOT NULL,
    command_payload_json TEXT NOT NULL,
    requests_json TEXT NOT NULL DEFAULT '[]',
    orca_run_id TEXT,
    orca_task_id TEXT,
    orca_dispatch_id TEXT,
    orca_project_id TEXT,
    orca_repository_id TEXT,
    orca_host_setup_id TEXT,
    orca_worktree_id TEXT,
    orca_terminal_id TEXT,
    receipts_json TEXT NOT NULL DEFAULT '[]',
    partial_effects_json TEXT NOT NULL DEFAULT '[]',
    recovery_json TEXT NOT NULL DEFAULT '[]',
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    UNIQUE(idempotency_scope, idempotency_key)
);
CREATE INDEX IF NOT EXISTS orca_operations_initiative
    ON orca_operations(initiative_id, updated_at);
CREATE INDEX IF NOT EXISTS orca_operations_recovery
    ON orca_operations(operation_state, updated_at);
"#;

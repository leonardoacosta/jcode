use std::{fmt, panic::AssertUnwindSafe, path::Path};

use chrono::{DateTime, Utc};
use rusqlite::{Connection, OptionalExtension, Transaction, params};

use crate::{
    dedupe::DedupeKey,
    envelope::Envelope,
    record::{Classification, IntakeEvent, Record, RecordId},
    redact::Redactor,
    store::{Proposal, ProposalId, ProposalState, StoreError, TrackedWork, TrackedWorkId},
};

type Classifier = dyn Fn(&str) -> Result<Classification, String> + Send + Sync;

#[derive(Debug)]
pub enum SqliteStoreError {
    Database(rusqlite::Error),
    Serialization(serde_json::Error),
    Store(StoreError),
    InvalidData(&'static str),
}

impl fmt::Display for SqliteStoreError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Database(_) => formatter.write_str("intake database operation failed"),
            Self::Serialization(_) => formatter.write_str("intake data serialization failed"),
            Self::Store(error) => error.fmt(formatter),
            Self::InvalidData(message) => {
                write!(formatter, "invalid intake database data: {message}")
            }
        }
    }
}

impl std::error::Error for SqliteStoreError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Database(error) => Some(error),
            Self::Serialization(error) => Some(error),
            Self::Store(error) => Some(error),
            Self::InvalidData(_) => None,
        }
    }
}

impl From<rusqlite::Error> for SqliteStoreError {
    fn from(value: rusqlite::Error) -> Self {
        Self::Database(value)
    }
}

impl From<serde_json::Error> for SqliteStoreError {
    fn from(value: serde_json::Error) -> Self {
        Self::Serialization(value)
    }
}

/// SQLite-backed authority for inbound intake history and explicit promotion.
pub struct SqliteIntakeStore {
    connection: Connection,
    execution_budget: Option<usize>,
    redactor: Redactor,
    classifier: Box<Classifier>,
}

impl SqliteIntakeStore {
    pub fn open(
        path: impl AsRef<Path>,
        execution_budget: Option<usize>,
    ) -> Result<Self, SqliteStoreError> {
        Self::from_connection(Connection::open(path)?, execution_budget, default_classify)
    }

    pub fn open_in_memory(execution_budget: Option<usize>) -> Result<Self, SqliteStoreError> {
        Self::from_connection(
            Connection::open_in_memory()?,
            execution_budget,
            default_classify,
        )
    }

    pub fn open_with_classifier<F>(
        path: impl AsRef<Path>,
        execution_budget: Option<usize>,
        classifier: F,
    ) -> Result<Self, SqliteStoreError>
    where
        F: Fn(&str) -> Result<Classification, String> + Send + Sync + 'static,
    {
        Self::from_connection(Connection::open(path)?, execution_budget, classifier)
    }

    pub fn open_in_memory_with_classifier<F>(
        execution_budget: Option<usize>,
        classifier: F,
    ) -> Result<Self, SqliteStoreError>
    where
        F: Fn(&str) -> Result<Classification, String> + Send + Sync + 'static,
    {
        Self::from_connection(Connection::open_in_memory()?, execution_budget, classifier)
    }

    fn from_connection<F>(
        connection: Connection,
        execution_budget: Option<usize>,
        classifier: F,
    ) -> Result<Self, SqliteStoreError>
    where
        F: Fn(&str) -> Result<Classification, String> + Send + Sync + 'static,
    {
        connection.pragma_update(None, "foreign_keys", true)?;
        connection.pragma_update(None, "journal_mode", "WAL")?;
        connection.execute_batch(SCHEMA)?;
        Ok(Self {
            connection,
            execution_budget,
            redactor: Redactor::new(),
            classifier: Box::new(classifier),
        })
    }

    pub fn receive(
        &mut self,
        envelope: Envelope,
        raw_payload: serde_json::Value,
        operator: Option<String>,
    ) -> Result<RecordId, SqliteStoreError> {
        self.ingest(envelope, raw_payload, operator, true)
    }

    pub fn receive_unauthorized(
        &mut self,
        envelope: Envelope,
        raw_payload: serde_json::Value,
    ) -> Result<RecordId, SqliteStoreError> {
        self.ingest(envelope, raw_payload, None, false)
    }

    fn ingest(
        &mut self,
        mut envelope: Envelope,
        raw_payload: serde_json::Value,
        operator: Option<String>,
        authorized: bool,
    ) -> Result<RecordId, SqliteStoreError> {
        let dedupe_key = DedupeKey::new(
            &envelope.sender_identity,
            &envelope.conversation,
            envelope.content.as_deref(),
        );
        let clean_content = envelope
            .content
            .as_deref()
            .map(|content| self.redactor.scrub(content));
        let content_redactions = clean_content.as_ref().map_or(0, |outcome| outcome.count);
        envelope.content = clean_content.map(|outcome| outcome.text);
        let (raw_payload, raw_redactions) = self.redactor.scrub_json(&raw_payload);
        let raw_payload = serde_json::to_string(&raw_payload)?;
        let attachments = serde_json::to_string(&envelope.attachments)?;

        // This transaction is deliberately committed before any classifier is invoked.
        let transaction = self.connection.transaction()?;
        let prior = find_prior(&transaction, dedupe_key.as_str())?;
        transaction.execute(
            "INSERT INTO records (
                adapter, sender_identity, conversation, content, raw_payload, attachments,
                received_at, operator, dedupe_key, duplicate_of
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            params![
                envelope.adapter,
                envelope.sender_identity,
                envelope.conversation,
                envelope.content,
                raw_payload,
                attachments,
                envelope.received_at.to_rfc3339(),
                operator,
                dedupe_key.as_str(),
                prior.as_ref().map(|item| item.id.0),
            ],
        )?;
        let id = RecordId(transaction.last_insert_rowid().try_into().map_err(|_| {
            SqliteStoreError::InvalidData("record id is outside the supported range")
        })?);
        let redactions = content_redactions + raw_redactions;
        if redactions != 0 {
            insert_event(
                &transaction,
                &IntakeEvent::Redaction {
                    record: id,
                    count: redactions,
                },
            )?;
        }
        transaction.commit()?;

        if !authorized {
            self.connection.execute(
                "UPDATE records SET classification = 'unauthorized' WHERE id = ?1",
                [id.0],
            )?;
            return Ok(id);
        }

        if let Some(prior) = prior {
            if prior.executed || prior.classification_error {
                return Ok(id);
            }
            self.connection.execute(
                "UPDATE records SET duplicate_of = NULL, retry_of = ?1 WHERE id = ?2",
                params![prior.id.0, id.0],
            )?;
        }

        let content = envelope.content.as_deref().unwrap_or("");
        let classification =
            std::panic::catch_unwind(AssertUnwindSafe(|| (self.classifier)(content)));
        let classification = match classification {
            Ok(Ok(classification)) => classification,
            Ok(Err(error)) => {
                self.record_classification_failure(id, error)?;
                return Ok(id);
            }
            Err(payload) => {
                self.record_classification_failure(id, panic_message(payload))?;
                return Ok(id);
            }
        };

        let class = class_name(&classification);
        let transaction = self.connection.transaction()?;
        transaction.execute(
            "UPDATE records SET classification = ?1 WHERE id = ?2",
            params![class, id.0],
        )?;
        if let Some(budget) = self.execution_budget {
            let used: u64 = transaction
                .query_row(
                    "SELECT executed_count FROM class_counters WHERE class = ?1",
                    [class],
                    |row| row.get(0),
                )
                .optional()?
                .unwrap_or(0);
            if used >= budget as u64 {
                transaction.execute("UPDATE records SET deferred = 1 WHERE id = ?1", [id.0])?;
                insert_event(
                    &transaction,
                    &IntakeEvent::Deferral {
                        record: id,
                        class: class.to_owned(),
                    },
                )?;
                transaction.commit()?;
                return Ok(id);
            }
            transaction.execute(
                "INSERT INTO class_counters (class, executed_count) VALUES (?1, 1)
                 ON CONFLICT(class) DO UPDATE SET executed_count = executed_count + 1",
                [class],
            )?;
        }
        transaction.execute("UPDATE records SET executed = 1 WHERE id = ?1", [id.0])?;
        if classification == Classification::WorkRequest {
            transaction.execute(
                "INSERT INTO proposals (record_id, state) VALUES (?1, 'awaiting_approval')",
                [id.0],
            )?;
        }
        transaction.commit()?;
        Ok(id)
    }

    pub fn records(&self) -> Result<Vec<Record>, SqliteStoreError> {
        let mut statement = self.connection.prepare(
            "SELECT id, adapter, sender_identity, conversation, content, raw_payload, operator,
                    dedupe_key, duplicate_of, retry_of, classification, classification_error,
                    executed, deferred
             FROM records ORDER BY id",
        )?;
        let rows = statement.query_map([], row_to_record)?;
        rows.collect::<Result<Vec<_>, _>>().map_err(Into::into)
    }

    pub fn events(&self) -> Result<Vec<IntakeEvent>, SqliteStoreError> {
        let mut statement = self
            .connection
            .prepare("SELECT kind, record_id, count, class, error FROM events ORDER BY id")?;
        let rows = statement.query_map([], |row| {
            let kind: String = row.get(0)?;
            let record = RecordId(row.get(1)?);
            match kind.as_str() {
                "redaction" => Ok(IntakeEvent::Redaction {
                    record,
                    count: row
                        .get::<_, u64>(2)?
                        .try_into()
                        .map_err(|_| rusqlite::Error::IntegralValueOutOfRange(2, i64::MAX))?,
                }),
                "deferral" => Ok(IntakeEvent::Deferral {
                    record,
                    class: row.get(3)?,
                }),
                "classification_failure" => Ok(IntakeEvent::ClassificationFailure {
                    record,
                    error: row.get(4)?,
                }),
                _ => Err(rusqlite::Error::InvalidQuery),
            }
        })?;
        rows.collect::<Result<Vec<_>, _>>().map_err(Into::into)
    }

    pub fn proposals(&self) -> Result<Vec<Proposal>, SqliteStoreError> {
        let mut statement = self.connection.prepare(
            "SELECT id, record_id, state, approved_by, approved_at, approved_channel
             FROM proposals ORDER BY id",
        )?;
        let rows = statement.query_map([], |row| {
            let state: String = row.get(2)?;
            let approved_at: Option<String> = row.get(4)?;
            Ok(Proposal {
                id: ProposalId(row.get(0)?),
                record: RecordId(row.get(1)?),
                state: match state.as_str() {
                    "awaiting_approval" => ProposalState::AwaitingApproval,
                    "approved" => ProposalState::Approved,
                    _ => return Err(rusqlite::Error::InvalidQuery),
                },
                approved_by: row.get(3)?,
                approved_at: approved_at.map(|value| parse_time(&value)).transpose()?,
                approved_channel: row.get(5)?,
            })
        })?;
        rows.collect::<Result<Vec<_>, _>>().map_err(Into::into)
    }

    pub fn tracked_work(&self) -> Result<Vec<TrackedWork>, SqliteStoreError> {
        let mut statement = self
            .connection
            .prepare("SELECT id, record_id, proposal_id FROM tracked_work ORDER BY id")?;
        let rows = statement.query_map([], |row| {
            Ok(TrackedWork {
                id: TrackedWorkId(row.get(0)?),
                from_record: RecordId(row.get(1)?),
                from_proposal: ProposalId(row.get(2)?),
            })
        })?;
        rows.collect::<Result<Vec<_>, _>>().map_err(Into::into)
    }

    pub fn polling_offset(&self, adapter: &str) -> Result<Option<i64>, SqliteStoreError> {
        self.connection
            .query_row(
                "SELECT next_offset FROM polling_offsets WHERE adapter = ?1",
                [adapter],
                |row| row.get(0),
            )
            .optional()
            .map_err(Into::into)
    }

    pub fn set_polling_offset(
        &mut self,
        adapter: &str,
        next_offset: i64,
    ) -> Result<(), SqliteStoreError> {
        self.connection.execute(
            "INSERT INTO polling_offsets (adapter, next_offset) VALUES (?1, ?2)
             ON CONFLICT(adapter) DO UPDATE SET next_offset = excluded.next_offset",
            params![adapter, next_offset],
        )?;
        Ok(())
    }

    pub fn approve(
        &mut self,
        proposal: ProposalId,
        approver: String,
        at: DateTime<Utc>,
        channel: String,
    ) -> Result<TrackedWorkId, SqliteStoreError> {
        let transaction = self.connection.transaction()?;
        let item: Option<(u64, String)> = transaction
            .query_row(
                "SELECT record_id, state FROM proposals WHERE id = ?1",
                [proposal.0],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .optional()?;
        let (record, state) =
            item.ok_or_else(|| SqliteStoreError::Store(StoreError::ProposalNotFound(proposal)))?;
        if state == "approved" {
            return Err(SqliteStoreError::Store(
                StoreError::ProposalAlreadyApproved(proposal),
            ));
        }
        transaction.execute(
            "UPDATE proposals SET state = 'approved', approved_by = ?1, approved_at = ?2,
                                  approved_channel = ?3 WHERE id = ?4",
            params![approver, at.to_rfc3339(), channel, proposal.0],
        )?;
        transaction.execute(
            "INSERT INTO tracked_work (record_id, proposal_id) VALUES (?1, ?2)",
            params![record, proposal.0],
        )?;
        let id = TrackedWorkId(transaction.last_insert_rowid().try_into().map_err(|_| {
            SqliteStoreError::InvalidData("tracked work id is outside the supported range")
        })?);
        transaction.commit()?;
        Ok(id)
    }

    fn record_classification_failure(
        &mut self,
        id: RecordId,
        error: String,
    ) -> Result<(), SqliteStoreError> {
        let error = self.redactor.scrub(&error).text;
        let transaction = self.connection.transaction()?;
        transaction.execute(
            "UPDATE records SET classification_error = ?1 WHERE id = ?2",
            params![error, id.0],
        )?;
        insert_event(
            &transaction,
            &IntakeEvent::ClassificationFailure { record: id, error },
        )?;
        transaction.commit()?;
        Ok(())
    }
}

#[derive(Debug)]
struct PriorRecord {
    id: RecordId,
    executed: bool,
    classification_error: bool,
}

fn find_prior(
    transaction: &Transaction<'_>,
    dedupe_key: &str,
) -> Result<Option<PriorRecord>, rusqlite::Error> {
    transaction
        .query_row(
            "SELECT id, executed, classification_error IS NOT NULL
             FROM records WHERE dedupe_key = ?1 ORDER BY id LIMIT 1",
            [dedupe_key],
            |row| {
                Ok(PriorRecord {
                    id: RecordId(row.get(0)?),
                    executed: row.get(1)?,
                    classification_error: row.get(2)?,
                })
            },
        )
        .optional()
}

fn insert_event(transaction: &Transaction<'_>, event: &IntakeEvent) -> Result<(), rusqlite::Error> {
    match event {
        IntakeEvent::Redaction { record, count } => {
            transaction.execute(
                "INSERT INTO events (record_id, kind, count) VALUES (?1, 'redaction', ?2)",
                params![record.0, *count as u64],
            )?;
        }
        IntakeEvent::Deferral { record, class } => {
            transaction.execute(
                "INSERT INTO events (record_id, kind, class) VALUES (?1, 'deferral', ?2)",
                params![record.0, class],
            )?;
        }
        IntakeEvent::ClassificationFailure { record, error } => {
            transaction.execute(
                "INSERT INTO events (record_id, kind, error) VALUES (?1, 'classification_failure', ?2)",
                params![record.0, error],
            )?;
        }
    }
    Ok(())
}

fn row_to_record(row: &rusqlite::Row<'_>) -> Result<Record, rusqlite::Error> {
    let raw_payload: String = row.get(5)?;
    let dedupe_key: String = row.get(7)?;
    let classification: Option<String> = row.get(10)?;
    Ok(Record {
        id: RecordId(row.get(0)?),
        adapter: row.get(1)?,
        sender_identity: row.get(2)?,
        conversation: row.get(3)?,
        content: row.get(4)?,
        raw_payload: serde_json::from_str(&raw_payload).map_err(|error| {
            rusqlite::Error::FromSqlConversionFailure(
                5,
                rusqlite::types::Type::Text,
                Box::new(error),
            )
        })?,
        operator: row.get(6)?,
        dedupe_key: serde_json::from_str(&format!("\"{dedupe_key}\"")).map_err(|error| {
            rusqlite::Error::FromSqlConversionFailure(
                7,
                rusqlite::types::Type::Text,
                Box::new(error),
            )
        })?,
        duplicate_of: row.get::<_, Option<u64>>(8)?.map(RecordId),
        retry_of: row.get::<_, Option<u64>>(9)?.map(RecordId),
        classification: classification
            .map(|value| parse_classification(&value))
            .transpose()?,
        classification_error: row.get(11)?,
        executed: row.get(12)?,
        deferred: row.get(13)?,
    })
}

fn parse_classification(value: &str) -> Result<Classification, rusqlite::Error> {
    match value {
        "work_request" => Ok(Classification::WorkRequest),
        "research_request" => Ok(Classification::ResearchRequest),
        "status_request" => Ok(Classification::StatusRequest),
        "unrecognized" => Ok(Classification::Unrecognized),
        "unauthorized" => Ok(Classification::Unauthorized),
        _ => Err(rusqlite::Error::InvalidQuery),
    }
}

fn parse_time(value: &str) -> Result<DateTime<Utc>, rusqlite::Error> {
    DateTime::parse_from_rfc3339(value)
        .map(|time| time.with_timezone(&Utc))
        .map_err(|error| {
            rusqlite::Error::FromSqlConversionFailure(
                0,
                rusqlite::types::Type::Text,
                Box::new(error),
            )
        })
}

fn class_name(classification: &Classification) -> &'static str {
    match classification {
        Classification::WorkRequest => "work_request",
        Classification::ResearchRequest => "research_request",
        Classification::StatusRequest => "status_request",
        Classification::Unrecognized => "unrecognized",
        Classification::Unauthorized => "unauthorized",
    }
}

fn default_classify(content: &str) -> Result<Classification, String> {
    let lower = content.to_ascii_lowercase();
    if lower.contains("status") {
        Ok(Classification::StatusRequest)
    } else if lower.contains("research") || lower.starts_with("find ") {
        Ok(Classification::ResearchRequest)
    } else if lower.contains("build")
        || lower.contains("implement")
        || lower.contains("deploy")
        || lower.contains("do it")
    {
        Ok(Classification::WorkRequest)
    } else {
        Ok(Classification::Unrecognized)
    }
}

fn panic_message(payload: Box<dyn std::any::Any + Send>) -> String {
    if let Some(message) = payload.downcast_ref::<&str>() {
        (*message).to_owned()
    } else if let Some(message) = payload.downcast_ref::<String>() {
        message.clone()
    } else {
        "classifier panicked".to_owned()
    }
}

const SCHEMA: &str = r#"
CREATE TABLE IF NOT EXISTS records (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    adapter TEXT NOT NULL,
    sender_identity TEXT NOT NULL,
    conversation TEXT NOT NULL,
    content TEXT,
    raw_payload TEXT NOT NULL,
    attachments TEXT NOT NULL,
    received_at TEXT NOT NULL,
    operator TEXT,
    dedupe_key TEXT NOT NULL,
    duplicate_of INTEGER REFERENCES records(id),
    retry_of INTEGER REFERENCES records(id),
    classification TEXT,
    classification_error TEXT,
    executed INTEGER NOT NULL DEFAULT 0,
    deferred INTEGER NOT NULL DEFAULT 0
);
CREATE INDEX IF NOT EXISTS records_dedupe_key ON records(dedupe_key, id);
CREATE TABLE IF NOT EXISTS events (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    record_id INTEGER NOT NULL REFERENCES records(id),
    kind TEXT NOT NULL,
    count INTEGER,
    class TEXT,
    error TEXT
);
CREATE TABLE IF NOT EXISTS proposals (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    record_id INTEGER NOT NULL REFERENCES records(id),
    state TEXT NOT NULL,
    approved_by TEXT,
    approved_at TEXT,
    approved_channel TEXT
);
CREATE TABLE IF NOT EXISTS tracked_work (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    record_id INTEGER NOT NULL REFERENCES records(id),
    proposal_id INTEGER NOT NULL UNIQUE REFERENCES proposals(id)
);
CREATE TABLE IF NOT EXISTS class_counters (
    class TEXT PRIMARY KEY,
    executed_count INTEGER NOT NULL
);
CREATE TABLE IF NOT EXISTS polling_offsets (
    adapter TEXT PRIMARY KEY,
    next_offset INTEGER NOT NULL
);
"#;

#[cfg(test)]
mod tests {
    use chrono::{TimeZone, Utc};
    use serde_json::json;
    use tempfile::tempdir;

    use super::*;

    fn envelope(content: &str) -> Envelope {
        Envelope {
            adapter: "test".to_owned(),
            sender_identity: "person:1".to_owned(),
            conversation: "conversation:1".to_owned(),
            content: Some(content.to_owned()),
            attachments: Vec::new(),
            received_at: Utc.with_ymd_and_hms(2026, 8, 15, 18, 0, 0).unwrap(),
        }
    }

    #[test]
    fn reopen_after_receive_recovers_record() {
        let directory = tempdir().unwrap();
        let path = directory.path().join("intake.db");
        let id = SqliteIntakeStore::open(&path, None)
            .unwrap()
            .receive(envelope("status please"), json!({"delivery": 1}), None)
            .unwrap();
        let reopened = SqliteIntakeStore::open(&path, None).unwrap();
        let records = reopened.records().unwrap();
        assert_eq!(records.len(), 1);
        assert_eq!(records[0].id, id);
        assert!(records[0].executed);
    }

    #[test]
    fn reopen_after_approval_recovers_proposal_work_and_audit_identity() {
        let directory = tempdir().unwrap();
        let path = directory.path().join("intake.db");
        let at = Utc.with_ymd_and_hms(2026, 8, 15, 17, 0, 0).unwrap();
        let mut store = SqliteIntakeStore::open(&path, None).unwrap();
        let record = store
            .receive(envelope("build it"), json!({}), None)
            .unwrap();
        let proposal = store.proposals().unwrap()[0].id;
        store
            .approve(proposal, "op:leo".to_owned(), at, "telegram".to_owned())
            .unwrap();
        drop(store);

        let reopened = SqliteIntakeStore::open(&path, None).unwrap();
        let proposals = reopened.proposals().unwrap();
        let work = reopened.tracked_work().unwrap();
        assert_eq!(proposals[0].state, ProposalState::Approved);
        assert_eq!(proposals[0].approved_by.as_deref(), Some("op:leo"));
        assert_eq!(proposals[0].approved_at, Some(at));
        assert_eq!(proposals[0].approved_channel.as_deref(), Some("telegram"));
        assert_eq!(work[0].from_record, record);
        assert_eq!(work[0].from_proposal, proposal);
    }

    #[test]
    fn classifier_failure_survives_reopen() {
        let directory = tempdir().unwrap();
        let path = directory.path().join("intake.db");
        SqliteIntakeStore::open_with_classifier(&path, None, |_| Err("no class".to_owned()))
            .unwrap()
            .receive(envelope("ambiguous"), json!({}), None)
            .unwrap();
        let reopened = SqliteIntakeStore::open(&path, None).unwrap();
        assert_eq!(
            reopened.records().unwrap()[0]
                .classification_error
                .as_deref(),
            Some("no class")
        );
        assert!(matches!(
            reopened.events().unwrap()[0],
            IntakeEvent::ClassificationFailure { .. }
        ));
    }

    #[test]
    fn credentials_are_absent_from_database_bytes() {
        let directory = tempdir().unwrap();
        let path = directory.path().join("intake.db");
        let token = "123456789:ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghi";
        let mut store = SqliteIntakeStore::open(&path, None).unwrap();
        store
            .receive(
                envelope(&format!("use {token}")),
                json!({"nested": {"token": token}}),
                None,
            )
            .unwrap();
        drop(store);
        let bytes = std::fs::read(path).unwrap();
        assert!(
            !bytes
                .windows(token.len())
                .any(|window| window == token.as_bytes())
        );
    }

    #[test]
    fn duplicate_lookup_works_after_reopen() {
        let directory = tempdir().unwrap();
        let path = directory.path().join("intake.db");
        let first = SqliteIntakeStore::open(&path, None)
            .unwrap()
            .receive(envelope("status please"), json!({"delivery": 1}), None)
            .unwrap();
        let mut reopened = SqliteIntakeStore::open(&path, None).unwrap();
        reopened
            .receive(envelope("status please"), json!({"delivery": 2}), None)
            .unwrap();
        assert_eq!(reopened.records().unwrap()[1].duplicate_of, Some(first));
    }

    #[test]
    fn deferred_resend_is_a_retry() {
        let mut store = SqliteIntakeStore::open_in_memory(Some(0)).unwrap();
        let first = store
            .receive(envelope("build it"), json!({"delivery": 1}), None)
            .unwrap();
        store
            .receive(envelope("build it"), json!({"delivery": 2}), None)
            .unwrap();
        let records = store.records().unwrap();
        assert_eq!(records[1].retry_of, Some(first));
        assert_eq!(records[1].duplicate_of, None);
        assert!(records[1].deferred);
    }

    #[test]
    fn per_class_budget_survives_reopen() {
        let directory = tempdir().unwrap();
        let path = directory.path().join("intake.db");
        let mut store = SqliteIntakeStore::open(&path, Some(1)).unwrap();
        store
            .receive(envelope("status one"), json!({}), None)
            .unwrap();
        drop(store);
        let mut reopened = SqliteIntakeStore::open(&path, Some(1)).unwrap();
        reopened
            .receive(envelope("status two"), json!({}), None)
            .unwrap();
        reopened
            .receive(envelope("build dashboard"), json!({}), None)
            .unwrap();
        let records = reopened.records().unwrap();
        assert!(records[1].deferred);
        assert!(records[2].executed);
    }

    #[test]
    fn unauthorized_never_promotes() {
        let directory = tempdir().unwrap();
        let path = directory.path().join("intake.db");
        SqliteIntakeStore::open(&path, None)
            .unwrap()
            .receive_unauthorized(envelope("build it"), json!({}))
            .unwrap();
        let reopened = SqliteIntakeStore::open(&path, None).unwrap();
        let records = reopened.records().unwrap();
        assert_eq!(
            records[0].classification,
            Some(Classification::Unauthorized)
        );
        assert!(!records[0].executed);
        assert!(reopened.proposals().unwrap().is_empty());
        assert!(reopened.tracked_work().unwrap().is_empty());
    }
}

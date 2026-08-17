use serde::{Deserialize, Serialize};

use crate::dedupe::DedupeKey;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct RecordId(pub u64);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Record {
    pub id: RecordId,
    pub adapter: String,
    pub sender_identity: String,
    pub conversation: String,
    pub content: Option<String>,
    pub raw_payload: serde_json::Value,
    pub operator: Option<String>,
    pub dedupe_key: DedupeKey,
    pub duplicate_of: Option<RecordId>,
    pub retry_of: Option<RecordId>,
    pub classification: Option<Classification>,
    pub classification_error: Option<String>,
    pub executed: bool,
    pub deferred: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Classification {
    WorkRequest,
    ResearchRequest,
    StatusRequest,
    Unrecognized,
    Unauthorized,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum IntakeEvent {
    Redaction { record: RecordId, count: usize },
    Deferral { record: RecordId, class: String },
    ClassificationFailure { record: RecordId, error: String },
}

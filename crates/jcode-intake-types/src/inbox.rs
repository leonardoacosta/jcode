use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::{
    Classification, DedupeKey, Proposal, ProposalId, ProposalState, RecordId, TrackedWorkId,
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceIdentity {
    pub adapter: String,
    pub sender_identity: String,
    pub conversation: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DecisionInboxStatus {
    AwaitingApproval,
    Approved,
    ReadOnly,
    Unrecognized,
    Unauthorized,
    Deferred,
    ClassificationFailed,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DecisionProposal {
    pub id: ProposalId,
    pub state: ProposalState,
    pub approved_by: Option<String>,
    pub approved_at: Option<DateTime<Utc>>,
    pub approved_channel: Option<String>,
}

impl From<&Proposal> for DecisionProposal {
    fn from(proposal: &Proposal) -> Self {
        Self {
            id: proposal.id,
            state: proposal.state,
            approved_by: proposal.approved_by.clone(),
            approved_at: proposal.approved_at,
            approved_channel: proposal.approved_channel.clone(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DecisionInboxItem {
    pub record_id: RecordId,
    pub source: SourceIdentity,
    pub received_at: DateTime<Utc>,
    pub content: Option<String>,
    pub category: Option<Classification>,
    pub status: DecisionInboxStatus,
    pub proposal: Option<DecisionProposal>,
    pub tracked_work: Option<TrackedWorkId>,
    pub dedupe_key: DedupeKey,
    pub duplicate_deliveries: usize,
    pub retry_deliveries: usize,
    pub redacted: bool,
    pub raw_payload_retained: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DecisionInboxSnapshot {
    pub generated_at: DateTime<Utc>,
    pub items: Vec<DecisionInboxItem>,
}

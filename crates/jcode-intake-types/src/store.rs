use std::{collections::HashMap, fmt, panic::AssertUnwindSafe};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::{
    dedupe::DedupeKey,
    envelope::Envelope,
    record::{Classification, IntakeEvent, Record, RecordId},
    redact::Redactor,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ProposalId(pub u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TrackedWorkId(pub u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProposalState {
    AwaitingApproval,
    Approved,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Proposal {
    pub id: ProposalId,
    pub record: RecordId,
    pub state: ProposalState,
    pub approved_by: Option<String>,
    pub approved_at: Option<DateTime<Utc>>,
    pub approved_channel: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrackedWork {
    pub id: TrackedWorkId,
    pub from_record: RecordId,
    pub from_proposal: ProposalId,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StoreError {
    ProposalNotFound(ProposalId),
    ProposalAlreadyApproved(ProposalId),
}

impl fmt::Display for StoreError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ProposalNotFound(id) => write!(formatter, "proposal {} was not found", id.0),
            Self::ProposalAlreadyApproved(id) => {
                write!(formatter, "proposal {} is already approved", id.0)
            }
        }
    }
}

impl std::error::Error for StoreError {}

type Classifier = dyn Fn(&str) -> Result<Classification, String> + Send + Sync;

/// In-memory authority for inbound intake history and explicit promotion.
pub struct IntakeStore {
    records: Vec<Record>,
    events: Vec<IntakeEvent>,
    proposals: Vec<Proposal>,
    tracked_work: Vec<TrackedWork>,
    seen: HashMap<DedupeKey, RecordId>,
    executed_by_class: HashMap<String, usize>,
    execution_budget: Option<usize>,
    redactor: Redactor,
    classifier: Box<Classifier>,
    next_record_id: u64,
}

impl Default for IntakeStore {
    fn default() -> Self {
        Self::new(None)
    }
}

impl IntakeStore {
    #[must_use]
    pub fn new(execution_budget: Option<usize>) -> Self {
        Self::with_classifier(execution_budget, default_classify)
    }

    #[must_use]
    pub fn with_classifier<F>(execution_budget: Option<usize>, classifier: F) -> Self
    where
        F: Fn(&str) -> Result<Classification, String> + Send + Sync + 'static,
    {
        Self {
            records: Vec::new(),
            events: Vec::new(),
            proposals: Vec::new(),
            tracked_work: Vec::new(),
            seen: HashMap::new(),
            executed_by_class: HashMap::new(),
            execution_budget,
            redactor: Redactor::new(),
            classifier: Box::new(classifier),
            next_record_id: 1,
        }
    }

    /// Scrubs, records, deduplicates, classifies, and applies admission control,
    /// in that order. The returned id always names a retained record.
    pub fn receive(
        &mut self,
        envelope: Envelope,
        raw_payload: serde_json::Value,
        operator: Option<String>,
    ) -> RecordId {
        self.ingest(envelope, raw_payload, operator, true)
    }

    /// Record a message from a sender that is not authorized.
    ///
    /// The message is retained in full, but it is never classified, executed,
    /// or promoted. Authorization is checked before interpretation so an
    /// unauthorized request cannot reach the classifier at all.
    pub fn receive_unauthorized(
        &mut self,
        envelope: Envelope,
        raw_payload: serde_json::Value,
    ) -> RecordId {
        self.ingest(envelope, raw_payload, None, false)
    }

    fn ingest(
        &mut self,
        mut envelope: Envelope,
        raw_payload: serde_json::Value,
        operator: Option<String>,
        authorized: bool,
    ) -> RecordId {
        // The key must observe the original content, but the original itself must
        // never enter any retained structure.
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

        let id = RecordId(self.next_record_id);
        self.next_record_id += 1;
        let prior_id = self.seen.get(&dedupe_key).copied();

        // This append is intentionally before duplicate resolution and classification.
        self.records.push(Record {
            id,
            adapter: envelope.adapter,
            sender_identity: envelope.sender_identity,
            conversation: envelope.conversation,
            content: envelope.content,
            raw_payload,
            operator,
            dedupe_key: dedupe_key.clone(),
            duplicate_of: prior_id,
            retry_of: None,
            classification: None,
            classification_error: None,
            executed: false,
            deferred: false,
        });
        self.seen.entry(dedupe_key).or_insert(id);

        let redactions = content_redactions + raw_redactions;
        if redactions != 0 {
            self.events.push(IntakeEvent::Redaction {
                record: id,
                count: redactions,
            });
        }

        if !authorized {
            self.last_record_mut(id).classification = Some(Classification::Unauthorized);
            return id;
        }

        if let Some(prior_id) = prior_id {
            let prior = self
                .records
                .iter()
                .find(|record| record.id == prior_id)
                .expect("seen record ids must refer to retained records");
            if prior.executed || prior.classification_error.is_some() {
                return id;
            }
            let record = self.last_record_mut(id);
            record.retry_of = Some(prior_id);
            record.duplicate_of = None;
        }

        let content = self
            .records
            .last()
            .and_then(|record| record.content.as_deref())
            .unwrap_or("")
            .to_owned();
        let classification =
            std::panic::catch_unwind(AssertUnwindSafe(|| (self.classifier)(&content)));
        let classification = match classification {
            Ok(Ok(classification)) => classification,
            Ok(Err(error)) => {
                self.record_classification_failure(id, error);
                return id;
            }
            Err(payload) => {
                let error = panic_message(payload);
                self.record_classification_failure(id, error);
                return id;
            }
        };
        self.last_record_mut(id).classification = Some(classification.clone());

        let class = class_name(&classification).to_owned();
        if let Some(budget) = self.execution_budget {
            let used = self.executed_by_class.get(&class).copied().unwrap_or(0);
            if used >= budget {
                self.last_record_mut(id).deferred = true;
                self.events
                    .push(IntakeEvent::Deferral { record: id, class });
                return id;
            }
            self.executed_by_class.insert(class, used + 1);
        }

        self.last_record_mut(id).executed = true;
        if matches!(classification, Classification::WorkRequest) {
            self.proposals.push(Proposal {
                id: ProposalId(self.proposals.len() as u64 + 1),
                record: id,
                state: ProposalState::AwaitingApproval,
                approved_by: None,
                approved_at: None,
                approved_channel: None,
            });
        }
        id
    }

    #[must_use]
    pub fn records(&self) -> &[Record] {
        &self.records
    }

    #[must_use]
    pub fn events(&self) -> &[IntakeEvent] {
        &self.events
    }

    #[must_use]
    pub fn proposals(&self) -> &[Proposal] {
        &self.proposals
    }

    #[must_use]
    pub fn tracked_work(&self) -> &[TrackedWork] {
        &self.tracked_work
    }

    pub fn approve(
        &mut self,
        proposal: ProposalId,
        approver: String,
        at: DateTime<Utc>,
        channel: String,
    ) -> Result<TrackedWorkId, StoreError> {
        let item = self
            .proposals
            .iter_mut()
            .find(|item| item.id == proposal)
            .ok_or(StoreError::ProposalNotFound(proposal))?;
        if item.state == ProposalState::Approved {
            return Err(StoreError::ProposalAlreadyApproved(proposal));
        }
        item.state = ProposalState::Approved;
        item.approved_by = Some(approver);
        item.approved_at = Some(at);
        item.approved_channel = Some(channel);

        let id = TrackedWorkId(self.tracked_work.len() as u64 + 1);
        self.tracked_work.push(TrackedWork {
            id,
            from_record: item.record,
            from_proposal: proposal,
        });
        Ok(id)
    }

    fn last_record_mut(&mut self, id: RecordId) -> &mut Record {
        let record = self.records.last_mut().expect("receive appended a record");
        debug_assert_eq!(record.id, id);
        record
    }

    fn record_classification_failure(&mut self, id: RecordId, error: String) {
        self.last_record_mut(id).classification_error = Some(error.clone());
        self.events
            .push(IntakeEvent::ClassificationFailure { record: id, error });
    }
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

#[cfg(test)]
mod tests {
    use chrono::{TimeZone, Utc};
    use serde_json::json;

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
    fn classifier_failure_still_leaves_a_record() {
        let mut store = IntakeStore::with_classifier(None, |_| panic!("classifier blew up"));
        let id = store.receive(envelope("ambiguous"), json!({"text": "ambiguous"}), None);

        assert_eq!(store.records().len(), 1);
        assert_eq!(store.records()[0].id, id);
        assert_eq!(
            store.records()[0].classification_error.as_deref(),
            Some("classifier blew up")
        );
    }

    #[test]
    fn replay_of_an_executed_message_is_a_duplicate() {
        let mut store = IntakeStore::new(None);
        let first = store.receive(envelope("what is the status"), json!({"delivery": 1}), None);
        let second = store.receive(envelope("what is the status"), json!({"delivery": 2}), None);

        assert!(store.records()[0].executed);
        assert_eq!(store.records()[1].duplicate_of, Some(first));
        assert_eq!(store.records()[1].retry_of, None);
        assert_eq!(second, store.records()[1].id);
    }

    #[test]
    fn resend_of_a_deferred_message_is_a_retry_and_is_not_swallowed() {
        let mut store = IntakeStore::new(Some(0));
        let first = store.receive(envelope("build it"), json!({"delivery": 1}), None);
        store.receive(envelope("build it"), json!({"delivery": 2}), None);

        assert!(store.records()[0].deferred);
        assert_eq!(store.records()[1].retry_of, Some(first));
        assert_eq!(store.records()[1].duplicate_of, None);
        assert!(store.records()[1].deferred);
        assert!(store.records()[1].classification.is_some());
    }

    #[test]
    fn status_request_cannot_exhaust_work_request_budget() {
        let mut store = IntakeStore::new(Some(1));
        store.receive(envelope("what is the status one"), json!({}), None);
        store.receive(envelope("what is the status two"), json!({}), None);
        store.receive(envelope("build a dashboard"), json!({}), None);

        assert!(store.records()[1].deferred);
        assert!(store.records()[2].executed);
        assert!(!store.records()[2].deferred);
        assert_eq!(store.proposals().len(), 1);
    }

    #[test]
    fn no_tracked_work_exists_until_approve_is_called() {
        let mut store = IntakeStore::new(None);
        store.receive(envelope("build a dashboard"), json!({}), None);

        assert_eq!(store.proposals().len(), 1);
        assert_eq!(store.proposals()[0].state, ProposalState::AwaitingApproval);
        assert!(store.tracked_work().is_empty());
    }

    #[test]
    fn approve_records_identity_time_and_channel() {
        let mut store = IntakeStore::new(None);
        let record = store.receive(envelope("build a dashboard"), json!({}), None);
        let at = Utc.with_ymd_and_hms(2026, 8, 15, 17, 0, 0).unwrap();
        let proposal = store.proposals()[0].id;
        store
            .approve(proposal, "op:leo".to_owned(), at, "telegram".to_owned())
            .unwrap();

        let approved = &store.proposals()[0];
        assert_eq!(approved.state, ProposalState::Approved);
        assert_eq!(approved.approved_by.as_deref(), Some("op:leo"));
        assert_eq!(approved.approved_at, Some(at));
        assert_eq!(approved.approved_channel.as_deref(), Some("telegram"));
        assert_eq!(store.tracked_work()[0].from_record, record);
        assert_eq!(store.tracked_work()[0].from_proposal, proposal);
    }

    #[test]
    fn every_message_is_retained_including_duplicates_and_deferred_ones() {
        let mut store = IntakeStore::new(Some(1));
        store.receive(envelope("what is the status"), json!({"delivery": 1}), None);
        store.receive(envelope("what is the status"), json!({"delivery": 2}), None);
        store.receive(
            envelope("status for another thing"),
            json!({"delivery": 3}),
            None,
        );

        assert_eq!(store.records().len(), 3);
        assert!(store.records()[1].duplicate_of.is_some());
        assert!(store.records()[2].deferred);
    }

    #[test]
    fn credentials_are_scrubbed_before_record_or_event_storage() {
        let token = "123456789:ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghi";
        let mut store = IntakeStore::new(None);
        store.receive(
            envelope(&format!("use {token}")),
            json!({"nested": {"token": token}}),
            None,
        );

        let stored = format!("{:?}{:?}", store.records(), store.events());
        assert!(!stored.contains(token));
        assert!(stored.contains("[REDACTED]"));
    }
}

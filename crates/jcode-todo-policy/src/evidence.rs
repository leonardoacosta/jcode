//! Redacted continuation diagnostics. No prompt, command, or tool-output text is accepted.
use crate::{FollowUpKind, TodoItem, canonical_todo_status};
use serde::Serialize;

/// Fixed keys, enums and scalar counts keep every serialized record below this cap.
pub const MAX_CONTINUATION_EVIDENCE_BYTES: usize = 2048;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CheckResult {
    Pass,
    Fail,
    #[default]
    NotRun,
}

impl From<bool> for CheckResult {
    fn from(passed: bool) -> Self {
        if passed { Self::Pass } else { Self::Fail }
    }
}

/// Scheduler predicates, not evidence that external required commands passed.
#[derive(Clone, Debug, Default, Serialize)]
pub struct SchedulerChecks {
    pub ownership_delivery: CheckResult,
    pub completion_confidence: CheckResult,
    pub confidence_spike: CheckResult,
    pub gate_budget: CheckResult,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ContinuationReason {
    #[default]
    Disabled,
    PendingFollowup,
    UnchangedFinalResponse,
    LongSessionReview,
    NoTodos,
    GateDigest,
    UnchangedOwnershipAssessment,
    Ownership,
    CompletionValidation,
    ConfidenceSpike,
    GateExhausted,
    FinalResponse,
    FinalResponseAlreadyRequested,
    UnchangedTodos,
    IncompleteTodos,
}

#[derive(Clone, Debug, Default, Serialize)]
pub struct TodoCounts {
    pub pending: usize,
    pub in_progress: usize,
    pub completed: usize,
    pub cancelled: usize,
    pub unknown: usize,
}

#[derive(Clone, Debug, Default, Serialize)]
pub struct ContinuationEvidence {
    pub kind: Option<FollowUpKind>,
    pub reason: ContinuationReason,
    pub todos: TodoCounts,
    /// None when no comparable fingerprint was checked on this path.
    pub fingerprint_changed: Option<bool>,
    pub scheduler_checks: SchedulerChecks,
    /// Required command outcomes are unavailable at the scheduler boundary.
    pub required_checks_available: bool,
    pub waiting: bool,
    /// A declared dependency exists. Its identity and prose are omitted.
    pub blocked: bool,
    pub truncated: bool,
}

impl ContinuationEvidence {
    pub fn new(todos: &[TodoItem]) -> Self {
        let mut record = Self::default();
        for todo in todos {
            match canonical_todo_status(&todo.status) {
                Some("pending") => record.todos.pending += 1,
                Some("in_progress") => record.todos.in_progress += 1,
                Some("completed") => record.todos.completed += 1,
                Some("cancelled") => record.todos.cancelled += 1,
                _ => record.todos.unknown += 1,
            }
            record.blocked |= !todo.blocked_by.is_empty();
            // Free text is deliberately omitted, never clipped into the log.
            record.truncated |= !todo.content.is_empty()
                || !todo.id.is_empty()
                || !todo.priority.is_empty()
                || todo.details.is_some()
                || todo.group.is_some()
                || todo.assigned_to.is_some()
                || !todo.blocked_by.is_empty()
                || canonical_todo_status(&todo.status).is_none();
        }
        record
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{FollowUpPolicy, TodoSnapshot};

    #[test]
    fn continuation_evidence_is_bounded_and_redacts_todo_text() {
        let secret = "PRIVATE_TOOL_OUTPUT".repeat(10_000);
        let todos = vec![TodoItem {
            content: secret.clone(),
            details: Some(secret.clone()),
            status: " IN-PROGRESS ".into(),
            blocked_by: vec![secret.clone()],
            ..Default::default()
        }];
        let record = ContinuationEvidence::new(&todos);
        let json = serde_json::to_string(&record).unwrap();
        assert!(json.len() <= MAX_CONTINUATION_EVIDENCE_BYTES);
        assert!(!json.contains("PRIVATE_TOOL_OUTPUT"));
        assert!(record.truncated);
        assert_eq!(record.todos.in_progress, 1);
        assert!(record.blocked);
    }

    #[test]
    fn continuation_evidence_fixed_schema_bounds_worst_case_counts_and_reasons() {
        let record = ContinuationEvidence {
            todos: TodoCounts {
                pending: usize::MAX,
                in_progress: usize::MAX,
                completed: usize::MAX,
                cancelled: usize::MAX,
                unknown: usize::MAX,
            },
            kind: Some(FollowUpKind::CompletionValidation),
            reason: ContinuationReason::UnchangedOwnershipAssessment,
            ..Default::default()
        };
        assert!(serde_json::to_string(&record).unwrap().len() <= MAX_CONTINUATION_EVIDENCE_BYTES);
    }

    #[test]
    fn continuation_evidence_records_only_explicit_observed_checks() {
        let mut record = ContinuationEvidence::new(&[]);
        assert_eq!(
            record.scheduler_checks.ownership_delivery,
            CheckResult::NotRun
        );
        record.scheduler_checks.ownership_delivery = CheckResult::Fail;
        let value = serde_json::to_value(&record).unwrap();
        assert_eq!(value["scheduler_checks"]["ownership_delivery"], "fail");
        assert_eq!(
            value["scheduler_checks"]["completion_confidence"],
            "not_run"
        );
    }

    #[test]
    fn continuation_evidence_does_not_change_policy_or_fingerprints() {
        let snapshot = TodoSnapshot {
            todos: vec![TodoItem {
                status: "pending".into(),
                ..Default::default()
            }],
            ..Default::default()
        };
        let mut plain = FollowUpPolicy::new(true);
        let mut recorded = plain.clone();
        for _ in 0..3 {
            let decision = recorded.decide(&snapshot);
            let mut record = ContinuationEvidence::new(&snapshot.todos);
            record.kind = decision.follow_up.as_ref().map(|follow_up| follow_up.kind);
            serde_json::to_string(&record).unwrap();
            assert_eq!(decision, plain.decide(&snapshot));
            assert_eq!(recorded, plain);
        }
    }
}

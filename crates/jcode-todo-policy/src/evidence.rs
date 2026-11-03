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

/// Retain only bounded explicit bindings for the current turn. Failures are sticky.
#[derive(Clone, Debug, Default)]
pub struct RequiredChecks {
    entries: Vec<(String, CheckResult)>,
    truncated: bool,
}

impl RequiredChecks {
    pub fn observe_batch(&mut self, batch: jcode_task_types::RequiredCheckBatch, error: bool) {
        let batch = batch.bounded();
        self.truncated |= batch.truncated;
        for outcome in batch.outcomes {
            self.observe(&outcome, error);
        }
    }

    pub fn observe(&mut self, outcome: &jcode_task_types::RequiredCheckOutcome, tool_error: bool) {
        if !jcode_task_types::RequiredCheckOutcome::valid_id(&outcome.required_check_id) {
            self.truncated = true;
            return;
        }
        let result = if tool_error || outcome.tool_error {
            CheckResult::Fail
        } else if !outcome.foreground || !outcome.completed {
            CheckResult::NotRun
        } else {
            outcome.passed().into()
        };
        if let Some((_, prior)) = self
            .entries
            .iter_mut()
            .find(|(id, _)| id == &outcome.required_check_id)
        {
            if *prior != CheckResult::Fail {
                *prior = result;
            }
        } else if self.entries.len() < 8 {
            self.entries
                .push((outcome.required_check_id.clone(), result));
        } else {
            self.truncated = true;
        }
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct RequiredCheckEvidence {
    /// Turn-local pseudonym. Supplied names never enter diagnostic logs.
    pub check_id: String,
    pub result: CheckResult,
}

#[derive(Clone, Debug, Default, Serialize)]
pub struct ContinuationEvidence {
    pub kind: Option<FollowUpKind>,
    pub reason: ContinuationReason,
    pub todos: TodoCounts,
    /// None when no comparable fingerprint was checked on this path.
    pub fingerprint_changed: Option<bool>,
    pub scheduler_checks: SchedulerChecks,
    pub required_checks_available: bool,
    pub required_checks: Vec<RequiredCheckEvidence>,
    pub waiting: bool,
    /// A declared dependency exists. Its identity and prose are omitted.
    pub blocked: bool,
    pub truncated: bool,
}

impl ContinuationEvidence {
    pub fn record_required_checks(&mut self, checks: &RequiredChecks) {
        self.required_checks_available = !checks.entries.is_empty();
        self.truncated |= checks.truncated || !checks.entries.is_empty();
        self.required_checks = checks
            .entries
            .iter()
            .enumerate()
            .map(|(index, (_, result))| RequiredCheckEvidence {
                check_id: format!("check_{}", index + 1),
                result: *result,
            })
            .collect();
    }

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
    fn continuation_evidence_required_checks_redact_bound_deduplicate_and_preserve_failure() {
        let mut checks = RequiredChecks::default();
        for (id, code) in [("SECRET", 7), ("SECRET", 0), ("unrelated", 0)] {
            checks.observe(
                &jcode_task_types::RequiredCheckOutcome {
                    required_check_id: id.into(),
                    foreground: true,
                    completed: true,
                    exit_code: Some(code),
                    tool_error: false,
                },
                false,
            );
        }
        let mut record = ContinuationEvidence::default();
        record.record_required_checks(&checks);
        let value = serde_json::to_value(&record).unwrap();
        assert_eq!(value["required_checks"][0]["result"], "fail");
        assert_eq!(value["required_checks"][1]["result"], "pass");
        assert_eq!(record.required_checks.len(), 2);
        assert_ne!(
            record.required_checks[0].check_id,
            record.required_checks[1].check_id
        );
        let first = record.required_checks[0].check_id.clone();
        record.record_required_checks(&checks);
        assert_eq!(record.required_checks[0].check_id, first);
        assert!(record.required_checks_available);
        for n in 0..100 {
            checks.observe(
                &jcode_task_types::RequiredCheckOutcome {
                    required_check_id: format!("PRIVATE_{n}"),
                    foreground: true,
                    completed: true,
                    exit_code: Some(0),
                    tool_error: false,
                },
                false,
            );
        }
        record.record_required_checks(&checks);
        let json = serde_json::to_string(&record).unwrap();
        assert!(json.len() <= MAX_CONTINUATION_EVIDENCE_BYTES);
        assert!(!json.contains("SECRET"));
        assert!(!json.contains("PRIVATE"));
        assert!(record.truncated);
        assert_eq!(record.required_checks.len(), 8);
    }

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
        let mut record = ContinuationEvidence {
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
        let mut checks = RequiredChecks::default();
        for n in 0..8 {
            checks.observe(
                &jcode_task_types::RequiredCheckOutcome {
                    required_check_id: format!("check{n}"),
                    foreground: true,
                    completed: false,
                    exit_code: None,
                    tool_error: false,
                },
                false,
            );
        }
        record.record_required_checks(&checks);
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

#[cfg(test)]
mod required_check_failure_tests {
    use super::*;

    #[test]
    fn continuation_evidence_required_checks_never_promote_missing_or_error_status() {
        for (foreground, completed, exit_code, tool_error, event_error, expected) in [
            (false, false, Some(0), false, false, CheckResult::NotRun),
            (true, false, Some(0), false, false, CheckResult::NotRun),
            (true, true, None, false, false, CheckResult::Fail),
            (true, true, Some(0), true, false, CheckResult::Fail),
            (true, true, Some(0), false, true, CheckResult::Fail),
        ] {
            let mut checks = RequiredChecks::default();
            checks.observe(
                &jcode_task_types::RequiredCheckOutcome {
                    required_check_id: "check".into(),
                    foreground,
                    completed,
                    exit_code,
                    tool_error,
                },
                event_error,
            );
            let mut evidence = ContinuationEvidence::default();
            evidence.record_required_checks(&checks);
            assert_eq!(evidence.required_checks[0].result, expected);
        }
    }
}

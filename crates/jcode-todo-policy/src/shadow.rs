//! Observation-only semantic continuation records and offline comparison.

use crate::evidence::{CheckResult, ContinuationEvidence, ContinuationReason};
use serde::{Deserialize, Serialize};

pub const MAX_SHADOW_REPLAY_BYTES: usize = 4 * 1024 * 1024;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ShadowVerdict {
    Continue,
    Stop,
    Uncertain,
}

/// Immutable original decision, never an instruction to the scheduler.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ShadowRecord {
    pub decision_id: String,
    pub deterministic_queued: bool,
    pub evidence: ContinuationEvidence,
    pub verdict: ShadowVerdict,
    pub latency_ms: u64,
    /// Supervision evaluations started for this decision, not token usage.
    pub calls: usize,
}

#[derive(Debug, Default, Serialize)]
pub struct ShadowReplayReport {
    pub records: usize,
    pub missed_required_checks: usize,
    pub unknown_required_checks: usize,
    pub hypothetical_redundant_turns: usize,
    pub disagreements: usize,
    pub calls: usize,
    pub latency_ms: u64,
    pub uncertain: usize,
}

/// Offline only. Unknown check provenance is reported, never counted as passed.
pub fn replay_shadow_json(input: &[u8]) -> Result<ShadowReplayReport, String> {
    if input.len() > MAX_SHADOW_REPLAY_BYTES {
        return Err("shadow replay input exceeds 4 MiB".into());
    }
    let records: Vec<ShadowRecord> = serde_json::from_slice(input).map_err(|_| {
        "shadow replay requires complete evidence records in a JSON array".to_string()
    })?;
    if records.is_empty() {
        return Err("shadow replay requires at least one evidence record".into());
    }
    let mut ids = std::collections::HashSet::new();
    let mut report = ShadowReplayReport::default();
    for record in records {
        let evidence = &record.evidence;
        if record.decision_id.is_empty()
            || record.decision_id.len() > 64
            || !ids.insert(record.decision_id)
            || record.calls > 1
            || evidence.required_checks.len() > 8
            || evidence.required_checks_available != !evidence.required_checks.is_empty()
            || (evidence.required_checks_complete && !evidence.required_checks_available)
            || evidence
                .required_checks
                .iter()
                .enumerate()
                .any(|(index, check)| check.check_id != format!("check_{}", index + 1))
        {
            return Err("shadow replay has invalid or duplicate evidence bindings".into());
        }
        report.records += 1;
        report.calls += record.calls;
        report.latency_ms = report
            .latency_ms
            .checked_add(record.latency_ms)
            .ok_or_else(|| "shadow replay latency total overflow".to_string())?;
        report.uncertain += usize::from(record.verdict == ShadowVerdict::Uncertain);
        report.unknown_required_checks += usize::from(!evidence.required_checks_complete);
        if record.verdict == ShadowVerdict::Stop {
            report.missed_required_checks += evidence
                .required_checks
                .iter()
                .filter(|check| check.result != CheckResult::Pass)
                .count();
            let redundant = evidence.waiting
                || evidence.blocked
                || (evidence.fingerprint_changed == Some(false)
                    && matches!(
                        evidence.reason,
                        ContinuationReason::UnchangedTodos
                            | ContinuationReason::UnchangedFinalResponse
                            | ContinuationReason::UnchangedOwnershipAssessment
                    ));
            report.hypothetical_redundant_turns += usize::from(
                record.deterministic_queued
                    && redundant
                    && evidence.required_checks_available
                    && evidence.required_checks_complete
                    && evidence
                        .required_checks
                        .iter()
                        .all(|check| check.result == CheckResult::Pass),
            );
        }
        report.disagreements += usize::from(match record.verdict {
            ShadowVerdict::Continue => !record.deterministic_queued,
            ShadowVerdict::Stop => record.deterministic_queued,
            ShadowVerdict::Uncertain => false,
        });
    }
    Ok(report)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{Value, json};

    fn row() -> Value {
        json!({
            "decision_id":"poke-1", "deterministic_queued":true,
            "verdict":"stop", "latency_ms":12, "calls":1,
            "evidence": {
                "kind":null, "reason":"unchanged_todos",
                "todos":{"pending":1,"in_progress":0,"completed":0,"cancelled":0,"unknown":0},
                "fingerprint_changed":false,
                "scheduler_checks":{"ownership_delivery":"not_run","completion_confidence":"not_run","confidence_spike":"not_run","gate_budget":"not_run"},
                "required_checks_available":true,
                "required_checks_complete":true,
                "required_checks":[{"check_id":"check_1","result":"pass"}],
                "waiting":false,"blocked":false,"truncated":false
            }
        })
    }

    #[test]
    fn semantic_shadow_replay_reports_cost_and_redundancy_without_savings_claims() {
        let report = replay_shadow_json(&serde_json::to_vec(&vec![row()]).unwrap()).unwrap();
        assert_eq!(report.records, 1);
        assert_eq!(report.missed_required_checks, 0);
        assert_eq!(report.hypothetical_redundant_turns, 1);
        assert_eq!(report.calls, 1);
        assert_eq!(report.latency_ms, 12);
        assert_eq!(report.uncertain, 0);
    }

    #[test]
    fn semantic_shadow_replay_dropped_required_checks_are_unknown_never_redundant() {
        use crate::evidence::RequiredChecks;
        for batch in [false, true] {
            for count in [8, 9] {
                let outcomes: Vec<_> = (0..count)
                    .map(|index| crate::RequiredCheckOutcome {
                        required_check_id: format!("required_{index}"),
                        foreground: true,
                        completed: true,
                        exit_code: Some(if index == 8 { 7 } else { 0 }),
                        tool_error: false,
                    })
                    .collect();
                let mut checks = RequiredChecks::default();
                if batch {
                    checks.observe_batch(
                        crate::RequiredCheckBatch {
                            outcomes,
                            truncated: false,
                        },
                        false,
                    );
                } else {
                    for outcome in outcomes {
                        checks.observe(&outcome, false);
                    }
                }
                let mut evidence = ContinuationEvidence {
                    reason: ContinuationReason::UnchangedTodos,
                    fingerprint_changed: Some(false),
                    ..Default::default()
                };
                evidence.record_required_checks(&checks);
                assert!(
                    evidence.truncated,
                    "pseudonym redaction remains independent"
                );
                let record = ShadowRecord {
                    decision_id: "capacity-control".into(),
                    deterministic_queued: true,
                    evidence,
                    verdict: ShadowVerdict::Stop,
                    calls: 1,
                    latency_ms: 1,
                };
                let json = serde_json::to_vec(&vec![record]).unwrap();
                let report = replay_shadow_json(&json).unwrap();
                assert_eq!(
                    report.unknown_required_checks,
                    usize::from(count == 9),
                    "batch={batch} count={count}"
                );
                assert_eq!(
                    report.hypothetical_redundant_turns,
                    usize::from(count == 8),
                    "batch={batch} count={count}"
                );
            }
        }
        let mut legacy = row();
        legacy["evidence"]
            .as_object_mut()
            .unwrap()
            .remove("required_checks_complete");
        let report = replay_shadow_json(&serde_json::to_vec(&vec![legacy]).unwrap()).unwrap();
        assert_eq!(report.unknown_required_checks, 1);
        assert_eq!(report.hypothetical_redundant_turns, 0);
    }

    #[test]
    fn semantic_shadow_replay_rejects_missing_evidence_and_overflow() {
        for input in [
            json!([]),
            json!([{}]),
            json!([{"evidence":null}]),
            json!([row(), row()]),
        ] {
            assert!(replay_shadow_json(&serde_json::to_vec(&input).unwrap()).is_err());
        }
        let mut huge = row();
        huge["latency_ms"] = json!(u64::MAX);
        let mut another = row();
        another["decision_id"] = json!("another");
        assert!(replay_shadow_json(&serde_json::to_vec(&vec![huge, another]).unwrap()).is_err());
        assert!(replay_shadow_json(&vec![b' '; MAX_SHADOW_REPLAY_BYTES + 1]).is_err());
        for (path, value) in [
            ("verdict", json!("pass")),
            ("calls", json!(2)),
            ("decision_id", json!("x".repeat(65))),
        ] {
            let mut invalid = row();
            invalid[path] = value;
            assert!(replay_shadow_json(&serde_json::to_vec(&vec![invalid]).unwrap()).is_err());
        }
        let mut invalid = row();
        invalid["evidence"]["required_checks"][0]["result"] = json!("missing");
        assert!(replay_shadow_json(&serde_json::to_vec(&vec![&invalid]).unwrap()).is_err());
        invalid["evidence"]["required_checks"] = json!([]);
        assert!(replay_shadow_json(&serde_json::to_vec(&vec![invalid]).unwrap()).is_err());
    }

    #[test]
    fn semantic_shadow_replay_controls_failed_missing_waits_blocked_and_premature_stop() {
        let mut controls = vec![];
        for (index, result) in ["fail", "not_run", "pass", "pass"].iter().enumerate() {
            let mut item = row();
            item["decision_id"] = json!(format!("control-{index}"));
            item["evidence"]["required_checks"][0]["result"] = json!(result);
            if index == 2 {
                item["evidence"]["waiting"] = json!(true);
            }
            if index == 3 {
                item["evidence"]["blocked"] = json!(true);
            }
            controls.push(item);
        }
        let mut premature = row();
        premature["decision_id"] = json!("premature");
        premature["evidence"]["reason"] = json!("incomplete_todos");
        premature["evidence"]["fingerprint_changed"] = json!(true);
        controls.push(premature);
        let mut unknown = row();
        unknown["decision_id"] = json!("unknown");
        unknown["evidence"]["required_checks_available"] = json!(false);
        unknown["evidence"]["required_checks_complete"] = json!(false);
        unknown["evidence"]["required_checks"] = json!([]);
        unknown["verdict"] = json!("uncertain");
        controls.push(unknown);
        let report = replay_shadow_json(&serde_json::to_vec(&controls).unwrap()).unwrap();
        assert_eq!(report.missed_required_checks, 2);
        assert_eq!(report.hypothetical_redundant_turns, 2);
        assert_eq!(report.unknown_required_checks, 1);
        assert_eq!(report.uncertain, 1);
    }
}

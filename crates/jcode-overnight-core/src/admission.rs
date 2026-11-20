//! Evidence-gated coordinator turns. A turn starts only after an `admitted`
//! decision citing a mission start, a wake boundary, a new signal or remaining
//! eligible work. Pure so the supervisor and its tests share one rule.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AdmissionVerdict {
    Admitted,
    Waiting,
    Blocked,
}

impl AdmissionVerdict {
    pub fn label(self) -> &'static str {
        match self {
            Self::Admitted => "admitted",
            Self::Waiting => "waiting",
            Self::Blocked => "blocked",
        }
    }
}

/// Outcome of the deterministic task-admission policy for this run. A signal or
/// credential never upgrades it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TaskGate {
    /// Run carries no persisted task authorization (legacy manual run).
    NotApplicable,
    Ready,
    Waiting(String),
    Blocked(String),
}

#[derive(Debug, Clone, Default)]
pub struct AdmissionInput<'a> {
    pub first_turn: bool,
    /// Wake/handoff boundary that needs its prompt, e.g. `target_wake`.
    pub boundary: Option<&'a str>,
    /// New worker/background completion or user message since the last turn.
    pub signal: Option<&'a str>,
    /// Task cards still `active` or `deferred` after the previous turn.
    pub eligible_tasks: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AdmissionEvent {
    pub verdict: AdmissionVerdict,
    pub reason: String,
    /// What the decision cites: `mission`, a boundary, a signal or `tasks:N`.
    pub cited: Option<String>,
}

pub fn decide_admission(gate: &TaskGate, input: &AdmissionInput<'_>) -> AdmissionEvent {
    let event = |verdict, reason: &str, cited: Option<String>| AdmissionEvent {
        verdict,
        reason: reason.to_string(),
        cited,
    };
    match gate {
        TaskGate::Blocked(reason) => return event(AdmissionVerdict::Blocked, reason, None),
        TaskGate::Waiting(reason) if input.boundary.is_none() => {
            return event(AdmissionVerdict::Waiting, reason, None);
        }
        _ => {}
    }
    if let Some(boundary) = input.boundary {
        event(
            AdmissionVerdict::Admitted,
            "Wake boundary reached",
            Some(boundary.to_string()),
        )
    } else if input.first_turn {
        event(
            AdmissionVerdict::Admitted,
            "Run mission start",
            Some("mission".into()),
        )
    } else if let Some(signal) = input.signal {
        event(
            AdmissionVerdict::Admitted,
            "New signal arrived",
            Some(signal.to_string()),
        )
    } else if input.eligible_tasks > 0 {
        event(
            AdmissionVerdict::Admitted,
            "Eligible task remains",
            Some(format!("tasks:{}", input.eligible_tasks)),
        )
    } else {
        event(
            AdmissionVerdict::Waiting,
            "No eligible work and no new signal; waiting for a signal or the wake boundary",
            None,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn input() -> AdmissionInput<'static> {
        AdmissionInput::default()
    }

    #[test]
    fn idle_run_waits_and_cites_nothing() {
        let d = decide_admission(&TaskGate::NotApplicable, &input());
        assert_eq!(d.verdict, AdmissionVerdict::Waiting);
        assert_eq!(d.cited, None);
    }

    #[test]
    fn each_admitted_decision_cites_its_evidence() {
        let gate = TaskGate::NotApplicable;
        let first = decide_admission(
            &gate,
            &AdmissionInput {
                first_turn: true,
                ..input()
            },
        );
        assert_eq!(first.cited.as_deref(), Some("mission"));
        let signal = decide_admission(
            &gate,
            &AdmissionInput {
                signal: Some("background:abc"),
                ..input()
            },
        );
        assert_eq!(signal.cited.as_deref(), Some("background:abc"));
        let task = decide_admission(
            &gate,
            &AdmissionInput {
                eligible_tasks: 2,
                ..input()
            },
        );
        assert_eq!(task.cited.as_deref(), Some("tasks:2"));
        let wake = decide_admission(
            &gate,
            &AdmissionInput {
                boundary: Some("target_wake"),
                ..input()
            },
        );
        assert_eq!(wake.cited.as_deref(), Some("target_wake"));
    }

    #[test]
    fn signal_or_boundary_never_overrides_a_blocking_task_gate() {
        let gate = TaskGate::Blocked("Task is cancelled".into());
        let d = decide_admission(
            &gate,
            &AdmissionInput {
                first_turn: true,
                signal: Some("x"),
                boundary: Some("b"),
                eligible_tasks: 3,
            },
        );
        assert_eq!(d.verdict, AdmissionVerdict::Blocked);
        assert_eq!(d.reason, "Task is cancelled");
    }

    #[test]
    fn waiting_task_gate_defers_signals_but_not_wake_boundaries() {
        let gate = TaskGate::Waiting("Existing live attempt a".into());
        let signal = decide_admission(
            &gate,
            &AdmissionInput {
                signal: Some("x"),
                ..input()
            },
        );
        assert_eq!(signal.verdict, AdmissionVerdict::Waiting);
        let wake = decide_admission(
            &gate,
            &AdmissionInput {
                boundary: Some("target_wake"),
                ..input()
            },
        );
        assert_eq!(wake.verdict, AdmissionVerdict::Admitted);
    }
}

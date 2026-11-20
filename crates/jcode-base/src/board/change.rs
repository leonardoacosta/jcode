//! The one allowed write: change an issue's workflow state.
//!
//! Order inside [`execute`] (every step before dispatch fails closed):
//! 1. ADO scope: refused, before anything else. No audit record, no provider
//!    call. ([`Confirmation::confirm`] refuses it too, so an ADO change can
//!    never even reach a confirmation.)
//! 2. The confirmation must match the request exactly.
//! 3. The world is re-resolved now and must equal the confirmed scope.
//! 4. The live state is read and must equal the expected prior state.
//! 5. An intent record is appended. If that fails, nothing is dispatched.
//! 6. `set_issue_state` is called once.
//!
//! A transport-level failure is `Uncertain`. There is no automatic retry:
//! Linear's idempotency contract for `save_issue` is UNVERIFIED, so a blind
//! retry could apply twice or overwrite a newer state. [`reconcile`] reads
//! the live state and records what happened; a further change needs a new
//! confirmation and therefore a new operation id.
//!
//! Only a state id can be written: the request has no field for labels,
//! assignees, comments or any other issue content, and
//! [`BoardProvider`] has no method that could carry one.

use super::audit::{
    AuditEvent, AuditLog, AuditResult, ConfirmationRecord, IntentRecord, OutcomeRecord,
};
use super::binding::{BoardScope, BoardUnavailable};
use super::provider::{BoardError, BoardProvider};
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StatusChangeRequest {
    pub scope: BoardScope,
    pub issue_id: String,
    /// Workflow-state id the user saw the issue in.
    pub expected_prior_state: String,
    pub requested_state: String,
}

/// Why a change was refused. No provider call and no audit record happened.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Refusal {
    /// ADO boards are read-only in this version.
    ReadOnly,
    /// The confirmation was given for a different change.
    ConfirmationMismatch,
    /// Requested state equals the expected prior state.
    NoChange,
    /// The active world no longer matches the confirmed binding.
    WorldChanged(Option<BoardUnavailable>),
}

impl fmt::Display for Refusal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ReadOnly => write!(f, "ADO boards are read-only in this version"),
            Self::ConfirmationMismatch => {
                write!(f, "confirmation does not match this status change")
            }
            Self::NoChange => write!(f, "issue is already expected to be in that state"),
            Self::WorldChanged(Some(why)) => write!(f, "world changed before write: {why}"),
            Self::WorldChanged(None) => write!(f, "world changed before write"),
        }
    }
}

/// An explicit user confirmation of exactly one status change.
///
/// Fields are private and the only constructor is [`Confirmation::confirm`],
/// so holding one means the UI asked the user about this exact issue, world,
/// binding, prior state and requested state. Declining or dismissing means
/// never calling `confirm`: no call, no write. [`execute`] consumes it, and
/// each `confirm` mints a new operation id, so a duplicate confirmation is a
/// new operation, never a replay.
#[derive(Debug, PartialEq, Eq)]
pub struct Confirmation {
    op_id: String,
    confirmed_at: String,
    request: StatusChangeRequest,
}

impl Confirmation {
    /// Record the user's confirmation of `request`. Refused for ADO scopes
    /// and for a no-op change, so those can never be offered for
    /// confirmation.
    pub fn confirm(request: &StatusChangeRequest) -> Result<Self, Refusal> {
        precheck(request)?;
        Ok(Self {
            op_id: uuid::Uuid::new_v4().to_string(),
            confirmed_at: now(),
            request: request.clone(),
        })
    }

    pub fn op_id(&self) -> &str {
        &self.op_id
    }
}

fn precheck(request: &StatusChangeRequest) -> Result<(), Refusal> {
    if request.scope.read_only() {
        return Err(Refusal::ReadOnly);
    }
    if request.requested_state == request.expected_prior_state {
        return Err(Refusal::NoChange);
    }
    Ok(())
}

/// A dispatched change whose result is unknown. Stays attributed to its
/// original scope and operation id whatever world is active later.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PendingOp {
    pub op_id: String,
    pub scope: BoardScope,
    pub issue_id: String,
    pub expected_prior_state: String,
    pub requested_state: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChangeOutcome {
    Refused(Refusal),
    /// The live state could not be read; nothing was written or audited.
    ReadFailed(BoardError),
    /// Live state differs from the expected prior state; nothing written.
    Conflict {
        live_state: String,
    },
    /// The intent record could not be appended; nothing was dispatched.
    AuditFailed(String),
    Applied {
        op_id: String,
        /// Set when the outcome record could not be appended.
        outcome_unrecorded: Option<String>,
    },
    Failed {
        op_id: String,
        error: BoardError,
        outcome_unrecorded: Option<String>,
    },
    /// Result unknown. Do not retry automatically; call [`reconcile`].
    Uncertain {
        pending: PendingOp,
        error: BoardError,
        outcome_unrecorded: Option<String>,
    },
}

fn now() -> String {
    chrono::Utc::now().to_rfc3339()
}

fn record_outcome(audit: &AuditLog, op_id: &str, result: AuditResult) -> Option<String> {
    audit
        .append(&AuditEvent::Outcome(OutcomeRecord {
            op_id: op_id.to_string(),
            ts: now(),
            result,
        }))
        .err()
        .map(|error| error.to_string())
}

/// Perform one confirmed status change. See the module docs for the order of
/// checks. `current_scope` re-resolves the active world at call time (for
/// example `|| resolve_board(&cwd)`).
pub async fn execute(
    request: &StatusChangeRequest,
    confirmation: Confirmation,
    provider: &dyn BoardProvider,
    audit: &AuditLog,
    current_scope: &(dyn Fn() -> Result<BoardScope, BoardUnavailable> + Sync),
) -> ChangeOutcome {
    if let Err(refusal) = precheck(request) {
        return ChangeOutcome::Refused(refusal);
    }
    if confirmation.request != *request {
        return ChangeOutcome::Refused(Refusal::ConfirmationMismatch);
    }
    match current_scope() {
        Ok(now_scope) if now_scope == request.scope => {}
        Ok(_) => return ChangeOutcome::Refused(Refusal::WorldChanged(None)),
        Err(why) => return ChangeOutcome::Refused(Refusal::WorldChanged(Some(why))),
    }
    let live = match provider
        .issue_state(&request.scope, &request.issue_id)
        .await
    {
        Ok(live) => live,
        Err(error) => return ChangeOutcome::ReadFailed(error),
    };
    if live != request.expected_prior_state {
        return ChangeOutcome::Conflict { live_state: live };
    }

    let op_id = confirmation.op_id;
    let intent = AuditEvent::Intent(IntentRecord {
        op_id: op_id.clone(),
        ts: now(),
        world: request.scope.world.clone(),
        backend: request.scope.backend.as_str().to_string(),
        team: request.scope.team.clone(),
        project: request.scope.project.clone(),
        issue_id: request.issue_id.clone(),
        expected_prior_state: request.expected_prior_state.clone(),
        requested_state: request.requested_state.clone(),
        confirmation: ConfirmationRecord {
            confirmed_at: confirmation.confirmed_at,
        },
    });
    if let Err(error) = audit.append(&intent) {
        return ChangeOutcome::AuditFailed(error.to_string());
    }

    // Re-resolve the world once more, immediately before dispatch: the live
    // read and the audit append above can take a while. A change here is
    // recorded (the intent is already in the log) and nothing is sent.
    let refusal = match current_scope() {
        Ok(now_scope) if now_scope == request.scope => None,
        Ok(_) => Some(Refusal::WorldChanged(None)),
        Err(why) => Some(Refusal::WorldChanged(Some(why))),
    };
    if let Some(refusal) = refusal {
        let _ = record_outcome(audit, &op_id, AuditResult::Failed);
        return ChangeOutcome::Refused(refusal);
    }

    match provider
        .set_issue_state(&request.scope, &request.issue_id, &request.requested_state)
        .await
    {
        Ok(()) => {
            let outcome_unrecorded = record_outcome(audit, &op_id, AuditResult::Applied);
            ChangeOutcome::Applied {
                op_id,
                outcome_unrecorded,
            }
        }
        Err(error @ BoardError::Uncertain(_)) => {
            let outcome_unrecorded = record_outcome(audit, &op_id, AuditResult::Uncertain);
            ChangeOutcome::Uncertain {
                pending: PendingOp {
                    op_id,
                    scope: request.scope.clone(),
                    issue_id: request.issue_id.clone(),
                    expected_prior_state: request.expected_prior_state.clone(),
                    requested_state: request.requested_state.clone(),
                },
                error,
                outcome_unrecorded,
            }
        }
        Err(error) => {
            let outcome_unrecorded = record_outcome(audit, &op_id, AuditResult::Failed);
            ChangeOutcome::Failed {
                op_id,
                error,
                outcome_unrecorded,
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReconcileOutcome {
    /// Live state is the requested state.
    Applied,
    /// Live state is still the expected prior state. Nothing was retried.
    NotApplied,
    /// Live state is neither. Recorded as not applied.
    Diverged { live_state: String },
    /// Could not read the issue; the operation stays uncertain, no record.
    Unknown(BoardError),
}

/// Read the live state of a pending operation (read-only, through its
/// original scope) and append a `Reconciled` outcome. Never writes to the
/// tracker and never retries. A new attempt needs a new [`Confirmation`].
pub async fn reconcile(
    pending: &PendingOp,
    provider: &dyn BoardProvider,
    audit: &AuditLog,
) -> (ReconcileOutcome, Option<String>) {
    let live = match provider
        .issue_state(&pending.scope, &pending.issue_id)
        .await
    {
        Ok(live) => live,
        Err(error) => return (ReconcileOutcome::Unknown(error), None),
    };
    let outcome = if live == pending.requested_state {
        ReconcileOutcome::Applied
    } else if live == pending.expected_prior_state {
        ReconcileOutcome::NotApplied
    } else {
        ReconcileOutcome::Diverged { live_state: live }
    };
    let applied = outcome == ReconcileOutcome::Applied;
    let unrecorded = record_outcome(audit, &pending.op_id, AuditResult::Reconciled { applied });
    (outcome, unrecorded)
}

#[cfg(test)]
#[path = "change_tests.rs"]
mod tests;

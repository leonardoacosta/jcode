//! World-bound issue board: binding, scoped provider reads, cache, audit and
//! the constrained status-change contract.
//!
//! Spec: `openspec/changes/bind-world-issue-board/`. Everything here fails
//! closed: an unresolved, ambiguous or refused world yields no provider call,
//! and there is no fallback to another binding, backend, cache entry or
//! last-used scope.

mod audit;
mod binding;
mod cache;
mod change;
mod linear;
mod provider;

pub use audit::{
    AuditEvent, AuditLog, AuditResult, ConfirmationRecord, IntentRecord, OutcomeRecord,
};
pub use binding::{BoardScope, BoardUnavailable, resolve_board, resolve_board_from};
pub use cache::{BoardCache, BoardSnapshot, CacheKey, auth_identity};
pub use change::{
    ChangeOutcome, Confirmation, PendingOp, ReconcileOutcome, Refusal, StatusChangeRequest,
    execute, reconcile,
};
pub use jcode_config_types::BoardBackend;
pub use linear::{LinearMcpProvider, McpCaller, McpManagerCaller, ToolError};
pub use provider::{
    AdoUnavailableProvider, BoardError, BoardIssue, BoardProvider, WorkflowState, provider_for,
};

#[cfg(test)]
pub(crate) fn test_scope(backend: BoardBackend) -> BoardScope {
    BoardScope {
        world: "w".into(),
        backend,
        team: "team-1".into(),
        project: None,
        server: "Linear".into(),
        canonical_root: std::path::PathBuf::from("/r"),
    }
}

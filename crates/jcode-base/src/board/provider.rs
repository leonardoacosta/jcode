//! Backend-neutral board reads and the single allowed write.

use super::binding::BoardScope;
use async_trait::async_trait;
use jcode_config_types::BoardBackend;
use std::fmt;
use std::sync::Arc;

#[derive(Debug, Clone, PartialEq)]
pub struct WorkflowState {
    pub id: String,
    pub name: String,
    pub position: f64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BoardIssue {
    pub id: String,
    pub identifier: String,
    pub title: String,
    pub state_id: String,
}

/// Why a provider call did not succeed. None of these switches provider,
/// backend or binding.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BoardError {
    /// Authentication or permission failure.
    Unauthorized(String),
    /// No adapter, or the provider cannot be reached.
    Unavailable(String),
    /// The provider answered with an error or an unusable body.
    Provider(String),
    /// A write was sent but its result is unknown (transport failure).
    Uncertain(String),
}

impl fmt::Display for BoardError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unauthorized(m) => write!(f, "unauthorized: {m}"),
            Self::Unavailable(m) => write!(f, "unavailable: {m}"),
            Self::Provider(m) => write!(f, "provider error: {m}"),
            Self::Uncertain(m) => write!(f, "uncertain result: {m}"),
        }
    }
}

impl std::error::Error for BoardError {}

/// Order columns the way the provider orders workflow states: by `position`,
/// ties and NaN keep provider order (stable sort).
pub(super) fn sort_states(states: &mut [WorkflowState]) {
    states.sort_by(|a, b| a.position.total_cmp(&b.position));
}

#[async_trait]
pub trait BoardProvider: Send + Sync {
    /// Workflow states of the bound team, in provider order.
    async fn workflow_states(&self, scope: &BoardScope) -> Result<Vec<WorkflowState>, BoardError>;
    /// Issues inside the binding only (team, and project when bound).
    async fn issues(&self, scope: &BoardScope) -> Result<Vec<BoardIssue>, BoardError>;
    /// Live workflow-state id of one issue.
    async fn issue_state(&self, scope: &BoardScope, issue_id: &str) -> Result<String, BoardError>;
    /// The only write: set the workflow state of one issue.
    async fn set_issue_state(
        &self,
        scope: &BoardScope,
        issue_id: &str,
        state_id: &str,
    ) -> Result<(), BoardError>;
}

/// ADO has no adapter in this build. Every call fails `Unavailable`.
pub struct AdoUnavailableProvider;

const NO_ADO: &str = "no ADO adapter in this build";

#[async_trait]
impl BoardProvider for AdoUnavailableProvider {
    async fn workflow_states(&self, _: &BoardScope) -> Result<Vec<WorkflowState>, BoardError> {
        Err(BoardError::Unavailable(NO_ADO.into()))
    }
    async fn issues(&self, _: &BoardScope) -> Result<Vec<BoardIssue>, BoardError> {
        Err(BoardError::Unavailable(NO_ADO.into()))
    }
    async fn issue_state(&self, _: &BoardScope, _: &str) -> Result<String, BoardError> {
        Err(BoardError::Unavailable(NO_ADO.into()))
    }
    async fn set_issue_state(&self, _: &BoardScope, _: &str, _: &str) -> Result<(), BoardError> {
        Err(BoardError::Unavailable(NO_ADO.into()))
    }
}

/// The provider for a scope's backend. `linear` is used only for Linear
/// scopes; an ADO scope always gets [`AdoUnavailableProvider`], never Linear.
pub fn provider_for(scope: &BoardScope, linear: Arc<dyn BoardProvider>) -> Arc<dyn BoardProvider> {
    match scope.backend {
        BoardBackend::Linear => linear,
        BoardBackend::Ado => Arc::new(AdoUnavailableProvider),
    }
}

#[cfg(test)]
#[path = "provider_tests.rs"]
mod tests;

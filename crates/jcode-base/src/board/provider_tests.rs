use super::*;

use crate::board::test_scope as scope;

fn state(id: &str, position: f64) -> WorkflowState {
    WorkflowState {
        id: id.into(),
        name: id.into(),
        position,
        kind: String::new(),
    }
}

#[test]
fn states_sort_by_position_then_provider_order() {
    let mut states = vec![
        state("c", 2.0),
        state("a", 0.0),
        state("b2", 1.0),
        state("b1", 1.0),
    ];
    sort_states(&mut states);
    let ids: Vec<_> = states.iter().map(|s| s.id.as_str()).collect();
    assert_eq!(ids, ["a", "b2", "b1", "c"]);
}

struct Panics;

#[async_trait]
impl BoardProvider for Panics {
    async fn workflow_states(&self, _: &BoardScope) -> Result<Vec<WorkflowState>, BoardError> {
        panic!("linear must not be called for ADO")
    }
    async fn issues(&self, _: &BoardScope) -> Result<Vec<BoardIssue>, BoardError> {
        panic!("linear must not be called for ADO")
    }
    async fn issue_state(&self, _: &BoardScope, _: &str) -> Result<String, BoardError> {
        panic!("linear must not be called for ADO")
    }
    async fn set_issue_state(&self, _: &BoardScope, _: &str, _: &str) -> Result<(), BoardError> {
        panic!("linear must not be called for ADO")
    }
}

#[tokio::test]
async fn ado_scope_never_reaches_linear_and_is_unavailable() {
    let scope = scope(BoardBackend::Ado);
    let provider = provider_for(&scope, Arc::new(Panics));
    assert!(
        matches!(provider.workflow_states(&scope).await, Err(BoardError::Unavailable(m)) if m == "no ADO adapter in this build")
    );
    assert!(
        matches!(provider.issues(&scope).await, Err(BoardError::Unavailable(m)) if m == "no ADO adapter in this build")
    );
    assert!(matches!(
        provider.issue_state(&scope, "i").await,
        Err(BoardError::Unavailable(_))
    ));
    assert!(matches!(
        provider.set_issue_state(&scope, "i", "s").await,
        Err(BoardError::Unavailable(_))
    ));
}

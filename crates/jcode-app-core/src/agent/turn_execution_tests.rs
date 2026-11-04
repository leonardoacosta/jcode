use super::*;
use async_trait::async_trait;
use std::sync::atomic::{AtomicUsize, Ordering};

#[tokio::test]
async fn plan_mode_nonempty_retry_does_not_reset_judge_budget() {
    use crate::session::judge_feedback::{self, FeedbackAdmission};
    let _home = crate::auth::test_sandbox::AuthTestSandbox::new().unwrap();
    let (mut agent, _) = mock_agent().await;
    let parent = agent.session_id().to_owned();
    let child = || {
        let mut s = crate::session::Session::create(Some(parent.clone()), None);
        s.worker_role = Some("judge".into());
        s.save().unwrap();
        s.id
    };
    judge_feedback::begin_user_turn_once(&parent, "plan_same").unwrap();
    assert!(matches!(
        judge_feedback::admit(&parent, &child(), "CONTINUE: spent", 1).unwrap(),
        Some(FeedbackAdmission::Continue(_))
    ));
    let _guard = crate::tool::PlanTurnGuard::new(&parent);
    judge_feedback::begin_user_turn_once(&parent, "plan_same").unwrap();
    agent
        .append_user_context_message("nonempty retry", vec![])
        .unwrap();
    assert!(matches!(
        judge_feedback::admit(&parent, &child(), "CONTINUE: no refund", 1).unwrap(),
        Some(FeedbackAdmission::Note(_))
    ));
}

#[tokio::test]
async fn judge_feedback_budget_resets_only_for_genuine_user_turns() {
    use crate::session::judge_feedback::{self, FeedbackAdmission};
    let _home = crate::auth::test_sandbox::AuthTestSandbox::new().unwrap();
    let (mut agent, calls) = mock_agent().await;
    let child = |parent: &str| {
        let mut judge = crate::session::Session::create(Some(parent.into()), None);
        judge.worker_role = Some("judge".into());
        judge.save().unwrap();
        judge
    };
    let parent = agent.session_id().to_string();
    let first = child(&parent);
    assert!(matches!(
        judge_feedback::admit(&parent, &first.id, "CONTINUE: first", 1).unwrap(),
        Some(FeedbackAdmission::Continue(_))
    ));
    for (display, continues) in [
        (Some(crate::session::StoredDisplayRole::System), false),
        (None, true),
    ] {
        agent
            .append_user_context_message_with_display_role(
                "automatic continuation",
                vec![],
                display,
                continues,
            )
            .unwrap();
        assert!(matches!(
            judge_feedback::admit(&parent, &child(&parent).id, "CONTINUE: capped", 1).unwrap(),
            Some(FeedbackAdmission::Note(_))
        ));
    }
    agent
        .append_user_context_message("genuine input", vec![])
        .unwrap();
    assert!(matches!(
        judge_feedback::admit(&parent, &child(&parent).id, "CONTINUE: new turn", 1).unwrap(),
        Some(FeedbackAdmission::Continue(_))
    ));
    assert!(matches!(
        judge_feedback::admit(&parent, &first.id, "CONTINUE: old duplicate", 1).unwrap(),
        Some(FeedbackAdmission::Note(_))
    ));
    agent.queue_soft_interrupt(
        "automatic DM".into(),
        vec![],
        false,
        jcode_agent_runtime::SoftInterruptSource::System,
    );
    agent.inject_soft_interrupts();
    assert!(matches!(
        judge_feedback::admit(&parent, &child(&parent).id, "CONTINUE: system keeps cap", 1)
            .unwrap(),
        Some(FeedbackAdmission::Note(_))
    ));
    agent.queue_soft_interrupt(
        "genuine user interleave".into(),
        vec![],
        false,
        jcode_agent_runtime::SoftInterruptSource::User,
    );
    agent.inject_soft_interrupts();
    assert!(matches!(
        judge_feedback::admit(
            &parent,
            &child(&parent).id,
            "CONTINUE: interleave resets",
            1
        )
        .unwrap(),
        Some(FeedbackAdmission::Continue(_))
    ));
    assert_eq!(calls.load(Ordering::SeqCst), 0);
}

#[derive(Clone)]
struct MockProvider {
    calls: Arc<AtomicUsize>,
}

#[async_trait]
impl Provider for MockProvider {
    async fn complete(
        &self,
        _: &[Message],
        _: &[ToolDefinition],
        _: &str,
        _: Option<&str>,
    ) -> Result<crate::provider::EventStream> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        Ok(Box::pin(futures::stream::iter(vec![
            Ok(StreamEvent::TextDelta("mock answer".into())),
            Ok(StreamEvent::MessageEnd {
                stop_reason: Some("end_turn".into()),
            }),
        ])))
    }
    fn name(&self) -> &str {
        "mock"
    }
    fn model(&self) -> String {
        "mock-model".into()
    }
    fn fork(&self) -> Arc<dyn Provider> {
        Arc::new(self.clone())
    }
}

async fn mock_agent() -> (Agent, Arc<AtomicUsize>) {
    let calls = Arc::new(AtomicUsize::new(0));
    let provider: Arc<dyn Provider> = Arc::new(MockProvider {
        calls: calls.clone(),
    });
    let registry = Registry::new(provider.clone()).await;
    let mut agent = Agent::new(provider, registry);
    agent.session.is_debug = false;
    (agent, calls)
}

fn texts(agent: &Agent, role: Role) -> Vec<String> {
    agent
        .session
        .messages
        .iter()
        .filter(|m| m.role == role)
        .flat_map(|m| m.content.iter())
        .filter_map(|b| match b {
            ContentBlock::Text { text, .. } => Some(text.clone()),
            _ => None,
        })
        .collect()
}

#[tokio::test]
async fn run_once_capture_returns_streamed_text_and_records_turn() {
    let _home = crate::auth::test_sandbox::AuthTestSandbox::new().unwrap();
    let (mut agent, calls) = mock_agent().await;
    let out = agent.run_once_capture("hello there").await.unwrap();
    assert!(out.contains("mock answer"), "{out}");
    assert_eq!(calls.load(Ordering::SeqCst), 1);
    assert!(
        texts(&agent, Role::User)
            .iter()
            .any(|t| t.contains("hello there"))
    );
    assert!(
        texts(&agent, Role::Assistant)
            .iter()
            .any(|t| t.contains("mock answer"))
    );
}

#[tokio::test]
async fn append_user_context_message_persists_without_model_call() {
    let _home = crate::auth::test_sandbox::AuthTestSandbox::new().unwrap();
    let (mut agent, calls) = mock_agent().await;
    let before = agent.message_count();
    agent
        .append_user_context_message("context only", vec![("image/png".into(), "AAAA".into())])
        .unwrap();
    assert_eq!(calls.load(Ordering::SeqCst), 0);
    assert_eq!(agent.message_count(), before + 1);
    let last = agent.session.messages.last().unwrap();
    assert_eq!(last.role, Role::User);
    assert!(matches!(last.content[0], ContentBlock::Image { .. }));
    assert!(
        matches!(&last.content[1], ContentBlock::Text { text, .. } if text.contains("context only"))
    );
}

#[tokio::test]
async fn continue_turn_after_failover_runs_streaming_turn() {
    let _home = crate::auth::test_sandbox::AuthTestSandbox::new().unwrap();
    let (mut agent, calls) = mock_agent().await;
    agent
        .append_user_context_message("retry me", Vec::new())
        .unwrap();
    let (tx, _rx) = mpsc::unbounded_channel();
    agent
        .continue_turn_after_failover(Some("ignored reminder".into()), tx)
        .await
        .unwrap();
    assert_eq!(calls.load(Ordering::SeqCst), 1);
    assert!(
        texts(&agent, Role::Assistant)
            .iter()
            .any(|t| t.contains("mock answer"))
    );
    assert!(
        !texts(&agent, Role::User)
            .iter()
            .any(|t| t.contains("ignored reminder"))
    );
}

#[tokio::test]
async fn continue_turn_after_failover_noops_on_graceful_shutdown() {
    let _home = crate::auth::test_sandbox::AuthTestSandbox::new().unwrap();
    let (mut agent, calls) = mock_agent().await;
    agent.request_graceful_shutdown();
    let before = agent.message_count();
    let (tx, _rx) = mpsc::unbounded_channel();
    agent.continue_turn_after_failover(None, tx).await.unwrap();
    assert_eq!(calls.load(Ordering::SeqCst), 0);
    assert_eq!(agent.message_count(), before);
}

#[tokio::test]
async fn fresh_session_has_valid_lease_and_is_not_stale() {
    let _home = crate::auth::test_sandbox::AuthTestSandbox::new().unwrap();
    let (agent, _) = mock_agent().await;
    agent.ensure_session_lease().unwrap();
    assert!(!agent.session_copy_is_stale());
}

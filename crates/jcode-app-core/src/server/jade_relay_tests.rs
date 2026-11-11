use super::*;

#[test]
fn relay_listener_config_is_opt_in_and_requires_credentials() {
    let cfg = SafetyConfig::default();
    assert!(RelayListenerConfig::from_safety(&cfg).is_none());

    let cfg = SafetyConfig {
        jade_relay_enabled: true,
        jade_relay_reply_enabled: true,
        ..SafetyConfig::default()
    };
    assert!(RelayListenerConfig::from_safety(&cfg).is_none());
}

#[test]
fn relay_listener_config_accepts_complete_opt_in_config() {
    let cfg = SafetyConfig {
        jade_relay_enabled: true,
        jade_relay_reply_enabled: true,
        jade_relay_api_base: Some("https://example.com/api".to_string()),
        jade_relay_token: Some("tok".to_string()),
        jade_relay_token_id: Some("alice-token".to_string()),
        jade_relay_user_id: Some("alice".to_string()),
        jade_relay_session_id: Some("sess-1".to_string()),
        ..SafetyConfig::default()
    };
    let parsed = RelayListenerConfig::from_safety(&cfg).expect("complete config");
    assert_eq!(parsed.api.api_base, "https://example.com/api/");
    assert_eq!(parsed.api.token, "tok");
    assert_eq!(parsed.api.token_id.as_deref(), Some("alice-token"));
    assert_eq!(parsed.api.user_id.as_deref(), Some("alice"));
    assert_eq!(parsed.session_id, "sess-1");
}

#[test]
fn relay_launch_config_is_separately_opt_in() {
    let cfg = SafetyConfig {
        jade_relay_enabled: true,
        jade_relay_api_base: Some("https://example.com".to_string()),
        jade_relay_token: Some("tok".to_string()),
        ..SafetyConfig::default()
    };
    assert!(RelayLaunchConfig::from_safety(&cfg).is_none());

    let cfg = SafetyConfig {
        jade_relay_launch_enabled: true,
        jade_relay_launch_working_dir: Some("/tmp/project".to_string()),
        ..cfg
    };
    let parsed = RelayLaunchConfig::from_safety(&cfg).expect("launch opt-in config");
    assert_eq!(parsed.api.api_base, "https://example.com/");
    assert_eq!(parsed.default_working_dir.as_deref(), Some("/tmp/project"));
}

#[test]
fn launch_request_reads_structured_data() {
    let event = RelayEvent {
        seq: 7,
        event_type: "launch".to_string(),
        text: Some("hello from web".to_string()),
        data: Some(serde_json::json!({
            "working_dir": "/tmp/repo",
            "model": "openai:gpt-test",
            "provider": "openai",
            "selfdev": true,
        })),
    };
    let parsed = LaunchRequest::from_event(&event, Some("/fallback")).expect("launch request");
    assert_eq!(parsed.text, "hello from web");
    assert_eq!(parsed.working_dir.as_deref(), Some("/tmp/repo"));
    assert_eq!(parsed.model.as_deref(), Some("openai:gpt-test"));
    assert_eq!(parsed.provider_key.as_deref(), Some("openai"));
    assert!(parsed.selfdev);
}

#[test]
fn relay_session_listener_polls_prompt_and_cancel_commands() {
    assert_eq!(session_command_event_types_param(), "types=prompt,cancel");
}

#[test]
fn relay_event_type_defaults_to_prompt_for_legacy_events() {
    let legacy = RelayEvent {
        seq: 1,
        event_type: String::new(),
        text: Some("hello".to_string()),
        data: None,
    };
    assert_eq!(legacy.event_type(), "prompt");

    let cancel = RelayEvent {
        event_type: "cancel".to_string(),
        ..legacy
    };
    assert_eq!(cancel.event_type(), "cancel");
}

#[test]
fn relay_url_encoding_matches_jade_api_expectations() {
    assert_eq!(urlencoding_encode("sess-relay-test"), "sess-relay-test");
    assert_eq!(urlencoding_encode("a/b c"), "a%2Fb%20c");
    assert_eq!(urlencoding_encode("user.name~1_2"), "user.name~1_2");
}

#[test]
fn truncation_preserves_short_text_and_marks_long_text() {
    assert_eq!(truncate_chars("hello", 10), "hello");
    assert_eq!(truncate_chars("abcdef", 4), "abc…");
}

#[test]
fn pending_queue_is_fifo_and_drains_once() {
    let id = "test-pending-fifo";
    assert_eq!(enqueue_pending(id, "a".into()), (1, None));
    assert_eq!(enqueue_pending(id, "b".into()), (2, None));
    assert_eq!(take_pending(id), vec!["a".to_string(), "b".to_string()]);
    assert!(take_pending(id).is_empty());
}

#[test]
fn pending_queue_drops_oldest_past_bound() {
    let id = "test-pending-bound";
    for i in 0..PENDING_QUEUE_LIMIT {
        assert_eq!(enqueue_pending(id, i.to_string()).1, None);
    }
    let (len, dropped) = enqueue_pending(id, "new".into());
    assert_eq!((len, dropped.as_deref()), (PENDING_QUEUE_LIMIT, Some("0")));
    let drained = take_pending(id);
    assert_eq!(drained.first().map(String::as_str), Some("1"));
    assert_eq!(drained.last().map(String::as_str), Some("new"));
}

struct DrainTestProvider;

#[async_trait::async_trait]
impl crate::provider::Provider for DrainTestProvider {
    async fn complete(
        &self,
        _messages: &[crate::message::Message],
        _tools: &[crate::message::ToolDefinition],
        _system: &str,
        _resume_session_id: Option<&str>,
    ) -> Result<crate::provider::EventStream> {
        Ok(Box::pin(futures::stream::iter(vec![
            Ok(crate::message::StreamEvent::TextDelta("ok".to_string())),
            Ok(crate::message::StreamEvent::MessageEnd { stop_reason: None }),
        ])))
    }
    fn name(&self) -> &str {
        "test"
    }
    fn fork(&self) -> Arc<dyn crate::provider::Provider> {
        Arc::new(Self)
    }
}

#[tokio::test]
async fn pending_messages_drain_as_one_turn_after_busy_turn_ends() {
    let provider: Arc<dyn crate::provider::Provider> = Arc::new(DrainTestProvider);
    let registry = crate::tool::Registry::new(provider.clone()).await;
    let agent = Arc::new(tokio::sync::Mutex::new(crate::agent::Agent::new(
        provider, registry,
    )));
    let swarm_members = Arc::new(RwLock::new(HashMap::new()));
    let id = "test-turn-end-drain";

    // Simulate the in-flight turn holding the agent lock.
    let busy = agent.lock().await;
    let before = busy.message_count();
    assert_eq!(enqueue_pending(id, "first queued".into()).0, 1);
    spawn_pending_drain(
        id.to_string(),
        Arc::clone(&agent),
        Arc::clone(&swarm_members),
    );
    assert_eq!(enqueue_pending(id, "second queued".into()).0, 2);
    tokio::time::sleep(std::time::Duration::from_millis(50)).await;
    assert_eq!(busy.message_count(), before, "must not run while busy");
    drop(busy);

    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
    loop {
        let done = {
            let a = agent.lock().await;
            a.message_count() >= before + 2 && take_pending(id).is_empty()
        };
        if done {
            break;
        }
        assert!(std::time::Instant::now() < deadline, "drain never ran");
        tokio::time::sleep(std::time::Duration::from_millis(20)).await;
    }
    let history = format!("{:?}", agent.lock().await.get_history());
    let first = history.find("first queued").expect("first delivered");
    let second = history.find("second queued").expect("second delivered");
    assert!(first < second, "FIFO order preserved in drained turn");
}

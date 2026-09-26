use super::*;
use crate::{
    message::{Message, StreamEvent, ToolDefinition},
    provider::{EventStream, Provider},
};
use anyhow::Result;
use async_stream::stream;
use async_trait::async_trait;
use std::{
    collections::VecDeque,
    sync::{Arc, Mutex},
};

#[derive(Clone, Default)]
struct StreamingTestProvider {
    calls: Arc<Mutex<Vec<(String, String, Vec<Message>)>>>,
    responses: Arc<Mutex<VecDeque<Vec<StreamEvent>>>>,
    fail: bool,
    delay: std::time::Duration,
}

impl StreamingTestProvider {
    fn responding(events: Vec<StreamEvent>) -> Self {
        Self {
            responses: Arc::new(Mutex::new(VecDeque::from([events]))),
            ..Self::default()
        }
    }
}

#[async_trait]
impl Provider for StreamingTestProvider {
    async fn complete(
        &self,
        messages: &[Message],
        _tools: &[ToolDefinition],
        system: &str,
        _resume: Option<&str>,
    ) -> Result<EventStream> {
        self.calls
            .lock()
            .unwrap()
            .push((system.to_owned(), String::new(), messages.to_vec()));
        if self.fail {
            anyhow::bail!("intentional provider failure")
        }
        let events = self
            .responses
            .lock()
            .unwrap()
            .pop_front()
            .unwrap_or_default();
        Ok(Box::pin(
            stream! { for event in events { yield Ok(event); } },
        ))
    }
    async fn complete_split(
        &self,
        messages: &[Message],
        _tools: &[ToolDefinition],
        static_prompt: &str,
        dynamic_prompt: &str,
        _resume: Option<&str>,
    ) -> Result<EventStream> {
        self.calls.lock().unwrap().push((
            static_prompt.to_owned(),
            dynamic_prompt.to_owned(),
            messages.to_vec(),
        ));
        if self.fail {
            anyhow::bail!("intentional provider failure")
        }
        if !self.delay.is_zero() {
            tokio::time::sleep(self.delay).await;
        }
        let events = self
            .responses
            .lock()
            .unwrap()
            .pop_front()
            .unwrap_or_default();
        Ok(Box::pin(
            stream! { for event in events { yield Ok(event); } },
        ))
    }
    fn name(&self) -> &str {
        "automation-test"
    }
    fn fork(&self) -> Arc<dyn Provider> {
        Arc::new(self.clone())
    }
}

// Isolate both JCODE_HOME and environment-secret discovery for each test.
struct TestEnv {
    _lock: std::sync::MutexGuard<'static, ()>,
    _home: EnvVarGuard,
}
struct EnvVarGuard {
    key: &'static str,
    old: Option<std::ffi::OsString>,
}
impl EnvVarGuard {
    fn set(key: &'static str, value: &std::path::Path) -> Self {
        let old = std::env::var_os(key);
        crate::env::set_var(key, value);
        Self { key, old }
    }
}
impl Drop for EnvVarGuard {
    fn drop(&mut self) {
        if let Some(old) = self.old.take() {
            crate::env::set_var(self.key, old);
        } else {
            crate::env::remove_var(self.key);
        }
    }
}
async fn setup() -> (
    TestEnv,
    tempfile::TempDir,
    Arc<tokio::sync::Mutex<Store>>,
    Automation,
) {
    let lock = crate::storage::lock_test_env();
    let temp = tempfile::tempdir().unwrap();
    let home = EnvVarGuard::set("JCODE_HOME", temp.path());
    let root = temp.path().join("workspace");
    std::fs::create_dir_all(root.join(".jcode/skills/demo")).unwrap();
    std::fs::write(
        root.join(".jcode/skills/demo/SKILL.md"),
        "---\nname: demo\ndescription: test\n---\nDo the requested test task.",
    )
    .unwrap();
    let store = Arc::new(tokio::sync::Mutex::new(
        Store::open(temp.path().join("state.json")).unwrap(),
    ));
    let now = Utc::now() - chrono::Duration::seconds(120);
    let automation = Automation {
        id: "automation-test".into(),
        skill: "demo".into(),
        arguments: "arg-value".into(),
        working_dir: root,
        schedule: crate::automations::schedule::Schedule::Interval { seconds: 60 },
        enabled: true,
        created_at: now,
        next_due: now,
        provider: None,
        model: None,
        error: None,
    };
    store
        .lock()
        .await
        .create(automation.clone(), temp.path())
        .unwrap();
    (
        TestEnv {
            _lock: lock,
            _home: home,
        },
        temp,
        store,
        automation,
    )
}

async fn claim(store: &tokio::sync::Mutex<Store>) -> Run {
    store.lock().await.claim_due(Utc::now()).unwrap().unwrap().1
}

#[tokio::test]
async fn executes_skill_in_workspace_and_records_session_source_and_output() {
    let (_env, _temp, store, automation) = setup().await;
    let claimed = claim(&store).await;
    {
        let provider = StreamingTestProvider::responding(vec![
            StreamEvent::TextDelta("automation complete".into()),
            StreamEvent::MessageEnd { stop_reason: None },
        ]);
        let (out, status) = execute(
            &automation,
            &claimed,
            store.clone(),
            Arc::new(provider.clone()),
            CancellationToken::new(),
        )
        .await
        .unwrap();
        assert_eq!(status, RunStatus::Success);
        assert_eq!(out, "automation complete");
        let (_static_prompt, dynamic_prompt, messages) =
            provider.calls.lock().unwrap().first().cloned().unwrap();
        assert!(
            dynamic_prompt.contains("# Active Skill")
                && dynamic_prompt.contains("# Skill: demo")
                && dynamic_prompt.contains("Do the requested test task."),
            "active skill missing from dynamic system prompt: {dynamic_prompt}"
        );
        assert!(
            messages
                .iter()
                .any(|message| format!("{:?}", message.content).contains("arg-value")),
            "automation arguments missing from user request"
        );
        let db = store.lock().await;
        let recorded = db.runs().iter().find(|r| r.id == claimed.id).unwrap();
        let session_id = recorded.session_id.as_deref().expect("session ID recorded");
        assert!(
            recorded.skill_source.as_deref().unwrap().starts_with(
                automation
                    .working_dir
                    .join(".jcode/skills/demo/SKILL.md")
                    .to_str()
                    .unwrap()
            )
        );
        let skill =
            crate::skill::SkillRegistry::load_for_working_dir(Some(&automation.working_dir))
                .unwrap();
        let skill = skill.get("demo").unwrap();
        assert!(
            recorded
                .skill_source
                .as_deref()
                .unwrap()
                .ends_with(&sha256(&skill.content))
        );
        assert_eq!(
            crate::session::Session::load(session_id)
                .unwrap()
                .working_dir
                .as_deref(),
            automation.working_dir.to_str()
        );
    }
}

#[tokio::test]
async fn provider_failure_is_an_error_not_success() {
    let (_env, _temp, store, automation) = setup().await;
    let claimed = claim(&store).await;
    let provider = StreamingTestProvider {
        fail: true,
        ..Default::default()
    };
    let err = execute(
        &automation,
        &claimed,
        store,
        Arc::new(provider),
        CancellationToken::new(),
    )
    .await
    .unwrap_err();
    assert!(
        err.to_string().contains("intentional provider failure")
            || err.to_string().contains("automation agent turn failed")
    );
}

#[tokio::test]
async fn missing_working_directory_pauses_automation() {
    let (_env, temp, store, automation) = setup().await;
    let claimed = claim(&store).await;
    let mut automation = automation;
    automation.working_dir = temp.path().join("missing");
    let error = execute(
        &automation,
        &claimed,
        store.clone(),
        Arc::new(StreamingTestProvider::default()),
        CancellationToken::new(),
    )
    .await
    .unwrap_err();
    assert!(
        error
            .to_string()
            .contains("working directory is unavailable")
    );
}

#[tokio::test]
async fn missing_skill_is_reported_without_provider_call() {
    let (_env, _temp, store, mut automation) = setup().await;
    let claimed = claim(&store).await;
    automation.skill = "not-installed".into();
    let provider = StreamingTestProvider::default();
    let error = execute(
        &automation,
        &claimed,
        store,
        Arc::new(provider.clone()),
        CancellationToken::new(),
    )
    .await
    .unwrap_err();
    assert!(error.to_string().contains("skill unavailable"));
    assert!(provider.calls.lock().unwrap().is_empty());
}

#[tokio::test]
async fn cancellation_acknowledges_interruption_and_keeps_run_slot_claimed() {
    let (_env, _temp, store, automation) = setup().await;
    let claimed = claim(&store).await;
    let token = CancellationToken::new();
    let provider = StreamingTestProvider {
        delay: std::time::Duration::from_millis(100),
        ..StreamingTestProvider::responding(vec![
            StreamEvent::TextDelta("late output".into()),
            StreamEvent::MessageEnd { stop_reason: None },
        ])
    };
    let automation_for_task = automation.clone();
    let claimed_for_task = claimed.clone();
    let store_for_task = store.clone();
    let run_token = token.clone();
    let task = tokio::spawn(async move {
        execute(
            &automation_for_task,
            &claimed_for_task,
            store_for_task,
            Arc::new(provider),
            run_token,
        )
        .await
    });
    tokio::time::sleep(std::time::Duration::from_millis(10)).await;
    token.cancel();
    let (_, status) = task.await.unwrap().unwrap();
    assert_eq!(status, RunStatus::Interrupted);
    assert!(
        store
            .lock()
            .await
            .claim_due(Utc::now() + chrono::Duration::seconds(1))
            .unwrap()
            .is_none()
    );
}

#[tokio::test]
async fn ask_user_event_marks_run_blocked() {
    let (_env, _temp, store, automation) = setup().await;
    let claimed = claim(&store).await;
    let provider = StreamingTestProvider::responding(vec![
        StreamEvent::ToolUseStart {
            id: "ask1".into(),
            name: "ask_user_question".into(),
        },
        StreamEvent::ToolInputDelta("{}".into()),
        StreamEvent::ToolUseEnd,
        StreamEvent::MessageEnd { stop_reason: None },
    ]);
    let (_, status) = execute(
        &automation,
        &claimed,
        store,
        Arc::new(provider),
        CancellationToken::new(),
    )
    .await
    .unwrap();
    assert_eq!(status, RunStatus::Blocked);
}

#[tokio::test]
async fn unsupported_provider_selection_fails_before_provider_call() {
    let (_env, _temp, store, mut automation) = setup().await;
    let claimed = claim(&store).await;
    automation.provider = Some("provider-that-does-not-exist".into());
    let provider = StreamingTestProvider::default();
    let error = execute(
        &automation,
        &claimed,
        store,
        Arc::new(provider.clone()),
        CancellationToken::new(),
    )
    .await
    .unwrap_err();
    assert!(error.to_string().contains("unknown automation provider"));
    assert!(provider.calls.lock().unwrap().is_empty());
}

#[tokio::test]
async fn listener_retries_after_initial_bind_failure() {
    let _env_lock = crate::storage::lock_test_env();
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().canonicalize().unwrap();
    let occupied = tokio::net::TcpListener::bind(("127.0.0.1", 0))
        .await
        .unwrap();
    let port = occupied.local_addr().unwrap().port();
    let mut cfg = config::Config::new(root, temp.path().join("socket"), port);
    cfg.control_token = "a".repeat(64);
    let store = Arc::new(tokio::sync::Mutex::new(
        Store::open(temp.path().join("listener-state.json")).unwrap(),
    ));
    let cancel = CancellationToken::new();
    let task = tokio::spawn(listener_retry(cfg, store, cancel.clone()));
    tokio::time::sleep(std::time::Duration::from_millis(50)).await;
    drop(occupied);
    let deadline = tokio::time::Instant::now() + std::time::Duration::from_secs(3);
    loop {
        if tokio::net::TcpStream::connect(("127.0.0.1", port))
            .await
            .is_ok()
        {
            break;
        }
        assert!(
            tokio::time::Instant::now() < deadline,
            "listener did not recover after bind failure"
        );
        tokio::time::sleep(std::time::Duration::from_millis(25)).await;
    }
    cancel.cancel();
    task.await.unwrap().unwrap();
}

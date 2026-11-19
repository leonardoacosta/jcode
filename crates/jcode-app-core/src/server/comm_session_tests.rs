#![cfg_attr(test, allow(clippy::await_holding_lock))]

use super::{
    CoordinatorSpawnIdentity, ensure_spawn_coordinator_swarm, prepare_visible_spawn_session,
    register_visible_spawned_member, resolve_coordinator_spawn_identity, resolve_spawn_working_dir,
    resolve_stop_target_session, resolve_swarm_spawn_selection, spawn_admission_lock,
    swarm_stop_allowed_by_owner,
};
use crate::agent::Agent;
use crate::message::{Message, ToolDefinition};
use crate::protocol::{NotificationType, ServerEvent};
use crate::provider::{EventStream, Provider};
use crate::server::{SwarmEventType, SwarmMember, VersionedPlan};
use crate::tool::Registry;
use anyhow::Result;
use async_trait::async_trait;
use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::Arc;
use std::sync::atomic::AtomicU64;
use std::time::Instant;
use tokio::sync::{Mutex, RwLock, broadcast, mpsc};

#[test]
fn reviewer_startup_uses_automatic_final_report_without_extra_tool_roundtrip() {
    let prompt = "Review exact source and checks.";
    let review = super::swarm_startup_message(prompt, Some("reviewer"));
    assert!(review.starts_with(prompt));
    assert!(review.contains("automatically records your final assistant response"));
    assert!(review.contains("Do not call the swarm report tool"));
    assert!(!review.contains("Before finishing, call the swarm tool"));
    assert_eq!(super::swarm_startup_message(prompt, None), prompt);
    assert_eq!(super::swarm_startup_message(prompt, Some("edit")), prompt);
}

struct MockProvider;

struct RoutingProvider(std::sync::Mutex<String>);

#[async_trait]
impl Provider for RoutingProvider {
    async fn complete(
        &self,
        _: &[Message],
        _: &[ToolDefinition],
        _: &str,
        _: Option<&str>,
    ) -> Result<EventStream> {
        anyhow::bail!("routing regression must not make model calls")
    }
    fn name(&self) -> &str {
        "mock"
    }
    fn model(&self) -> String {
        self.0.lock().unwrap().clone()
    }
    fn set_model(&self, model: &str) -> Result<()> {
        *self.0.lock().unwrap() = model
            .rsplit_once(':')
            .map(|(_, model)| model)
            .unwrap_or(model)
            .to_string();
        Ok(())
    }
    fn fork(&self) -> Arc<dyn Provider> {
        Arc::new(Self(std::sync::Mutex::new(self.model())))
    }
}

#[async_trait]
impl Provider for MockProvider {
    async fn complete(
        &self,
        _messages: &[Message],
        _tools: &[ToolDefinition],
        _system: &str,
        _resume_session_id: Option<&str>,
    ) -> Result<EventStream> {
        Err(anyhow::anyhow!("mock provider should not be called"))
    }

    fn name(&self) -> &str {
        "mock"
    }

    fn fork(&self) -> Arc<dyn Provider> {
        Arc::new(MockProvider)
    }
}

fn member(
    session_id: &str,
    swarm_id: Option<&str>,
    role: &str,
) -> (SwarmMember, mpsc::UnboundedReceiver<ServerEvent>) {
    let (event_tx, event_rx) = mpsc::unbounded_channel();
    (
        SwarmMember {
            session_id: session_id.to_string(),
            event_tx,
            event_txs: HashMap::new(),
            working_dir: None,
            swarm_id: swarm_id.map(|id| id.to_string()),
            swarm_enabled: true,
            status: "ready".to_string(),
            detail: None,
            friendly_name: Some(session_id.to_string()),
            report_back_to_session_id: None,
            latest_completion_report: None,
            role: role.to_string(),
            joined_at: Instant::now(),
            last_status_change: Instant::now(),
            is_headless: false,
            output_tail: None,
            todo_progress: None,
            todo_items: Vec::new(),
            runtime: crate::protocol::SwarmMemberRuntime::default(),
            task_label: None,
        },
        event_rx,
    )
}

async fn test_agent_with_working_dir(session_id: &str, working_dir: &str) -> Arc<Mutex<Agent>> {
    let provider: Arc<dyn Provider> = Arc::new(MockProvider);
    let registry = Registry::new(provider.clone()).await;
    let mut session = crate::session::Session::create_with_id(session_id.to_string(), None, None);
    session.model = Some("mock".to_string());
    session.working_dir = Some(working_dir.to_string());
    let mut agent = Agent::new_with_session(provider, registry, session, None);
    agent.set_working_dir(working_dir);
    Arc::new(Mutex::new(agent))
}

#[tokio::test]
async fn resolve_spawn_working_dir_prefers_explicit_then_spawner_agent_dir() {
    let sessions = Arc::new(crate::server::session_manager::SessionManager::from_agents(
        HashMap::new(),
    ));
    sessions
        .register_new(
            "req".to_string(),
            test_agent_with_working_dir("req", "/tmp/spawner-agent").await,
        )
        .await
        .unwrap();
    let swarm_members = Arc::new(RwLock::new(HashMap::new()));

    assert_eq!(
        resolve_spawn_working_dir(
            Some("/tmp/explicit".to_string()),
            "req",
            &sessions,
            &swarm_members,
        )
        .await
        .as_deref(),
        Some("/tmp/explicit")
    );
    assert_eq!(
        resolve_spawn_working_dir(None, "req", &sessions, &swarm_members)
            .await
            .as_deref(),
        Some("/tmp/spawner-agent")
    );
}

#[tokio::test]
async fn resolve_spawn_working_dir_falls_back_to_member_dir() {
    let sessions = Arc::new(crate::server::session_manager::SessionManager::from_agents(
        HashMap::new(),
    ));
    let swarm_members = Arc::new(RwLock::new(HashMap::new()));
    let (mut req_member, _rx) = member("req", Some("swarm-1"), "coordinator");
    req_member.working_dir = Some(std::path::PathBuf::from("/tmp/member-dir"));
    swarm_members
        .write()
        .await
        .insert("req".to_string(), req_member);

    assert_eq!(
        resolve_spawn_working_dir(None, "req", &sessions, &swarm_members)
            .await
            .as_deref(),
        Some("/tmp/member-dir")
    );
}

#[test]
fn stop_permission_defaults_to_sessions_spawned_by_requesting_coordinator() {
    let (mut owned, _owned_rx) = member("worker-owned", Some("swarm-1"), "agent");
    owned.report_back_to_session_id = Some("coord".to_string());
    let (mut user_created, _user_rx) = member("worker-user", Some("swarm-1"), "agent");
    user_created.report_back_to_session_id = None;
    let (mut other_owned, _other_rx) = member("worker-other", Some("swarm-1"), "agent");
    other_owned.report_back_to_session_id = Some("other-coord".to_string());

    assert!(swarm_stop_allowed_by_owner("coord", &owned, false));
    assert!(!swarm_stop_allowed_by_owner("coord", &user_created, false));
    assert!(!swarm_stop_allowed_by_owner("coord", &other_owned, false));
    assert!(swarm_stop_allowed_by_owner("coord", &user_created, true));
}

#[tokio::test]
async fn stop_target_resolves_unique_friendly_name_and_suffix() {
    let swarm_members = Arc::new(RwLock::new(HashMap::new()));
    let (mut worker, _worker_rx) = member("session_jellyfish_1234_abcd", Some("swarm-1"), "agent");
    worker.friendly_name = Some("jellyfish".to_string());
    swarm_members
        .write()
        .await
        .insert(worker.session_id.clone(), worker);

    assert_eq!(
        resolve_stop_target_session("swarm-1", "jellyfish", &swarm_members)
            .await
            .as_deref(),
        Ok("session_jellyfish_1234_abcd")
    );
    assert_eq!(
        resolve_stop_target_session("swarm-1", "abcd", &swarm_members)
            .await
            .as_deref(),
        Ok("session_jellyfish_1234_abcd")
    );
}

#[tokio::test]
async fn stop_target_rejects_ambiguous_friendly_name() {
    let swarm_members = Arc::new(RwLock::new(HashMap::new()));
    let (mut first, _first_rx) = member("session_bear_1", Some("swarm-1"), "agent");
    first.friendly_name = Some("bear".to_string());
    let (mut second, _second_rx) = member("session_bear_2", Some("swarm-1"), "agent");
    second.friendly_name = Some("bear".to_string());
    let mut members = swarm_members.write().await;
    members.insert(first.session_id.clone(), first);
    members.insert(second.session_id.clone(), second);
    drop(members);

    let err = resolve_stop_target_session("swarm-1", "bear", &swarm_members)
        .await
        .expect_err("ambiguous friendly names should be rejected");
    assert!(err.contains("Ambiguous swarm session 'bear'"));
}

#[tokio::test]
async fn register_visible_spawned_member_marks_startup_as_running() {
    let swarm_members = Arc::new(RwLock::new(HashMap::new()));
    let swarms_by_id = Arc::new(RwLock::new(HashMap::new()));
    let event_history = Arc::new(RwLock::new(VecDeque::new()));
    let event_counter = Arc::new(AtomicU64::new(0));
    let (swarm_event_tx, _swarm_event_rx) = broadcast::channel(8);

    register_visible_spawned_member(
        "child-1",
        "swarm-1",
        Some("/tmp/worktree"),
        true,
        Some("owner"),
        &swarm_members,
        &swarms_by_id,
        &event_history,
        &event_counter,
        &swarm_event_tx,
    )
    .await;

    let members = swarm_members.read().await;
    let member = members.get("child-1").expect("spawned member should exist");
    assert_eq!(member.status, "running");
    assert_eq!(member.detail.as_deref(), Some("startup queued"));
    assert_eq!(member.swarm_id.as_deref(), Some("swarm-1"));
    assert_eq!(
        member.working_dir.as_deref(),
        Some(std::path::Path::new("/tmp/worktree"))
    );
    drop(members);

    assert!(
        swarms_by_id
            .read()
            .await
            .get("swarm-1")
            .is_some_and(|members| members.contains("child-1"))
    );

    let history = event_history.read().await;
    assert!(history.iter().any(|event| {
            event.session_id == "child-1"
                && matches!(event.event, SwarmEventType::MemberChange { ref action } if action == "joined")
        }));
}

#[test]
fn judge_feedback_visible_spawn_records_parent_before_launch() {
    let _home = crate::auth::test_sandbox::AuthTestSandbox::new().unwrap();
    let (id, launched) = prepare_visible_spawn_session(
        None,
        None,
        None,
        None,
        None,
        false,
        None,
        None,
        Some("judge"),
        Some("judge-parent"),
        |id, _, _, _| {
            let session = crate::session::Session::load_startup_stub(id).unwrap();
            assert_eq!(session.worker_role.as_deref(), Some("judge"));
            assert_eq!(session.parent_id.as_deref(), Some("judge-parent"));
            Ok(true)
        },
    )
    .unwrap();
    assert!(launched);
    assert!(!id.is_empty());
}

#[test]
fn prepare_visible_council_spawn_persists_policy_and_startup_before_launch() {
    let _guard = crate::storage::lock_test_env();
    let temp_home = tempfile::TempDir::new().expect("temp home");
    crate::env::set_var("JCODE_HOME", temp_home.path());

    let worktree = tempfile::TempDir::new().expect("temp worktree");
    let startup = "Please start by auditing prompt delivery.";

    let (session_id, launched) = prepare_visible_spawn_session(
        Some(worktree.path().to_str().expect("utf8 worktree path")),
        None,
        None,
        None,
        None,
        false,
        Some(startup),
        Some("council-visible-test"),
        None,
        None,
        |session_id, _cwd: &std::path::Path, _selfdev, provider_key| {
            assert_eq!(provider_key, None);
            // This is the disk boundary used by a separately launched client.
            let session = crate::session::Session::load_startup_stub(session_id).unwrap();
            assert_eq!(session.council_id.as_deref(), Some("council-visible-test"));
            let provider: Arc<dyn Provider> = Arc::new(MockProvider);
            let _agent = Agent::new_with_session(provider, Registry::empty(), session, None);
            assert_eq!(
                crate::tool::session_tool_policy_allows_tool_for_test(session_id, "edit"),
                Some(false)
            );
            assert_eq!(
                crate::tool::session_tool_policy_allows_tool_for_test(session_id, "read"),
                Some(true)
            );
            let path = crate::storage::jcode_dir()
                .expect("jcode dir")
                .join(format!("client-input-{}", session_id));
            let data = std::fs::read_to_string(&path).expect("startup file should exist");
            assert!(
                data.contains(startup),
                "startup payload should be written before launch"
            );
            assert!(
                data.contains(r#""submit_on_restore":true"#),
                "startup payload should auto-submit on restore"
            );
            Ok(true)
        },
    )
    .expect("visible spawn preparation should succeed");

    assert!(launched);
    let path = crate::storage::jcode_dir()
        .expect("jcode dir")
        .join(format!("client-input-{}", session_id));
    assert!(
        path.exists(),
        "startup file should remain for launched visible session"
    );

    crate::env::remove_var("JCODE_HOME");
}

#[test]
fn prepare_visible_research_spawn_enforces_policy_in_new_process() {
    let _guard = crate::storage::lock_test_env();
    let temp_home = tempfile::TempDir::new().unwrap();
    let previous_home = std::env::var_os("JCODE_HOME");
    crate::env::set_var("JCODE_HOME", temp_home.path());
    let result = prepare_visible_spawn_session(
        Some(temp_home.path().to_str().unwrap()),
        None,
        None,
        None,
        None,
        false,
        None,
        None,
        Some("research"),
        None,
        |session_id, _, _, _| {
            for mode in ["startup", "remote", "full"] {
                let output = std::process::Command::new(std::env::current_exe().unwrap())
                    .args([
                        "--exact",
                        "agent::tests::research_executor_rejects_escape_tools_and_allows_read",
                    ])
                    .env("JCODE_HOME", temp_home.path())
                    .env("JCODE_RESEARCH_SESSION", session_id)
                    .env("JCODE_RESEARCH_LOAD_MODE", mode)
                    .output()?;
                anyhow::ensure!(
                    output.status.success(),
                    "{mode}: {}{}",
                    String::from_utf8_lossy(&output.stdout),
                    String::from_utf8_lossy(&output.stderr)
                );
            }
            Ok(true)
        },
    );
    match previous_home {
        Some(home) => crate::env::set_var("JCODE_HOME", home),
        None => crate::env::remove_var("JCODE_HOME"),
    }
    assert!(result.unwrap().1);
}

#[test]
fn prepare_visible_spawn_session_cleans_startup_when_launch_not_started() {
    let _guard = crate::storage::lock_test_env();
    let temp_home = tempfile::TempDir::new().expect("temp home");
    crate::env::set_var("JCODE_HOME", temp_home.path());

    let worktree = tempfile::TempDir::new().expect("temp worktree");

    let (session_id, launched) = prepare_visible_spawn_session(
        Some(worktree.path().to_str().expect("utf8 worktree path")),
        None,
        None,
        None,
        None,
        false,
        Some("Do the thing."),
        None,
        None,
        None,
        |session_id, _cwd: &std::path::Path, _selfdev, _provider_key| {
            crate::session_env::set_session_env(
                session_id,
                HashMap::from([("SPAWN_SECRET".into(), "test-only".into())]),
            )
            .unwrap();
            Ok(false)
        },
    )
    .expect("visible spawn preparation should succeed even when launch is skipped");

    assert!(!launched);
    assert!(crate::session_env::session_env(&session_id).is_empty());
    let path = crate::storage::jcode_dir()
        .expect("jcode dir")
        .join(format!("client-input-{}", session_id));
    assert!(
        !path.exists(),
        "startup file should be removed when visible launch does not start"
    );
    assert!(
        !crate::session::session_exists(&session_id),
        "prepared session should be cleaned up when visible launch does not start"
    );

    crate::env::remove_var("JCODE_HOME");
}

#[test]
fn prepare_visible_spawn_session_cleans_session_when_launch_errors() {
    let _guard = crate::storage::lock_test_env();
    let temp_home = tempfile::TempDir::new().expect("temp home");
    crate::env::set_var("JCODE_HOME", temp_home.path());

    let worktree = tempfile::TempDir::new().expect("temp worktree");

    let error = prepare_visible_spawn_session(
        Some(worktree.path().to_str().expect("utf8 worktree path")),
        None,
        None,
        None,
        None,
        false,
        Some("Do the thing."),
        None,
        None,
        None,
        |_session_id, _cwd: &std::path::Path, _selfdev, _provider_key| {
            Err(anyhow::anyhow!("launch failed"))
        },
    )
    .expect_err("visible spawn preparation should surface launch error");

    assert!(error.to_string().contains("launch failed"));
    let sessions_dir = crate::storage::jcode_dir()
        .expect("jcode dir")
        .join("sessions");
    let remaining_sessions = std::fs::read_dir(&sessions_dir)
        .map(|entries| entries.count())
        .unwrap_or(0);
    assert_eq!(
        remaining_sessions, 0,
        "failed visible launch should not leave orphan prepared sessions"
    );

    crate::env::remove_var("JCODE_HOME");
}

#[test]
fn prepare_visible_spawn_session_persists_and_launches_provider_key_for_openrouter_model() {
    let _guard = crate::storage::lock_test_env();
    let temp_home = tempfile::TempDir::new().expect("temp home");
    crate::env::set_var("JCODE_HOME", temp_home.path());

    let worktree = tempfile::TempDir::new().expect("temp worktree");
    let (session_id, launched) = prepare_visible_spawn_session(
        Some(worktree.path().to_str().expect("utf8 worktree path")),
        Some("openai/gpt-5.4@OpenAI"),
        None,
        None,
        None,
        false,
        None,
        None,
        None,
        None,
        |_session_id, _cwd: &std::path::Path, _selfdev, provider_key| {
            assert_eq!(provider_key, Some("openrouter"));
            Ok(true)
        },
    )
    .expect("visible spawn preparation should succeed");

    assert!(launched);
    let session = crate::session::Session::load(&session_id).expect("prepared session should save");
    assert_eq!(session.model.as_deref(), Some("openai/gpt-5.4@OpenAI"));
    assert_eq!(session.provider_key.as_deref(), Some("openrouter"));

    crate::env::remove_var("JCODE_HOME");
}

#[test]
fn prepare_visible_spawn_session_persists_requested_effort() {
    let _guard = crate::storage::lock_test_env();
    let temp_home = tempfile::TempDir::new().expect("temp home");
    crate::env::set_var("JCODE_HOME", temp_home.path());

    let worktree = tempfile::TempDir::new().expect("temp worktree");
    let (session_id, launched) = prepare_visible_spawn_session(
        Some(worktree.path().to_str().expect("utf8 worktree path")),
        Some("gpt-5.5"),
        None,
        None,
        Some("low"),
        false,
        None,
        None,
        None,
        None,
        |_session_id, _cwd: &std::path::Path, _selfdev, _provider_key| Ok(true),
    )
    .expect("visible spawn preparation should succeed");

    assert!(launched);
    let session = crate::session::Session::load(&session_id).expect("prepared session should save");
    assert_eq!(session.model.as_deref(), Some("gpt-5.5"));
    assert_eq!(
        session.reasoning_effort.as_deref(),
        Some("low"),
        "requested effort should persist so the headed client restores it"
    );

    crate::env::remove_var("JCODE_HOME");
}

#[test]
fn prepare_visible_spawn_session_prefers_parent_provider_key_over_model_guess() {
    let _guard = crate::storage::lock_test_env();
    let temp_home = tempfile::TempDir::new().expect("temp home");
    crate::env::set_var("JCODE_HOME", temp_home.path());

    let worktree = tempfile::TempDir::new().expect("temp worktree");
    let (session_id, launched) = prepare_visible_spawn_session(
        Some(worktree.path().to_str().expect("utf8 worktree path")),
        Some("gpt-5.4"),
        Some("ollama"),
        None,
        None,
        false,
        None,
        None,
        None,
        None,
        |_session_id, _cwd: &std::path::Path, _selfdev, provider_key| {
            assert_eq!(provider_key, Some("ollama"));
            Ok(true)
        },
    )
    .expect("visible spawn preparation should succeed");

    assert!(launched);
    let session = crate::session::Session::load(&session_id).expect("prepared session should save");
    assert_eq!(session.model.as_deref(), Some("gpt-5.4"));
    assert_eq!(session.provider_key.as_deref(), Some("ollama"));

    crate::env::remove_var("JCODE_HOME");
}

fn coordinator_identity(
    model: Option<&str>,
    provider_key: Option<&str>,
    route_api_method: Option<&str>,
) -> CoordinatorSpawnIdentity {
    CoordinatorSpawnIdentity {
        model: model.map(str::to_string),
        provider_key: provider_key.map(str::to_string),
        route_api_method: route_api_method.map(str::to_string),
        is_canary: false,
    }
}

#[test]
fn resolve_swarm_spawn_model_prefers_configured_model_over_coordinator_model() {
    let selection = resolve_swarm_spawn_selection(
        None,
        Some("openai/gpt-5.4@OpenAI".to_string()),
        &coordinator_identity(
            Some("nvidia/llama-3.3-nemotron-super-49b-v1"),
            Some("nvidia"),
            Some("openai-compatible:nvidia-nim"),
        ),
    );

    assert_eq!(selection.model.as_deref(), Some("openai/gpt-5.4@OpenAI"));
    assert_eq!(selection.provider_key.as_deref(), Some("openrouter"));
    // A different configured model must not inherit the coordinator's route.
    assert_eq!(selection.route_api_method, None);
}

#[test]
fn resolve_swarm_spawn_model_inherits_coordinator_when_unconfigured() {
    let selection = resolve_swarm_spawn_selection(
        None,
        None,
        &coordinator_identity(
            Some("nvidia/llama-3.3-nemotron-super-49b-v1"),
            Some("nvidia"),
            Some("openai-compatible:nvidia-nim"),
        ),
    );

    assert_eq!(
        selection.model.as_deref(),
        Some("nvidia/llama-3.3-nemotron-super-49b-v1")
    );
    assert_eq!(selection.provider_key.as_deref(), Some("nvidia"));
    assert_eq!(
        selection.route_api_method.as_deref(),
        Some("openai-compatible:nvidia-nim")
    );
}

#[test]
fn resolve_swarm_spawn_model_inherits_coordinator_auth_route_for_oauth_vs_api() {
    // Regression: a coordinator on the Claude API route must spawn agents on
    // the same API route, not Claude OAuth (the config default).
    let selection = resolve_swarm_spawn_selection(
        None,
        None,
        &coordinator_identity(
            Some("claude-opus-4-6"),
            Some("claude-api"),
            Some("claude-api"),
        ),
    );

    assert_eq!(selection.model.as_deref(), Some("claude-opus-4-6"));
    assert_eq!(selection.provider_key.as_deref(), Some("claude-api"));
    assert_eq!(selection.route_api_method.as_deref(), Some("claude-api"));
}

#[test]
fn resolve_swarm_spawn_model_keeps_provider_key_when_config_matches_coordinator() {
    let selection = resolve_swarm_spawn_selection(
        None,
        Some("custom-model".to_string()),
        &coordinator_identity(
            Some("custom-model"),
            Some("custom-provider"),
            Some("custom-route"),
        ),
    );

    assert_eq!(selection.model.as_deref(), Some("custom-model"));
    assert_eq!(selection.provider_key.as_deref(), Some("custom-provider"));
    assert_eq!(selection.route_api_method.as_deref(), Some("custom-route"));
}

#[test]
fn resolve_swarm_spawn_model_openai_api_prefix_pins_api_route_over_coordinator() {
    // `agents.swarm_model = "openai-api:gpt-5.5"` must spawn agents on GPT-5.5
    // via the OpenAI API key route, regardless of the coordinator's model/auth.
    let selection = resolve_swarm_spawn_selection(
        None,
        Some("openai-api:gpt-5.5".to_string()),
        &coordinator_identity(
            Some("claude-opus-4-8"),
            Some("claude-oauth"),
            Some("claude-oauth"),
        ),
    );

    assert_eq!(selection.model.as_deref(), Some("gpt-5.5"));
    assert_eq!(selection.provider_key.as_deref(), Some("openai-api-key"));
    assert_eq!(
        selection.route_api_method.as_deref(),
        Some("openai-api-key")
    );
}

#[test]
fn resolve_swarm_spawn_model_auth_route_prefixes_pin_expected_routes() {
    for (configured, expected_model, expected_key) in [
        ("openai-api:gpt-5.5", "gpt-5.5", "openai-api-key"),
        ("openai-oauth:gpt-5.5", "gpt-5.5", "openai-oauth"),
        (
            "claude-api:claude-opus-4-8",
            "claude-opus-4-8",
            "anthropic-api-key",
        ),
        (
            "claude-oauth:claude-opus-4-8",
            "claude-opus-4-8",
            "claude-oauth",
        ),
    ] {
        let selection = resolve_swarm_spawn_selection(
            None,
            Some(configured.to_string()),
            &coordinator_identity(
                Some("some-other-model"),
                Some("some-key"),
                Some("some-route"),
            ),
        );
        assert_eq!(
            selection.model.as_deref(),
            Some(expected_model),
            "configured {configured:?} model",
        );
        assert_eq!(
            selection.provider_key.as_deref(),
            Some(expected_key),
            "configured {configured:?} provider_key",
        );
        assert_eq!(
            selection.route_api_method.as_deref(),
            Some(expected_key),
            "configured {configured:?} route_api_method",
        );
    }
}

#[test]
fn resolve_swarm_spawn_model_inherit_sentinel_uses_coordinator_model() {
    for sentinel in ["inherit", "INHERIT", "coordinator", " inherit ", ""] {
        let selection = resolve_swarm_spawn_selection(
            None,
            Some(sentinel.to_string()),
            &coordinator_identity(
                Some("nvidia/llama-3.3-nemotron-super-49b-v1"),
                Some("nvidia"),
                Some("openai-compatible:nvidia-nim"),
            ),
        );

        assert_eq!(
            selection.model.as_deref(),
            Some("nvidia/llama-3.3-nemotron-super-49b-v1"),
            "sentinel {sentinel:?} should inherit coordinator model",
        );
        assert_eq!(
            selection.provider_key.as_deref(),
            Some("nvidia"),
            "sentinel {sentinel:?} should inherit coordinator provider key",
        );
        assert_eq!(
            selection.route_api_method.as_deref(),
            Some("openai-compatible:nvidia-nim"),
            "sentinel {sentinel:?} should inherit coordinator auth route",
        );
    }
}

#[test]
fn resolve_swarm_spawn_model_requested_model_overrides_configured_pin() {
    for requested in ["openai-api:gpt-5.5", "  openai-api:gpt-5.5 \t"] {
        let selection = resolve_swarm_spawn_selection(
            Some(requested.to_string()),
            Some("claude-oauth:claude-opus-4-8".to_string()),
            &coordinator_identity(
                Some("claude-fable-5"),
                Some("claude-oauth"),
                Some("claude-oauth"),
            ),
        );

        assert_eq!(selection.model.as_deref(), Some("gpt-5.5"));
        assert_eq!(selection.provider_key.as_deref(), Some("openai-api-key"));
        assert_eq!(
            selection.route_api_method.as_deref(),
            Some("openai-api-key")
        );
    }
}

#[test]
fn resolve_swarm_spawn_model_requested_inherit_overrides_configured_pin() {
    for requested in [
        "inherit",
        "INHERIT",
        "coordinator",
        " COORDINATOR ",
        " inherit ",
    ] {
        let selection = resolve_swarm_spawn_selection(
            Some(requested.to_string()),
            Some("openai-api:gpt-5.5".to_string()),
            &coordinator_identity(
                Some("claude-fable-5"),
                Some("claude-api"),
                Some("claude-api"),
            ),
        );

        assert_eq!(selection.model.as_deref(), Some("claude-fable-5"));
        assert_eq!(selection.provider_key.as_deref(), Some("claude-api"));
        assert_eq!(selection.route_api_method.as_deref(), Some("claude-api"));
    }
}

#[test]
fn resolve_swarm_spawn_model_requested_matching_coordinator_model_keeps_route() {
    let selection = resolve_swarm_spawn_selection(
        Some(" custom-model ".to_string()),
        Some("openai-api:gpt-5.5".to_string()),
        &coordinator_identity(
            Some("custom-model"),
            Some("custom-provider"),
            Some("custom-route"),
        ),
    );

    assert_eq!(selection.model.as_deref(), Some("custom-model"));
    assert_eq!(selection.provider_key.as_deref(), Some("custom-provider"));
    assert_eq!(selection.route_api_method.as_deref(), Some("custom-route"));
}

#[test]
fn resolve_swarm_spawn_model_blank_requested_model_falls_back_to_config() {
    for requested in ["", "   ", "\t\n"] {
        let selection = resolve_swarm_spawn_selection(
            Some(requested.to_string()),
            Some("openai-api:gpt-5.5".to_string()),
            &coordinator_identity(
                Some("claude-fable-5"),
                Some("claude-oauth"),
                Some("claude-oauth"),
            ),
        );

        assert_eq!(selection.model.as_deref(), Some("gpt-5.5"));
        assert_eq!(selection.provider_key.as_deref(), Some("openai-api-key"));
        assert_eq!(
            selection.route_api_method.as_deref(),
            Some("openai-api-key")
        );
    }
}

#[test]
fn resolve_swarm_spawn_model_omitted_request_trims_configured_model() {
    let selection = resolve_swarm_spawn_selection(
        None,
        Some(" \topenai-api:gpt-5.5 \n".to_string()),
        &coordinator_identity(
            Some("claude-fable-5"),
            Some("claude-oauth"),
            Some("claude-oauth"),
        ),
    );

    assert_eq!(selection.model.as_deref(), Some("gpt-5.5"));
    assert_eq!(selection.provider_key.as_deref(), Some("openai-api-key"));
    assert_eq!(
        selection.route_api_method.as_deref(),
        Some("openai-api-key")
    );
}

#[test]
fn resolve_swarm_spawn_model_blank_requested_model_inherits_when_unconfigured() {
    let selection = resolve_swarm_spawn_selection(
        Some(" \t\n".to_string()),
        None,
        &coordinator_identity(
            Some("custom-model"),
            Some("custom-provider"),
            Some("custom-route"),
        ),
    );

    assert_eq!(selection.model.as_deref(), Some("custom-model"));
    assert_eq!(selection.provider_key.as_deref(), Some("custom-provider"));
    assert_eq!(selection.route_api_method.as_deref(), Some("custom-route"));
}

#[tokio::test]
async fn coordinator_identity_uses_live_agent_when_lock_is_available() {
    let agent = test_agent_with_working_dir("coord", "/tmp/coord").await;
    let live_model = agent.lock().await.provider_model();
    let sessions = Arc::new(crate::server::session_manager::SessionManager::from_agents(
        HashMap::new(),
    ));
    sessions
        .register_new("coord".to_string(), Arc::clone(&agent))
        .await
        .unwrap();

    let identity = resolve_coordinator_spawn_identity("coord", &sessions).await;
    assert_eq!(identity.model.as_deref(), Some(live_model.as_str()));
}

#[tokio::test]
async fn coordinator_identity_falls_back_to_persisted_session_when_agent_busy() {
    let _guard = crate::storage::lock_test_env();
    let temp_home = tempfile::TempDir::new().expect("temp home");
    crate::env::set_var("JCODE_HOME", temp_home.path());

    let agent = test_agent_with_working_dir("coord_busy", "/tmp/coord").await;

    // Persist a coordinator session that records a concrete model + auth route.
    // Persist after the agent is built so it reflects the authoritative on-disk
    // snapshot the spawn path will read when the agent lock is unavailable.
    let mut session = crate::session::Session::create_with_id("coord_busy".to_string(), None, None);
    session.model = Some("claude-opus-4-6".to_string());
    session.provider_key = Some("claude-api".to_string());
    session.route_api_method = Some("claude-api".to_string());
    session
        .save_prepared()
        .expect("persist coordinator session");

    // Hold the agent lock to simulate a coordinator mid-turn: the spawn path
    // must not block and must read the persisted identity instead of defaults.
    let sessions = Arc::new(crate::server::session_manager::SessionManager::from_agents(
        HashMap::new(),
    ));
    sessions
        .register_new("coord_busy".to_string(), Arc::clone(&agent))
        .await
        .unwrap();

    let _held = agent.lock().await;
    let identity = resolve_coordinator_spawn_identity("coord_busy", &sessions).await;
    assert_eq!(identity.model.as_deref(), Some("claude-opus-4-6"));
    assert_eq!(identity.provider_key.as_deref(), Some("claude-api"));
    assert_eq!(identity.route_api_method.as_deref(), Some("claude-api"));

    crate::env::remove_var("JCODE_HOME");
}

#[tokio::test]
async fn global_agent_model_request_updates_server_config_not_session() {
    let _env = crate::storage::lock_test_env();
    let temp = tempfile::TempDir::new().unwrap();
    let previous = std::env::var_os("JCODE_HOME");
    crate::env::set_var("JCODE_HOME", temp.path());
    let agent = test_agent_with_working_dir("routing_global", "/tmp/coord").await;
    let overrides = crate::server::client_actions::set_global_agent_model(
        &agent,
        "routing_global",
        "swarm",
        Some("openai-api:gpt-5.5".into()),
    )
    .await
    .unwrap();
    assert!(
        overrides.is_empty(),
        "a global change must not pin the session"
    );
    assert_eq!(
        crate::config::Config::load().agents.swarm_model.as_deref(),
        Some("openai-api:gpt-5.5")
    );
    crate::server::client_actions::set_global_agent_model(&agent, "routing_global", "swarm", None)
        .await
        .unwrap();
    assert_eq!(crate::config::Config::load().agents.swarm_model, None);
    assert!(
        crate::server::client_actions::set_global_agent_model(
            &agent,
            "routing_global",
            "bogus",
            Some("m".into())
        )
        .await
        .is_err()
    );
    if let Some(previous) = previous {
        crate::env::set_var("JCODE_HOME", previous);
    } else {
        crate::env::remove_var("JCODE_HOME");
    }
}

#[tokio::test]
async fn read_only_spawn_preserves_routing_and_policy_while_coordinator_busy() {
    let _env = crate::storage::lock_test_env();
    let temp = tempfile::TempDir::new().unwrap();
    let previous = std::env::var_os("JCODE_HOME");
    crate::env::set_var("JCODE_HOME", temp.path());
    let agent = test_agent_with_working_dir("routing_busy", "/tmp/coord").await;
    let mut session = crate::session::Session::create_with_id("routing_busy".into(), None, None);
    session.model = Some("claude-opus-4-6".into());
    session.provider_key = Some("claude-api".into());
    session.route_api_method = Some("claude-api".into());
    session.save_prepared().unwrap();
    let sessions = Arc::new(crate::server::session_manager::SessionManager::from_agents(
        HashMap::from([("routing_busy".to_string(), agent.clone())]),
    ));
    let held = agent.lock().await;
    tokio::time::timeout(
        std::time::Duration::from_millis(250),
        crate::server::client_actions::set_session_agent_model(
            &agent,
            "routing_busy",
            "swarm",
            Some("openai-api:gpt-5.5".into()),
        ),
    )
    .await
    .expect("setter must not await busy Agent lock")
    .unwrap();
    let coordinator = resolve_coordinator_spawn_identity("routing_busy", &sessions).await;
    let configured =
        super::session_swarm_model("routing_busy", &sessions, Some("global-model".into())).await;
    let selection = resolve_swarm_spawn_selection(None, configured, &coordinator);
    assert_eq!(selection.model.as_deref(), Some("gpt-5.5"));
    assert_eq!(selection.provider_key.as_deref(), Some("openai-api-key"));
    let provider: Arc<dyn Provider> = Arc::new(RoutingProvider(std::sync::Mutex::new(
        "initial-model".into(),
    )));
    let state = crate::server::SwarmState::new(
        HashMap::new(),
        HashMap::new(),
        HashMap::new(),
        HashMap::new(),
    );
    let mcp = Arc::new(crate::mcp::SharedMcpPool::new(
        crate::mcp::McpConfig::default(),
    ));
    for (council_id, worker_role) in [
        (Some("council-headless-test"), None),
        (None, Some("research")),
    ] {
        let child_id = tokio::time::timeout(
            std::time::Duration::from_secs(10),
            super::spawn_swarm_agent(
                "routing_busy",
                "routing-swarm",
                None,
                Some(temp.path().to_string_lossy().into_owned()),
                None,
                Some(crate::config::SwarmSpawnMode::Headless),
                None,
                None,
                Some("routing test".into()),
                council_id.map(str::to_string),
                worker_role.map(str::to_string),
                &sessions,
                &Arc::new(RwLock::new(String::new())),
                &provider,
                &state.members,
                &state.swarms_by_id,
                &state.coordinators,
                &state.plans,
                &Arc::new(RwLock::new(VecDeque::new())),
                &Arc::new(AtomicU64::new(0)),
                &broadcast::channel(16).0,
                &mcp,
                &Arc::new(RwLock::new(HashMap::new())),
                &Arc::new(RwLock::new(HashMap::new())),
            ),
        )
        .await
        .expect("actual spawn must not await coordinator lock")
        .unwrap();
        let child = sessions.read().await.get(&child_id).cloned().unwrap();
        let child = child.lock().await;
        assert_eq!(child.provider_model(), "gpt-5.5");
        assert_eq!(
            crate::tool::session_tool_policy_allows_tool_for_test(&child_id, "write"),
            Some(false)
        );
        assert_eq!(
            crate::tool::session_tool_policy_allows_tool_for_test(&child_id, "read"),
            Some(true)
        );
        let persisted = crate::session::Session::load_startup_stub(&child_id).unwrap();
        assert_eq!(persisted.council_id.as_deref(), council_id);
        assert_eq!(persisted.worker_role.as_deref(), worker_role);
        assert_eq!(child.session_provider_key().as_deref(), Some("openai-api"));
        drop(child);
    }
    crate::server::client_actions::set_session_agent_model(
        &agent,
        "routing_busy",
        "swarm",
        Some("inherit".into()),
    )
    .await
    .unwrap();
    let configured =
        super::session_swarm_model("routing_busy", &sessions, Some("global-model".into())).await;
    let selection = resolve_swarm_spawn_selection(None, configured, &coordinator);
    assert_eq!(selection.model.as_deref(), Some("claude-opus-4-6"));
    assert_eq!(selection.provider_key.as_deref(), Some("claude-api"));
    crate::server::client_actions::set_session_agent_model(&agent, "routing_busy", "swarm", None)
        .await
        .unwrap();
    assert_eq!(
        super::session_swarm_model("routing_busy", &sessions, Some("global-model".into()))
            .await
            .as_deref(),
        Some("global-model")
    );
    drop(held);
    if let Some(previous) = previous {
        crate::env::set_var("JCODE_HOME", previous);
    } else {
        crate::env::remove_var("JCODE_HOME");
    }
}

#[tokio::test]
async fn spawn_bootstraps_coordinator_when_swarm_has_none() {
    let swarm_members = Arc::new(RwLock::new(HashMap::new()));
    let swarms_by_id = Arc::new(RwLock::new(HashMap::from([(
        "swarm-1".to_string(),
        HashSet::from(["req".to_string()]),
    )])));
    let swarm_coordinators = Arc::new(RwLock::new(HashMap::new()));
    let swarm_plans = Arc::new(RwLock::new(HashMap::<String, VersionedPlan>::new()));
    let (req_member, _req_rx) = member("req", Some("swarm-1"), "agent");
    swarm_members
        .write()
        .await
        .insert("req".to_string(), req_member);
    let (client_event_tx, mut client_event_rx) = mpsc::unbounded_channel();

    let swarm_id = ensure_spawn_coordinator_swarm(
        1,
        "req",
        &client_event_tx,
        &swarm_members,
        &swarms_by_id,
        &swarm_coordinators,
        &swarm_plans,
        32,
    )
    .await;

    assert_eq!(swarm_id.as_deref(), Some("swarm-1"));
    assert_eq!(
        swarm_coordinators
            .read()
            .await
            .get("swarm-1")
            .map(String::as_str),
        Some("req")
    );
    assert_eq!(
        swarm_members
            .read()
            .await
            .get("req")
            .map(|member| member.role.as_str()),
        Some("coordinator")
    );
    assert!(matches!(
        client_event_rx.recv().await,
        Some(ServerEvent::Notification {
            notification_type: NotificationType::Message { .. },
            message,
            ..
        }) if message == "You are the coordinator for this swarm."
    ));
}

#[tokio::test]
async fn nested_agent_cannot_spawn_regardless_of_root_effort() {
    // Swarms are one-level fan-out. A spawned child cannot grow another
    // generation, even if the root still reports the retired deep sentinel.
    for (root_id, effort) in [
        ("light-root-no-recursion", Some("swarm")),
        ("deep-root-no-recursion", Some("swarm-deep")),
        ("normal-root-no-recursion", None),
    ] {
        crate::session_effort::forget_session_effort(root_id);
        crate::session_effort::record_session_effort(root_id, effort);
        let swarm_id = format!("swarm-{root_id}");
        let child_id = format!("child-{root_id}");
        let swarm_members = Arc::new(RwLock::new(HashMap::new()));
        let swarms_by_id = Arc::new(RwLock::new(HashMap::from([(
            swarm_id.clone(),
            HashSet::from([child_id.clone(), root_id.to_string()]),
        )])));
        let swarm_coordinators = Arc::new(RwLock::new(HashMap::from([(
            swarm_id.clone(),
            root_id.to_string(),
        )])));
        let swarm_plans = Arc::new(RwLock::new(HashMap::<String, VersionedPlan>::new()));
        let (mut child_member, _child_rx) = member(&child_id, Some(&swarm_id), "agent");
        child_member.report_back_to_session_id = Some(root_id.to_string());
        let (root_member, _root_rx) = member(root_id, Some(&swarm_id), "coordinator");
        let mut members = swarm_members.write().await;
        members.insert(child_id.clone(), child_member);
        members.insert(root_id.to_string(), root_member);
        drop(members);
        let (client_event_tx, mut client_event_rx) = mpsc::unbounded_channel();

        let refused = ensure_spawn_coordinator_swarm(
            2,
            &child_id,
            &client_event_tx,
            &swarm_members,
            &swarms_by_id,
            &swarm_coordinators,
            &swarm_plans,
            32,
        )
        .await;

        crate::session_effort::forget_session_effort(root_id);
        assert!(refused.is_none());
        assert_eq!(
            swarm_coordinators
                .read()
                .await
                .get(&swarm_id)
                .map(String::as_str),
            Some(root_id)
        );
        assert_eq!(
            swarm_members
                .read()
                .await
                .get(&child_id)
                .map(|member| member.role.as_str()),
            Some("agent")
        );
        assert!(matches!(
            client_event_rx.recv().await,
            Some(ServerEvent::Error { message, .. })
                if message.contains("Recursive swarm spawning is disabled")
                    && message.contains(&format!("Only the root session ({root_id}) may spawn agents"))
        ));
    }
}

#[tokio::test]
async fn spawn_rejected_when_member_limit_reached() {
    use crate::server::swarm::MAX_SWARM_MEMBERS;

    // Fill the swarm to the member cap; the next spawn must be refused.
    let swarm_members = Arc::new(RwLock::new(HashMap::new()));
    let swarms_by_id = Arc::new(RwLock::new(HashMap::new()));
    let swarm_coordinators = Arc::new(RwLock::new(HashMap::from([(
        "swarm-1".to_string(),
        "root".to_string(),
    )])));
    let swarm_plans = Arc::new(RwLock::new(HashMap::<String, VersionedPlan>::new()));
    {
        let mut members = swarm_members.write().await;
        let (root, _rx) = member("root", Some("swarm-1"), "coordinator");
        members.insert("root".to_string(), root);
        // Add filler members so the swarm holds exactly MAX_SWARM_MEMBERS total.
        for idx in 1..MAX_SWARM_MEMBERS {
            let id = format!("agent-{idx}");
            let (mut m, _rx) = member(&id, Some("swarm-1"), "agent");
            m.report_back_to_session_id = Some("root".to_string());
            members.insert(id, m);
        }
    }
    let (client_event_tx, mut client_event_rx) = mpsc::unbounded_channel();

    let refused = ensure_spawn_coordinator_swarm(
        7,
        "root",
        &client_event_tx,
        &swarm_members,
        &swarms_by_id,
        &swarm_coordinators,
        &swarm_plans,
        0,
    )
    .await;
    assert!(refused.is_none());
    assert!(matches!(
        client_event_rx.recv().await,
        Some(ServerEvent::Error { message, .. })
            if message.contains("Swarm member limit reached")
    ));
}

#[tokio::test]
async fn terminal_members_do_not_consume_spawn_capacity() {
    use crate::server::swarm::MAX_SWARM_MEMBERS;

    let swarm_members = Arc::new(RwLock::new(HashMap::new()));
    let swarms_by_id = Arc::new(RwLock::new(HashMap::new()));
    let swarm_coordinators = Arc::new(RwLock::new(HashMap::from([(
        "swarm-1".to_string(),
        "root".to_string(),
    )])));
    let swarm_plans = Arc::new(RwLock::new(HashMap::<String, VersionedPlan>::new()));
    {
        let mut members = swarm_members.write().await;
        let (root, _rx) = member("root", Some("swarm-1"), "coordinator");
        members.insert("root".to_string(), root);
        for idx in 0..MAX_SWARM_MEMBERS {
            let id = format!("historical-{idx}");
            let (mut historical, _rx) = member(&id, Some("swarm-1"), "agent");
            historical.status = if idx % 2 == 0 {
                "completed".to_string()
            } else {
                "stopped".to_string()
            };
            historical.latest_completion_report = Some(format!("report {idx}"));
            historical.report_back_to_session_id = Some("root".to_string());
            members.insert(id, historical);
        }
    }
    let (client_event_tx, _client_event_rx) = mpsc::unbounded_channel();

    let allowed = ensure_spawn_coordinator_swarm(
        7,
        "root",
        &client_event_tx,
        &swarm_members,
        &swarms_by_id,
        &swarm_coordinators,
        &swarm_plans,
        32,
    )
    .await;

    assert_eq!(allowed.as_deref(), Some("swarm-1"));
}

#[tokio::test]
async fn spawn_rejected_at_configured_live_agent_limit() {
    let swarm_members = Arc::new(RwLock::new(HashMap::new()));
    let swarms_by_id = Arc::new(RwLock::new(HashMap::new()));
    let swarm_coordinators = Arc::new(RwLock::new(HashMap::from([(
        "swarm-1".to_string(),
        "root".to_string(),
    )])));
    let swarm_plans = Arc::new(RwLock::new(HashMap::<String, VersionedPlan>::new()));
    {
        let mut members = swarm_members.write().await;
        let (root, _rx) = member("root", Some("swarm-1"), "coordinator");
        members.insert("root".to_string(), root);
        for idx in 0..2 {
            let id = format!("agent-{idx}");
            let (mut worker, _rx) = member(&id, Some("swarm-1"), "agent");
            worker.report_back_to_session_id = Some("root".to_string());
            members.insert(id, worker);
        }
    }
    let (client_event_tx, mut client_event_rx) = mpsc::unbounded_channel();

    let refused = ensure_spawn_coordinator_swarm(
        7,
        "root",
        &client_event_tx,
        &swarm_members,
        &swarms_by_id,
        &swarm_coordinators,
        &swarm_plans,
        2,
    )
    .await;

    assert!(refused.is_none());
    assert!(matches!(
        client_event_rx.recv().await,
        Some(ServerEvent::Error { message, .. })
            if message.contains("Swarm live-agent limit reached (max 2")
    ));
}

#[tokio::test]
async fn spawn_admission_lock_serializes_per_swarm_only() {
    use std::time::Duration;

    let key = format!("lock-test-{}", std::process::id());
    let same_a = spawn_admission_lock(&key);
    let same_b = spawn_admission_lock(&key);
    let other = spawn_admission_lock(&format!("{key}-other"));

    let held = same_a.lock().await;
    assert!(
        tokio::time::timeout(Duration::from_millis(10), same_b.lock())
            .await
            .is_err()
    );
    assert!(
        tokio::time::timeout(Duration::from_millis(100), other.lock())
            .await
            .is_ok()
    );
    drop(held);
    assert!(
        tokio::time::timeout(Duration::from_millis(100), same_b.lock())
            .await
            .is_ok()
    );
}

#[path = "comm_session_effort_tests.rs"]
mod effort;

fn is_nonterminal_stop_frame(event: &ServerEvent) -> bool {
    matches!(
        event,
        ServerEvent::StopRequested { .. } | ServerEvent::Draining { .. }
    )
}

/// Terminal frames only: the additive `stop_requested`/`draining` frames are
/// asserted explicitly by `stop_emits_nonterminal_frames_before_the_terminal_reply`.
async fn stop_after_member_removal(requester: &str, target: &str, force: bool) -> Vec<ServerEvent> {
    stop_frames_after_member_removal(requester, target, force)
        .await
        .into_iter()
        .filter(|event| !is_nonterminal_stop_frame(event))
        .collect()
}

async fn stop_frames_after_member_removal(
    requester: &str,
    target: &str,
    force: bool,
) -> Vec<ServerEvent> {
    let (tx, mut rx) = mpsc::unbounded_channel();
    let swarm_members = Arc::new(RwLock::new(HashMap::new()));
    let swarms_by_id = Arc::new(RwLock::new(HashMap::new()));
    let coordinators = Arc::new(RwLock::new(HashMap::new()));
    let plans = Arc::new(RwLock::new(HashMap::new()));
    let channels = Arc::new(RwLock::new(HashMap::new()));
    let history = Arc::new(RwLock::new(VecDeque::new()));
    let counter = Arc::new(AtomicU64::new(0));
    let (swarm_event_tx, _swarm_event_rx) = broadcast::channel(8);
    let queues = Arc::new(RwLock::new(HashMap::new()));
    super::handle_comm_stop(
        7,
        requester.to_string(),
        target.to_string(),
        force,
        &tx,
        &Arc::new(crate::server::session_manager::SessionManager::new()),
        &swarm_members,
        &swarms_by_id,
        &coordinators,
        &plans,
        &channels,
        &channels,
        &history,
        &counter,
        &swarm_event_tx,
        &queues,
        &crate::server::SwarmMutationRuntime::default(),
    )
    .await;
    drop(tx);
    let mut events = Vec::new();
    while let Some(event) = rx.recv().await {
        events.push(event);
    }
    events
}

async fn next_terminal_stop_frame(rx: &mut mpsc::UnboundedReceiver<ServerEvent>) -> ServerEvent {
    loop {
        let event = rx.recv().await.expect("stop result");
        if !is_nonterminal_stop_frame(&event) {
            return event;
        }
    }
}

#[tokio::test]
async fn stop_emits_nonterminal_frames_before_the_terminal_reply() {
    let _env = crate::storage::lock_test_env();
    let runtime = tempfile::TempDir::new().expect("runtime dir");
    let previous = std::env::var_os("JCODE_RUNTIME_DIR");
    crate::env::set_var("JCODE_RUNTIME_DIR", runtime.path());
    let control = crate::server::session_control::store();
    control.register("frame-worker", Some("coord")).unwrap();
    let token = control.admit("frame-worker").unwrap();
    let work = token.reserve().unwrap();
    let frames = stop_frames_after_member_removal("coord", "frame-worker", false).await;
    drop(work);
    match previous {
        Some(value) => crate::env::set_var("JCODE_RUNTIME_DIR", value),
        None => crate::env::remove_var("JCODE_RUNTIME_DIR"),
    }
    match frames.as_slice() {
        [
            ServerEvent::StopRequested {
                request_id: 7,
                generation,
                terminal: false,
                ..
            },
            ServerEvent::Draining {
                request_id: 7,
                terminal: false,
                ..
            },
            ServerEvent::Error { id: 7, .. },
        ] => assert_eq!(*generation, token.generation),
        other => panic!("expected stop_requested, draining, then terminal: {other:?}"),
    }
}

#[tokio::test]
async fn stop_authorizes_removed_member_from_retained_owner_record() {
    let _env = crate::storage::lock_test_env();
    let runtime = tempfile::TempDir::new().expect("runtime dir");
    let previous = std::env::var_os("JCODE_RUNTIME_DIR");
    crate::env::set_var("JCODE_RUNTIME_DIR", runtime.path());
    let control = crate::server::session_control::store();
    for id in ["orphan-owner", "orphan-other", "orphan-force"] {
        control.register(id, Some("coord")).expect("register");
    }

    let owner = stop_after_member_removal("coord", "orphan-owner", false).await;
    let unrelated = stop_after_member_removal("stranger", "orphan-other", false).await;
    let forced = stop_after_member_removal("stranger", "orphan-force", true).await;

    match previous {
        Some(value) => crate::env::set_var("JCODE_RUNTIME_DIR", value),
        None => crate::env::remove_var("JCODE_RUNTIME_DIR"),
    }
    assert!(
        matches!(owner.as_slice(), [ServerEvent::Done { id: 7 }]),
        "{owner:?}"
    );
    assert!(
        matches!(unrelated.as_slice(), [ServerEvent::Error { message, .. }] if message.contains("Refusing to stop")),
        "{unrelated:?}"
    );
    assert!(
        matches!(forced.as_slice(), [ServerEvent::Done { id: 7 }]),
        "{forced:?}"
    );
    let states: Vec<_> = ["orphan-owner", "orphan-other", "orphan-force"]
        .iter()
        .map(|id| {
            crate::server::session_control::ControlStore::new(
                runtime.path().join("durable-state").join("session-control"),
            )
            .inspect(id)
            .unwrap()
            .unwrap()
            .state
        })
        .collect();
    use crate::server::session_control::ControlState::{Runnable, Stopped};
    assert_eq!(states, vec![Stopped, Runnable, Stopped]);
}

#[tokio::test]
async fn aliases_stay_swarm_local_while_exact_id_survives_owner_reconnect() {
    let _env = crate::storage::lock_test_env();
    let runtime = tempfile::TempDir::new().expect("runtime dir");
    let previous = std::env::var_os("JCODE_RUNTIME_DIR");
    crate::env::set_var("JCODE_RUNTIME_DIR", runtime.path());
    let control = crate::server::session_control::store();
    control.register("alias-owner", None).unwrap();
    control
        .register("session_otter_alias_1", Some("alias-owner"))
        .unwrap();

    // The owner reconnected into a different swarm than the worker's.
    let (tx, mut rx) = mpsc::unbounded_channel();
    let swarm_members = Arc::new(RwLock::new(HashMap::new()));
    let (mut owner, _owner_rx) = member("alias-owner", Some("swarm-new"), "coordinator");
    owner.friendly_name = Some("boss".to_string());
    let (mut worker, _worker_rx) = member("session_otter_alias_1", Some("swarm-old"), "agent");
    worker.friendly_name = Some("otter".to_string());
    worker.report_back_to_session_id = Some("alias-owner".to_string());
    {
        let mut members = swarm_members.write().await;
        members.insert(owner.session_id.clone(), owner);
        members.insert(worker.session_id.clone(), worker);
    }
    let resolved_other_swarm =
        resolve_stop_target_session("swarm-new", "otter", &swarm_members).await;

    let run = |target: &'static str| {
        let tx = tx.clone();
        let swarm_members = Arc::clone(&swarm_members);
        async move {
            super::handle_comm_stop(
                9,
                "alias-owner".to_string(),
                target.to_string(),
                false,
                &tx,
                &Arc::new(crate::server::session_manager::SessionManager::new()),
                &swarm_members,
                &Arc::new(RwLock::new(HashMap::new())),
                &Arc::new(RwLock::new(HashMap::new())),
                &Arc::new(RwLock::new(HashMap::new())),
                &Arc::new(RwLock::new(HashMap::new())),
                &Arc::new(RwLock::new(HashMap::new())),
                &Arc::new(RwLock::new(VecDeque::new())),
                &Arc::new(AtomicU64::new(0)),
                &broadcast::channel(8).0,
                &Arc::new(RwLock::new(HashMap::new())),
                &crate::server::SwarmMutationRuntime::default(),
            )
            .await
        }
    };
    run("otter").await;
    let alias_event = next_terminal_stop_frame(&mut rx).await;
    let after_alias = control
        .inspect("session_otter_alias_1")
        .unwrap()
        .unwrap()
        .state;
    run("session_otter_alias_1").await;
    let exact_event = next_terminal_stop_frame(&mut rx).await;
    let after_exact = control
        .inspect("session_otter_alias_1")
        .unwrap()
        .unwrap()
        .state;

    match previous {
        Some(value) => crate::env::set_var("JCODE_RUNTIME_DIR", value),
        None => crate::env::remove_var("JCODE_RUNTIME_DIR"),
    }
    assert!(
        resolved_other_swarm.is_err(),
        "alias crossed swarm boundary"
    );
    assert!(
        matches!(&alias_event, ServerEvent::Error { message, .. } if !message.contains("settled")),
        "{alias_event:?}"
    );
    use crate::server::session_control::ControlState::{Runnable, Stopped};
    assert_eq!(after_alias, Runnable, "cross-swarm alias stopped a worker");
    assert!(
        matches!(exact_event, ServerEvent::Done { id: 9 }),
        "{exact_event:?}"
    );
    assert_eq!(after_exact, Stopped);
}

async fn status_after_member_removal(requester: &str, target: &str) -> ServerEvent {
    let (tx, mut rx) = mpsc::unbounded_channel();
    crate::server::comm_sync::handle_comm_status(
        11,
        requester.to_string(),
        target.to_string(),
        &Arc::new(crate::server::session_manager::SessionManager::new()),
        &Arc::new(RwLock::new(HashMap::new())),
        &Arc::new(RwLock::new(HashMap::new())),
        &crate::server::FileTouchService::new(),
        &tx,
    )
    .await;
    drop(tx);
    rx.recv().await.expect("status event")
}

#[tokio::test]
async fn status_resolves_removed_member_by_exact_id_for_owner_only() {
    let _env = crate::storage::lock_test_env();
    let runtime = tempfile::TempDir::new().expect("runtime dir");
    let previous = std::env::var_os("JCODE_RUNTIME_DIR");
    crate::env::set_var("JCODE_RUNTIME_DIR", runtime.path());
    let control = crate::server::session_control::store();
    control.register("left-worker", Some("coord")).unwrap();
    control.admit("left-worker").unwrap();
    control
        .request_stop("coord", "left-worker", false, "coord:1")
        .unwrap();

    let owner = status_after_member_removal("coord", "left-worker").await;
    let stranger = status_after_member_removal("stranger", "left-worker").await;
    let unknown = status_after_member_removal("coord", "never-existed").await;

    match previous {
        Some(value) => crate::env::set_var("JCODE_RUNTIME_DIR", value),
        None => crate::env::remove_var("JCODE_RUNTIME_DIR"),
    }
    let ServerEvent::CommStatusResponse { snapshot, .. } = owner else {
        panic!("owner should receive control state, got {owner:?}");
    };
    let control = snapshot.control.expect("control state present");
    assert_eq!(control.state, "stop_requested");
    assert_eq!(control.revoked_through, 1);
    assert_eq!(control.stop_request.as_deref(), Some("coord:1"));
    assert!(
        matches!(&stranger, ServerEvent::Error { message, .. } if message.contains("Refusing")),
        "{stranger:?}"
    );
    assert!(
        matches!(&unknown, ServerEvent::Error { message, .. } if message.contains("not in the same swarm")),
        "{unknown:?}"
    );
}

#[tokio::test]
async fn stop_with_unsettled_work_reports_unresolved_then_retry_settles() {
    let _env = crate::storage::lock_test_env();
    let runtime = tempfile::TempDir::new().expect("runtime dir");
    let previous = std::env::var_os("JCODE_RUNTIME_DIR");
    crate::env::set_var("JCODE_RUNTIME_DIR", runtime.path());
    let control = crate::server::session_control::store();
    control.register("busy-worker", Some("coord")).unwrap();
    let token = control.admit("busy-worker").unwrap();
    let work = token.reserve().unwrap();

    let first = stop_after_member_removal("coord", "busy-worker", false).await;
    let mid = control.inspect("busy-worker").unwrap().unwrap();
    let repeat_while_busy = stop_after_member_removal("coord", "busy-worker", false).await;
    drop(work);
    let retry = stop_after_member_removal("coord", "busy-worker", false).await;
    let settled = control.inspect("busy-worker").unwrap().unwrap();

    match previous {
        Some(value) => crate::env::set_var("JCODE_RUNTIME_DIR", value),
        None => crate::env::remove_var("JCODE_RUNTIME_DIR"),
    }
    for events in [&first, &repeat_while_busy] {
        match events.as_slice() {
            [ServerEvent::Error { message, .. }] => {
                assert!(
                    message.contains("unresolved session=busy-worker"),
                    "{message}"
                );
                assert!(message.contains("request=coord:7"), "{message}");
                assert!(
                    message.contains(&format!("generation={}", token.generation)),
                    "{message}"
                );
                assert!(message.contains("settlement not confirmed"), "{message}");
                assert!(
                    message.contains("communicate action=status target_session=busy-worker"),
                    "{message}"
                );
            }
            other => panic!("unsettled stop must not look successful: {other:?}"),
        }
    }
    use crate::server::session_control::ControlState::{Stopped, Unresolved};
    assert_eq!(mid.state, Unresolved);
    assert!(control.admit("busy-worker").is_err());
    assert!(
        matches!(retry.as_slice(), [ServerEvent::Done { id: 7 }]),
        "{retry:?}"
    );
    assert_eq!(settled.state, Stopped);
    assert!(settled.revision > mid.revision);
}

#[tokio::test]
async fn stop_persistence_failure_reports_error_and_keeps_worker_revoked() {
    let _env = crate::storage::lock_test_env();
    let runtime = tempfile::TempDir::new().expect("runtime dir");
    let previous = std::env::var_os("JCODE_RUNTIME_DIR");
    crate::env::set_var("JCODE_RUNTIME_DIR", runtime.path());
    let control = crate::server::session_control::store();
    control
        .register("persist-fail-worker", Some("coord"))
        .unwrap();
    let token = control.admit("persist-fail-worker").unwrap();
    let path = control.path("persist-fail-worker").unwrap();
    std::fs::remove_file(&path).unwrap();
    std::fs::create_dir(&path).unwrap();

    let events = stop_after_member_removal("coord", "persist-fail-worker", false).await;
    let reserve_after = control.reserve(&token).is_err();
    let admit_after = control.admit("persist-fail-worker").is_err();

    std::fs::remove_dir(&path).unwrap();
    match previous {
        Some(value) => crate::env::set_var("JCODE_RUNTIME_DIR", value),
        None => crate::env::remove_var("JCODE_RUNTIME_DIR"),
    }
    assert!(
        matches!(events.as_slice(), [ServerEvent::Error { .. }]),
        "failed durable stop must not report success: {events:?}"
    );
    assert!(
        reserve_after,
        "work was admitted after a failed durable stop"
    );
    assert!(
        admit_after,
        "new generation admitted after a failed durable stop"
    );
}

/// Drives the real `spawn_swarm_agent` against an overnight run capped at one
/// worker. Two concurrent spawns must admit exactly one, and a refused spawn
/// surfaces a visible reason and a `worker_spawn_refused` event.
#[tokio::test]
async fn overnight_max_agents_cap_admits_exactly_one_of_two_concurrent_spawns() {
    let _sandbox = crate::auth::test_sandbox::AuthTestSandbox::new().unwrap();
    let temp = tempfile::TempDir::new().unwrap();
    let manifest = crate::overnight::test_run_manifest(
        "spawn-cap-real",
        "cap_req",
        crate::overnight::OvernightLimits {
            max_agents: Some(1),
            ..Default::default()
        },
    );
    let sessions = Arc::new(crate::server::session_manager::SessionManager::from_agents(
        HashMap::new(),
    ));
    sessions
        .register_new(
            "cap_req".to_string(),
            test_agent_with_working_dir("cap_req", &temp.path().to_string_lossy()).await,
        )
        .await
        .unwrap();
    let provider: Arc<dyn Provider> = Arc::new(MockProvider);
    let state = crate::server::SwarmState::new(
        HashMap::new(),
        HashMap::new(),
        HashMap::new(),
        HashMap::new(),
    );
    let mcp = Arc::new(crate::mcp::SharedMcpPool::new(
        crate::mcp::McpConfig::default(),
    ));
    let global_session_id = Arc::new(RwLock::new(String::new()));
    let event_history = Arc::new(RwLock::new(VecDeque::new()));
    let event_counter = Arc::new(AtomicU64::new(0));
    let swarm_event_tx = broadcast::channel(16).0;
    let interrupts = Arc::new(RwLock::new(HashMap::new()));
    let connections = Arc::new(RwLock::new(HashMap::new()));
    let spawn = || {
        super::spawn_swarm_agent(
            "cap_req",
            "cap-swarm",
            None,
            Some(temp.path().to_string_lossy().into_owned()),
            None,
            Some(crate::config::SwarmSpawnMode::Headless),
            None,
            None,
            Some("cap test".into()),
            None,
            None,
            &sessions,
            &global_session_id,
            &provider,
            &state.members,
            &state.swarms_by_id,
            &state.coordinators,
            &state.plans,
            &event_history,
            &event_counter,
            &swarm_event_tx,
            &mcp,
            &interrupts,
            &connections,
        )
    };
    let (first, second) = tokio::time::timeout(
        std::time::Duration::from_secs(20),
        async { tokio::join!(spawn(), spawn()) },
    )
    .await
    .expect("spawns finish");
    let results = [first, second];
    assert_eq!(
        results.iter().filter(|result| result.is_ok()).count(),
        1,
        "exactly one concurrent spawn fits under max_agents=1: {results:?}"
    );
    let error = results.iter().find_map(|r| r.as_ref().err()).unwrap();
    assert!(error.to_string().contains("max 1"), "{error}");
    let live = state
        .members
        .read()
        .await
        .values()
        .filter(|member| member.report_back_to_session_id.as_deref() == Some("cap_req"))
        .count();
    assert_eq!(live, 1);
    let events = crate::overnight::read_events(&manifest).unwrap();
    assert!(events.iter().any(|event| event.kind == "worker_spawn_refused"));

    // A third spawn after cancellation is refused even if capacity were free.
    let mut cancelling = crate::overnight::load_manifest("spawn-cap-real").unwrap();
    cancelling.status = crate::overnight::OvernightRunStatus::CancelRequested;
    cancelling.cancel_requested_at = Some(chrono::Utc::now());
    crate::overnight::save_manifest(&cancelling).unwrap();
    let error = spawn().await.unwrap_err();
    assert!(error.to_string().contains("being cancelled"), "{error}");
}

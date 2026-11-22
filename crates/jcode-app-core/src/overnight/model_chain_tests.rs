use super::*;
use crate::message::{Message, StreamEvent, ToolDefinition};
use async_trait::async_trait;
use std::sync::Mutex;

const ALL_KINDS: [ProviderFailureKind; 7] = [
    ProviderFailureKind::AuthExpired,
    ProviderFailureKind::AuthRevoked,
    ProviderFailureKind::QuotaExhausted,
    ProviderFailureKind::RateLimited,
    ProviderFailureKind::ModelUnavailable,
    ProviderFailureKind::Transient,
    ProviderFailureKind::Permanent,
];

fn names(models: &[&str]) -> Vec<String> {
    models.iter().map(|m| m.to_string()).collect()
}

fn route(served: &[String]) -> RouteModels {
    RouteModels {
        served: served.to_vec(),
        unfiltered_reason: None,
        refreshed: false,
    }
}

#[test]
fn every_failure_kind_has_the_specified_advance_behaviour() {
    for kind in ALL_KINDS {
        let expected = matches!(
            kind,
            ProviderFailureKind::Transient
                | ProviderFailureKind::RateLimited
                | ProviderFailureKind::ModelUnavailable
                | ProviderFailureKind::QuotaExhausted
        );
        assert_eq!(kind_advances_chain(kind), expected, "{kind:?}");
        let mut chain = ModelChain::starting_at(names(&["a", "b"]), "a");
        let outcome = chain.advance(kind);
        if expected {
            assert_eq!(outcome, Advance::Next("b".into()), "{kind:?}");
        } else {
            assert_eq!(outcome, Advance::Hold, "{kind:?}");
            assert_eq!(chain.tried(), ["a"], "a held chain does not move");
        }
    }
}

#[test]
fn exhaustion_lists_every_model_tried_in_order() {
    let mut chain = ModelChain::starting_at(names(&["a", "b", "c"]), "a");
    assert_eq!(
        chain.advance(ProviderFailureKind::ModelUnavailable),
        Advance::Next("b".into())
    );
    assert_eq!(
        chain.advance(ProviderFailureKind::RateLimited),
        Advance::Next("c".into())
    );
    assert_eq!(
        chain.advance(ProviderFailureKind::Transient),
        Advance::Exhausted(names(&["a", "b", "c"]))
    );
    // Stays exhausted.
    assert!(matches!(
        chain.advance(ProviderFailureKind::QuotaExhausted),
        Advance::Exhausted(_)
    ));
}

#[test]
fn explicit_model_outside_the_chain_counts_as_tried_then_chain_applies() {
    let mut chain = ModelChain::starting_at(names(&["w1", "w2"]), "explicit-model");
    assert_eq!(
        chain.advance(ProviderFailureKind::ModelUnavailable),
        Advance::Next("w1".into())
    );
    assert_eq!(
        chain.advance(ProviderFailureKind::ModelUnavailable),
        Advance::Next("w2".into())
    );
    assert_eq!(
        chain.advance(ProviderFailureKind::ModelUnavailable),
        Advance::Exhausted(names(&["explicit-model", "w1", "w2"]))
    );
}

#[test]
fn current_model_spelling_variants_find_their_chain_position() {
    let chain = ModelChain::starting_at(
        names(&["claude-opus-5.5", "gpt-6-sol"]),
        "omni:claude-opus-5-5",
    );
    assert_eq!(chain.tried(), ["claude-opus-5.5"]);
}

#[test]
fn preflight_filter_drops_unserved_models_and_keeps_route_spelling() {
    let (kept, dropped) = filter_served(
        &names(&["claude-opus-5.5", "gpt-6-sol", "qwen3.5-35b"]),
        &names(&["GPT-6-SOL", "qwen3.5-35b"]),
    );
    assert_eq!(kept, ["GPT-6-SOL", "qwen3.5-35b"]);
    assert_eq!(dropped, ["claude-opus-5.5"]);
}

#[test]
fn empty_served_list_keeps_every_chain_model() {
    let (kept, dropped) = filter_served(&names(&["a", "b"]), &[]);
    assert_eq!(kept, ["a", "b"]);
    assert!(dropped.is_empty());
}

#[test]
fn plan_blocks_when_a_chain_is_left_empty_and_names_unserved_models() {
    let config = OvernightConfig::default();
    let served = names(&["claude-opus-5.5", "gpt-6-sol"]);
    let reasons = plan(&config, &route(&served)).unwrap_err();
    assert_eq!(reasons.len(), 1, "{reasons:?}");
    assert!(reasons[0].contains("worker"), "{reasons:?}");
    for model in &config.worker_model_chain {
        assert!(reasons[0].contains(model.as_str()), "{reasons:?}");
    }
}

#[test]
fn plan_starts_on_the_next_served_model_and_records_dropped() {
    let config = OvernightConfig::default();
    let served = names(&["gpt-6-sol", "claude-sonnet-5.5"]);
    let plan = plan(&config, &route(&served)).unwrap();
    assert_eq!(
        plan.coordinator.first().map(String::as_str),
        Some("gpt-6-sol")
    );
    assert_eq!(plan.dropped_coordinator[0], "claude-opus-5.5");
    assert_eq!(plan.worker, ["claude-sonnet-5.5"]);
}

#[test]
fn plan_with_no_model_list_keeps_default_chains_whole() {
    let config = OvernightConfig::default();
    let plan = plan(&config, &route(&[])).unwrap();
    assert_eq!(plan.coordinator, config.coordinator_model_chain);
    assert_eq!(plan.worker, config.worker_model_chain);
}

#[test]
fn select_first_skips_models_the_route_refuses() {
    let (selected, skipped) = select_first(&names(&["a", "b", "c"]), |model| {
        if model == "c" {
            Ok(())
        } else {
            anyhow::bail!("not served")
        }
    });
    assert_eq!(selected.as_deref(), Some("c"));
    assert_eq!(skipped.len(), 2);
}

/// Provider that fails the models in `failing` with `error` and serves the
/// models in `served`. Its name and account never change on a model switch.
#[derive(Clone)]
struct ChainProvider {
    model: Arc<Mutex<String>>,
    failing: Vec<String>,
    served: Vec<String>,
    error: String,
    calls: Arc<Mutex<Vec<String>>>,
}

impl ChainProvider {
    fn new(start: &str, failing: &[&str], served: &[&str], error: &str) -> Self {
        Self {
            model: Arc::new(Mutex::new(start.to_string())),
            failing: names(failing),
            served: names(served),
            error: error.to_string(),
            calls: Arc::default(),
        }
    }
    fn calls(&self) -> Vec<String> {
        self.calls.lock().unwrap().clone()
    }
}

#[async_trait]
impl Provider for ChainProvider {
    async fn complete(
        &self,
        _: &[Message],
        _: &[ToolDefinition],
        _: &str,
        _: Option<&str>,
    ) -> Result<crate::provider::EventStream> {
        let model = self.model.lock().unwrap().clone();
        self.calls.lock().unwrap().push(model.clone());
        if self.failing.contains(&model) {
            anyhow::bail!("{}", self.error);
        }
        Ok(Box::pin(futures::stream::iter(vec![
            Ok(StreamEvent::TextDelta(format!("answered on {model}"))),
            Ok(StreamEvent::MessageEnd {
                stop_reason: Some("end_turn".into()),
            }),
        ])))
    }
    fn name(&self) -> &str {
        "chain-fake"
    }
    fn model(&self) -> String {
        self.model.lock().unwrap().clone()
    }
    fn set_model(&self, model: &str) -> Result<()> {
        if !self.served.iter().any(|m| m == model) {
            anyhow::bail!("model {model} is not served");
        }
        *self.model.lock().unwrap() = model.to_string();
        Ok(())
    }
    fn available_models_for_switching(&self) -> Vec<String> {
        self.served.clone()
    }
    fn fork(&self) -> Arc<dyn Provider> {
        Arc::new(self.clone())
    }
}

struct Fixture {
    _sandbox: crate::auth::test_sandbox::AuthTestSandbox,
}

fn fixture() -> Fixture {
    Fixture {
        _sandbox: crate::auth::test_sandbox::AuthTestSandbox::new().unwrap(),
    }
}

async fn agent_on(provider: &ChainProvider) -> Agent {
    let provider: Arc<dyn Provider> = Arc::new(provider.clone());
    let registry = Registry::new(provider.clone()).await;
    Agent::new(provider, registry)
}

fn kinds(manifest: &OvernightManifest) -> Vec<String> {
    read_events(manifest)
        .unwrap()
        .into_iter()
        .map(|event| event.kind)
        .collect()
}

#[tokio::test]
async fn supervised_turn_advances_twice_on_model_unavailable_then_succeeds() {
    let _env = fixture();
    let provider = ChainProvider::new(
        "m1",
        &["m1", "m2"],
        &["m1", "m2", "m3"],
        "model m1 not found",
    );
    let mut agent = agent_on(&provider).await;
    let manifest = test_run_manifest(
        "chain-two-advances",
        agent.session_id(),
        OvernightLimits::default(),
    );
    let provider_name_before = agent.provider_name();
    let mut chain = ModelChain::starting_at(names(&["m1", "m2", "m3"]), "m1");

    let turn = run_turn_with_chain(&mut agent, &manifest, "do the work", &mut chain)
        .await
        .unwrap();
    let ChainTurn::Finished(output) = turn else {
        panic!("expected a finished turn");
    };
    assert!(output.contains("answered on m3"), "{output}");
    assert_eq!(provider.calls(), ["m1", "m2", "m3"]);

    let events = read_events(&manifest).unwrap();
    let advances: Vec<_> = events
        .iter()
        .filter(|event| event.kind == "model_chain_advanced")
        .collect();
    assert_eq!(advances.len(), 2);
    assert_eq!(advances[0].details["from"], "m1");
    assert_eq!(advances[0].details["to"], "m2");
    assert_eq!(advances[0].details["kind"], "ModelUnavailable");
    assert_eq!(advances[1].details["to"], "m3");
    assert_eq!(
        agent.provider_name(),
        provider_name_before,
        "route unchanged"
    );
    assert_eq!(
        load_manifest(&manifest.run_id).unwrap().status,
        OvernightRunStatus::Running
    );
    // The prompt was persisted once and retried, not appended three times.
    let prompts = agent
        .session_for_split()
        .messages
        .iter()
        .filter(|m| {
            m.role == Role::User
                && m.content.iter().any(|block| {
                    matches!(block, crate::message::ContentBlock::Text { text, .. } if text.contains("do the work"))
                })
        })
        .count();
    assert_eq!(prompts, 1);
}

#[tokio::test]
async fn auth_revoked_blocks_without_advancing() {
    let _env = fixture();
    let provider = ChainProvider::new(
        "m1",
        &["m1"],
        &["m1", "m2"],
        "AuthRevoked: refresh token has been revoked",
    );
    let mut agent = agent_on(&provider).await;
    let manifest = test_run_manifest(
        "chain-auth-revoked",
        agent.session_id(),
        OvernightLimits::default(),
    );
    let mut chain = ModelChain::starting_at(names(&["m1", "m2"]), "m1");

    let turn = run_turn_with_chain(&mut agent, &manifest, "work", &mut chain)
        .await
        .unwrap();
    assert!(matches!(turn, ChainTurn::Ended));
    assert_eq!(provider.calls(), ["m1"], "no retry on another model");
    assert_eq!(agent.provider_model(), "m1");
    let done = load_manifest(&manifest.run_id).unwrap();
    assert_eq!(done.status, OvernightRunStatus::Blocked);
    let kinds = kinds(&done);
    assert!(!kinds.contains(&"model_chain_advanced".to_string()));
    assert!(kinds.contains(&"run_blocked".to_string()));
    let blocked = read_events(&done)
        .unwrap()
        .into_iter()
        .find(|event| event.kind == "run_blocked")
        .unwrap();
    assert!(
        blocked.summary.contains("AuthRevoked"),
        "{}",
        blocked.summary
    );
}

#[tokio::test]
async fn exhausted_coordinator_chain_blocks_and_names_every_model() {
    let _env = fixture();
    let provider = ChainProvider::new(
        "m1",
        &["m1", "m2", "m3"],
        &["m1", "m2", "m3"],
        "HTTP 429 too many requests",
    );
    let mut agent = agent_on(&provider).await;
    let manifest = test_run_manifest(
        "chain-exhausted",
        agent.session_id(),
        OvernightLimits::default(),
    );
    let mut chain = ModelChain::starting_at(names(&["m1", "m2", "m3"]), "m1");

    let turn = run_turn_with_chain(&mut agent, &manifest, "work", &mut chain)
        .await
        .unwrap();
    assert!(matches!(turn, ChainTurn::Ended));
    assert_eq!(provider.calls(), ["m1", "m2", "m3"]);
    let done = load_manifest(&manifest.run_id).unwrap();
    assert_eq!(done.status, OvernightRunStatus::Blocked);
    let events = read_events(&done).unwrap();
    let exhausted = events
        .iter()
        .find(|event| event.kind == "model_chain_exhausted")
        .expect("exhaustion recorded");
    for model in ["m1", "m2", "m3"] {
        assert!(exhausted.summary.contains(model), "{}", exhausted.summary);
    }
    let blocked = events
        .iter()
        .find(|event| event.kind == "run_blocked")
        .unwrap();
    for model in ["m1", "m2", "m3"] {
        assert!(blocked.summary.contains(model), "{}", blocked.summary);
    }
}

fn run_with_plan(
    run_id: &str,
    coordinator: &str,
    coordinator_models: &[&str],
    worker: &[&str],
) -> OvernightManifest {
    let manifest = test_run_manifest(run_id, coordinator, OvernightLimits::default());
    persist_plan(
        &manifest.run_dir,
        &ChainPlan {
            coordinator: names(coordinator_models),
            worker: names(worker),
            ..Default::default()
        },
    )
    .unwrap();
    manifest
}

#[test]
fn worker_spawn_default_uses_chain_for_run_owner_only_and_explicit_wins() {
    let _env = fixture();
    let manifest = run_with_plan("chain-spawn", "owner-session", &["c1"], &["w1", "w2"]);
    // Overnight owner without an explicit model: first worker-chain model,
    // pinned to the owner's route.
    assert_eq!(
        resolve_spawn_request(None, &manifest.coordinator_session_id, None).as_deref(),
        Some("w1")
    );
    assert_eq!(
        resolve_spawn_request(Some("  ".into()), "owner-session", None).as_deref(),
        Some("w1"),
        "a blank model counts as none"
    );
    // Explicit model wins.
    assert_eq!(
        resolve_spawn_request(Some("explicit".into()), "owner-session", None).as_deref(),
        Some("explicit")
    );
    // A normal session keeps its normal selection.
    assert_eq!(resolve_spawn_request(None, "normal-session", None), None);
    assert_eq!(
        resolve_spawn_request(Some("x".into()), "normal-session", None).as_deref(),
        Some("x")
    );
    assert_eq!(
        worker_chain_default_model("owner-session").as_deref(),
        Some("w1")
    );
    assert_eq!(worker_chain_default_model("normal-session"), None);
}

#[tokio::test]
async fn worker_chain_advances_then_exhaustion_fails_only_that_worker() {
    let _env = fixture();
    let provider = ChainProvider::new(
        "w1",
        &["w1", "w2", "w3"],
        &["w1", "w2", "w3"],
        "model not available",
    );
    let mut worker = agent_on(&provider).await;
    let manifest = run_with_plan(
        "chain-worker",
        "owner-session",
        &["c1"],
        &["w1", "w2", "w3"],
    );
    crate::server::session_control::store()
        .register(worker.session_id(), Some("owner-session"))
        .unwrap();

    let error = "model w1 not available";
    assert!(matches!(
        try_advance_for_session(&mut worker, error).await,
        Some(SessionAdvance::Switched)
    ));
    assert_eq!(worker.provider_model(), "w2");
    assert!(matches!(
        try_advance_for_session(&mut worker, error).await,
        Some(SessionAdvance::Switched)
    ));
    assert_eq!(worker.provider_model(), "w3");
    let SessionAdvance::Stop(Some(message)) =
        try_advance_for_session(&mut worker, error).await.unwrap()
    else {
        panic!("expected exhaustion");
    };
    for model in ["w1", "w2", "w3"] {
        assert!(message.contains(model), "{message}");
    }

    let after = load_manifest(&manifest.run_id).unwrap();
    assert_eq!(after.status, OvernightRunStatus::Running, "run continues");
    let kinds = kinds(&after);
    assert_eq!(
        kinds
            .iter()
            .filter(|k| *k == "model_chain_advanced")
            .count(),
        2
    );
    assert!(kinds.contains(&"worker_model_chain_exhausted".to_string()));
    assert!(!kinds.contains(&"run_blocked".to_string()));
}

#[tokio::test]
async fn worker_auth_failure_never_advances() {
    let _env = fixture();
    let provider = ChainProvider::new("w1", &["w1"], &["w1", "w2"], "x");
    let mut worker = agent_on(&provider).await;
    run_with_plan("chain-worker-auth", "owner-session", &["c1"], &["w1", "w2"]);
    crate::server::session_control::store()
        .register(worker.session_id(), Some("owner-session"))
        .unwrap();
    assert!(matches!(
        try_advance_for_session(&mut worker, "AuthRevoked: token revoked").await,
        Some(SessionAdvance::Stop(None))
    ));
    assert_eq!(worker.provider_model(), "w1");
}

#[tokio::test]
async fn normal_session_never_switches_models_automatically() {
    let _env = fixture();
    let provider = ChainProvider::new("m1", &["m1"], &["m1", "m2"], "model m1 not found");
    let mut agent = agent_on(&provider).await;
    // An overnight run exists, but for another session.
    run_with_plan("chain-other", "someone-else", &["m1", "m2"], &["m1", "m2"]);
    assert!(
        try_advance_for_session(&mut agent, "model m1 not found")
            .await
            .is_none()
    );
    assert_eq!(agent.provider_model(), "m1");
}

#[test]
fn visible_run_advances_a_bare_provider_and_blocks_on_exhaustion() {
    let _env = fixture();
    let provider = ChainProvider::new("m1", &[], &["m1", "m2"], "x");
    // Visible run: the coordinator session is also the parent session.
    let mut manifest = run_with_plan("chain-visible", "visible-session", &["m1", "m2"], &["w1"]);
    manifest.parent_session_id = "visible-session".into();
    save_manifest(&manifest).unwrap();
    let rt = tokio::runtime::Builder::new_current_thread()
        .build()
        .unwrap();

    let advance = try_advance_provider_for_session(
        "visible-session",
        &provider,
        None,
        &|| None,
        "model m1 unavailable",
    );
    assert!(matches!(advance, Some(SessionAdvance::Switched)));
    assert_eq!(provider.model(), "m2");
    assert_eq!(provider.name(), "chain-fake");

    let advance = try_advance_provider_for_session(
        "visible-session",
        &provider,
        None,
        &|| None,
        "model m2 unavailable",
    );
    let Some(SessionAdvance::Stop(Some(message))) = advance else {
        panic!("expected exhaustion");
    };
    assert!(
        message.contains("m1") && message.contains("m2"),
        "{message}"
    );
    let done = load_manifest("chain-visible").unwrap();
    assert_eq!(done.status, OvernightRunStatus::Blocked);

    // Any other session is untouched.
    assert!(
        try_advance_provider_for_session(
            "normal",
            &provider,
            None,
            &|| None,
            "model m2 unavailable"
        )
        .is_none()
    );
    drop(rt);
}

/// Catalog-backed route: until `loaded` is set, the model list is only the
/// current model. `refresh_model_catalog` loads it when `refresh_ok`.
struct CatalogProvider {
    loaded: Arc<std::sync::atomic::AtomicBool>,
    refresh_ok: bool,
    refreshes: Arc<std::sync::atomic::AtomicUsize>,
    served: Vec<String>,
}

impl CatalogProvider {
    fn new(refresh_ok: bool, served: &[&str]) -> Self {
        Self {
            loaded: Arc::default(),
            refresh_ok,
            refreshes: Arc::default(),
            served: names(served),
        }
    }
}

#[async_trait]
impl Provider for CatalogProvider {
    async fn complete(
        &self,
        _: &[Message],
        _: &[ToolDefinition],
        _: &str,
        _: Option<&str>,
    ) -> Result<crate::provider::EventStream> {
        anyhow::bail!("unused")
    }
    fn name(&self) -> &str {
        "catalog-fake"
    }
    fn model(&self) -> String {
        "claude-sonnet-5.5".into()
    }
    fn set_model(&self, _: &str) -> Result<()> {
        Ok(())
    }
    fn available_models_for_switching(&self) -> Vec<String> {
        if self.loaded.load(std::sync::atomic::Ordering::SeqCst) {
            self.served.clone()
        } else {
            vec![self.model()]
        }
    }
    fn model_list_is_authoritative(&self) -> bool {
        self.loaded.load(std::sync::atomic::Ordering::SeqCst)
    }
    async fn refresh_model_catalog(
        &self,
    ) -> Result<jcode_provider_core::ModelCatalogRefreshSummary> {
        self.refreshes
            .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        if !self.refresh_ok {
            anyhow::bail!("catalog endpoint unreachable");
        }
        self.loaded.store(true, std::sync::atomic::Ordering::SeqCst);
        Ok(Default::default())
    }
    fn fork(&self) -> Arc<dyn Provider> {
        unimplemented!()
    }
}

#[test]
fn cold_catalog_that_stays_cold_keeps_full_chains_and_notes_it() {
    let config = OvernightConfig::default();
    let provider = CatalogProvider::new(false, &[]);
    let routes = route_models(&provider);
    assert!(routes.served.is_empty(), "unknown, not the current model");
    let reason = routes.unfiltered_reason.clone().unwrap();
    assert!(reason.contains("route catalog unavailable; chains unfiltered"));
    assert!(reason.contains("catalog endpoint unreachable"));
    assert_eq!(
        provider.refreshes.load(std::sync::atomic::Ordering::SeqCst),
        1
    );

    let plan = plan(&config, &routes).expect("a cold catalog must not block the run");
    assert_eq!(plan.coordinator, config.coordinator_model_chain);
    assert_eq!(plan.worker, config.worker_model_chain);
    assert!(plan.dropped_coordinator.is_empty() && plan.dropped_worker.is_empty());
    assert_eq!(plan.unfiltered_reason, Some(reason));
}

#[test]
fn cold_catalog_that_loads_after_refresh_filters_normally() {
    let config = OvernightConfig::default();
    let provider = CatalogProvider::new(true, &["gpt-6-sol", "claude-sonnet-5.5", "gpt-6-luna"]);
    let routes = route_models(&provider);
    assert_eq!(routes.unfiltered_reason, None);
    assert_eq!(routes.served.len(), 3);
    let plan = plan(&config, &routes).unwrap();
    assert_eq!(
        plan.coordinator,
        ["gpt-6-sol", "claude-sonnet-5.5", "gpt-6-luna"]
    );
    assert_eq!(plan.dropped_coordinator[0], "claude-opus-5.5");
    assert_eq!(plan.unfiltered_reason, None);
}

#[test]
fn warm_or_catalogless_route_never_refreshes() {
    let provider = CatalogProvider::new(true, &["claude-sonnet-5.5"]);
    provider
        .loaded
        .store(true, std::sync::atomic::Ordering::SeqCst);
    let routes = route_models(&provider);
    assert_eq!(routes.served, ["claude-sonnet-5.5"]);
    assert_eq!(
        provider.refreshes.load(std::sync::atomic::Ordering::SeqCst),
        0
    );

    // Default `model_list_is_authoritative` is true: static-list routes skip the refresh.
    let fixed = ChainProvider::new("m1", &[], &["m1", "m2"], "");
    assert_eq!(route_models(&fixed).served, ["m1", "m2"]);
}

#[test]
fn unfiltered_note_is_recorded_in_the_preflight_event() {
    let _env = fixture();
    let manifest = test_run_manifest("chain-unfiltered", "owner", OvernightLimits::default());
    let plan = ChainPlan {
        coordinator: names(&["a"]),
        worker: names(&["b"]),
        unfiltered_reason: Some("route catalog unavailable; chains unfiltered".into()),
        ..Default::default()
    };
    record_plan(&manifest, &plan);
    let event = read_events(&manifest)
        .unwrap()
        .into_iter()
        .find(|e| e.kind == "model_chain_preflight")
        .unwrap();
    assert!(
        event
            .summary
            .contains("route catalog unavailable; chains unfiltered")
    );
    assert_eq!(
        event.details["unfiltered_reason"],
        "route catalog unavailable; chains unfiltered"
    );
}

/// Serves its whole list but refuses every `set_model`, like a route that
/// lists models yet cannot switch to them.
#[derive(Clone)]
struct RefusingProvider(Vec<String>);

#[async_trait]
impl Provider for RefusingProvider {
    async fn complete(
        &self,
        _: &[Message],
        _: &[ToolDefinition],
        _: &str,
        _: Option<&str>,
    ) -> Result<crate::provider::EventStream> {
        anyhow::bail!("unused")
    }
    fn name(&self) -> &str {
        "refusing-fake"
    }
    fn model(&self) -> String {
        "not-in-chain".into()
    }
    fn set_model(&self, model: &str) -> Result<()> {
        anyhow::bail!("cannot switch to {model}")
    }
    fn available_models_for_switching(&self) -> Vec<String> {
        self.0.clone()
    }
    fn fork(&self) -> Arc<dyn Provider> {
        Arc::new(self.clone())
    }
}

#[tokio::test]
async fn visible_start_keeps_the_session_model_on_a_route_without_a_model_list() {
    let _env = fixture();
    // Fixed single-model route: no list, cannot switch. Nothing to enforce.
    let provider: Arc<dyn Provider> = Arc::new(RefusingProvider(Vec::new()));
    let registry = Registry::new(provider.clone()).await;
    let launch = start_overnight_run(OvernightStartOptions {
        duration: OvernightDuration { minutes: 60 },
        mission: None,
        parent_session: Session::create(None, None),
        provider,
        registry,
        working_dir: None,
        use_current_session: true,
        limits: OvernightLimits::default(),
    })
    .expect("a route with no model list must not block the start");
    assert_eq!(launch.manifest.model, "not-in-chain");
}

#[tokio::test]
async fn visible_start_blocks_when_the_route_accepts_no_chain_model() {
    let _env = fixture();
    let config = OvernightConfig::default();
    let provider: Arc<dyn Provider> = Arc::new(RefusingProvider(
        config
            .coordinator_model_chain
            .iter()
            .chain(&config.worker_model_chain)
            .cloned()
            .collect(),
    ));
    let registry = Registry::new(provider.clone()).await;
    let error = match start_overnight_run(OvernightStartOptions {
        duration: OvernightDuration { minutes: 60 },
        mission: None,
        parent_session: Session::create(None, None),
        provider,
        registry,
        working_dir: None,
        use_current_session: true,
        limits: OvernightLimits::default(),
    }) {
        Ok(_) => panic!("a route that refuses every chain model must block the start"),
        Err(error) => error.to_string(),
    };
    assert!(
        error.contains("accepted no coordinator chain model"),
        "{error}"
    );
    assert!(error.contains("cannot switch to"), "{error}");
    assert!(latest_manifest().unwrap().is_none(), "no run was created");
}

#[test]
fn visible_held_failure_blocks_the_run_with_the_reason() {
    let _env = fixture();
    let provider = ChainProvider::new("m1", &[], &["m1", "m2"], "x");
    let mut manifest = run_with_plan("chain-visible-held", "visible-held", &["m1", "m2"], &["w1"]);
    manifest.parent_session_id = "visible-held".into();
    save_manifest(&manifest).unwrap();

    let advance = try_advance_provider_for_session(
        "visible-held",
        &provider,
        None,
        &|| None,
        "AuthRevoked: token revoked",
    );
    let Some(SessionAdvance::Stop(Some(reason))) = advance else {
        panic!("a held coordinator failure must stop with a reason");
    };
    assert!(
        reason.contains("AuthRevoked") && reason.contains("does not advance"),
        "{reason}"
    );
    assert_eq!(provider.model(), "m1", "a held chain does not move");
    let done = load_manifest("chain-visible-held").unwrap();
    assert_eq!(done.status, OvernightRunStatus::Blocked);
    assert!(kinds(&done).contains(&"run_blocked".to_string()));
}

#[test]
fn visible_setter_refuses_a_switch_that_changes_the_api_method() {
    let provider = ChainProvider::new("m1", &[], &["m1", "m2"], "x");
    let method = Mutex::new("openai-api".to_string());
    // The route reports a different API method once the model changed.
    let route = || {
        let current = provider.model();
        Some(if current == "m1" {
            method.lock().unwrap().clone()
        } else {
            "openai-oauth".to_string()
        })
    };
    let error = set_provider_model_on_same_route(&provider, None, &route, "m2")
        .unwrap_err()
        .to_string();
    assert!(error.contains("refusing to switch"), "{error}");
    assert_eq!(provider.model(), "m1", "the switch is undone");

    // Same API method before and after: allowed.
    let stable = || Some("openai-api".to_string());
    set_provider_model_on_same_route(&provider, None, &stable, "m2").unwrap();
    assert_eq!(provider.model(), "m2");
}

#[test]
fn plan_file_is_versioned_and_other_versions_are_derived_again() {
    let _env = fixture();
    let config = crate::config::config().overnight.clone();

    // Current version: read as written, nothing derived.
    let current = run_with_plan("plan-current", "owner", &["c1"], &["w1"]);
    assert_eq!(coordinator_models(&current), ["c1"]);
    assert!(!kinds(&current).contains(&"model_chain_plan_derived".to_string()));
    let raw = std::fs::read_to_string(current.run_dir.join(PLAN_FILE)).unwrap();
    assert!(
        raw.contains("\"version\": 1") || raw.contains("\"version\":1"),
        "{raw}"
    );

    // Legacy file without a version: derived from config, persisted, recorded.
    let legacy = test_run_manifest("plan-legacy", "owner", OvernightLimits::default());
    std::fs::create_dir_all(&legacy.run_dir).unwrap();
    std::fs::write(
        legacy.run_dir.join(PLAN_FILE),
        r#"{"coordinator":["stale"],"worker":["stale"]}"#,
    )
    .unwrap();
    assert_eq!(
        coordinator_models(&legacy),
        dedupe(&config.coordinator_model_chain)
    );
    assert_eq!(
        chain_models(&legacy, ChainRole::Worker),
        dedupe(&config.worker_model_chain)
    );
    let events: Vec<_> = kinds(&legacy)
        .into_iter()
        .filter(|k| k == "model_chain_plan_derived")
        .collect();
    assert_eq!(
        events.len(),
        1,
        "derived once, then read back from the persisted file"
    );
    let rewritten: ChainPlan = storage::read_json(&legacy.run_dir.join(PLAN_FILE)).unwrap();
    assert_eq!(rewritten.version, PLAN_VERSION);
    assert!(
        rewritten
            .unfiltered_reason
            .unwrap()
            .contains("derived on resume")
    );

    // A derived plan was never route-filtered: it drives advances, but does not
    // pick the model of a newly spawned worker.
    let stale = test_run_manifest("plan-stale", "stale-owner", OvernightLimits::default());
    std::fs::create_dir_all(&stale.run_dir).unwrap();
    save_manifest(&stale).unwrap();
    assert_eq!(resolve_spawn_request(None, "stale-owner", None), None);
    assert_eq!(worker_chain_default_model("stale-owner"), None);

    // Missing file: same derivation.
    let missing = test_run_manifest("plan-missing", "owner", OvernightLimits::default());
    assert_eq!(
        coordinator_models(&missing),
        dedupe(&config.coordinator_model_chain)
    );
    assert!(kinds(&missing).contains(&"model_chain_plan_derived".to_string()));
}

#[test]
fn preflight_event_notes_a_refreshed_catalog() {
    let _env = fixture();
    let manifest = test_run_manifest("chain-refreshed", "owner", OvernightLimits::default());
    let plan = ChainPlan {
        coordinator: names(&["a"]),
        worker: names(&["b"]),
        dropped_coordinator: names(&["x"]),
        catalog_refreshed: true,
        ..Default::default()
    };
    record_plan(&manifest, &plan);
    let event = read_events(&manifest)
        .unwrap()
        .into_iter()
        .find(|e| e.kind == "model_chain_preflight")
        .unwrap();
    assert!(
        event
            .summary
            .contains("route catalog refreshed before filtering")
    );
    assert!(event.summary.contains("dropped (not served by route)"));
    assert!(!event.summary.contains("unfiltered"));
}

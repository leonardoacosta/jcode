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
    let first = select_first(&names(&["a", "b", "c"]), |model| {
        if model == "c" {
            Ok(())
        } else {
            anyhow::bail!("not served")
        }
    });
    assert_eq!(first.selected.as_deref(), Some("c"));
    assert_eq!(first.skipped.len(), 2);
    assert!(first.fatal.is_none());
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
#[derive(Clone)]
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

fn route_models_for(provider: &(impl Provider + Clone + 'static)) -> RouteModels {
    let provider: Arc<dyn Provider> = Arc::new(provider.clone());
    route_models(&provider)
}

#[test]
fn cold_catalog_that_stays_cold_keeps_full_chains_and_notes_it() {
    let config = OvernightConfig::default();
    let provider = CatalogProvider::new(false, &[]);
    let routes = route_models_for(&provider);
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
    let routes = route_models_for(&provider);
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
    let routes = route_models_for(&provider);
    assert_eq!(routes.served, ["claude-sonnet-5.5"]);
    assert_eq!(
        provider.refreshes.load(std::sync::atomic::Ordering::SeqCst),
        0
    );

    // Default `model_list_is_authoritative` is true: static-list routes skip the refresh.
    let fixed = ChainProvider::new("m1", &[], &["m1", "m2"], "");
    assert_eq!(route_models_for(&fixed).served, ["m1", "m2"]);
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
    // The run says what it did, like the supervised path does.
    let events = read_events(&launch.manifest).unwrap();
    let started = events
        .iter()
        .find(|e| e.kind == "model_chain_started")
        .expect("model_chain_started is recorded");
    assert!(
        started.summary.contains("no model list") && started.summary.contains("not-in-chain"),
        "{}",
        started.summary
    );
    let preflight = events
        .iter()
        .find(|e| e.kind == "model_chain_preflight")
        .expect("model_chain_preflight is recorded");
    assert!(
        preflight
            .summary
            .contains("route exposes no model list; kept session model not-in-chain"),
        "{}",
        preflight.summary
    );
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

fn route_serving(served: &[&str]) -> RouteModels {
    route(&names(served))
}

fn raw_plan_file(manifest: &OvernightManifest, body: &str) {
    std::fs::create_dir_all(&manifest.run_dir).unwrap();
    std::fs::write(manifest.run_dir.join(PLAN_FILE), body).unwrap();
}

fn count_kind(manifest: &OvernightManifest, kind: &str) -> usize {
    kinds(manifest).iter().filter(|k| *k == kind).count()
}

/// H1: a run resumed without a usable plan is route-filtered once, at resume,
/// through the real plan file; workers then start on the filtered worker chain.
#[test]
fn resume_filters_a_derived_plan_and_workers_use_it() {
    let _env = fixture();
    let config = crate::config::config().overnight.clone();
    let served_worker = config.worker_model_chain[1].clone();
    let served_coordinator = config.coordinator_model_chain[1].clone();
    assert_ne!(served_worker, config.worker_model_chain[0]);

    for (run_id, body) in [
        (
            "h1-v0",
            Some(r#"{"coordinator":["stale"],"worker":["stale"]}"#),
        ),
        ("h1-none", None),
        (
            "h1-v99",
            Some(r#"{"version":99,"coordinator":["x"],"worker":["x"]}"#),
        ),
    ] {
        let owner = format!("{run_id}-owner");
        let manifest = test_run_manifest(run_id, &owner, OvernightLimits::default());
        if let Some(body) = body {
            raw_plan_file(&manifest, body);
        }
        // Before resume: nothing checked against the route may drive a spawn.
        assert_eq!(resolve_spawn_request(None, &owner, None), None, "{run_id}");

        let route = route_serving(&[&served_coordinator, &served_worker]);
        let plan = ensure_plan_with(&manifest, || route).unwrap();
        // The served list is shared, so the coordinator chain keeps every model
        // the route serves, in chain order (the worker model is in it too).
        let (expected_coordinator, _) = filter_served(
            &config.coordinator_model_chain,
            &[served_coordinator.clone(), served_worker.clone()],
        );
        assert_eq!(plan.coordinator, expected_coordinator, "{run_id}");
        assert!(plan.coordinator.len() < config.coordinator_model_chain.len());
        assert_eq!(plan.worker, [served_worker.clone()], "{run_id}");
        assert!(plan.derived && !plan.route_unchecked, "{run_id}");

        // The persisted file is the filtered one, and the first spawn uses it.
        let stored: ChainPlan = storage::read_json(&manifest.run_dir.join(PLAN_FILE)).unwrap();
        assert_eq!(stored, plan, "{run_id}");
        assert_eq!(
            resolve_spawn_request(None, &owner, None).as_deref(),
            Some(served_worker.as_str()),
            "{run_id}: worker[0] of the filtered chain"
        );
        assert_eq!(
            worker_chain_default_model(&owner).as_deref(),
            Some(served_worker.as_str())
        );
        assert_eq!(
            count_kind(&manifest, "model_chain_plan_derived"),
            1,
            "{run_id}"
        );

        // A second resume reads the stored plan: no new route load, no event.
        let again = ensure_plan_with(&manifest, || panic!("route must not load twice")).unwrap();
        assert_eq!(again, plan, "{run_id}");
        assert_eq!(
            count_kind(&manifest, "model_chain_plan_derived"),
            1,
            "{run_id}"
        );
    }
}

/// H1: a plan derived without a route (advance path before resume) is replaced
/// by the route-filtered one at resume.
#[test]
fn resume_replaces_a_route_unchecked_plan() {
    let _env = fixture();
    let config = crate::config::config().overnight.clone();
    let manifest = test_run_manifest(
        "h1-unchecked",
        "h1-unchecked-owner",
        OvernightLimits::default(),
    );
    let unchecked = plan_for_run(&manifest);
    assert!(unchecked.route_unchecked);
    assert_eq!(worker_chain_default_model("h1-unchecked-owner"), None);

    let keep = config.worker_model_chain[2].clone();
    let route = route_serving(&[&config.coordinator_model_chain[0], &keep]);
    let plan = ensure_plan_with(&manifest, || route).unwrap();
    assert_eq!(plan.worker, [keep.clone()]);
    assert!(!plan.route_unchecked);
    assert_eq!(
        worker_chain_default_model("h1-unchecked-owner").as_deref(),
        Some(keep.as_str())
    );
}

/// H1: a route that serves no model of a chain blocks the resume, and the old
/// file is left alone.
#[test]
fn resume_blocks_when_the_route_serves_no_chain_model_and_keeps_the_file() {
    let _env = fixture();
    let manifest = test_run_manifest("h1-block", "h1-block-owner", OvernightLimits::default());
    let body = r#"{"version":99,"coordinator":["x"],"worker":["x"]}"#;
    raw_plan_file(&manifest, body);
    let reasons = ensure_plan_with(&manifest, || route_serving(&["unrelated"])).unwrap_err();
    assert!(
        reasons
            .iter()
            .any(|r| r.contains("serves no coordinator chain model")),
        "{reasons:?}"
    );
    assert_eq!(
        std::fs::read_to_string(manifest.run_dir.join(PLAN_FILE)).unwrap(),
        body
    );
    assert_eq!(count_kind(&manifest, "model_chain_plan_derived"), 0);
}

/// L: one stale plan yields one derived event even when several readers race,
/// and the file it replaces is kept as `model-chains.json.v<N>.bak`.
#[test]
fn deriving_records_once_under_a_race_and_backs_up_the_old_file() {
    let _env = fixture();
    let manifest = test_run_manifest("l-race", "l-race-owner", OvernightLimits::default());
    let body = r#"{"version":99,"coordinator":["newer"],"worker":["newer"],"extra":true}"#;
    raw_plan_file(&manifest, body);

    std::thread::scope(|scope| {
        for _ in 0..8 {
            scope.spawn(|| {
                let _ = plan_for_run(&manifest);
            });
        }
    });
    assert_eq!(count_kind(&manifest, "model_chain_plan_derived"), 1);
    let backup = manifest.run_dir.join("model-chains.json.v99.bak");
    assert_eq!(std::fs::read_to_string(backup).unwrap(), body, "bytes kept");
    let rewritten: ChainPlan = storage::read_json(&manifest.run_dir.join(PLAN_FILE)).unwrap();
    assert_eq!(rewritten.version, PLAN_VERSION);

    // A versionless v0 file and an unreadable one are kept too.
    for (run_id, body, suffix) in [
        ("l-v0", r#"{"coordinator":["old"],"worker":["old"]}"#, "v0"),
        ("l-junk", "{not json", "unreadable"),
    ] {
        let manifest = test_run_manifest(run_id, run_id, OvernightLimits::default());
        raw_plan_file(&manifest, body);
        let _ = plan_for_run(&manifest);
        let backup = manifest
            .run_dir
            .join(format!("model-chains.json.{suffix}.bak"));
        assert_eq!(std::fs::read_to_string(backup).unwrap(), body, "{run_id}");
    }
}

/// L: when the old file cannot be backed up it is not overwritten.
#[cfg(unix)]
#[test]
fn an_unbackable_plan_file_is_never_overwritten() {
    use std::os::unix::fs::PermissionsExt;
    let _env = fixture();
    let manifest = test_run_manifest("l-nobackup", "l-nobackup-owner", OvernightLimits::default());
    let body = r#"{"version":99,"coordinator":["newer"],"worker":["newer"]}"#;
    raw_plan_file(&manifest, body);
    let dir = &manifest.run_dir;
    std::fs::set_permissions(dir, std::fs::Permissions::from_mode(0o555)).unwrap();
    // Root ignores directory modes; nothing to prove then.
    let writable = std::fs::write(dir.join("probe"), "x").is_ok();
    let plan = plan_for_run(&manifest);
    std::fs::set_permissions(dir, std::fs::Permissions::from_mode(0o755)).unwrap();
    if writable {
        return;
    }
    assert!(plan.route_unchecked, "still usable in memory");
    assert_eq!(std::fs::read_to_string(dir.join(PLAN_FILE)).unwrap(), body);
}

/// L: dropping the version filter in `current_plan` would let an old file with
/// no route check pick a new worker's model.
#[test]
fn current_plan_ignores_files_without_the_current_version_or_a_route_check() {
    let _env = fixture();
    for (run_id, body) in [
        ("cp-v0", r#"{"coordinator":["stale"],"worker":["stale"]}"#),
        (
            "cp-v99",
            r#"{"version":99,"coordinator":["s"],"worker":["s"]}"#,
        ),
        (
            "cp-unchecked",
            r#"{"version":1,"coordinator":["s"],"worker":["s"],"route_unchecked":true}"#,
        ),
    ] {
        let owner = format!("{run_id}-owner");
        let manifest = test_run_manifest(run_id, &owner, OvernightLimits::default());
        raw_plan_file(&manifest, body);
        assert!(current_plan(&manifest).is_none(), "{run_id}");
        assert_eq!(resolve_spawn_request(None, &owner, None), None, "{run_id}");
        assert_eq!(worker_chain_default_model(&owner), None, "{run_id}");
    }
    let ok = run_with_plan("cp-ok", "cp-ok-owner", &["c1"], &["w1"]);
    assert_eq!(current_plan(&ok).unwrap().worker, ["w1"]);
}

/// A provider whose switch drifts the route (name or display name changes with
/// the model), and whose undo can be made to fail.
#[derive(Clone)]
struct DriftProvider {
    model: Arc<Mutex<String>>,
    served: Vec<String>,
    fail_restore_to: Option<String>,
}

impl DriftProvider {
    fn new(start: &str, served: &[String], fail_restore_to: Option<&str>) -> Self {
        Self {
            model: Arc::new(Mutex::new(start.to_string())),
            served: served.to_vec(),
            fail_restore_to: fail_restore_to.map(str::to_string),
        }
    }
}

#[async_trait]
impl Provider for DriftProvider {
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
        "drift-fake"
    }
    /// The profile in use follows the model: a switch changes the route.
    fn display_name(&self) -> String {
        format!("profile-for-{}", self.model.lock().unwrap())
    }
    fn model(&self) -> String {
        self.model.lock().unwrap().clone()
    }
    fn set_model(&self, model: &str) -> Result<()> {
        if self.fail_restore_to.as_deref() == Some(model) {
            anyhow::bail!("cannot return to {model}");
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

fn all_chain_models() -> Vec<String> {
    let config = OvernightConfig::default();
    config
        .coordinator_model_chain
        .iter()
        .chain(&config.worker_model_chain)
        .cloned()
        .collect()
}

/// M3: the route identity covers providers that are neither OpenAI nor Claude.
/// With `&|| None` (the old closure) this switch would have been accepted.
#[test]
fn route_identity_catches_a_drift_on_any_provider() {
    let provider = DriftProvider::new("start", &all_chain_models(), None);
    let identity = || provider_route_identity(&provider);
    let error = set_provider_model_on_same_route(&provider, None, &identity, "next")
        .unwrap_err()
        .to_string();
    assert!(error.contains("refusing to switch"), "{error}");
    assert_eq!(provider.model(), "start", "the switch is undone");

    // The old closure accepted it: proof the identity is what refuses.
    set_provider_model_on_same_route(&provider, None, &|| None, "next").unwrap();
    assert_eq!(provider.model(), "next");
}

#[tokio::test]
async fn visible_start_blocks_when_every_switch_drifts_the_route() {
    let _env = fixture();
    let provider: Arc<dyn Provider> =
        Arc::new(DriftProvider::new("start", &all_chain_models(), None));
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
        Ok(_) => panic!("a switch that drifts the route must not be accepted"),
        Err(error) => error.to_string(),
    };
    assert!(
        error.contains("accepted no coordinator chain model"),
        "{error}"
    );
    assert!(error.contains("refusing to switch"), "{error}");
}

/// M1: a refused switch whose undo fails is fatal, not skipped.
#[test]
fn a_failed_route_restore_is_fatal_for_selection_and_advances() {
    let _env = fixture();
    let identity_of = |p: &DriftProvider| provider_route_identity(p);

    let provider = DriftProvider::new("start", &all_chain_models(), Some("start"));
    let error =
        set_provider_model_on_same_route(&provider, None, &|| identity_of(&provider), "next")
            .unwrap_err();
    assert!(is_route_restore_failure(&error), "{error:#}");
    assert!(
        format!("{error:#}").contains("restoring start failed"),
        "{error:#}"
    );

    // Selection stops at the first fatal error instead of trying the next model.
    let tried = Mutex::new(Vec::new());
    let first = select_first(&names(&["a", "b"]), |model| {
        tried.lock().unwrap().push(model.to_string());
        Err(RouteRestoreFailed("undo failed".into()).into())
    });
    assert!(first.selected.is_none());
    assert!(first.fatal.as_deref().unwrap().contains("undo failed"));
    assert_eq!(
        tried.lock().unwrap().len(),
        1,
        "no further candidate is tried"
    );

    // A visible coordinator advance blocks the run with a distinct event.
    let provider = DriftProvider::new("m1", &names(&["m1", "m2"]), Some("m1"));
    let mut manifest = run_with_plan("m1-visible", "m1-visible", &["m1", "m2"], &["w1"]);
    manifest.parent_session_id = "m1-visible".into();
    save_manifest(&manifest).unwrap();
    let advance = try_advance_provider_for_session(
        "m1-visible",
        &provider,
        None,
        &|| identity_of(&provider),
        "model m1 unavailable",
    );
    let Some(SessionAdvance::Stop(Some(reason))) = advance else {
        panic!("a failed restore must stop the run with a reason");
    };
    assert!(reason.contains("restoring m1 failed"), "{reason}");
    let done = load_manifest("m1-visible").unwrap();
    assert_eq!(done.status, OvernightRunStatus::Blocked);
    let recorded = kinds(&done);
    assert!(recorded.contains(&"model_chain_route_restore_failed".to_string()));
    assert!(recorded.contains(&"run_blocked".to_string()));
}

/// M1 through an agent: the same-route setter reports the failed undo.
#[tokio::test]
async fn agent_route_restore_failure_is_fatal() {
    let _env = fixture();
    let drift = DriftProvider::new("start", &names(&["start", "next"]), Some("start"));
    let provider: Arc<dyn Provider> = Arc::new(drift);
    let registry = Registry::new(provider.clone()).await;
    let mut agent = Agent::new(provider, registry);
    let error = agent.set_model_on_same_route("next").unwrap_err();
    assert!(is_route_restore_failure(&error), "{error:#}");
    // The provider stayed on `next`, so selection through a fresh agent sees
    // the same refusal and reports it as fatal too.
    let drift = DriftProvider::new("start", &names(&["start", "next"]), Some("start"));
    let provider: Arc<dyn Provider> = Arc::new(drift);
    let registry = Registry::new(provider.clone()).await;
    let mut agent = Agent::new(provider, registry);
    let first = select_first(&names(&["next"]), |m| agent.set_model_on_same_route(m));
    assert!(first.fatal.is_some(), "{:?}", first.skipped);
}

/// H2: a Permanent error (context limit, oversized request) does not stop a
/// coordinator in the chain; only auth failures do.
#[test]
fn only_auth_failures_stop_a_visible_coordinator() {
    let _env = fixture();
    for (run_id, error, stops) in [
        (
            "h2-ctx",
            "prompt is too long: maximum context length exceeded",
            false,
        ),
        ("h2-413", "413 request_too_large", false),
        ("h2-auth", "AuthRevoked: token revoked", true),
        ("h2-expired", "401 unauthorized: token has expired", true),
    ] {
        let provider = ChainProvider::new("m1", &[], &["m1", "m2"], "x");
        let mut manifest = run_with_plan(run_id, run_id, &["m1", "m2"], &["w1"]);
        manifest.parent_session_id = run_id.into();
        save_manifest(&manifest).unwrap();
        let advance = try_advance_provider_for_session(run_id, &provider, None, &|| None, error);
        match (stops, advance) {
            (true, Some(SessionAdvance::Stop(Some(_)))) => {
                assert_eq!(
                    load_manifest(run_id).unwrap().status,
                    OvernightRunStatus::Blocked
                );
            }
            (false, Some(SessionAdvance::Stop(None))) => {
                let loaded = load_manifest(run_id).unwrap();
                assert_eq!(loaded.status, OvernightRunStatus::Running, "{run_id}");
                assert!(
                    !kinds(&loaded).contains(&"run_blocked".to_string()),
                    "{run_id}"
                );
            }
            (_, other) => panic!(
                "{run_id}: unexpected {}",
                match other {
                    None => "none".to_string(),
                    Some(SessionAdvance::Switched) => "switched".to_string(),
                    Some(SessionAdvance::Stop(m)) => format!("stop({m:?})"),
                }
            ),
        }
        assert_eq!(provider.model(), "m1", "{run_id}: no switch");
    }
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

/// A refresh that blocks its thread synchronously (no await point) must not
/// hold the start path past the timeout.
#[test]
fn bounded_refresh_gives_up_on_a_synchronously_blocking_refresh() {
    #[derive(Clone)]
    struct Blocking;
    #[async_trait]
    impl Provider for Blocking {
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
            "blocking"
        }
        fn model(&self) -> String {
            "m".into()
        }
        async fn refresh_model_catalog(
            &self,
        ) -> Result<jcode_provider_core::ModelCatalogRefreshSummary> {
            std::thread::sleep(std::time::Duration::from_secs(5));
            Ok(Default::default())
        }
        fn fork(&self) -> Arc<dyn Provider> {
            Arc::new(self.clone())
        }
    }
    let started = std::time::Instant::now();
    let result = refresh_catalog_bounded(Arc::new(Blocking), std::time::Duration::from_millis(200));
    assert!(started.elapsed() < std::time::Duration::from_secs(2));
    assert!(result.unwrap_err().contains("timed out"));
}

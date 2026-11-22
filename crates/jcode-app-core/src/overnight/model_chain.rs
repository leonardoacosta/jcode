//! Per-role model chains for overnight runs.
//!
//! An overnight coordinator and the workers it owns each walk an ordered model
//! chain when a turn fails with a recoverable model failure. Every step stays
//! on the session's own provider route and account (pinned through the same
//! route prefix session restore uses); a switch that would change provider or
//! account is refused. This is an overnight-only exception: sessions without an
//! active run never switch models automatically.

use super::*;
use crate::config::OvernightConfig;
use crate::message::Role;
use crate::server::session_control;
use jcode_provider_core::{ProviderFailureKind, classify_provider_failure};
use serde::{Deserialize, Serialize};

const PLAN_FILE: &str = "model-chains.json";
const MAX_OWNER_HOPS: usize = 8;

/// Whether `kind` may move a chain to its next model.
pub(crate) fn kind_advances_chain(kind: ProviderFailureKind) -> bool {
    matches!(
        kind,
        ProviderFailureKind::Transient
            | ProviderFailureKind::RateLimited
            | ProviderFailureKind::ModelUnavailable
            | ProviderFailureKind::QuotaExhausted
    )
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Advance {
    /// Switch to this model next.
    Next(String),
    /// Every chain model has been tried (listed in order).
    Exhausted(Vec<String>),
    /// The failure kind never advances a chain (auth, permanent).
    Hold,
}

/// Ordered model list with a cursor. Pure: no provider access.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ModelChain {
    models: Vec<String>,
    next_index: usize,
    tried: Vec<String>,
}

impl ModelChain {
    pub(crate) fn new(models: Vec<String>) -> Self {
        Self {
            models,
            next_index: 0,
            tried: Vec::new(),
        }
    }

    /// A chain whose cursor sits on `current`. A current model outside the
    /// chain (for example an explicit worker model) counts as tried and the
    /// chain applies from its first model.
    pub(crate) fn starting_at(models: Vec<String>, current: &str) -> Self {
        let current_key = model_key(current);
        let mut chain = Self::new(models);
        match chain
            .models
            .iter()
            .position(|model| model_key(model) == current_key)
        {
            Some(position) => {
                chain.next_index = position + 1;
                chain.tried = chain.models[..=position].to_vec();
            }
            None => chain.tried = vec![current.trim().to_string()],
        }
        chain
    }

    /// Next untried model, without a failure (used to choose the first model).
    pub(crate) fn take_next(&mut self) -> Option<String> {
        let model = self.models.get(self.next_index)?.clone();
        self.next_index += 1;
        self.tried.push(model.clone());
        Some(model)
    }

    pub(crate) fn advance(&mut self, kind: ProviderFailureKind) -> Advance {
        if !kind_advances_chain(kind) {
            return Advance::Hold;
        }
        match self.take_next() {
            Some(model) => Advance::Next(model),
            None => Advance::Exhausted(self.tried.clone()),
        }
    }

    pub(crate) fn tried(&self) -> &[String] {
        &self.tried
    }
}

/// Comparison key for model names: case, `provider:` prefix, `@pin` suffix and
/// `.`/`-` spelling differences do not make two names different models.
fn model_key(model: &str) -> String {
    let model = model.trim();
    let model = model.split('@').next().unwrap_or(model);
    let model = model.rsplit(':').next().unwrap_or(model);
    model.to_ascii_lowercase().replace('.', "-")
}

fn dedupe(models: &[String]) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for model in models.iter().map(|m| m.trim()).filter(|m| !m.is_empty()) {
        if !out.iter().any(|o| model_key(o) == model_key(model)) {
            out.push(model.to_string());
        }
    }
    out
}

/// Keep chain models the route serves, using the route's own spelling. An empty
/// `served` list means the route has no model list, so every model is kept.
/// Returns `(kept, dropped)`.
pub(crate) fn filter_served(chain: &[String], served: &[String]) -> (Vec<String>, Vec<String>) {
    let chain = dedupe(chain);
    if served.is_empty() {
        return (chain, Vec::new());
    }
    let mut kept = Vec::new();
    let mut dropped = Vec::new();
    for model in chain {
        let key = model_key(&model);
        match served.iter().find(|s| model_key(s) == key) {
            Some(spelling) => kept.push(spelling.trim().to_string()),
            None => dropped.push(model),
        }
    }
    (kept, dropped)
}

/// Layout version of `model-chains.json`. A file with another (or no) version
/// is not trusted: the plan is derived again from config (see `plan_for_run`).
const PLAN_VERSION: u32 = 1;

/// Chains after the route filter, persisted next to the run's manifest.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct ChainPlan {
    /// 0 when absent: files written before versioning.
    #[serde(default)]
    pub version: u32,
    pub coordinator: Vec<String>,
    pub worker: Vec<String>,
    #[serde(default)]
    pub dropped_coordinator: Vec<String>,
    #[serde(default)]
    pub dropped_worker: Vec<String>,
    /// Set when the route's model list could not be loaded, so the chains were
    /// kept whole instead of filtered.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub unfiltered_reason: Option<String>,
    /// The route catalog was cold at start and was refreshed before filtering.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub catalog_refreshed: bool,
    /// Built on resume from config (no current plan file), not at start.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub derived: bool,
    /// Derived without a provider, so the route was never consulted. Such a
    /// plan may drive chain advances (each switch is checked by the route) but
    /// never picks a new worker's model, and resume replaces it.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub route_unchecked: bool,
}

impl Default for ChainPlan {
    fn default() -> Self {
        Self {
            version: PLAN_VERSION,
            coordinator: Vec::new(),
            worker: Vec::new(),
            dropped_coordinator: Vec::new(),
            dropped_worker: Vec::new(),
            unfiltered_reason: None,
            catalog_refreshed: false,
            derived: false,
            route_unchecked: false,
        }
    }
}

const CATALOG_REFRESH_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(15);
const CATALOG_UNAVAILABLE: &str = "route catalog unavailable; chains unfiltered";

/// The route's served models for preflight filtering, plus why the list is
/// empty when the catalog could not be loaded.
pub(crate) struct RouteModels {
    pub served: Vec<String>,
    pub unfiltered_reason: Option<String>,
    /// The catalog was not loaded at start, so a refresh was attempted.
    pub refreshed: bool,
}

/// Served models for `provider`'s route. A catalog-backed route whose catalog
/// is not loaded gets one bounded refresh; if that does not produce a real
/// list, `served` is empty (unknown) so `filter_served` keeps every chain
/// model instead of shrinking the chains to the current model.
pub(crate) fn route_models(provider: &Arc<dyn Provider>) -> RouteModels {
    route_models_with(provider.as_ref(), || {
        refresh_catalog_bounded(Arc::clone(provider), CATALOG_REFRESH_TIMEOUT)
    })
}

fn route_models_with(
    provider: &dyn Provider,
    refresh: impl FnOnce() -> std::result::Result<(), String>,
) -> RouteModels {
    let mut failure = None;
    let refreshed = !provider.model_list_is_authoritative();
    if refreshed {
        failure = refresh().err();
    }
    if provider.model_list_is_authoritative() {
        return RouteModels {
            served: provider.available_models_for_switching(),
            unfiltered_reason: None,
            refreshed,
        };
    }
    RouteModels {
        refreshed,
        served: Vec::new(),
        unfiltered_reason: Some(match failure {
            Some(error) => format!("{CATALOG_UNAVAILABLE} ({error})"),
            None => CATALOG_UNAVAILABLE.to_string(),
        }),
    }
}

/// Run `refresh_model_catalog` on its own detached thread and runtime, and wait
/// at most `timeout` for the result. The wait is a channel timeout, not a
/// future timeout, so a refresh that blocks its thread synchronously (file or
/// process IO inside an async fn) still cannot hold up the start; the stray
/// thread finishes or stays blocked on its own and is never joined.
fn refresh_catalog_bounded(
    provider: Arc<dyn Provider>,
    timeout: std::time::Duration,
) -> std::result::Result<(), String> {
    let (tx, rx) = std::sync::mpsc::channel();
    std::thread::Builder::new()
        .name("overnight-catalog-refresh".into())
        .spawn(move || {
            let result = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .map_err(|error| error.to_string())
                .and_then(|runtime| {
                    runtime
                        .block_on(provider.refresh_model_catalog())
                        .map(|_| ())
                        .map_err(|error| format!("{error:#}"))
                });
            let _ = tx.send(result);
        })
        .map_err(|error| error.to_string())?;
    match rx.recv_timeout(timeout) {
        Ok(result) => result,
        Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
            Err(format!("timed out after {}s", timeout.as_secs()))
        }
        Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => {
            Err("catalog refresh panicked".to_string())
        }
    }
}

/// Filter both chains against the route. `Err` carries the block reasons when
/// either chain is left empty.
pub(crate) fn plan(
    config: &OvernightConfig,
    route: &RouteModels,
) -> Result<ChainPlan, Vec<String>> {
    let served = route.served.as_slice();
    let (coordinator, dropped_coordinator) = filter_served(&config.coordinator_model_chain, served);
    let (worker, dropped_worker) = filter_served(&config.worker_model_chain, served);
    let mut reasons = Vec::new();
    for (role, kept, dropped) in [
        ("coordinator", &coordinator, &dropped_coordinator),
        ("worker", &worker, &dropped_worker),
    ] {
        if kept.is_empty() {
            reasons.push(if dropped.is_empty() {
                format!("overnight {role} model chain is empty")
            } else {
                format!(
                    "selected route serves no {role} chain model; unserved: {}",
                    dropped.join(", ")
                )
            });
        }
    }
    if reasons.is_empty() {
        Ok(ChainPlan {
            coordinator,
            worker,
            dropped_coordinator,
            dropped_worker,
            unfiltered_reason: route.unfiltered_reason.clone(),
            catalog_refreshed: route.refreshed,
            ..Default::default()
        })
    } else {
        Err(reasons)
    }
}

pub(crate) fn persist_plan(run_dir: &Path, plan: &ChainPlan) -> Result<()> {
    storage::write_json(&run_dir.join(PLAN_FILE), plan)
}

/// What the run directory holds for the plan.
enum Stored {
    Current(ChainPlan),
    /// Missing, unreadable or another version: needs deriving again.
    Stale(String),
}

fn read_stored(run_dir: &Path) -> Stored {
    match storage::read_json::<ChainPlan>(&run_dir.join(PLAN_FILE)) {
        Ok(plan) if plan.version == PLAN_VERSION => Stored::Current(plan),
        Ok(plan) => Stored::Stale(format!(
            "plan version {} is not {PLAN_VERSION}",
            plan.version
        )),
        Err(_) => Stored::Stale("plan missing or unreadable".to_string()),
    }
}

/// Serializes deriving so one stale plan yields one derived event.
static PLAN_DERIVE_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

/// Copy a plan file this build cannot use (another version, or unreadable) to
/// `model-chains.json.v<N>.bak` before it is replaced. `Ok(())` also when no
/// file exists; `Err` when the copy failed, in which case the file must stay.
fn back_up_unusable_plan(run_dir: &Path) -> std::io::Result<()> {
    let raw = match std::fs::read(run_dir.join(PLAN_FILE)) {
        Ok(raw) => raw,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(error) => return Err(error),
    };
    let suffix = match serde_json::from_slice::<serde_json::Value>(&raw) {
        Ok(value) => format!(
            "v{}",
            value.get("version").and_then(|v| v.as_u64()).unwrap_or(0)
        ),
        Err(_) => "unreadable".to_string(),
    };
    let backup = run_dir.join(format!("{PLAN_FILE}.{suffix}.bak"));
    if backup.exists() {
        return Ok(());
    }
    std::fs::write(backup, raw)
}

/// Persist a derived plan (backing up an unusable file first) and record it
/// once. When the old file cannot be backed up it is left alone and the plan
/// is only used in memory: an unknown newer plan is never destroyed.
fn store_derived(manifest: &OvernightManifest, cause: &str, plan: ChainPlan) -> ChainPlan {
    if back_up_unusable_plan(&manifest.run_dir).is_err()
        || persist_plan(&manifest.run_dir, &plan).is_err()
    {
        return plan;
    }
    let _ = record_event(
        manifest,
        "model_chain_plan_derived",
        format!(
            "Model chain plan derived from config on resume: {cause}; {}",
            plan_summary(&plan)
        ),
        serde_json::to_value(&plan).unwrap_or_else(|_| json!({})),
        true,
    );
    plan
}

/// Resume hook: make sure the run has a plan this build trusts. A missing,
/// unreadable or other-version file (a run begun by an older build) is derived
/// again from config and filtered against the route exactly like a fresh start
/// (`route` loads the served models), then persisted so new workers and chain
/// advances use the filtered chains. `Err` carries the block reasons when the
/// route serves no model of a chain; nothing is written then.
pub(crate) fn ensure_plan_with(
    manifest: &OvernightManifest,
    route: impl FnOnce() -> RouteModels,
) -> Result<ChainPlan, Vec<String>> {
    let cause = match read_stored(&manifest.run_dir) {
        Stored::Current(plan) if !plan.route_unchecked => return Ok(plan),
        Stored::Current(_) => "route was not consulted when the plan was derived".to_string(),
        Stored::Stale(cause) => cause,
    };
    let route = route();
    let plan = ChainPlan {
        derived: true,
        ..plan(&crate::config::config().overnight, &route)?
    };
    let _guard = PLAN_DERIVE_LOCK.lock().unwrap_or_else(|p| p.into_inner());
    // Another resume may have stored a checked plan while the route loaded.
    if let Stored::Current(current) = read_stored(&manifest.run_dir)
        && !current.route_unchecked
    {
        return Ok(current);
    }
    Ok(store_derived(manifest, &cause, plan))
}

pub(crate) fn ensure_plan(
    manifest: &OvernightManifest,
    provider: &Arc<dyn Provider>,
) -> Result<ChainPlan, Vec<String>> {
    ensure_plan_with(manifest, || route_models(provider))
}

/// The run's plan for chain advances. Without a current file the plan is
/// derived from config without consulting the route (no provider is at hand
/// here): it is persisted as `route_unchecked` so the event is recorded once,
/// it never picks a new worker's model, and resume replaces it. The advance
/// path itself skips every model the route refuses.
fn plan_for_run(manifest: &OvernightManifest) -> ChainPlan {
    if let Stored::Current(plan) = read_stored(&manifest.run_dir) {
        return plan;
    }
    let _guard = PLAN_DERIVE_LOCK.lock().unwrap_or_else(|p| p.into_inner());
    let cause = match read_stored(&manifest.run_dir) {
        Stored::Current(plan) => return plan,
        Stored::Stale(cause) => cause,
    };
    let config = &crate::config::config().overnight;
    let plan = ChainPlan {
        coordinator: dedupe(&config.coordinator_model_chain),
        worker: dedupe(&config.worker_model_chain),
        unfiltered_reason: Some(format!(
            "derived on resume ({cause}); route not consulted, chains unfiltered"
        )),
        derived: true,
        route_unchecked: true,
        ..Default::default()
    };
    store_derived(manifest, &cause, plan)
}

/// One line naming both chains and why they were kept whole or trimmed.
fn plan_summary(plan: &ChainPlan) -> String {
    let mut notes = Vec::new();
    if plan.catalog_refreshed {
        notes.push("route catalog refreshed before filtering".to_string());
    }
    if let Some(reason) = plan.unfiltered_reason.as_deref() {
        notes.push(reason.to_string());
    } else if !plan.dropped_coordinator.is_empty() || !plan.dropped_worker.is_empty() {
        notes.push(format!(
            "dropped (not served by route): coordinator [{}], worker [{}]",
            plan.dropped_coordinator.join(", "),
            plan.dropped_worker.join(", ")
        ));
    }
    format!(
        "Model chains: coordinator [{}], worker [{}]{}",
        plan.coordinator.join(" > "),
        plan.worker.join(" > "),
        notes.iter().map(|n| format!("; {n}")).collect::<String>()
    )
}

/// Record the plan as a run event (dropped models included).
pub(crate) fn record_plan(manifest: &OvernightManifest, plan: &ChainPlan) {
    let _ = record_event(
        manifest,
        "model_chain_preflight",
        plan_summary(plan),
        serde_json::to_value(plan).unwrap_or_else(|_| json!({})),
        true,
    );
}

/// Route-pinned model request: the session's provider key becomes a model
/// prefix so a bare chain name cannot resolve to another provider.
pub(crate) fn pinned_request(model: &str, provider_key: Option<&str>) -> String {
    crate::provider::MultiProvider::model_switch_request_for_session_model(model, provider_key)
}

/// API method of the route `provider` is serving, as far as the provider can
/// tell: the account-qualified OAuth method, or the API-key method. `None` when
/// it only knows its name (profile routes are told apart by `name()`).
pub(crate) fn provider_route_api_method(provider: &dyn Provider) -> Option<String> {
    use jcode_provider_core::ResolvedCredential;
    let base = match provider.name().to_ascii_lowercase().as_str() {
        "openai" => "openai",
        "claude" | "anthropic" => "claude",
        _ => return None,
    };
    match (
        provider.oauth_account_label(),
        provider.active_resolved_credential(),
    ) {
        (Some(label), _) => Some(format!("{base}-oauth@{label}")),
        (None, Some(ResolvedCredential::Oauth)) => Some(format!("{base}-oauth")),
        (None, Some(ResolvedCredential::ApiKey)) => Some(format!("{base}-api")),
        (None, None) => None,
    }
}

/// A stable identity of the route `provider` is serving, for every provider:
/// its name, its display name (the OpenAI-compatible profile in use) and the
/// API method as far as the provider can tell. A model switch that changes any
/// part left the route.
pub fn provider_route_identity(provider: &dyn Provider) -> Option<String> {
    Some(format!(
        "{}|{}|{}",
        provider.name().to_ascii_lowercase(),
        provider.display_name(),
        provider_route_api_method(provider).unwrap_or_default()
    ))
}

/// A model switch whose undo failed: the provider may be left on a different
/// route than the one the run was started on, so the run must not go on.
#[derive(Debug)]
pub(crate) struct RouteRestoreFailed(pub String);

impl std::fmt::Display for RouteRestoreFailed {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for RouteRestoreFailed {}

pub(crate) fn is_route_restore_failure(error: &anyhow::Error) -> bool {
    error.downcast_ref::<RouteRestoreFailed>().is_some()
}

/// Select `model` on `provider` without leaving its route: provider name,
/// OAuth account and API method (`route`) must all be unchanged afterwards,
/// otherwise the switch is undone and refused. Used for the visible run's
/// first model and its advances; agents use `Agent::set_model_on_same_route`.
/// `route` reports the API method in use (the TUI passes its own richer view).
pub(crate) fn set_provider_model_on_same_route(
    provider: &dyn Provider,
    provider_key: Option<&str>,
    route: &dyn Fn() -> Option<String>,
    model: &str,
) -> Result<()> {
    let name = provider.name().to_string();
    let account = provider.oauth_account_label();
    let api_method = route();
    let previous = provider.model();
    crate::provider::set_model_with_auth_refresh(provider, &pinned_request(model, provider_key))?;
    if provider.name() != name || provider.oauth_account_label() != account || route() != api_method
    {
        if let Err(error) = crate::provider::set_model_with_auth_refresh(
            provider,
            &pinned_request(&previous, provider_key),
        ) {
            return Err(RouteRestoreFailed(format!(
                "{model} left the selected route ({name}) and restoring {previous} failed: {error:#}"
            ))
            .into());
        }
        anyhow::bail!("{model} is not served on the selected route ({name}); refusing to switch");
    }
    Ok(())
}

/// Result of choosing the first chain model.
pub(crate) struct FirstModel {
    pub selected: Option<String>,
    /// Candidates the route refused, with their reason.
    pub skipped: Vec<String>,
    /// A refused switch could not be undone: the run must be blocked.
    pub fatal: Option<String>,
}

/// Choose the first chain model the route accepts.
pub(crate) fn select_first(
    models: &[String],
    mut apply: impl FnMut(&str) -> Result<()>,
) -> FirstModel {
    let mut chain = ModelChain::new(models.to_vec());
    let mut skipped = Vec::new();
    while let Some(model) = chain.take_next() {
        match apply(&model) {
            Ok(()) => {
                return FirstModel {
                    selected: Some(model),
                    skipped,
                    fatal: None,
                };
            }
            Err(error) if is_route_restore_failure(&error) => {
                return FirstModel {
                    selected: None,
                    skipped,
                    fatal: Some(format!("{error:#}")),
                };
            }
            Err(error) => skipped.push(format!("{model}: {error:#}")),
        }
    }
    FirstModel {
        selected: None,
        skipped,
        fatal: None,
    }
}

pub(crate) enum AdvanceOutcome {
    Switched {
        from: String,
        to: String,
        skipped: Vec<String>,
    },
    Exhausted {
        tried: Vec<String>,
        skipped: Vec<String>,
    },
    Held,
    /// A refused switch could not be undone (see [`RouteRestoreFailed`]).
    Fatal(String),
}

/// Move `agent` to the next chain model on its own route. Models the route
/// refuses are skipped and counted as tried.
pub(crate) fn advance_agent(
    agent: &mut Agent,
    chain: &mut ModelChain,
    kind: ProviderFailureKind,
) -> AdvanceOutcome {
    let from = agent.provider_model();
    let mut skipped = Vec::new();
    loop {
        match chain.advance(kind) {
            Advance::Hold => return AdvanceOutcome::Held,
            Advance::Exhausted(tried) => return AdvanceOutcome::Exhausted { tried, skipped },
            Advance::Next(model) => match agent.set_model_on_same_route(&model) {
                Ok(()) => {
                    agent.reset_provider_session();
                    return AdvanceOutcome::Switched {
                        from,
                        to: model,
                        skipped,
                    };
                }
                Err(error) if is_route_restore_failure(&error) => {
                    return AdvanceOutcome::Fatal(format!("{error:#}"));
                }
                Err(error) => skipped.push(format!("{model}: {error:#}")),
            },
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ChainRole {
    Coordinator,
    Worker,
}

impl ChainRole {
    fn label(self) -> &'static str {
        match self {
            Self::Coordinator => "coordinator",
            Self::Worker => "worker",
        }
    }
}

/// The active overnight run `session_id` belongs to and its role in it:
/// the coordinator itself, or a session owned (directly or transitively) by it.
pub(crate) fn role_for_session(session_id: &str) -> Option<(OvernightManifest, ChainRole)> {
    if let Some(manifest) = enforce::governing_run(session_id) {
        return Some((manifest, ChainRole::Coordinator));
    }
    let store = session_control::store();
    let mut current = session_id.to_string();
    for _ in 0..MAX_OWNER_HOPS {
        let owner = store.inspect(&current).ok().flatten()?.owner_session_id?;
        if let Some(manifest) = enforce::governing_run(&owner) {
            return Some((manifest, ChainRole::Worker));
        }
        current = owner;
    }
    None
}

/// Coordinator chain models for a run (the run's plan, derived from config when absent).
pub(super) fn coordinator_models(manifest: &OvernightManifest) -> Vec<String> {
    chain_models(manifest, ChainRole::Coordinator)
}

/// The persisted plan, only when it is current and was checked against the
/// route (a fresh start, or `ensure_plan` on resume). A plan that would have to
/// be derived again, or was derived without the route, must not decide which
/// model a new worker is spawned on (the route may not serve it).
fn current_plan(manifest: &OvernightManifest) -> Option<ChainPlan> {
    match read_stored(&manifest.run_dir) {
        Stored::Current(plan) if !plan.route_unchecked => Some(plan),
        _ => None,
    }
}

fn chain_models(manifest: &OvernightManifest, role: ChainRole) -> Vec<String> {
    let plan = plan_for_run(manifest);
    match role {
        ChainRole::Coordinator => plan.coordinator,
        ChainRole::Worker => plan.worker,
    }
}

/// Model request for a worker spawned by `requester_session_id`. An explicit
/// request always wins; otherwise a session that owns an active overnight run
/// gets the first served worker-chain model pinned to its own route. Every
/// other session is untouched (`None` keeps the normal selection).
pub(crate) fn resolve_spawn_request(
    requested: Option<String>,
    requester_session_id: &str,
    coordinator_provider_key: Option<&str>,
) -> Option<String> {
    if requested.as_deref().is_some_and(|m| !m.trim().is_empty()) {
        return requested;
    }
    let manifest = enforce::governing_run(requester_session_id)?;
    let model = current_plan(&manifest)?.worker.into_iter().next()?;
    Some(pinned_request(&model, coordinator_provider_key))
}

/// First served worker-chain model for display, without route pinning.
pub fn worker_chain_default_model(requester_session_id: &str) -> Option<String> {
    let manifest = enforce::governing_run(requester_session_id)?;
    current_plan(&manifest)?.worker.into_iter().next()
}

/// What a session-level failure hook decided.
pub enum SessionAdvance {
    /// Model switched on the same route: re-run the persisted turn.
    Switched,
    /// Stop retrying. `Some` replaces the turn error (chain exhausted).
    Stop(Option<String>),
}

/// Overnight-owned sessions only: advance the model chain after a failed turn.
/// Returns `None` for any other session, which must not switch automatically.
pub(crate) async fn try_advance_for_session(
    agent: &mut Agent,
    error: &str,
) -> Option<SessionAdvance> {
    let (manifest, role) = role_for_session(agent.session_id())?;
    let kind = classify_provider_failure(error);
    let mut chain = ModelChain::starting_at(chain_models(&manifest, role), &agent.provider_model());
    let session_id = agent.session_id().to_string();
    let outcome = advance_agent(agent, &mut chain, kind);
    let advance = settle_session_advance(&manifest, role, &session_id, kind, error, outcome);
    finish_coordinator_block(&manifest, role, &advance).await;
    if role == ChainRole::Coordinator
        && matches!(advance, SessionAdvance::Stop(Some(_)))
        && !enforce::is_supervised(&manifest)
    {
        block_visible_run(&manifest, &advance_reason(&advance));
    }
    Some(advance)
}

/// Same as [`try_advance_for_session`] for a session driven through a bare
/// provider (the local visible TUI run). `provider_key` pins the route.
pub fn try_advance_provider_for_session(
    session_id: &str,
    provider: &dyn Provider,
    provider_key: Option<&str>,
    route: &dyn Fn() -> Option<String>,
    error: &str,
) -> Option<SessionAdvance> {
    let (manifest, role) = role_for_session(session_id)?;
    let kind = classify_provider_failure(error);
    let from = provider.model();
    let mut chain = ModelChain::starting_at(chain_models(&manifest, role), &from);
    let mut skipped = Vec::new();
    let outcome = loop {
        match chain.advance(kind) {
            Advance::Hold => break AdvanceOutcome::Held,
            Advance::Exhausted(tried) => break AdvanceOutcome::Exhausted { tried, skipped },
            Advance::Next(model) => {
                match set_provider_model_on_same_route(provider, provider_key, route, &model) {
                    Ok(()) => {
                        break AdvanceOutcome::Switched {
                            from: from.clone(),
                            to: model,
                            skipped,
                        };
                    }
                    Err(error) if is_route_restore_failure(&error) => {
                        break AdvanceOutcome::Fatal(format!("{error:#}"));
                    }
                    Err(error) => skipped.push(format!("{model}: {error:#}")),
                }
            }
        }
    };
    let advance = settle_session_advance(&manifest, role, session_id, kind, error, outcome);
    // A visible run has no supervisor to drain: mark it blocked in place.
    if role == ChainRole::Coordinator
        && matches!(advance, SessionAdvance::Stop(Some(_)))
        && !enforce::is_supervised(&manifest)
    {
        block_visible_run(&manifest, &advance_reason(&advance));
    }
    Some(advance)
}

fn advance_reason(advance: &SessionAdvance) -> String {
    match advance {
        SessionAdvance::Stop(Some(reason)) => reason.clone(),
        _ => String::new(),
    }
}

fn settle_session_advance(
    manifest: &OvernightManifest,
    role: ChainRole,
    session_id: &str,
    kind: ProviderFailureKind,
    error: &str,
    outcome: AdvanceOutcome,
) -> SessionAdvance {
    match outcome {
        AdvanceOutcome::Switched { from, to, skipped } => {
            record_advance(manifest, role, session_id, &from, &to, kind, &skipped);
            SessionAdvance::Switched
        }
        AdvanceOutcome::Exhausted { tried, .. } => {
            let message = format!(
                "Overnight {} model chain exhausted after {kind:?}: tried {}. Last error: {error}",
                role.label(),
                tried.join(", ")
            );
            let _ = record_event(
                manifest,
                match role {
                    ChainRole::Coordinator => "model_chain_exhausted",
                    ChainRole::Worker => "worker_model_chain_exhausted",
                },
                message.clone(),
                json!({ "session_id": session_id, "role": role.label(), "tried": tried, "kind": format!("{kind:?}") }),
                true,
            );
            // A worker only fails itself: the run keeps going and the swarm
            // status path notifies the coordinator of the failed worker.
            SessionAdvance::Stop(Some(message))
        }
        AdvanceOutcome::Fatal(cause) => {
            let message = format!(
                "Overnight {} model chain stopped: {cause}. Last error: {error}",
                role.label()
            );
            let _ = record_event(
                manifest,
                "model_chain_route_restore_failed",
                message.clone(),
                json!({ "session_id": session_id, "role": role.label(), "kind": format!("{kind:?}") }),
                true,
            );
            SessionAdvance::Stop(Some(message))
        }
        // Only an auth failure ends a coordinator run here: nothing the chain
        // could switch to fixes it. Any other non-advancing failure (a
        // permanent error such as a context limit or an oversized request)
        // keeps its error and falls through to the session's own recovery.
        AdvanceOutcome::Held
            if role == ChainRole::Coordinator
                && matches!(
                    kind,
                    ProviderFailureKind::AuthExpired | ProviderFailureKind::AuthRevoked
                ) =>
        {
            SessionAdvance::Stop(Some(format!(
                "Overnight coordinator turn failed with {kind:?}, which does not advance the model chain: {error}"
            )))
        }
        AdvanceOutcome::Held => SessionAdvance::Stop(None),
    }
}

/// Supervised coordinator with an exhausted chain: end the run `blocked`
/// through `block_run`, which revokes admission and drains owned workers.
async fn finish_coordinator_block(
    manifest: &OvernightManifest,
    role: ChainRole,
    advance: &SessionAdvance,
) {
    if role == ChainRole::Coordinator
        && matches!(advance, SessionAdvance::Stop(Some(_)))
        && enforce::is_supervised(manifest)
    {
        let _ = block_run(manifest, &[advance_reason(advance)]).await;
    }
}

/// Whether `error` is a failure the chain moves on from (rate limit, quota,
/// transient or model unavailable). Callers that route context-limit or
/// oversized-request errors to recovery use it so a mixed message such as a
/// rate limit that mentions a context window still advances the chain.
pub fn error_advances_chain(error: &str) -> bool {
    kind_advances_chain(classify_provider_failure(error))
}

/// What re-arming a visible run after a TUI reload found.
pub enum VisibleRearm {
    /// No active visible run for this session (or another live process drives it).
    NotApplicable,
    /// The run is active again here: the caller re-arms auto-poke.
    Active(Box<OvernightManifest>),
    /// The run could not be continued and was closed (reason for the user).
    Ended(String),
}

/// A visible run has no supervisor to resume it: after a TUI reload or restart
/// nothing drives it. Re-establish it for `session_id`: close a run past its
/// grace window as `interrupted`, route-check the plan (`ensure_plan_with`,
/// blocking the run when the route serves no chain model), take ownership of
/// the manifest (the old pid may be gone, and the chain hook only governs runs
/// with a live owner) and reinstall the coordinator policy for this process.
/// A remote-attached TUI does not run the policy: the daemon finds the active
/// run from its manifest.
pub fn rearm_visible_run(
    session_id: &str,
    provider: impl FnOnce() -> Arc<dyn Provider>,
) -> VisibleRearm {
    let Some(manifest) = session_manifests_newest_first(session_id)
        .ok()
        .and_then(|manifests| manifests.into_iter().next())
    else {
        return VisibleRearm::NotApplicable;
    };
    if manifest.status.is_terminal() || enforce::is_supervised(&manifest) {
        return VisibleRearm::NotApplicable;
    }
    let me = std::process::id();
    let other_owner_alive = manifest.process_id != me
        && supervisor_is_alive(&manifest)
        && !matches!(
            manifest.supervisor_start_time,
            Some(start) if crate::platform::process_start_time(manifest.process_id)
                .is_some_and(|now| now != start)
        );
    if other_owner_alive {
        return VisibleRearm::NotApplicable;
    }
    if manifest.status == OvernightRunStatus::Running && Utc::now() > manifest.post_wake_grace_until
    {
        let reason = "the run is past its wake time and grace window".to_string();
        interrupt_visible_run(&manifest, &reason);
        return VisibleRearm::Ended(reason);
    }
    // A cancel in progress starts nothing new, so its plan is left alone. The
    // provider is only built when the plan has to be derived (older run).
    if manifest.status == OvernightRunStatus::Running
        && let Err(reasons) = ensure_plan_with(&manifest, || route_models(&provider()))
    {
        let reason = format!("Overnight blocked on resume: {}", reasons.join("; "));
        block_visible_run(&manifest, &reason);
        return VisibleRearm::Ended(reason);
    }
    let mut updated = load_manifest(&manifest.run_id).unwrap_or_else(|_| manifest.clone());
    if updated.status.is_terminal() {
        return VisibleRearm::NotApplicable;
    }
    updated.process_id = me;
    updated.supervisor_start_time = crate::platform::process_start_time(me);
    updated.heartbeat_at = Some(Utc::now());
    updated.last_activity_at = Utc::now();
    if save_manifest(&updated).is_err() {
        return VisibleRearm::Ended("could not save the run manifest".to_string());
    }
    let _ = record_event(
        &updated,
        "visible_run_rearmed",
        "Visible overnight run re-armed after a reload".to_string(),
        json!({ "previous_pid": manifest.process_id }),
        true,
    );
    crate::tool::coordinator_policy::install_for_overnight(session_id, &updated.run_id);
    VisibleRearm::Active(Box::new(updated))
}

fn interrupt_visible_run(manifest: &OvernightManifest, reason: &str) {
    let mut updated = load_manifest(&manifest.run_id).unwrap_or_else(|_| manifest.clone());
    if updated.status != OvernightRunStatus::Running {
        return;
    }
    updated.status = OvernightRunStatus::Interrupted;
    updated.completed_at = Some(Utc::now());
    updated.last_activity_at = Utc::now();
    let _ = save_manifest(&updated);
    let _ = record_event(
        &updated,
        "run_interrupted",
        format!("Visible overnight run not continued after reload: {reason}"),
        json!({ "reason": reason }),
        true,
    );
}

/// A visible run's session is the user's own and has no supervisor, so the run
/// is marked blocked without revoking admission.
fn block_visible_run(manifest: &OvernightManifest, reason: &str) {
    let mut updated = load_manifest(&manifest.run_id).unwrap_or_else(|_| manifest.clone());
    if updated.status.is_terminal() {
        return;
    }
    updated.status = OvernightRunStatus::Blocked;
    updated.completed_at = Some(Utc::now());
    updated.last_activity_at = Utc::now();
    let _ = save_manifest(&updated);
    let _ = record_event(
        &updated,
        "run_blocked",
        format!("Overnight run blocked (visible session, nothing drained): {reason}"),
        json!({ "reasons": [reason] }),
        true,
    );
}

fn record_advance(
    manifest: &OvernightManifest,
    role: ChainRole,
    session_id: &str,
    from: &str,
    to: &str,
    kind: ProviderFailureKind,
    skipped: &[String],
) {
    let _ = record_event(
        manifest,
        "model_chain_advanced",
        format!("{} model {from} -> {to} after {kind:?}", role.label()),
        json!({
            "session_id": session_id,
            "role": role.label(),
            "from": from,
            "to": to,
            "kind": format!("{kind:?}"),
            "skipped": skipped,
        }),
        true,
    );
}

pub(super) enum ChainTurn {
    Finished(String),
    Stopped(enforce::StopCause),
    /// The run was ended (blocked) here; the supervisor loop must stop.
    Ended,
}

/// Run one supervised coordinator turn, advancing `chain` on recoverable model
/// failures and retrying the same turn. Non-advancing failures and exhaustion
/// end the run `blocked` with a reason naming every model tried.
pub(super) async fn run_turn_with_chain(
    agent: &mut Agent,
    manifest: &OvernightManifest,
    prompt: &str,
    chain: &mut ModelChain,
) -> Result<ChainTurn> {
    let mut resume = false;
    loop {
        let error = match enforce::run_turn_monitored_with(agent, manifest, prompt, resume).await {
            Ok(enforce::TurnOutcome::Finished(output)) => return Ok(ChainTurn::Finished(output)),
            Ok(enforce::TurnOutcome::Stopped(cause)) => return Ok(ChainTurn::Stopped(cause)),
            Err(error) => error,
        };
        let message = crate::util::format_error_chain(&error);
        if let Some(cause) = enforce::pending_stop(&load_manifest(&manifest.run_id)?, None) {
            return Ok(ChainTurn::Stopped(cause));
        }
        let kind = classify_provider_failure(&message);
        let session_id = manifest.coordinator_session_id.clone();
        match advance_agent(agent, chain, kind) {
            AdvanceOutcome::Switched { from, to, skipped } => {
                record_advance(
                    manifest,
                    ChainRole::Coordinator,
                    &session_id,
                    &from,
                    &to,
                    kind,
                    &skipped,
                );
                // Retry the persisted request instead of appending the prompt again.
                resume = agent.last_message_role() == Some(Role::User);
            }
            AdvanceOutcome::Exhausted { tried, .. } => {
                let reason = format!(
                    "Coordinator model chain exhausted after {kind:?}: tried {}. Last error: {message}",
                    tried.join(", ")
                );
                let _ = record_event(
                    manifest,
                    "model_chain_exhausted",
                    reason.clone(),
                    json!({ "tried": tried, "kind": format!("{kind:?}") }),
                    true,
                );
                block_run(manifest, &[reason]).await?;
                return Ok(ChainTurn::Ended);
            }
            AdvanceOutcome::Fatal(cause) => {
                let reason =
                    format!("Coordinator model chain stopped: {cause}. Last error: {message}");
                let _ = record_event(
                    manifest,
                    "model_chain_route_restore_failed",
                    reason.clone(),
                    json!({ "session_id": session_id, "role": "coordinator", "kind": format!("{kind:?}") }),
                    true,
                );
                block_run(manifest, &[reason]).await?;
                return Ok(ChainTurn::Ended);
            }
            AdvanceOutcome::Held => {
                let reason = format!(
                    "Coordinator turn failed with {kind:?}, which does not advance the model chain (models tried: {}): {message}",
                    chain.tried().join(", ")
                );
                block_run(manifest, &[reason]).await?;
                return Ok(ChainTurn::Ended);
            }
        }
    }
}

#[cfg(test)]
#[path = "model_chain_tests.rs"]
mod tests;

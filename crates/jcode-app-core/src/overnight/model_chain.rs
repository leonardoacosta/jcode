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

/// Chains after the route filter, persisted next to the run's manifest.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct ChainPlan {
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
}

const CATALOG_REFRESH_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(15);
const CATALOG_UNAVAILABLE: &str = "route catalog unavailable; chains unfiltered";

/// The route's served models for preflight filtering, plus why the list is
/// empty when the catalog could not be loaded.
pub(crate) struct RouteModels {
    pub served: Vec<String>,
    pub unfiltered_reason: Option<String>,
}

/// Served models for `provider`'s route. A catalog-backed route whose catalog
/// is not loaded gets one bounded refresh; if that does not produce a real
/// list, `served` is empty (unknown) so `filter_served` keeps every chain
/// model instead of shrinking the chains to the current model.
pub(crate) fn route_models(provider: &dyn Provider) -> RouteModels {
    route_models_with(provider, refresh_catalog_bounded)
}

fn route_models_with(
    provider: &dyn Provider,
    refresh: impl FnOnce(&dyn Provider) -> std::result::Result<(), String>,
) -> RouteModels {
    let mut failure = None;
    if !provider.model_list_is_authoritative() {
        failure = refresh(provider).err();
    }
    if provider.model_list_is_authoritative() {
        return RouteModels {
            served: provider.available_models_for_switching(),
            unfiltered_reason: None,
        };
    }
    RouteModels {
        served: Vec::new(),
        unfiltered_reason: Some(match failure {
            Some(error) => format!("{CATALOG_UNAVAILABLE} ({error})"),
            None => CATALOG_UNAVAILABLE.to_string(),
        }),
    }
}

/// Run `refresh_model_catalog` on a scratch runtime in a scoped thread, so the
/// synchronous start path works from any caller (TUI thread, current-thread
/// or multi-thread tokio) and the wait is bounded.
fn refresh_catalog_bounded(provider: &dyn Provider) -> std::result::Result<(), String> {
    std::thread::scope(|scope| {
        scope
            .spawn(|| {
                let runtime = tokio::runtime::Builder::new_current_thread()
                    .enable_all()
                    .build()
                    .map_err(|error| error.to_string())?;
                runtime.block_on(async {
                    match tokio::time::timeout(
                        CATALOG_REFRESH_TIMEOUT,
                        provider.refresh_model_catalog(),
                    )
                    .await
                    {
                        Ok(Ok(_)) => Ok(()),
                        Ok(Err(error)) => Err(format!("{error:#}")),
                        Err(_) => Err(format!(
                            "timed out after {}s",
                            CATALOG_REFRESH_TIMEOUT.as_secs()
                        )),
                    }
                })
            })
            .join()
            .unwrap_or_else(|_| Err("catalog refresh panicked".to_string()))
    })
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
        })
    } else {
        Err(reasons)
    }
}

pub(crate) fn persist_plan(run_dir: &Path, plan: &ChainPlan) -> Result<()> {
    storage::write_json(&run_dir.join(PLAN_FILE), plan)
}

fn read_plan(run_dir: &Path) -> Option<ChainPlan> {
    storage::read_json(&run_dir.join(PLAN_FILE)).ok()
}

/// Record the plan as a run event (dropped models included).
pub(crate) fn record_plan(manifest: &OvernightManifest, plan: &ChainPlan) {
    let dropped = plan.dropped_coordinator.len() + plan.dropped_worker.len();
    let _ = record_event(
        manifest,
        "model_chain_preflight",
        format!(
            "Model chains: coordinator [{}], worker [{}]{}",
            plan.coordinator.join(" > "),
            plan.worker.join(" > "),
            if let Some(reason) = plan.unfiltered_reason.as_deref() {
                format!("; {reason}")
            } else if dropped == 0 {
                String::new()
            } else {
                format!(
                    "; dropped (not served by route): coordinator [{}], worker [{}]",
                    plan.dropped_coordinator.join(", "),
                    plan.dropped_worker.join(", ")
                )
            }
        ),
        serde_json::to_value(plan).unwrap_or_else(|_| json!({})),
        true,
    );
}

/// Route-pinned model request: the session's provider key becomes a model
/// prefix so a bare chain name cannot resolve to another provider.
pub(crate) fn pinned_request(model: &str, provider_key: Option<&str>) -> String {
    crate::provider::MultiProvider::model_switch_request_for_session_model(model, provider_key)
}

/// Select `model` on `provider` without leaving its route or account.
/// Used for the visible run's first model; agents use
/// `Agent::set_model_on_same_route`.
pub(crate) fn set_provider_model_on_same_route(
    provider: &dyn Provider,
    provider_key: Option<&str>,
    model: &str,
) -> Result<()> {
    let name = provider.name().to_string();
    let account = provider.oauth_account_label();
    let previous = provider.model();
    crate::provider::set_model_with_auth_refresh(provider, &pinned_request(model, provider_key))?;
    if provider.name() != name || provider.oauth_account_label() != account {
        let _ = crate::provider::set_model_with_auth_refresh(
            provider,
            &pinned_request(&previous, provider_key),
        );
        anyhow::bail!("{model} is not served on the selected route ({name}); refusing to switch");
    }
    Ok(())
}

/// Choose the first chain model the route accepts. Returns the selected model
/// and the candidates skipped with their reason.
pub(crate) fn select_first(
    models: &[String],
    mut apply: impl FnMut(&str) -> Result<()>,
) -> (Option<String>, Vec<String>) {
    let mut chain = ModelChain::new(models.to_vec());
    let mut skipped = Vec::new();
    while let Some(model) = chain.take_next() {
        match apply(&model) {
            Ok(()) => return (Some(model), skipped),
            Err(error) => skipped.push(format!("{model}: {error:#}")),
        }
    }
    (None, skipped)
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

/// Coordinator chain models for a run (route-filtered plan, else config).
pub(super) fn coordinator_models(manifest: &OvernightManifest) -> Vec<String> {
    chain_models(manifest, ChainRole::Coordinator)
}

fn chain_models(manifest: &OvernightManifest, role: ChainRole) -> Vec<String> {
    match (read_plan(&manifest.run_dir), role) {
        (Some(plan), ChainRole::Coordinator) => plan.coordinator,
        (Some(plan), ChainRole::Worker) => plan.worker,
        (None, ChainRole::Coordinator) => {
            dedupe(&crate::config::config().overnight.coordinator_model_chain)
        }
        // No persisted plan: the run did not go through start-time route
        // filtering, so no worker model is forced on its spawns.
        (None, ChainRole::Worker) => Vec::new(),
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
    let model = chain_models(&manifest, ChainRole::Worker)
        .into_iter()
        .next()?;
    Some(pinned_request(&model, coordinator_provider_key))
}

/// First served worker-chain model for display, without route pinning.
pub fn worker_chain_default_model(requester_session_id: &str) -> Option<String> {
    let manifest = enforce::governing_run(requester_session_id)?;
    chain_models(&manifest, ChainRole::Worker)
        .into_iter()
        .next()
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
                match set_provider_model_on_same_route(provider, provider_key, &model) {
                    Ok(()) => {
                        break AdvanceOutcome::Switched {
                            from: from.clone(),
                            to: model,
                            skipped,
                        };
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

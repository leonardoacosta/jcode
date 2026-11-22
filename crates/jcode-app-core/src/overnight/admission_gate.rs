//! Evidence gate for coordinator turns and the selected-route auth preflight.
//! Reuses the deterministic task policy (`revalidate_task_admission`) and the
//! existing cancel/limit enforcement; a signal or credential never authorizes
//! dispatch by itself.

use super::*;
use crate::auth::AuthState;
use chrono::DateTime;
use jcode_overnight_core::task_status_bucket as bucket;
use jcode_overnight_core::{
    AdmissionEvent, AdmissionInput, AdmissionVerdict, TaskGate, decide_admission,
};
use tokio::sync::broadcast::{Receiver, error::TryRecvError};

/// The login provider for the explicitly selected route. Precedence: the
/// session's recorded API method (distinguishes API-key from OAuth), then its
/// provider key, then the provider's own name. Never consults other accounts.
/// A provider key that is not a login provider names a custom `[providers.<key>]`
/// profile. Those all run on the openrouter runtime, so the provider name says
/// nothing about their auth and is never consulted; they resolve to `None`.
fn resolve_route(
    provider_name: &str,
    provider_key: Option<&str>,
    route_api_method: Option<&str>,
) -> Option<crate::provider_catalog::LoginProviderDescriptor> {
    if let Some(key) = provider_key
        && crate::provider_catalog::resolve_login_provider(key).is_none()
    {
        return None;
    }
    let method_id =
        route_api_method.and_then(
            |method| match method.split('@').next().unwrap_or_default() {
                "openai-api-key" => Some("openai-api"),
                "claude-api" => Some("anthropic-api"),
                "openai-oauth" | "chatgpt-web" => Some("openai"),
                "claude-oauth" => Some("claude"),
                "copilot" => Some("copilot"),
                "cursor" => Some("cursor"),
                "openrouter" => Some("openrouter"),
                _ => None,
            },
        );
    [method_id, provider_key, Some(provider_name)]
        .into_iter()
        .flatten()
        .find_map(crate::provider_catalog::resolve_login_provider)
}

/// Why the selected route cannot be used, or `None` when its auth is usable.
pub(super) fn classify_route_auth(id: &str, state: AuthState) -> Option<Vec<String>> {
    match state {
        AuthState::Available => None,
        AuthState::Expired => Some(vec![format!(
            "{id}: credentials expired. Run `jcode login --provider {id}`, then restart the run."
        )]),
        AuthState::NotConfigured => Some(vec![format!(
            "{id}: no credentials configured. Run `jcode login --provider {id}`, then restart the run."
        )]),
    }
}

/// Auth verdict for a custom `[providers.<name>]` profile. `None` when the
/// profile is unknown to config; `Some(None)` when usable; `Some(Some(reasons))`
/// when its key is required but missing.
///
/// The runtime applies a profile by mutating process-wide environment
/// variables, so its credential code cannot be called from here. This repeats
/// the rules of the two runtimes that serve a named profile:
///
/// - OpenAI-compatible and OpenRouter profiles are served by
///   `OpenRouterProvider::new_named_openai_compatible`
///   (`jcode-provider-openrouter-runtime/src/lib.rs`, the `auth` resolver in
///   that constructor, run once eagerly by `auth()?`). `auth = "none"` needs no
///   key. `Bearer` and `Header` need one: `api_key_env` is looked up in the
///   environment (and in `env_file` when set, `load_named_profile_api_key`),
///   then the inline `api_key` is used. `requires_api_key` is not read, there
///   is no localhost exception, and no default `OPENROUTER_API_KEY` or
///   `openrouter.env` is consulted. Without either key source the runtime
///   fails with "inline api_key not found".
/// - Anthropic-compatible profiles go through
///   `provider_catalog::apply_named_provider_profile_env_from_config` and
///   `provider::anthropic::load_anthropic_api_key`. `auth = "none"` needs no
///   key. An `api_key_env` wins over the inline key (the inline key is only
///   exported when none is named) and is read from the environment, then
///   `env_file`. Without one, `ANTHROPIC_AUTH_TOKEN` or `ANTHROPIC_API_KEY`
///   (`anthropic.env`) apply.
fn custom_profile_auth_block(name: &str) -> Option<Option<Vec<String>>> {
    use crate::config::{NamedProviderAuth, NamedProviderType};
    let config = crate::config::Config::load();
    let profile = config.providers.get(name)?;
    if profile.auth == NamedProviderAuth::None {
        return Some(None);
    }
    let non_empty = |value: &Option<String>| {
        value
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(ToString::to_string)
    };
    let key_env = non_empty(&profile.api_key_env);
    let inline = non_empty(&profile.api_key);
    let env_file = non_empty(&profile.env_file);
    let non_blank = |value: String| !value.trim().is_empty();
    let anthropic = matches!(
        profile.provider_type,
        NamedProviderType::AnthropicCompatible
    );
    // The Anthropic runtime refuses to start on an unsafe variable or file name
    // (`apply_named_provider_profile_env_from_config`), so a key can never be
    // loaded and admitting the route would only fail later. The OpenAI path has
    // no such check: with an `env_file` the shared loader already ignores bad
    // names, and without one a bare variable lookup is what the runtime does.
    if anthropic {
        if let Some(env) = key_env
            .as_deref()
            .filter(|env| !crate::provider_catalog::is_safe_env_key_name(env))
        {
            return Some(Some(vec![format!(
                "{name}: invalid api_key_env '{env}' (use A-Z, 0-9 and _), then restart the run."
            )]));
        }
        if let Some(file) = env_file
            .as_deref()
            .filter(|file| !crate::provider_catalog::is_safe_env_file_name(file))
        {
            return Some(Some(vec![format!(
                "{name}: invalid env_file '{file}' (a plain file name), then restart the run."
            )]));
        }
    }

    let (usable, key_label, file_label) = if anthropic {
        let usable = match key_env.as_deref() {
            Some(env) => {
                std::env::var(env).ok().is_some_and(non_blank)
                    || env_file.as_deref().is_some_and(|file| {
                        crate::provider_catalog::load_env_value_from_config_file(env, file)
                            .is_some_and(non_blank)
                    })
            }
            None => {
                inline.is_some()
                    || std::env::var("ANTHROPIC_AUTH_TOKEN")
                        .ok()
                        .is_some_and(non_blank)
                    || crate::provider_catalog::load_api_key_from_env_or_config(
                        "ANTHROPIC_API_KEY",
                        "anthropic.env",
                    )
                    .is_some()
            }
        };
        (
            usable,
            key_env
                .clone()
                .unwrap_or_else(|| "ANTHROPIC_API_KEY".to_string()),
            env_file
                .clone()
                .unwrap_or_else(|| "anthropic.env".to_string()),
        )
    } else {
        let from_env = key_env
            .as_deref()
            .and_then(|env| match env_file.as_deref() {
                Some(file) => crate::provider_catalog::load_api_key_from_env_or_config(env, file),
                None => std::env::var(env).ok().filter(|v| !v.trim().is_empty()),
            });
        (
            from_env.is_some() || inline.is_some(),
            key_env.clone().unwrap_or_else(|| "api_key".to_string()),
            env_file.clone().unwrap_or_default(),
        )
    };
    if usable {
        return Some(None);
    }
    let location = if file_label.is_empty() {
        key_label.clone()
    } else {
        format!("{key_label} (or {file_label})")
    };
    Some(Some(vec![format!(
        "{name}: API key not configured. Set {location}, then restart the run."
    )]))
}

/// Blocking reasons for the selected route. An unresolvable route is blocked as
/// `unclassified` rather than assumed usable, except a custom named profile,
/// which is judged by its own configured key.
pub(super) fn route_auth_block(
    provider_name: &str,
    provider_key: Option<&str>,
    route_api_method: Option<&str>,
) -> Option<Vec<String>> {
    if let Some(key) = provider_key
        && crate::provider_catalog::resolve_login_provider(key).is_none()
        && let Some(verdict) = custom_profile_auth_block(key)
    {
        return verdict;
    }
    let Some(descriptor) = resolve_route(provider_name, provider_key, route_api_method) else {
        return Some(vec![format!(
            "unclassified: selected route `{}` cannot be matched to a login provider, so its auth cannot be verified",
            provider_key.unwrap_or(provider_name)
        )]);
    };
    let state = crate::auth::AuthStatus::check_fast().state_for_provider(descriptor);
    classify_route_auth(descriptor.id, state)
}

/// Like [`route_auth_block`] but only for a route that resolves to a login
/// provider. A visible current-session run has no unattended supervisor, so an
/// unclassified route (custom profile) is not refused at start; supervised runs
/// still block it in their loop.
pub(super) fn resolved_route_auth_block(
    provider_name: &str,
    provider_key: Option<&str>,
    route_api_method: Option<&str>,
) -> Option<Vec<String>> {
    if let Some(key) = provider_key
        && resolve_route(provider_name, provider_key, route_api_method).is_none()
        && custom_profile_auth_block(key).is_some()
    {
        return route_auth_block(provider_name, provider_key, route_api_method);
    }
    resolve_route(provider_name, provider_key, route_api_method)?;
    route_auth_block(provider_name, provider_key, route_api_method)
}

/// Record the blocked admission decision, then end the run via [`block_run`].
pub(super) async fn block_on_reasons(
    manifest: &OvernightManifest,
    reasons: &[String],
) -> Result<()> {
    record_decision(
        manifest,
        &AdmissionEvent {
            verdict: AdmissionVerdict::Blocked,
            reason: reasons.join("; "),
            cited: None,
        },
    )?;
    block_run(manifest, reasons).await
}

/// End the run as `blocked` without a coordinator turn. Cancellation and limit
/// stops win, and owned work is revoked and drained like any other stop: a run
/// is not terminal while a worker may still be running.
pub(super) async fn block_run(manifest: &OvernightManifest, reasons: &[String]) -> Result<()> {
    let current = load_manifest(&manifest.run_id).unwrap_or_else(|_| manifest.clone());
    if let Some(cause) = enforce::pending_stop(&current, None) {
        return enforce::stop_until_settled(&current, cause).await;
    }
    let mut settlement = enforce::settle_owned_work(&current).await;
    for _ in 0..enforce::RESETTLE_ATTEMPTS {
        if settlement.is_settled() {
            break;
        }
        tokio::time::sleep(enforce::RESETTLE_INTERVAL).await;
        let current = load_manifest(&manifest.run_id).unwrap_or_else(|_| manifest.clone());
        if let Some(cause) = enforce::pending_stop(&current, None) {
            return enforce::stop_until_settled(&current, cause).await;
        }
        settlement = enforce::settle_owned_work(&current).await;
    }
    let settled = settlement.is_settled();
    let mut updated = load_manifest(&manifest.run_id).unwrap_or_else(|_| manifest.clone());
    updated.stop_settlement = Some(settlement.clone());
    updated.last_activity_at = Utc::now();
    if settled {
        updated.status = OvernightRunStatus::Blocked;
        updated.completed_at = Some(Utc::now());
    }
    save_manifest(&updated)?;
    let reason = reasons.join("; ");
    record_event(
        &updated,
        if settled {
            "run_blocked"
        } else {
            "run_blocked_unresolved"
        },
        if settled {
            format!("Overnight run blocked without a coordinator turn: {reason}")
        } else {
            format!("Overnight run blocked ({reason}) but owned work is not confirmed stopped")
        },
        json!({ "reasons": reasons, "settlement": settlement }),
        true,
    )?;
    if settled {
        usage::finalize(&updated).await;
    }
    render_review_html(&updated)?;
    Ok(())
}

pub(super) fn record_decision(
    manifest: &OvernightManifest,
    decision: &AdmissionEvent,
) -> Result<()> {
    record_event(
        manifest,
        "admission_decision",
        match &decision.cited {
            Some(cited) => format!(
                "{}: {} ({cited})",
                decision.verdict.label(),
                decision.reason
            ),
            None => format!("{}: {}", decision.verdict.label(), decision.reason),
        },
        serde_json::to_value(decision)?,
        true,
    )
}

/// The admission decision for one pass of the supervisor loop. Returns the
/// recorded decision and the active-card digest it was made against.
pub(super) fn decide_and_record(
    current: &OvernightManifest,
    first_turn: bool,
    boundary: Option<&'static str>,
    signal: Option<&str>,
    seen_cards: Option<&str>,
    last_waiting: bool,
) -> Result<(AdmissionEvent, String)> {
    let gate = task_gate(current)?;
    let (active, digest) = active_cards(current);
    let decision = decide_admission(
        &gate,
        &AdmissionInput {
            first_turn,
            boundary,
            signal,
            eligible_tasks: eligible_since(seen_cards, active, &digest),
        },
    );
    // A repeated `waiting` after a rescan with no new evidence adds nothing.
    if !(last_waiting && decision.verdict == AdmissionVerdict::Waiting) {
        record_decision(current, &decision)?;
    }
    Ok((decision, digest))
}

/// Deterministic task admission for this run, from original persisted bounds
/// and the current strict policy.
pub(super) fn task_gate(manifest: &OvernightManifest) -> Result<TaskGate> {
    if manifest.task_admission.is_none() {
        return Ok(TaskGate::NotApplicable);
    }
    let policy = crate::config::Config::load_strict()?.tasks;
    let decision = revalidate_task_admission(manifest, &policy).expect("snapshot checked");
    let reason = decision.reasons.join("; ");
    Ok(match decision.state {
        crate::config::AdmissionState::Ready => TaskGate::Ready,
        crate::config::AdmissionState::Waiting => TaskGate::Waiting(reason),
        crate::config::AdmissionState::Blocked | crate::config::AdmissionState::NeedsHuman => {
            TaskGate::Blocked(reason)
        }
    })
}

/// Cards the coordinator still marks active, with a digest of their ids and
/// statuses. A card unchanged since the last turn is not new evidence.
pub(super) fn active_cards(manifest: &OvernightManifest) -> (usize, String) {
    let mut active: Vec<String> = read_task_cards(manifest)
        .unwrap_or_default()
        .iter()
        .filter(|card| bucket(&card.status) == "active")
        .map(|card| format!("{}:{}:{}", card.id, card.title, card.status))
        .collect();
    active.sort();
    (active.len(), active.join("\n"))
}

/// Eligible-task evidence: active cards whose set changed since the last turn.
pub(super) fn eligible_since(seen: Option<&str>, active: usize, digest: &str) -> usize {
    if seen == Some(digest) { 0 } else { active }
}

#[derive(Default)]
pub(super) struct StepFlags {
    pub handoff_notice_sent: bool,
    pub morning_report_prompt_sent: bool,
    pub final_wrapup_prompt_sent: bool,
}

pub(super) enum Step {
    Done,
    /// Next prompt, and the boundary it answers (boundaries admit a turn).
    Prompt(String, Option<&'static str>),
}

/// What follows a turn or an elapsed wait: wake-boundary prompts, otherwise
/// the plain continuation, which the admission gate may still defer.
pub(super) async fn next_step(after: &OvernightManifest, flags: &mut StepFlags) -> Result<Step> {
    let now = Utc::now();
    if now < after.target_wake_at {
        return Ok(Step::Prompt(build_continuation_prompt(after), None));
    }
    if !flags.morning_report_prompt_sent && after.morning_report_posted_at.is_none() {
        let mut updated = after.clone();
        updated.morning_report_posted_at = Some(now);
        save_manifest(&updated)?;
        record_event(
            &updated,
            "morning_report_requested",
            "Target wake time reached; requesting morning report".to_string(),
            json!({ "target_wake_at": updated.target_wake_at }),
            true,
        )?;
        flags.morning_report_prompt_sent = true;
        return Ok(Step::Prompt(
            build_morning_report_prompt(&updated),
            Some("target_wake"),
        ));
    }
    if now < after.post_wake_grace_until {
        record_event(
            after,
            "post_wake_continuation",
            "Morning report is posted; allowing bounded post-wake continuation".to_string(),
            json!({ "post_wake_grace_until": after.post_wake_grace_until }),
            true,
        )?;
        return Ok(Step::Prompt(
            build_post_wake_continuation_prompt(after),
            None,
        ));
    }
    if !flags.final_wrapup_prompt_sent {
        record_event(
            after,
            "post_wake_grace_expired",
            "Post-wake grace window expired; requesting final wrap-up".to_string(),
            json!({ "post_wake_grace_until": after.post_wake_grace_until }),
            true,
        )?;
        flags.final_wrapup_prompt_sent = true;
        return Ok(Step::Prompt(
            build_final_wrapup_prompt(after),
            Some("post_wake_grace_expired"),
        ));
    }
    mark_completed(
        after,
        OvernightRunStatus::Completed,
        "Morning report turn completed",
    )
    .await?;
    Ok(Step::Done)
}

/// The next time the run must act without any signal.
pub(super) fn next_boundary(
    manifest: &OvernightManifest,
    flags: &StepFlags,
    now: DateTime<Utc>,
) -> DateTime<Utc> {
    if now < manifest.target_wake_at {
        if !flags.handoff_notice_sent && manifest.handoff_ready_at > now {
            manifest.handoff_ready_at
        } else {
            manifest.target_wake_at
        }
    } else {
        manifest.post_wake_grace_until
    }
}

pub(super) enum Wait {
    Signal(String),
    /// The bus dropped events: durable state must be re-scanned, and nothing
    /// is admitted unless that scan finds evidence.
    Rescan,
    Boundary,
    Stop,
}

/// Subscribes to worker/background completion events for the coordinator
/// session. Subscribed before the first turn so nothing that arrives during a
/// turn is lost.
pub(super) struct SignalWatch {
    session: String,
    rx: Receiver<crate::bus::BusEvent>,
}

impl SignalWatch {
    pub(super) fn new(session: &str) -> Self {
        Self {
            session: session.to_string(),
            rx: crate::bus::Bus::global().subscribe(),
        }
    }

    fn signal_for(&self, event: &crate::bus::BusEvent) -> Option<String> {
        use crate::bus::BusEvent::*;
        match event {
            BackgroundTaskCompleted(e) if e.session_id == self.session => {
                Some(format!("background:{}", e.task_id))
            }
            SwarmWorkerReported(e) if e.session_id == self.session => {
                Some(format!("worker:{}", e.worker_session_id))
            }
            SwarmAwaitCompleted(e) if e.session_id == self.session => {
                Some("swarm_await".to_string())
            }
            _ => None,
        }
    }

    /// First pending signal, without blocking. Drains everything queued so one
    /// burst admits one turn.
    pub(super) fn drain(&mut self, queued_user_messages: usize) -> Option<String> {
        let mut found = (queued_user_messages > 0).then(|| "user_message".to_string());
        loop {
            match self.rx.try_recv() {
                Ok(event) => found = found.or_else(|| self.signal_for(&event)),
                // Lagged: events were dropped. Durable state is re-scanned by the
                // caller's admission pass; the loss itself is not evidence.
                Err(TryRecvError::Lagged(_)) => {}
                Err(_) => return found,
            }
        }
    }

    /// Sleep until a signal, the boundary, or a pending stop. No turn runs and
    /// no manifest state is re-read for content while idle.
    pub(super) async fn wait(
        &mut self,
        agent: &Agent,
        manifest: &OvernightManifest,
        deadline: DateTime<Utc>,
    ) -> Wait {
        let run_id = manifest.run_id.as_str();
        let mut unreadable = 0u32;
        let until = (deadline - Utc::now()).to_std().unwrap_or_default();
        let sleep = tokio::time::sleep(until);
        tokio::pin!(sleep);
        let mut poll = tokio::time::interval(enforce::STOP_POLL_INTERVAL);
        loop {
            tokio::select! {
                _ = &mut sleep => return Wait::Boundary,
                _ = poll.tick() => {
                    if agent.soft_interrupt_count() > 0 {
                        return Wait::Signal("user_message".into());
                    }
                    match load_manifest(run_id) {
                        Ok(current) => {
                            unreadable = 0;
                            if enforce::pending_stop(&current, None).is_some() {
                                return Wait::Stop;
                            }
                        }
                        // Tolerate transient read errors like `run_turn_monitored`.
                        Err(error) => {
                            unreadable += 1;
                            if unreadable == enforce::UNREADABLE_POLLS_BEFORE_RECORD {
                                let _ = record_event(
                                    manifest,
                                    "manifest_unreadable",
                                    format!("Run manifest unreadable for {unreadable} polls while waiting; cancellation and limits cannot be observed: {error:#}"),
                                    json!({ "polls": unreadable }),
                                    true,
                                );
                            }
                        }
                    }
                }
                event = self.rx.recv() => match event {
                    Ok(event) => {
                        if let Some(signal) = self.signal_for(&event) {
                            return Wait::Signal(signal);
                        }
                    }
                    Err(tokio::sync::broadcast::error::RecvError::Lagged(_)) => return Wait::Rescan,
                    Err(_) => {
                        // Bus closed: fall back to the boundary and stop polls.
                        (&mut sleep).await;
                        return Wait::Boundary;
                    }
                },
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::message::{Message, StreamEvent, ToolDefinition};
    use crate::server::session_control;
    use async_trait::async_trait;
    use std::sync::atomic::{AtomicUsize, Ordering};

    struct CountingProvider {
        name: &'static str,
        calls: Arc<AtomicUsize>,
    }

    #[async_trait]
    impl Provider for CountingProvider {
        async fn complete(
            &self,
            _: &[Message],
            _: &[ToolDefinition],
            _: &str,
            _: Option<&str>,
        ) -> Result<crate::provider::EventStream> {
            self.calls.fetch_add(1, Ordering::SeqCst);
            Ok(Box::pin(futures::stream::iter(vec![
                Ok(StreamEvent::TextDelta("no eligible work remains".into())),
                Ok(StreamEvent::MessageEnd {
                    stop_reason: Some("end_turn".into()),
                }),
            ])))
        }
        fn name(&self) -> &str {
            self.name
        }
        fn model(&self) -> String {
            "counting-model".into()
        }
        fn fork(&self) -> Arc<dyn Provider> {
            Arc::new(Self {
                name: self.name,
                calls: self.calls.clone(),
            })
        }
    }

    /// Sandbox whose selected route (`openrouter`) has a usable API key.
    fn sandbox() -> crate::auth::test_sandbox::AuthTestSandbox {
        let sandbox = crate::auth::test_sandbox::AuthTestSandbox::new().unwrap();
        crate::env::set_var("OPENROUTER_API_KEY", "test-key");
        sandbox
    }

    const ROUTE: &str = "openrouter";

    fn kinds(manifest: &OvernightManifest) -> Vec<String> {
        read_events(manifest)
            .unwrap()
            .into_iter()
            .map(|event| event.kind)
            .collect()
    }

    fn turns(run_id: &str) -> usize {
        kinds(&load_manifest(run_id).unwrap())
            .iter()
            .filter(|kind| *kind == "coordinator_turn_started")
            .count()
    }

    fn decisions(manifest: &OvernightManifest, verdict: &str) -> Vec<String> {
        read_events(manifest)
            .unwrap()
            .into_iter()
            .filter(|e| e.kind == "admission_decision" && e.summary.starts_with(verdict))
            .map(|e| e.summary)
            .collect()
    }

    async fn until(what: &str, mut ready: impl FnMut() -> bool) {
        for _ in 0..400 {
            if ready() {
                return;
            }
            tokio::time::sleep(Duration::from_millis(25)).await;
        }
        panic!("timed out waiting for {what}");
    }

    struct Harness {
        manifest: OvernightManifest,
        calls: Arc<AtomicUsize>,
        task: tokio::task::JoinHandle<Result<()>>,
    }

    fn new_manifest(run_id: &str) -> (OvernightManifest, Session) {
        let child = Session::create(None, Some("gate".into()));
        let manifest =
            crate::overnight::test_run_manifest(run_id, &child.id, OvernightLimits::default());
        session_control::store().register(&child.id, None).unwrap();
        (manifest, child)
    }

    async fn launch(
        manifest: OvernightManifest,
        child: Session,
        provider_name: &'static str,
    ) -> Harness {
        let calls = Arc::new(AtomicUsize::new(0));
        let provider: Arc<dyn Provider> = Arc::new(CountingProvider {
            name: provider_name,
            calls: calls.clone(),
        });
        let registry = Registry::new(provider.clone()).await;
        let task = tokio::spawn(run_supervisor(
            manifest.clone(),
            child,
            provider,
            registry,
            false,
        ));
        Harness {
            manifest,
            calls,
            task,
        }
    }

    async fn start(run_id: &str, provider_name: &'static str) -> Harness {
        let (manifest, child) = new_manifest(run_id);
        launch(manifest, child, provider_name).await
    }

    fn write_card(manifest: &OvernightManifest, id: &str, status: &str) {
        std::fs::create_dir_all(&manifest.task_cards_dir).unwrap();
        std::fs::write(
            manifest.task_cards_dir.join(format!("{id}.json")),
            format!(r#"{{"id":"{id}","title":"task {id}","status":"{status}"}}"#),
        )
        .unwrap();
    }

    fn complete_background(session: &str, id: &str) {
        crate::bus::Bus::global().publish(crate::bus::BusEvent::BackgroundTaskCompleted(
            crate::bus::BackgroundTaskCompleted {
                task_id: id.into(),
                tool_name: "bash".into(),
                display_name: None,
                session_id: session.into(),
                status: crate::bus::BackgroundTaskStatus::Completed,
                exit_code: Some(0),
                output_preview: String::new(),
                output_file: PathBuf::new(),
                duration_secs: 0.1,
                notify: true,
                wake: true,
            },
        ));
    }

    /// Overflow the bus (capacity 256) so a parked receiver lags.
    fn overflow_bus() {
        for _ in 0..600 {
            crate::bus::Bus::global().publish(crate::bus::BusEvent::ClientBinaryChanged);
        }
    }

    fn cancel(run_id: &str) {
        let mut manifest = load_manifest(run_id).unwrap();
        manifest.status = OvernightRunStatus::CancelRequested;
        manifest.cancel_requested_at = Some(Utc::now());
        save_manifest(&manifest).unwrap();
    }

    async fn finish(harness: Harness) {
        tokio::time::timeout(Duration::from_secs(10), harness.task)
            .await
            .expect("waiting run stops")
            .unwrap()
            .unwrap();
    }

    #[test]
    fn route_resolution_prefers_the_recorded_api_method() {
        let id = |name, key, method| resolve_route(name, key, method).map(|d| d.id);
        assert_eq!(
            id("OpenAI", None, Some("openai-api-key")),
            Some("openai-api")
        );
        assert_eq!(
            id("OpenAI", Some("openai"), Some("openai-oauth@work")),
            Some("openai")
        );
        assert_eq!(id("Claude", Some("claude"), None), Some("claude"));
        assert_eq!(
            id("anthropic", None, Some("claude-api")),
            Some("anthropic-api")
        );
        assert_eq!(id("synthetic-route", None, None), None);
    }

    #[test]
    fn route_auth_classifies_states_with_remedies() {
        assert!(classify_route_auth("openai-api", AuthState::Available).is_none());
        let expired = classify_route_auth("openai", AuthState::Expired).unwrap();
        assert!(
            expired[0].contains("jcode login --provider openai"),
            "{expired:?}"
        );
        let missing = classify_route_auth("anthropic-api", AuthState::NotConfigured).unwrap();
        assert!(missing[0].contains("anthropic-api"), "{missing:?}");
    }

    #[test]
    fn selected_route_blocks_on_missing_auth_and_unknown_routes_are_unclassified() {
        let _env = sandbox();
        let reasons = route_auth_block("OpenAI", None, Some("openai-api-key")).expect("no key");
        assert_eq!(reasons.len(), 1, "one route, one reason: {reasons:?}");
        assert!(reasons[0].contains("openai-api"), "{reasons:?}");
        // The selected, authenticated route is the only one consulted.
        assert!(route_auth_block(ROUTE, None, None).is_none());
        let unknown = route_auth_block("synthetic-route", None, None).unwrap();
        assert!(unknown[0].starts_with("unclassified"), "{unknown:?}");
        // The visible start path does not refuse a custom, unclassified route.
        assert!(resolved_route_auth_block("synthetic-route", None, None).is_none());
        assert!(resolved_route_auth_block("OpenAI", None, Some("openai-api-key")).is_some());
    }

    fn write_omni_profile() {
        let path = crate::config::Config::path().expect("config path");
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(
            &path,
            "[providers.omni]\ntype = \"openai-compatible\"\nbase_url = \"https://omni.example.com/v1\"\napi_key_env = \"JCODE_PROVIDER_OMNI_API_KEY\"\nenv_file = \"omni.env\"\n",
        )
        .unwrap();
    }

    #[test]
    fn custom_profile_route_is_judged_by_its_own_key_not_the_openrouter_runtime() {
        let _env = sandbox();
        write_omni_profile();
        // Runtime name is `openrouter` (all compatible profiles run there) and
        // OPENROUTER_API_KEY is set by `sandbox()`, so a wrong fallback would
        // pass; clear it to prove the profile key decides.
        crate::env::remove_var("OPENROUTER_API_KEY");
        let blocked = route_auth_block("openrouter", Some("omni"), None).expect("key missing");
        assert_eq!(blocked.len(), 1, "{blocked:?}");
        assert!(blocked[0].starts_with("omni:"), "{blocked:?}");
        assert!(
            blocked[0].contains("JCODE_PROVIDER_OMNI_API_KEY") && blocked[0].contains("omni.env"),
            "{blocked:?}"
        );
        assert!(!blocked[0].contains("openrouter"), "{blocked:?}");
        assert!(resolved_route_auth_block("openrouter", Some("omni"), None).is_some());

        crate::env::set_var("JCODE_PROVIDER_OMNI_API_KEY", "omni-key");
        assert!(route_auth_block("openrouter", Some("omni"), None).is_none());
        assert!(resolved_route_auth_block("openrouter", Some("omni"), None).is_none());
    }

    fn write_profile(body: &str) {
        let path = crate::config::Config::path().expect("config path");
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(&path, body).unwrap();
    }

    fn profile_blocked(name: &str) -> bool {
        route_auth_block("openrouter", Some(name), None).is_some()
    }

    #[test]
    fn custom_profile_auth_none_is_never_blocked() {
        let _env = sandbox();
        crate::env::remove_var("OPENROUTER_API_KEY");
        write_profile(
            "[providers.noauth]\ntype = \"openai-compatible\"\nbase_url = \"https://gw.example.com/v1\"\nauth = \"none\"\n\
             [providers.local_none]\ntype = \"openai-compatible\"\nbase_url = \"http://localhost:8080/v1\"\nauth = \"none\"\nrequires_api_key = false\n\
             [providers.anth_none]\ntype = \"anthropic-compatible\"\nbase_url = \"https://gw.example.com\"\nauth = \"none\"\n",
        );
        for name in ["noauth", "local_none", "anth_none"] {
            assert!(!profile_blocked(name), "{name} must not be blocked");
            assert!(resolved_route_auth_block("openrouter", Some(name), None).is_none());
        }
    }

    /// The runtime (`new_named_openai_compatible`) never reads
    /// `requires_api_key` and has no localhost exception: Bearer and Header
    /// profiles fail to start without a key.
    #[test]
    fn requires_api_key_false_does_not_waive_the_key_for_bearer_or_header() {
        let _env = sandbox();
        crate::env::remove_var("OPENROUTER_API_KEY");
        write_profile(
            "[providers.nokey]\ntype = \"openai-compatible\"\nbase_url = \"https://gw.example.com/v1\"\nrequires_api_key = false\napi_key_env = \"GW_NOKEY_KEY\"\n\
             [providers.hdr]\ntype = \"openai-compatible\"\nbase_url = \"https://gw.example.com/v1\"\nauth = \"header\"\nrequires_api_key = false\napi_key_env = \"GW_HDR_KEY\"\n\
             [providers.local]\ntype = \"openai-compatible\"\nbase_url = \"http://localhost:8080/v1\"\napi_key_env = \"GW_LOCAL_KEY\"\n",
        );
        for (name, var) in [
            ("nokey", "GW_NOKEY_KEY"),
            ("hdr", "GW_HDR_KEY"),
            ("local", "GW_LOCAL_KEY"),
        ] {
            assert!(profile_blocked(name), "{name} needs {var}");
            crate::env::set_var(var, "k");
            assert!(!profile_blocked(name), "{name} usable with {var}");
            crate::env::remove_var(var);
        }
    }

    /// With neither `api_key_env` nor `api_key` the runtime fails with
    /// "inline api_key not found". It never falls back to OPENROUTER_API_KEY
    /// or openrouter.env, so admission must not either.
    #[test]
    fn custom_profile_without_key_source_does_not_borrow_the_openrouter_defaults() {
        let _env = sandbox();
        write_profile(
            "[providers.bare]\ntype = \"openai-compatible\"\nbase_url = \"https://gw.example.com/v1\"\n",
        );
        // sandbox() set OPENROUTER_API_KEY: a wrong fallback would pass.
        let reasons = route_auth_block("openrouter", Some("bare"), None).expect("blocked");
        assert!(reasons[0].starts_with("bare:"), "{reasons:?}");
        assert!(!reasons[0].contains("OPENROUTER"), "{reasons:?}");
        let file = crate::storage::app_config_dir()
            .unwrap()
            .join("openrouter.env");
        std::fs::write(&file, "OPENROUTER_API_KEY=from-file\n").unwrap();
        assert!(profile_blocked("bare"), "openrouter.env is not a source");
    }

    #[test]
    fn custom_profile_key_sources_match_the_runtime() {
        let _env = sandbox();
        crate::env::remove_var("OPENROUTER_API_KEY");
        write_profile(
            "[providers.inline]\ntype = \"openai-compatible\"\nbase_url = \"https://gw.example.com/v1\"\napi_key = \"inline-secret\"\n\
             [providers.envfile]\ntype = \"openai-compatible\"\nbase_url = \"https://gw.example.com/v1\"\napi_key_env = \"GW_FILE_KEY\"\nenv_file = \"gw.env\"\n\
             [providers.envvar]\ntype = \"openai-compatible\"\nbase_url = \"https://gw.example.com/v1\"\napi_key_env = \"GW_VAR_KEY\"\n\
             [providers.filewithoutvar]\ntype = \"openai-compatible\"\nbase_url = \"https://gw.example.com/v1\"\napi_key_env = \"GW_NOFILE_KEY\"\n\
             [providers.fallback]\ntype = \"openai-compatible\"\nbase_url = \"https://gw.example.com/v1\"\napi_key = \"inline-fallback\"\napi_key_env = \"GW_FALLBACK_KEY\"\n\
             [providers.missing]\ntype = \"openai-compatible\"\nbase_url = \"https://gw.example.com/v1\"\napi_key_env = \"GW_MISSING_KEY\"\nenv_file = \"missing.env\"\n",
        );
        // Inline key alone.
        assert!(!profile_blocked("inline"));
        // Env file source.
        assert!(profile_blocked("envfile"));
        let dir = crate::storage::app_config_dir().unwrap();
        std::fs::write(dir.join("gw.env"), "GW_FILE_KEY=from-file\n").unwrap();
        assert!(!profile_blocked("envfile"));
        // Env var source.
        assert!(profile_blocked("envvar"));
        crate::env::set_var("GW_VAR_KEY", "from-env");
        assert!(!profile_blocked("envvar"));
        crate::env::remove_var("GW_VAR_KEY");
        // Without `env_file` the runtime reads only the process environment.
        std::fs::write(dir.join("openrouter.env"), "GW_NOFILE_KEY=x\n").unwrap();
        assert!(profile_blocked("filewithoutvar"));
        // The runtime falls back to the inline key when the named variable is
        // unset (`.or_else(|| api_key.clone())`).
        assert!(!profile_blocked("fallback"));
        // Missing key blocks, naming the variable and file.
        let reasons = route_auth_block("openrouter", Some("missing"), None).expect("blocked");
        assert!(
            reasons[0].contains("GW_MISSING_KEY") && reasons[0].contains("missing.env"),
            "{reasons:?}"
        );
    }

    /// Anthropic-compatible profiles use `load_anthropic_api_key`: a named
    /// `api_key_env` wins over the inline key, and `requires_api_key` is unused.
    #[test]
    fn anthropic_compatible_profile_needs_its_named_key_even_with_an_inline_key() {
        let _env = sandbox();
        crate::env::remove_var("OPENROUTER_API_KEY");
        write_profile(
            "[providers.anth]\ntype = \"anthropic-compatible\"\nbase_url = \"https://gw.example.com\"\nrequires_api_key = false\napi_key_env = \"ANTH_GW_KEY\"\n\
             [providers.anth_inline]\ntype = \"anthropic-compatible\"\nbase_url = \"https://gw.example.com\"\napi_key = \"inline\"\n\
             [providers.anth_shadow]\ntype = \"anthropic-compatible\"\nbase_url = \"https://gw.example.com\"\napi_key = \"inline\"\napi_key_env = \"ANTH_SHADOW_KEY\"\n",
        );
        assert!(profile_blocked("anth"));
        crate::env::set_var("ANTH_GW_KEY", "k");
        assert!(!profile_blocked("anth"));
        crate::env::remove_var("ANTH_GW_KEY");
        assert!(!profile_blocked("anth_inline"));
        assert!(profile_blocked("anth_shadow"), "named env wins over inline");
    }

    /// The Anthropic runtime refuses an unsafe `api_key_env` or `env_file` at
    /// start, whatever key is available, so admission blocks it too. The OpenAI
    /// path degrades to "no key" through the shared loader and is unchanged.
    #[test]
    fn anthropic_profile_with_unsafe_env_names_is_blocked_like_the_runtime_refuses_it() {
        let _env = sandbox();
        crate::env::remove_var("OPENROUTER_API_KEY");
        write_profile(
            "[providers.badkey]\ntype = \"anthropic-compatible\"\nbase_url = \"https://gw.example.com\"\napi_key_env = \"lower_case\"\n\
             [providers.badfile]\ntype = \"anthropic-compatible\"\nbase_url = \"https://gw.example.com\"\napi_key_env = \"ANTH_OK_KEY\"\nenv_file = \"../escape.env\"\n\
             [providers.goodfile]\ntype = \"anthropic-compatible\"\nbase_url = \"https://gw.example.com\"\napi_key_env = \"ANTH_OK_KEY\"\nenv_file = \"anth.env\"\n",
        );
        crate::env::set_var("lower_case", "k");
        crate::env::set_var("ANTH_OK_KEY", "k");
        let reasons = route_auth_block("openrouter", Some("badkey"), None).expect("blocked");
        assert!(
            reasons[0].contains("invalid api_key_env 'lower_case'"),
            "{reasons:?}"
        );
        let reasons = route_auth_block("openrouter", Some("badfile"), None).expect("blocked");
        assert!(
            reasons[0].contains("invalid env_file '../escape.env'"),
            "{reasons:?}"
        );
        assert!(!profile_blocked("goodfile"));
        crate::env::remove_var("lower_case");
        crate::env::remove_var("ANTH_OK_KEY");
    }

    #[test]
    fn custom_profile_unknown_to_config_stays_unclassified_and_openrouter_key_still_blocks() {
        let _env = sandbox();
        let unknown = route_auth_block("openrouter", Some("ghost"), None).unwrap();
        assert!(unknown[0].starts_with("unclassified"), "{unknown:?}");
        assert!(resolved_route_auth_block("openrouter", Some("ghost"), None).is_none());

        // A real login-provider key keeps the normal login-provider check.
        crate::env::remove_var("OPENROUTER_API_KEY");
        let blocked = route_auth_block("openrouter", Some("openrouter"), None).unwrap();
        assert!(blocked[0].starts_with("openrouter:"), "{blocked:?}");
        crate::env::set_var("OPENROUTER_API_KEY", "test-key");
        assert!(route_auth_block("openrouter", Some("openrouter"), None).is_none());
    }

    #[test]
    fn card_digest_gates_task_evidence() {
        let _env = sandbox();
        let (manifest, _child) = new_manifest("gate-digest");
        assert_eq!(active_cards(&manifest).0, 0);
        write_card(&manifest, "a", "running");
        write_card(&manifest, "done", "completed");
        let (count, digest) = active_cards(&manifest);
        assert_eq!(count, 1, "only active cards are eligible evidence");
        assert_eq!(eligible_since(None, count, &digest), 1);
        assert_eq!(
            eligible_since(Some(&digest), count, &digest),
            0,
            "unchanged set"
        );
        write_card(&manifest, "a", "verifying");
        let (count, changed) = active_cards(&manifest);
        assert_ne!(changed, digest);
        assert_eq!(eligible_since(Some(&digest), count, &changed), 1);
    }

    #[tokio::test]
    async fn auth_failure_blocks_the_run_with_zero_turns_and_one_blocked_decision() {
        let _env = sandbox();
        crate::env::remove_var("OPENROUTER_API_KEY");
        let harness = start("gate-auth", ROUTE).await;
        harness.task.await.unwrap().unwrap();
        let done = load_manifest("gate-auth").unwrap();
        assert_eq!(done.status, OvernightRunStatus::Blocked);
        assert_eq!(harness.calls.load(Ordering::SeqCst), 0);
        assert_eq!(turns("gate-auth"), 0);
        assert!(kinds(&done).contains(&"run_blocked".to_string()));
        let blocked = decisions(&done, "blocked");
        assert_eq!(blocked.len(), 1, "no duplicate decision: {blocked:?}");
        assert!(blocked[0].contains("jcode login --provider"), "{blocked:?}");
    }

    #[tokio::test]
    async fn unclassified_route_blocks_without_a_turn() {
        let _env = sandbox();
        let harness = start("gate-unknown", "synthetic-route").await;
        harness.task.await.unwrap().unwrap();
        let done = load_manifest("gate-unknown").unwrap();
        assert_eq!(done.status, OvernightRunStatus::Blocked);
        assert_eq!(harness.calls.load(Ordering::SeqCst), 0);
        assert!(decisions(&done, "blocked")[0].contains("unclassified"));
    }

    #[tokio::test]
    async fn cancel_requested_at_launch_wins_over_an_auth_block() {
        let _env = sandbox();
        crate::env::remove_var("OPENROUTER_API_KEY");
        let (manifest, child) = new_manifest("gate-cancel-auth");
        cancel("gate-cancel-auth");
        let harness = launch(manifest, child, ROUTE).await;
        harness.task.await.unwrap().unwrap();
        let done = load_manifest("gate-cancel-auth").unwrap();
        assert_eq!(
            done.status,
            OvernightRunStatus::Cancelled,
            "cancel must win"
        );
        assert!(done.cancel_requested_at.is_some());
        assert_eq!(harness.calls.load(Ordering::SeqCst), 0);
        assert!(!kinds(&done).contains(&"run_blocked".to_string()));
    }

    #[tokio::test]
    async fn block_run_revokes_and_drains_owned_workers_before_going_terminal() {
        let _env = sandbox();
        let (manifest, child) = new_manifest("gate-block-settle");
        let store = session_control::store();
        store.register("gate-worker", Some(&child.id)).unwrap();
        let held = store.admit("gate-worker").unwrap();
        let work = held.reserve().unwrap();
        let release = tokio::spawn(async move {
            tokio::time::sleep(Duration::from_millis(100)).await;
            drop(work);
        });
        block_run(&manifest, &["Task is not authorized".into()])
            .await
            .unwrap();
        release.await.unwrap();
        assert!(
            store.admit("gate-worker").is_err(),
            "worker admission revoked"
        );
        let done = load_manifest("gate-block-settle").unwrap();
        assert_eq!(done.status, OvernightRunStatus::Blocked);
        assert!(done.stop_settlement.as_ref().unwrap().is_settled());
        assert!(kinds(&done).contains(&"run_blocked".to_string()));
    }

    #[tokio::test]
    async fn block_run_with_unsettled_workers_stays_non_terminal() {
        let _env = sandbox();
        let (manifest, child) = new_manifest("gate-block-unsettled");
        let store = session_control::store();
        store.register("stuck-worker", Some(&child.id)).unwrap();
        let held = store.admit("stuck-worker").unwrap();
        let work = held.reserve().unwrap();
        block_run(&manifest, &["Task is not authorized".into()])
            .await
            .unwrap();
        let after = load_manifest("gate-block-unsettled").unwrap();
        assert_eq!(
            after.status,
            OvernightRunStatus::Running,
            "no terminal claim"
        );
        assert!(after.completed_at.is_none());
        assert!(kinds(&after).contains(&"run_blocked_unresolved".to_string()));
        drop(work);
    }

    #[test]
    fn task_gate_decision_denies_despite_signal_boundary_and_cards() {
        let _env = sandbox();
        let (mut manifest, _child) = new_manifest("gate-task-pure");
        write_card(&manifest, "a", "running");
        // Unverified ownership, no authorization: strict policy denies.
        manifest.task_admission = Some(Default::default());
        let (decision, _) = decide_and_record(
            &manifest,
            true,
            Some("target_wake"),
            Some("background:x"),
            None,
            false,
        )
        .unwrap();
        assert_ne!(decision.verdict, AdmissionVerdict::Admitted, "{decision:?}");
        assert_eq!(decisions(&manifest, "admitted").len(), 0);
        assert_eq!(
            decisions(&manifest, "blocked").len() + decisions(&manifest, "waiting").len(),
            1
        );
    }

    /// A manifest that already carries task authority, written directly like a
    /// native admission would (a save cannot add authority to a legacy run).
    fn authorized(
        run_id: &str,
        ready: bool,
        dependencies_ready: bool,
    ) -> (OvernightManifest, Session) {
        let (mut manifest, child) = new_manifest(run_id);
        std::fs::write(
            storage::jcode_dir().unwrap().join("config.toml"),
            "[tasks]\nunattended = true\n",
        )
        .unwrap();
        manifest.working_dir = Some(storage::jcode_dir().unwrap().to_string_lossy().into());
        manifest.task_admission = Some(crate::config::TaskAdmissionSnapshot {
            scope: "bounded task".into(),
            authorization: Some(crate::config::TaskAuthorization {
                scope: "bounded task".into(),
                max_attempts: 2,
                max_runtime_seconds: 100,
                max_spend_microusd: 100,
            }),
            ownership_verified: ready,
            dependencies_ready,
            workspace_ready: true,
            ..Default::default()
        });
        storage::write_json(&manifest_path(run_id).unwrap(), &manifest).unwrap();
        (manifest, child)
    }

    #[tokio::test]
    async fn unknown_authority_unsupported_monetary_and_open_predecessor_dispatch_nothing() {
        let _env = sandbox();
        for (run_id, ready, deps) in [
            ("gate-unowned", false, true),
            ("gate-monetary", true, true),
            ("gate-predecessor", true, false),
        ] {
            let (manifest, child) = authorized(run_id, ready, deps);
            let harness = launch(manifest, child, ROUTE).await;
            let result = harness.task.await.unwrap();
            assert!(result.is_err(), "{run_id}: dispatch must be denied");
            assert_eq!(
                harness.calls.load(Ordering::SeqCst),
                0,
                "{run_id}: no provider call"
            );
            assert_eq!(turns(run_id), 0, "{run_id}: no coordinator turn");
        }
    }

    #[tokio::test]
    async fn idle_run_waits_and_one_completion_admits_exactly_one_turn() {
        let _env = sandbox();
        let harness = start("gate-idle", ROUTE).await;
        let manifest = harness.manifest.clone();
        until("first turn", || turns("gate-idle") == 1).await;
        until("waiting decision", || {
            !decisions(&manifest, "waiting").is_empty()
        })
        .await;
        tokio::time::sleep(Duration::from_millis(400)).await;
        assert_eq!(
            harness.calls.load(Ordering::SeqCst),
            1,
            "idle run re-prompted"
        );
        let before = decisions(&manifest, "waiting").len();
        assert_eq!(before, 1, "idle waiting records once, not per poll");

        // A signal for another session never wakes this run.
        complete_background("someone-else", "other");
        tokio::time::sleep(Duration::from_millis(200)).await;
        assert_eq!(harness.calls.load(Ordering::SeqCst), 1);

        complete_background(&manifest.coordinator_session_id, "t1");
        until("second turn", || harness.calls.load(Ordering::SeqCst) == 2).await;
        until("second wait", || decisions(&manifest, "waiting").len() == 2).await;
        tokio::time::sleep(Duration::from_millis(300)).await;
        assert_eq!(
            harness.calls.load(Ordering::SeqCst),
            2,
            "one signal, one turn"
        );
        let admitted = decisions(&manifest, "admitted");
        assert!(admitted[0].contains("mission"), "{admitted:?}");
        assert!(admitted[1].contains("background:t1"), "{admitted:?}");

        cancel("gate-idle");
        finish(harness).await;
        assert_eq!(
            load_manifest("gate-idle").unwrap().status,
            OvernightRunStatus::Cancelled
        );
    }

    #[tokio::test]
    async fn running_card_never_respins_and_a_changed_card_admits_exactly_one_turn() {
        let _env = sandbox();
        let (manifest, child) = new_manifest("gate-cards");
        write_card(&manifest, "w1", "running");
        let harness = launch(manifest.clone(), child, ROUTE).await;
        until("first turn", || turns("gate-cards") == 1).await;
        until("waiting", || !decisions(&manifest, "waiting").is_empty()).await;
        // The worker's card is still running and nothing changed: no extra turn,
        // even across forced rescans of durable state.
        tokio::time::sleep(Duration::from_millis(300)).await;
        overflow_bus();
        tokio::time::sleep(Duration::from_millis(400)).await;
        assert_eq!(
            harness.calls.load(Ordering::SeqCst),
            1,
            "running card re-admitted a turn"
        );
        assert_eq!(decisions(&manifest, "admitted").len(), 1);

        // The card changes (worker progressed): the next rescan admits one turn.
        write_card(&manifest, "w1", "verifying");
        overflow_bus();
        until("changed card turn", || {
            harness.calls.load(Ordering::SeqCst) == 2
        })
        .await;
        tokio::time::sleep(Duration::from_millis(400)).await;
        assert_eq!(
            harness.calls.load(Ordering::SeqCst),
            2,
            "one change, one turn"
        );
        assert!(decisions(&manifest, "admitted")[1].contains("tasks:1"));

        cancel("gate-cards");
        finish(harness).await;
    }

    #[tokio::test]
    async fn bus_overflow_alone_is_not_evidence() {
        let _env = sandbox();
        let harness = start("gate-lag", ROUTE).await;
        let manifest = harness.manifest.clone();
        until("first turn", || turns("gate-lag") == 1).await;
        until("waiting", || !decisions(&manifest, "waiting").is_empty()).await;
        for _ in 0..3 {
            overflow_bus();
            tokio::time::sleep(Duration::from_millis(300)).await;
        }
        assert_eq!(
            harness.calls.load(Ordering::SeqCst),
            1,
            "lag admitted a turn with no evidence"
        );
        assert_eq!(decisions(&manifest, "admitted").len(), 1);
        cancel("gate-lag");
        finish(harness).await;
    }

    #[tokio::test]
    async fn unreadable_manifest_while_waiting_is_tolerated_then_cancel_still_stops() {
        let _env = sandbox();
        let harness = start("gate-unreadable", ROUTE).await;
        let manifest = harness.manifest.clone();
        until("waiting", || !decisions(&manifest, "waiting").is_empty()).await;
        let path = manifest_path("gate-unreadable").unwrap();
        let good = load_manifest("gate-unreadable").unwrap();
        std::fs::write(&path, b"{not json").unwrap();
        until("unreadable recorded", || {
            kinds(&manifest).contains(&"manifest_unreadable".to_string())
        })
        .await;
        assert!(
            !harness.task.is_finished(),
            "a transient read error must not fail the run"
        );
        storage::write_json(&path, &good).unwrap();
        cancel("gate-unreadable");
        finish(harness).await;
        assert_eq!(
            load_manifest("gate-unreadable").unwrap().status,
            OvernightRunStatus::Cancelled
        );
    }

    #[tokio::test]
    async fn wake_boundary_admits_the_morning_report_turn() {
        let _env = sandbox();
        let (mut manifest, child) = new_manifest("gate-wake");
        let wake = Utc::now() + ChronoDuration::milliseconds(500);
        manifest.target_wake_at = wake;
        manifest.handoff_ready_at = wake;
        manifest.post_wake_grace_until = wake + ChronoDuration::hours(1);
        save_manifest(&manifest).unwrap();
        let harness = launch(manifest.clone(), child, ROUTE).await;
        until("morning report turn", || {
            harness.calls.load(Ordering::SeqCst) == 2
        })
        .await;
        assert!(
            decisions(&manifest, "admitted")
                .iter()
                .any(|d| d.contains("target_wake"))
        );
        assert!(kinds(&manifest).contains(&"morning_report_requested".to_string()));
        cancel("gate-wake");
        finish(harness).await;
    }
}

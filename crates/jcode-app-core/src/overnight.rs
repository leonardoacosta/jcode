use crate::agent::Agent;
use crate::provider::Provider;
use crate::session::{Session, SessionStatus};
use crate::storage;
use crate::tool::Registry;
use anyhow::{Context, Result};
use chrono::{Duration as ChronoDuration, Utc};
use serde_json::{Value, json};
#[cfg(unix)]
use std::ffi::CString;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

pub use jcode_overnight_core::{
    GitSnapshot, LimitBreach, LimitKind, OVERNIGHT_VERSION, OvernightCommand, OvernightDuration,
    OvernightEvent, OvernightLimits, OvernightManifest, OvernightPreflight, OvernightProgressCard,
    OvernightRunStatus, OvernightTaskCard, OvernightTaskCardAfter, OvernightTaskCardBefore,
    OvernightTaskCardSummary, OvernightTaskCardValidation, OvernightTaskStatusCounts,
    ResourceSnapshot, StopSettlement, UsageLimitSnapshot, UsageProjection, UsageProviderSnapshot,
    build_continuation_prompt, build_coordinator_prompt, build_final_wrapup_prompt,
    build_handoff_ready_prompt, build_morning_report_prompt, build_post_wake_continuation_prompt,
    build_progress_card_from_parts, build_review_html, build_signal_prompt,
    build_visible_current_session_prompt, event_class, format_log_markdown_from_events,
    format_minutes, format_status_markdown_from_summary, format_status_markdown_with_stale,
    git_summary, html_escape, overnight_usage, parse_duration, parse_overnight_command,
    preflight_summary, prompt_event_summary, render_task_cards_html, resource_summary,
    summarize_task_cards_slice, task_card_title, task_card_validated, task_status_bucket,
};

use admission_gate::{
    SignalWatch, Step, StepFlags, Wait, active_cards, block_on_reasons, block_run,
    decide_and_record, next_boundary, next_step, resolved_route_auth_block, route_auth_block,
};
use jcode_overnight_core::AdmissionVerdict;

const RESOURCE_SAMPLE_INTERVAL: Duration = Duration::from_secs(5 * 60);
const LONG_TURN_NOTICE_INTERVAL: Duration = Duration::from_secs(30 * 60);

mod admission;
mod admission_gate;
mod enforce;
pub mod model_chain;
pub mod restart;
mod usage;
pub use admission::{
    TaskAttemptClaim, restart_task_attempt, restart_task_attempt_with_provider, start_task_attempt,
    start_task_attempt_with_provider, task_monetary_account,
};
#[cfg(test)]
pub(crate) use enforce::tests::run_manifest as test_run_manifest;
pub use enforce::{
    ensure_worker_spawn_allowed, is_supervised, resettle_in_background,
    visible_mode_limit_rejection, worker_spawn_gate,
};
pub(crate) use model_chain::pinned_request as pin_model_request;
pub(crate) use usage::{note_session_account, observe_stream_usage};
pub use usage::{read_usage_reconciled, reconciled_path};

#[derive(Debug, Clone)]
pub struct OvernightLaunch {
    pub manifest: OvernightManifest,
    /// Initial coordinator prompt to enqueue in the visible launching session.
    /// When present, the TUI should run this as a normal user turn so tool calls,
    /// spawned agents, and streaming output are visible like any other session.
    pub initial_prompt: Option<String>,
}

pub struct OvernightStartOptions {
    pub duration: OvernightDuration,
    pub mission: Option<String>,
    pub parent_session: Session,
    pub provider: Arc<dyn Provider>,
    pub registry: Registry,
    pub working_dir: Option<PathBuf>,
    /// When true, run the overnight coordinator in the session that launched
    /// `/overnight` instead of forking an invisible child transcript.
    pub use_current_session: bool,
    /// Enforced run bounds. `OvernightLimits::standard()` for the default.
    pub limits: OvernightLimits,
}

/// Revalidate the original persisted task bounds against current user policy.
/// Legacy manual runs return None, never synthesized authorization from a clock.
pub fn revalidate_task_admission(
    manifest: &OvernightManifest,
    policy: &crate::config::TasksConfig,
) -> Option<crate::config::AdmissionDecision> {
    let mut snapshot = manifest.task_admission.clone()?;
    snapshot.cancelled |= manifest.cancel_requested_at.is_some()
        || !matches!(manifest.status, OvernightRunStatus::Running);
    snapshot.ownership_verified &= manifest.process_id == std::process::id();
    snapshot.workspace_ready &= manifest
        .working_dir
        .as_deref()
        .is_some_and(|path| Path::new(path).is_dir());
    let elapsed = Utc::now()
        .signed_duration_since(manifest.started_at)
        .num_seconds()
        .max(0) as u64;
    snapshot.runtime_seconds = snapshot.runtime_seconds.max(elapsed);
    Some(policy.admit(&snapshot))
}

/// Fail closed until the evidence-aware executor supplies atomic attempt/spend
/// reservations. A diagnostic ready decision never starts provider work.
pub fn ensure_task_dispatch(
    manifest: &OvernightManifest,
    policy: &crate::config::TasksConfig,
) -> Result<()> {
    if let Some(decision) = revalidate_task_admission(manifest, policy) {
        anyhow::ensure!(
            decision.state == crate::config::AdmissionState::Ready,
            "Task admission denied: {}",
            decision.reasons.join("; ")
        );
        anyhow::bail!(
            "Unattended task dispatch requires an authoritative attempt reservation and spend enforcer; snapshot alone is not execution authority"
        );
    }
    Ok(())
}

pub fn start_overnight_run(options: OvernightStartOptions) -> Result<OvernightLaunch> {
    if options.use_current_session
        && let Some(reason) = visible_mode_limit_rejection(&options.limits)
    {
        anyhow::bail!(reason);
    }
    if options.use_current_session
        && let Some(reasons) = resolved_route_auth_block(
            options.provider.name(),
            options.parent_session.provider_key.as_deref(),
            options.parent_session.route_api_method.as_deref(),
        )
    {
        anyhow::bail!(
            "Overnight blocked: selected provider route has no usable auth. {}",
            reasons.join("; ")
        );
    }
    // Chain models the route does not serve are dropped; an empty chain blocks.
    let chain_plan = model_chain::plan(
        &crate::config::config().overnight,
        &options.provider.available_models_for_switching(),
    )
    .map_err(|reasons| anyhow::anyhow!("Overnight blocked: {}", reasons.join("; ")))?;
    if options.use_current_session {
        // The visible session has no supervisor loop; pick its first served
        // coordinator model here. Same provider route and account throughout.
        let provider_key = options.parent_session.provider_key.clone();
        // A route that cannot switch models keeps the session's current model;
        // the chain still applies to later failures.
        let _ = model_chain::select_first(&chain_plan.coordinator, |model| {
            model_chain::set_provider_model_on_same_route(
                options.provider.as_ref(),
                provider_key.as_deref(),
                model,
            )
        });
    }
    let run_id = crate::id::new_id("overnight");
    let started_at = Utc::now();
    let duration = ChronoDuration::minutes(options.duration.minutes as i64);
    let target_wake_at = started_at + duration;
    let handoff_ready_at = target_wake_at - ChronoDuration::minutes(30).min(duration / 4);
    let post_wake_grace_until = target_wake_at + ChronoDuration::hours(2);
    let run_dir = run_dir(&run_id)?;
    let events_path = run_dir.join("events.jsonl");
    let human_log_path = run_dir.join("run.log");
    let review_path = run_dir.join("review.html");
    let review_notes_path = run_dir.join("review-notes.md");
    let preflight_path = run_dir.join("preflight.json");
    let task_cards_dir = run_dir.join("task-cards");
    let issue_drafts_dir = run_dir.join("issue-drafts");
    let validation_dir = run_dir.join("validation");
    std::fs::create_dir_all(&task_cards_dir)?;
    std::fs::create_dir_all(&issue_drafts_dir)?;
    std::fs::create_dir_all(&validation_dir)?;

    let mut child = if options.use_current_session {
        options.parent_session.clone()
    } else {
        create_coordinator_session(&options.parent_session, &options.mission)?
    };
    if let Some(working_dir) = options.working_dir.as_ref() {
        child.working_dir = Some(working_dir.to_string_lossy().to_string());
    }
    child.model = Some(options.provider.model());
    let coordinator_session_id = child.id.clone();
    let coordinator_session_name = child.display_name().to_string();
    let child_is_canary = child.is_canary;
    if !options.use_current_session {
        child.status = SessionStatus::Closed;
    }
    child.save()?;

    if !options.use_current_session
        && let Ok(todos) = crate::todo::load_todos(&options.parent_session.id)
    {
        let _ = crate::todo::save_todos(&coordinator_session_id, &todos);
    }

    let manifest = OvernightManifest {
        task_attempt: None,
        task_admission: None,
        version: OVERNIGHT_VERSION,
        run_id: run_id.clone(),
        parent_session_id: options.parent_session.id.clone(),
        coordinator_session_id: coordinator_session_id.clone(),
        coordinator_session_name,
        started_at,
        target_wake_at,
        handoff_ready_at,
        post_wake_grace_until,
        morning_report_posted_at: None,
        completed_at: None,
        cancel_requested_at: None,
        status: OvernightRunStatus::Running,
        mission: options.mission.clone(),
        working_dir: child.working_dir.clone(),
        provider_name: options.provider.name().to_string(),
        model: options.provider.model(),
        // Coordinator plus allowed helpers; drives the burn-rate projection.
        max_agents_guidance: options
            .limits
            .max_agents
            .map_or(2, |n| n.saturating_add(1).min(u32::from(u8::MAX)) as u8),
        limits: options.limits,
        stop_settlement: None,
        limit_breach: None,
        process_id: std::process::id(),
        supervisor_start_time: crate::platform::process_start_time(std::process::id()),
        heartbeat_at: Some(started_at),
        run_dir,
        events_path,
        human_log_path,
        review_path,
        review_notes_path,
        preflight_path,
        task_cards_dir,
        issue_drafts_dir,
        validation_dir,
        last_activity_at: started_at,
    };

    save_manifest(&manifest)?;
    model_chain::persist_plan(&manifest.run_dir, &chain_plan)?;
    write_initial_review_notes(&manifest)?;
    write_task_card_schema(&manifest)?;
    record_event(
        &manifest,
        "run_started",
        format!(
            "Started overnight run for {} (target wake: {})",
            format_minutes(options.duration.minutes),
            manifest.target_wake_at.to_rfc3339()
        ),
        json!({
            "mission": manifest.mission,
            "parent_session_id": manifest.parent_session_id,
            "coordinator_session_id": manifest.coordinator_session_id,
            "review_path": manifest.review_path,
        }),
        true,
    )?;
    model_chain::record_plan(&manifest, &chain_plan);
    render_review_html(&manifest)?;
    // Policy hook: restrict the coordinator session to the coordinator tool
    // allowlist (supervised child session and visible current session alike).
    // It lifts itself once the run is terminal.
    crate::tool::coordinator_policy::install_for_overnight(&coordinator_session_id, &run_id);

    let initial_prompt = if options.use_current_session {
        Some(build_visible_current_session_prompt(&manifest))
    } else {
        spawn_supervisor(
            manifest.clone(),
            child,
            options.provider,
            options.registry,
            child_is_canary,
        );
        None
    };

    Ok(OvernightLaunch {
        manifest,
        initial_prompt,
    })
}

fn create_coordinator_session(parent: &Session, mission: &Option<String>) -> Result<Session> {
    let title = Some(match mission {
        Some(mission) => format!("Overnight: {}", crate::util::truncate_str(mission, 48)),
        None => "Overnight coordinator".to_string(),
    });
    let mut child = Session::create(Some(parent.id.clone()), title);
    child.replace_messages(parent.messages.clone());
    child.compaction = parent.compaction.clone();
    child.provider_key = parent.provider_key.clone();
    child.route_api_method = parent.route_api_method.clone();
    child.reasoning_effort = parent.reasoning_effort.clone();
    child.subagent_model = parent.subagent_model.clone();
    child.improve_mode = parent.improve_mode;
    child.autoreview_enabled = Some(false);
    child.autojudge_enabled = Some(false);
    child.is_canary = parent.is_canary;
    child.testing_build = parent.testing_build.clone();
    child.working_dir = parent.working_dir.clone();
    child.provider_session_id = None;
    Ok(child)
}

/// Record a supervisor error. The run is closed as `Failed` only when no owned
/// work is unresolved: a recorded unresolved stop settlement means workers may
/// still be running, so the run stays non-terminal and can be cancelled again.
async fn record_supervisor_failure(manifest: &OvernightManifest, err: &anyhow::Error) {
    let mut updated = load_manifest(&manifest.run_id).unwrap_or(manifest.clone());
    let unresolved = matches!(
        updated.stop_settlement,
        Some(jcode_overnight_core::StopSettlement::Unresolved { .. })
    );
    if !unresolved {
        updated.status = OvernightRunStatus::Failed;
        updated.completed_at = Some(Utc::now());
        let _ = save_manifest(&updated);
    }
    let _ = record_event(
        &updated,
        if unresolved {
            "run_failed_unresolved"
        } else {
            "run_failed"
        },
        if unresolved {
            format!(
                "Overnight supervisor failed while owned work is not confirmed stopped; run left non-terminal: {err}"
            )
        } else {
            format!("Overnight supervisor failed: {err}")
        },
        json!({ "error": crate::util::format_error_chain(err) }),
        true,
    );
    if !unresolved {
        usage::finalize(&updated).await;
    }
    let _ = render_review_html(&updated);
}

fn spawn_supervisor(
    manifest: OvernightManifest,
    child: Session,
    provider: Arc<dyn Provider>,
    registry: Registry,
    child_is_canary: bool,
) {
    let fut = async move {
        if let Err(err) =
            run_supervisor(manifest.clone(), child, provider, registry, child_is_canary).await
        {
            record_supervisor_failure(&manifest, &err).await;
        }
    };

    if tokio::runtime::Handle::try_current().is_ok() {
        tokio::spawn(fut);
    } else {
        std::thread::spawn(move || match tokio::runtime::Runtime::new() {
            Ok(runtime) => runtime.block_on(fut),
            Err(err) => crate::logging::error(&format!(
                "Failed to start overnight supervisor runtime: {}",
                err
            )),
        });
    }
}

async fn run_supervisor(
    manifest: OvernightManifest,
    child: Session,
    provider: Arc<dyn Provider>,
    registry: Registry,
    child_is_canary: bool,
) -> Result<()> {
    if manifest.task_admission.is_some() {
        ensure_task_dispatch(&manifest, &crate::config::Config::load_strict()?.tasks)?;
    }
    // Only the explicitly selected route is checked: no other account is
    // probed and no route is substituted.
    // A cancel requested at launch wins over a preflight block.
    if let Some(cause) = enforce::pending_stop(&load_manifest(&manifest.run_id)?, None) {
        return enforce::stop_until_settled(&load_manifest(&manifest.run_id)?, cause).await;
    }
    if let Some(reasons) = route_auth_block(
        provider.name(),
        child.provider_key.as_deref(),
        child.route_api_method.as_deref(),
    ) {
        return block_on_reasons(&manifest, &reasons).await;
    }
    record_event(
        &manifest,
        "preflight_started",
        "Collecting overnight usage/resource/git preflight".to_string(),
        json!({}),
        true,
    )?;
    let preflight = gather_preflight(&manifest).await;
    storage::write_json(&manifest.preflight_path, &preflight)?;
    record_event(
        &manifest,
        "preflight_completed",
        preflight_summary(&preflight),
        serde_json::to_value(&preflight).unwrap_or_else(|_| json!({})),
        true,
    )?;
    render_review_html(&manifest)?;

    if child_is_canary {
        registry.register_selfdev_tools().await;
    }

    let _usage_run = usage::register_run(&manifest);
    let mut agent = Agent::new_with_session(provider, registry, child, None);
    let mut coordinator_chain = {
        let models = model_chain::coordinator_models(&manifest);
        let (selected, skipped) =
            model_chain::select_first(&models, |model| agent.set_model_on_same_route(model));
        match selected {
            Some(selected) => {
                record_event(
                    &manifest,
                    "model_chain_started",
                    format!("Coordinator starts on {selected}"),
                    json!({ "model": selected, "skipped": skipped }),
                    true,
                )?;
                model_chain::ModelChain::starting_at(models, &selected)
            }
            None => {
                // The route lists its models at preflight, so reaching here means
                // it cannot switch models at all. Keep the session's own model and
                // keep the chain for later failures instead of ending the run.
                let current = agent.provider_model();
                record_event(
                    &manifest,
                    "model_chain_started",
                    format!("Route accepted no chain model; coordinator stays on {current}"),
                    json!({ "model": current, "skipped": skipped }),
                    true,
                )?;
                model_chain::ModelChain::starting_at(models, &current)
            }
        }
    };
    let mut watch = SignalWatch::new(&manifest.coordinator_session_id);
    let mut seen_cards: Option<String> = None;
    let mut last_waiting = false;
    let mut next_prompt = build_coordinator_prompt(&manifest, &preflight);
    let mut flags = StepFlags::default();
    let mut first_turn = true;
    let mut boundary: Option<&'static str> = None;
    let mut pending_signal: Option<String> = None;

    loop {
        let current = load_manifest(&manifest.run_id)?;
        // Cancellation and limits win over everything else, including task
        // admission, so a stop is never turned into a failure by a denial.
        if let Some(cause) = enforce::pending_stop(&current, None) {
            enforce::stop_until_settled(&current, cause).await?;
            break;
        }

        let now = Utc::now();
        if !flags.handoff_notice_sent
            && now >= current.handoff_ready_at
            && now < current.target_wake_at
        {
            record_event(
                &current,
                "handoff_ready_notice",
                "Entering handoff-ready mode".to_string(),
                json!({ "target_wake_at": current.target_wake_at }),
                true,
            )?;
            next_prompt = build_handoff_ready_prompt(&current);
            flags.handoff_notice_sent = true;
            boundary = Some("handoff_ready");
        }

        // Admission: every turn needs recorded evidence. Task policy is
        // revalidated deterministically on every pass, signal or not.
        let drained = watch.drain(agent.soft_interrupt_count());
        let signal = pending_signal.take().or(drained);
        let (decision, _digest) = decide_and_record(
            &current,
            first_turn,
            boundary,
            signal.as_deref(),
            seen_cards.as_deref(),
            last_waiting,
        )?;
        last_waiting = decision.verdict == AdmissionVerdict::Waiting;
        match decision.verdict {
            AdmissionVerdict::Blocked => {
                // `record_decision` above already logged the blocked verdict.
                block_run(&current, &[decision.reason]).await?;
                break;
            }
            AdmissionVerdict::Waiting => {
                match watch
                    .wait(&agent, &current, next_boundary(&current, &flags, now))
                    .await
                {
                    // Stop and rescan fall through to the loop top, which
                    // re-checks stops and re-scans durable state.
                    Wait::Stop | Wait::Rescan => {}
                    Wait::Signal(signal) => pending_signal = Some(signal),
                    Wait::Boundary => {
                        match next_step(&load_manifest(&manifest.run_id)?, &mut flags).await? {
                            Step::Done => break,
                            Step::Prompt(prompt, due) => {
                                next_prompt = prompt;
                                boundary = due;
                            }
                        }
                    }
                }
                continue;
            }
            AdmissionVerdict::Admitted => {}
        }
        if let (Some(signal), None, false) = (signal.as_deref(), boundary, first_turn) {
            next_prompt = build_signal_prompt(signal, &current);
        }
        if current.task_admission.is_some() {
            // A file/snapshot is not an attempt reservation or a spend enforcer.
            // Keep this path closed until the evidence-aware executor owns both.
            // Token/time limits below only stop a run; they never authorize it.
            let policy = crate::config::Config::load_strict()?.tasks;
            ensure_task_dispatch(&current, &policy)?;
        }
        first_turn = false;

        record_event(
            &current,
            "coordinator_turn_started",
            prompt_event_summary(&next_prompt),
            json!({ "prompt_preview": crate::util::truncate_str(&next_prompt, 600) }),
            true,
        )?;
        render_review_html(&current)?;

        let output = match model_chain::run_turn_with_chain(
            &mut agent,
            &current,
            &next_prompt,
            &mut coordinator_chain,
        )
        .await?
        {
            model_chain::ChainTurn::Finished(output) => output,
            model_chain::ChainTurn::Stopped(cause) => {
                enforce::stop_until_settled(&load_manifest(&manifest.run_id)?, cause).await?;
                break;
            }
            // The chain helper already ended the run `blocked`.
            model_chain::ChainTurn::Ended => break,
        };
        let after_turn = load_manifest(&manifest.run_id)?;
        record_event(
            &after_turn,
            "coordinator_turn_completed",
            "Coordinator turn completed".to_string(),
            json!({ "output_preview": crate::util::truncate_str(&output, 4000) }),
            true,
        )?;
        render_review_html(&after_turn)?;

        let after_turn = load_manifest(&manifest.run_id)?;
        // Cards as the turn left them are the baseline: only a later change is
        // new evidence, so a still-running card never re-admits a turn.
        seen_cards = Some(active_cards(&after_turn).1);
        if let Some(cause) = enforce::pending_stop(&after_turn, None) {
            enforce::stop_until_settled(&after_turn, cause).await?;
            break;
        }
        match next_step(&after_turn, &mut flags).await? {
            Step::Done => break,
            Step::Prompt(prompt, due) => {
                next_prompt = prompt;
                boundary = due;
            }
        }
    }

    Ok(())
}

async fn gather_preflight(manifest: &OvernightManifest) -> OvernightPreflight {
    // Admission inspects only the selected route's auth. Querying every
    // connected provider's usage would probe unrelated accounts, so the
    // preflight carries no all-provider usage fetch.
    let mut usage = build_usage_projection(&[], manifest);
    usage.notes.push(
        "Usage was not queried at preflight: only the selected route is inspected, never other accounts."
            .to_string(),
    );
    let resources = gather_resource_snapshot(manifest.working_dir.as_deref().map(Path::new));
    let git = gather_git_snapshot(manifest.working_dir.as_deref().map(Path::new));
    OvernightPreflight {
        captured_at: Utc::now(),
        usage,
        resources,
        git,
    }
}

fn build_usage_projection(
    reports: &[crate::usage::ProviderUsage],
    manifest: &OvernightManifest,
) -> UsageProjection {
    let providers: Vec<UsageProviderSnapshot> = reports
        .iter()
        .map(|provider| UsageProviderSnapshot {
            provider_name: provider.provider_name.clone(),
            hard_limit_reached: provider.hard_limit_reached,
            error: provider.error.clone(),
            limits: provider
                .limits
                .iter()
                .map(|limit| UsageLimitSnapshot {
                    name: limit.name.clone(),
                    usage_percent: limit.usage_percent,
                    resets_at: limit.resets_at.clone(),
                })
                .collect(),
            extra_info: provider.extra_info.clone(),
        })
        .collect();

    let max_usage = providers
        .iter()
        .flat_map(|provider| provider.limits.iter().map(|limit| limit.usage_percent))
        .fold(None::<f32>, |acc, value| {
            Some(acc.unwrap_or(value).max(value))
        });
    let hard_limit = providers.iter().any(|provider| provider.hard_limit_reached);
    let has_errors = providers.iter().any(|provider| provider.error.is_some());
    let hours = manifest
        .target_wake_at
        .signed_duration_since(manifest.started_at)
        .num_minutes()
        .max(1) as f32
        / 60.0;
    let delta_min = (hours * 3.0).min(35.0);
    let delta_max = (hours * 7.0 * manifest.max_agents_guidance as f32 / 2.0).min(75.0);
    let projected_end_min = max_usage.map(|current| (current + delta_min).min(100.0));
    let projected_end_max = max_usage.map(|current| (current + delta_max).min(100.0));

    let risk = if hard_limit || projected_end_max.is_some_and(|value| value >= 95.0) {
        "high"
    } else if projected_end_max.is_some_and(|value| value >= 80.0) || has_errors {
        "medium"
    } else if max_usage.is_some() {
        "low"
    } else {
        "unknown"
    }
    .to_string();

    let confidence = if max_usage.is_some() && !has_errors {
        "medium"
    } else {
        "low"
    }
    .to_string();

    let mut notes = Vec::new();
    if providers.is_empty() {
        notes.push(
            "No connected-provider usage reports were available; projection is heuristic."
                .to_string(),
        );
    } else {
        notes.push("Projection uses provider usage percentages plus a conservative overnight burn-rate heuristic.".to_string());
    }
    notes.push("Usage is advisory evidence only: it never authorizes dispatch. Admission is decided by the selected route's auth and the deterministic task policy.".to_string());

    UsageProjection {
        captured_at: Utc::now(),
        risk,
        confidence,
        projected_delta_min_percent: max_usage.map(|_| delta_min),
        projected_delta_max_percent: max_usage.map(|_| delta_max),
        projected_end_min_percent: projected_end_min,
        projected_end_max_percent: projected_end_max,
        providers,
        notes,
    }
}

pub fn gather_resource_snapshot(working_dir: Option<&Path>) -> ResourceSnapshot {
    let (memory_total_mb, memory_available_mb, swap_total_mb, swap_free_mb) = detect_memory();
    let memory_used_percent =
        memory_total_mb
            .zip(memory_available_mb)
            .and_then(|(total, available)| {
                if total == 0 {
                    None
                } else {
                    Some(((total.saturating_sub(available)) as f32 / total as f32) * 100.0)
                }
            });
    let (load_one, cpu_count) = detect_load();
    let (battery_percent, battery_status) = detect_battery();
    let disk_available_gb = working_dir.and_then(disk_available_gb);

    ResourceSnapshot {
        captured_at: Utc::now(),
        memory_total_mb,
        memory_available_mb,
        memory_used_percent,
        swap_total_mb,
        swap_free_mb,
        load_one,
        cpu_count,
        battery_percent,
        battery_status,
        disk_available_gb,
    }
}

fn detect_memory() -> (Option<u64>, Option<u64>, Option<u64>, Option<u64>) {
    #[cfg(target_os = "linux")]
    {
        let Ok(contents) = std::fs::read_to_string("/proc/meminfo") else {
            return (None, None, None, None);
        };
        let mut total_kb = None;
        let mut available_kb = None;
        let mut swap_total_kb = None;
        let mut swap_free_kb = None;
        for line in contents.lines() {
            if let Some(rest) = line.strip_prefix("MemTotal:") {
                total_kb = parse_meminfo_kb(rest);
            } else if let Some(rest) = line.strip_prefix("MemAvailable:") {
                available_kb = parse_meminfo_kb(rest);
            } else if let Some(rest) = line.strip_prefix("SwapTotal:") {
                swap_total_kb = parse_meminfo_kb(rest);
            } else if let Some(rest) = line.strip_prefix("SwapFree:") {
                swap_free_kb = parse_meminfo_kb(rest);
            }
        }
        (
            total_kb.map(|kb| kb / 1024),
            available_kb.map(|kb| kb / 1024),
            swap_total_kb.map(|kb| kb / 1024),
            swap_free_kb.map(|kb| kb / 1024),
        )
    }
    #[cfg(not(target_os = "linux"))]
    {
        (None, None, None, None)
    }
}

#[cfg(target_os = "linux")]
fn parse_meminfo_kb(rest: &str) -> Option<u64> {
    rest.split_whitespace().next()?.parse().ok()
}

fn detect_load() -> (Option<f64>, Option<usize>) {
    #[cfg(target_os = "linux")]
    {
        let load = std::fs::read_to_string("/proc/loadavg")
            .ok()
            .and_then(|contents| contents.split_whitespace().next()?.parse::<f64>().ok());
        let cpus = std::thread::available_parallelism()
            .ok()
            .map(|value| value.get());
        (load, cpus)
    }
    #[cfg(not(target_os = "linux"))]
    {
        let cpus = std::thread::available_parallelism()
            .ok()
            .map(|value| value.get());
        (None, cpus)
    }
}

fn detect_battery() -> (Option<u8>, Option<String>) {
    #[cfg(target_os = "linux")]
    {
        let Ok(entries) = std::fs::read_dir("/sys/class/power_supply") else {
            return (None, None);
        };
        for entry in entries.flatten() {
            let path = entry.path();
            let name = entry.file_name().to_string_lossy().to_string();
            if !name.starts_with("BAT") {
                continue;
            }
            let percent = std::fs::read_to_string(path.join("capacity"))
                .ok()
                .and_then(|value| value.trim().parse::<u8>().ok());
            let status = std::fs::read_to_string(path.join("status"))
                .ok()
                .map(|value| value.trim().to_string());
            return (percent, status);
        }
        (None, None)
    }
    #[cfg(not(target_os = "linux"))]
    {
        (None, None)
    }
}

fn disk_available_gb(path: &Path) -> Option<f64> {
    #[cfg(unix)]
    {
        use std::os::unix::ffi::OsStrExt;
        let c_path = CString::new(path.as_os_str().as_bytes()).ok()?;
        let mut stat: libc::statvfs = unsafe { std::mem::zeroed() };
        let rc = unsafe { libc::statvfs(c_path.as_ptr(), &mut stat) };
        if rc != 0 {
            return None;
        }
        let bytes = stat.f_bavail as f64 * stat.f_frsize as f64;
        Some(bytes / 1024.0 / 1024.0 / 1024.0)
    }
    #[cfg(not(unix))]
    {
        let _ = path;
        None
    }
}

pub fn gather_git_snapshot(working_dir: Option<&Path>) -> GitSnapshot {
    let captured_at = Utc::now();
    let dir = working_dir.unwrap_or_else(|| Path::new("."));
    let branch = run_git(dir, &["branch", "--show-current"])
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty());
    match run_git(dir, &["status", "--short"]) {
        Ok(status) => {
            let dirty_summary: Vec<String> = status
                .lines()
                .filter(|line| !line.trim().is_empty())
                .take(20)
                .map(str::to_string)
                .collect();
            let dirty_count = status
                .lines()
                .filter(|line| !line.trim().is_empty())
                .count();
            GitSnapshot {
                captured_at,
                branch,
                dirty_count: Some(dirty_count),
                dirty_summary,
                error: None,
            }
        }
        Err(error) => GitSnapshot {
            captured_at,
            branch,
            dirty_count: None,
            dirty_summary: Vec::new(),
            error: Some(error),
        },
    }
}

fn run_git(dir: &Path, args: &[&str]) -> std::result::Result<String, String> {
    let output = std::process::Command::new("git")
        .args(args)
        .current_dir(dir)
        .output()
        .map_err(|err| format!("failed to run git {}: {}", args.join(" "), err))?;
    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).trim().to_string())
    }
}

pub fn overnight_root_dir() -> Result<PathBuf> {
    Ok(storage::jcode_dir()?.join("overnight"))
}

pub fn runs_dir() -> Result<PathBuf> {
    Ok(overnight_root_dir()?.join("runs"))
}

pub fn run_dir(run_id: &str) -> Result<PathBuf> {
    Ok(runs_dir()?.join(run_id))
}

pub fn manifest_path(run_id: &str) -> Result<PathBuf> {
    Ok(run_dir(run_id)?.join("manifest.json"))
}

pub fn save_manifest(manifest: &OvernightManifest) -> Result<()> {
    admission::save_manifest_at(&manifest_path(&manifest.run_id)?, manifest)
}

pub fn load_manifest(run_id: &str) -> Result<OvernightManifest> {
    admission::read_manifest_at(&manifest_path(run_id)?)
}

pub fn latest_manifest() -> Result<Option<OvernightManifest>> {
    let dir = runs_dir()?;
    if !dir.exists() {
        return Ok(None);
    }
    let mut manifests = Vec::new();
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        if !entry.file_type()?.is_dir() {
            continue;
        }
        let path = entry.path().join("manifest.json");
        if path.exists() {
            match admission::read_manifest_at(&path) {
                Ok(manifest) => manifests.push(manifest),
                Err(error) => crate::logging::warn(&format!(
                    "Skipping unreadable overnight manifest {}: {error}",
                    path.display()
                )),
            }
        }
    }
    manifests.sort_by_key(|manifest| manifest.started_at);
    Ok(manifests.pop())
}

/// Parseable manifests whose coordinator is `session_id`, newest first.
/// Unreadable manifests are skipped, like `latest_manifest`.
pub fn session_manifests_newest_first(session_id: &str) -> Result<Vec<OvernightManifest>> {
    let dir = runs_dir()?;
    if !dir.exists() {
        return Ok(Vec::new());
    }
    let mut manifests = Vec::new();
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        if !entry.file_type()?.is_dir() {
            continue;
        }
        let path = entry.path().join("manifest.json");
        if path.exists() {
            match admission::read_manifest_at(&path) {
                Ok(manifest) if manifest.coordinator_session_id == session_id => {
                    manifests.push(manifest)
                }
                Ok(_) => {}
                Err(error) => crate::logging::warn(&format!(
                    "Skipping unreadable overnight manifest {}: {error}",
                    path.display()
                )),
            }
        }
    }
    manifests.sort_by_key(|manifest| std::cmp::Reverse(manifest.started_at));
    Ok(manifests)
}

/// Whether the process that supervises this run still exists.
pub fn supervisor_is_alive(manifest: &OvernightManifest) -> bool {
    process_is_alive(manifest.process_id)
}

#[cfg(unix)]
fn process_is_alive(pid: u32) -> bool {
    // pid 0 and pids that wrap to negative `pid_t` (-1 means "every process")
    // would signal process groups, not a single supervisor.
    let Ok(pid) = libc::pid_t::try_from(pid) else {
        return false;
    };
    if pid == 0 {
        return false;
    }
    let result = unsafe { libc::kill(pid, 0) };
    result == 0 || std::io::Error::last_os_error().raw_os_error() == Some(libc::EPERM)
}

#[cfg(not(unix))]
fn process_is_alive(pid: u32) -> bool {
    pid != 0
}

/// An active (`Running`/`CancelRequested`) run whose supervisor is gone.
pub fn is_stale(manifest: &OvernightManifest) -> bool {
    stale_pid(manifest).is_some()
}

/// The dead supervisor pid of a stale run, if any.
pub fn stale_pid(manifest: &OvernightManifest) -> Option<u32> {
    (matches!(
        manifest.status,
        OvernightRunStatus::Running | OvernightRunStatus::CancelRequested
    ) && !supervisor_is_alive(manifest))
    .then_some(manifest.process_id)
}

/// Result of a cancel request. A run that already reached a terminal status
/// is rejected and left byte-for-byte unchanged.
#[derive(Debug, Clone)]
pub enum CancelOutcome {
    Requested(OvernightManifest),
    Rejected(OvernightManifest),
}

impl CancelOutcome {
    pub fn manifest(&self) -> &OvernightManifest {
        match self {
            Self::Requested(manifest) | Self::Rejected(manifest) => manifest,
        }
    }

    pub fn rejection_message(manifest: &OvernightManifest) -> String {
        format!(
            "Overnight run {} is already {}; unchanged.",
            manifest.run_id,
            manifest.status.label()
        )
    }
}

/// Record a cancel request, or reject it when the run is already terminal.
/// A repeated cancel of an unresolved supervised run re-runs settlement.
pub fn request_cancel(mut manifest: OvernightManifest) -> Result<CancelOutcome> {
    if manifest.status.is_terminal() {
        return Ok(CancelOutcome::Rejected(manifest));
    }
    manifest.status = OvernightRunStatus::CancelRequested;
    manifest.cancel_requested_at = Some(Utc::now());
    save_manifest(&manifest)?;
    record_event(
        &manifest,
        "cancel_requested",
        "User requested overnight cancellation".to_string(),
        json!({}),
        true,
    )?;
    render_review_html(&manifest)?;
    resettle_in_background(&manifest);
    Ok(CancelOutcome::Requested(manifest))
}

pub fn cancel_latest_run() -> Result<CancelOutcome> {
    request_cancel(latest_manifest()?.context("No overnight runs found")?)
}

pub fn read_events(manifest: &OvernightManifest) -> Result<Vec<OvernightEvent>> {
    if !manifest.events_path.exists() {
        return Ok(Vec::new());
    }
    let contents = std::fs::read_to_string(&manifest.events_path)?;
    Ok(contents
        .lines()
        .filter_map(|line| serde_json::from_str::<OvernightEvent>(line).ok())
        .collect())
}

pub fn read_task_cards(manifest: &OvernightManifest) -> Result<Vec<OvernightTaskCard>> {
    if !manifest.task_cards_dir.exists() {
        return Ok(Vec::new());
    }

    let mut paths = Vec::new();
    for entry in std::fs::read_dir(&manifest.task_cards_dir)? {
        let entry = entry?;
        let path = entry.path();
        if !entry.file_type()?.is_file() {
            continue;
        }
        let file_name = path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or_default();
        if file_name.starts_with('_')
            || path.extension().and_then(|ext| ext.to_str()) != Some("json")
        {
            continue;
        }
        paths.push(path);
    }
    paths.sort();

    let mut cards = Vec::new();
    for path in paths {
        let Ok(contents) = std::fs::read_to_string(&path) else {
            continue;
        };
        if let Ok(mut parsed) = serde_json::from_str::<Vec<OvernightTaskCard>>(&contents) {
            cards.append(&mut parsed);
        } else if let Ok(card) = serde_json::from_str::<OvernightTaskCard>(&contents) {
            cards.push(card);
        }
    }

    cards.retain(|card| !card.title.trim().is_empty() || !card.id.trim().is_empty());
    cards.sort_by(|a, b| {
        a.updated_at
            .cmp(&b.updated_at)
            .then_with(|| a.id.cmp(&b.id))
            .then_with(|| a.title.cmp(&b.title))
    });
    Ok(cards)
}

pub fn summarize_task_cards(manifest: &OvernightManifest) -> OvernightTaskCardSummary {
    summarize_task_cards_slice(&read_task_cards(manifest).unwrap_or_default())
}

pub fn format_progress_card_content(manifest: &OvernightManifest) -> Result<String> {
    Ok(serde_json::to_string(&build_progress_card(manifest))?)
}

pub fn latest_progress_card_content() -> Result<Option<String>> {
    latest_manifest()?
        .map(|manifest| format_progress_card_content(&manifest))
        .transpose()
}

pub fn build_progress_card(manifest: &OvernightManifest) -> OvernightProgressCard {
    let events = read_events(manifest).unwrap_or_default();
    let preflight = read_preflight(manifest);
    let task_cards = read_task_cards(manifest).unwrap_or_default();
    let mut card = build_progress_card_from_parts(
        manifest,
        &events,
        preflight.as_ref(),
        &task_cards,
        Utc::now(),
    );
    card.stale_pid = stale_pid(manifest);
    card
}

fn read_preflight(manifest: &OvernightManifest) -> Option<OvernightPreflight> {
    if !manifest.preflight_path.exists() {
        return None;
    }
    storage::read_json(&manifest.preflight_path).ok()
}

pub fn record_event(
    manifest: &OvernightManifest,
    kind: &str,
    summary: String,
    details: Value,
    meaningful: bool,
) -> Result<()> {
    let event = OvernightEvent {
        timestamp: Utc::now(),
        run_id: manifest.run_id.clone(),
        session_id: Some(manifest.coordinator_session_id.clone()),
        kind: kind.to_string(),
        summary: summary.clone(),
        details,
        meaningful,
    };

    if let Some(parent) = manifest.events_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let mut events = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&manifest.events_path)?;
    writeln!(events, "{}", serde_json::to_string(&event)?)?;

    if let Some(parent) = manifest.human_log_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let mut human = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&manifest.human_log_path)?;
    writeln!(
        human,
        "{} [{}] {}",
        event.timestamp.format("%H:%M:%S"),
        event.kind,
        summary
    )?;

    if meaningful {
        let mut updated = load_manifest(&manifest.run_id).unwrap_or_else(|_| manifest.clone());
        updated.last_activity_at = event.timestamp;
        let _ = save_manifest(&updated);
    }

    Ok(())
}

async fn mark_completed(
    manifest: &OvernightManifest,
    status: OvernightRunStatus,
    summary: &str,
) -> Result<()> {
    let mut updated = load_manifest(&manifest.run_id).unwrap_or_else(|_| manifest.clone());
    updated.status = status;
    updated.completed_at = Some(Utc::now());
    updated.last_activity_at = Utc::now();
    save_manifest(&updated)?;
    record_event(
        &updated,
        "run_completed",
        summary.to_string(),
        json!({ "status": updated.status.label() }),
        true,
    )?;
    usage::finalize(&updated).await;
    render_review_html(&updated)?;
    Ok(())
}

pub fn format_status_markdown(manifest: &OvernightManifest) -> String {
    let task_summary = summarize_task_cards(manifest);
    format_status_markdown_with_stale(manifest, &task_summary, Utc::now(), stale_pid(manifest))
}

pub fn format_log_markdown(manifest: &OvernightManifest, max_lines: usize) -> String {
    let events = read_events(manifest).unwrap_or_default();
    format_log_markdown_from_events(manifest, &events, max_lines)
}

fn write_initial_review_notes(manifest: &OvernightManifest) -> Result<()> {
    if manifest.review_notes_path.exists() {
        return Ok(());
    }
    let content = format!(
        "# Overnight review notes\n\nRun: `{}`\nCoordinator session: `{}`\nTarget wake time: `{}`\n\nThe coordinator must keep this file useful as the run progresses. Required sections for each meaningful task:\n\n## Executive summary\n\n- Status: running\n- Current task: not started\n- Verified fixes: 0\n- Issue drafts/posts: 0\n- Repo risk: unknown\n\n## Task reviews\n\nFor each task, include:\n\n### Task: <title>\n\n- Source: user request / GH issue / static analysis / failing test / code quality\n- Why chosen:\n- Verifiability:\n- Risk:\n- Outcome:\n\n#### Before\n\n- Observed behavior or code state:\n- Reproduction/evidence:\n\n#### After\n\n- Changed behavior or code state:\n- Validation run:\n- Files changed:\n\n## Decisions and skipped work\n\nRecord tasks considered but skipped, with reasons.\n\n## Open questions and next steps\n\nRecord user decisions needed and safe continuation options.\n",
        manifest.run_id,
        manifest.coordinator_session_id,
        manifest.target_wake_at.to_rfc3339(),
    );
    write_text_file(&manifest.review_notes_path, &content)
}

fn write_task_card_schema(manifest: &OvernightManifest) -> Result<()> {
    let schema_path = manifest.task_cards_dir.join("task-card-schema.md");
    if schema_path.exists() {
        return Ok(());
    }
    let content = r#"# Overnight task-card schema

Create one `*.json` file per meaningful task. Keep it current while you work. The generated review page and TUI progress card read these files continuously.

Required spirit: make the morning review objectively useful. Each completed or important attempted task should show why it was selected, what was true before, what changed after, and exactly how it was validated.

```json
{
  "id": "task-001",
  "title": "Fix deterministic provider reload timeout",
  "status": "active | completed | blocked | deferred | failed | skipped",
  "priority": "high | medium | low",
  "source": "GH issue | failing test | static analysis | code quality | user request",
  "why_selected": "Objective, bounded, high-confidence reason for choosing this task.",
  "verifiability": "How we can prove the problem and prove the fix.",
  "risk": "low | medium | high",
  "outcome": "Concise final outcome or current state.",
  "before": {
    "problem": "Observed bug/code state before work.",
    "evidence": ["validation/task-001-before.txt"]
  },
  "after": {
    "change": "Changed behavior or code state after work.",
    "files_changed": ["src/example.rs"],
    "evidence": ["validation/task-001-after.txt"]
  },
  "validation": {
    "commands": ["cargo test provider_reload"],
    "result": "passed",
    "evidence": ["validation/task-001-test.txt"]
  },
  "followups": ["Optional remaining safe next step"],
  "updated_at": "2026-05-01T08:00:00Z"
}
```

Notes:
- Use `active` for the task currently being worked.
- Use `completed` only when validation evidence exists or the task is intentionally documentation/issue-draft only.
- Use `blocked` when user input, credentials, external access, or taste is required.
- Use `deferred` or `skipped` when a task was considered but not pursued.
"#;
    write_text_file(&schema_path, content)
}

pub fn render_review_html(manifest: &OvernightManifest) -> Result<()> {
    let events = read_events(manifest).unwrap_or_default();
    let notes = std::fs::read_to_string(&manifest.review_notes_path).unwrap_or_else(|_| {
        "# Overnight review notes

Coordinator has not written notes yet."
            .to_string()
    });
    let preflight = if manifest.preflight_path.exists() {
        std::fs::read_to_string(&manifest.preflight_path).unwrap_or_default()
    } else {
        String::new()
    };
    let task_cards = read_task_cards(manifest).unwrap_or_default();
    let usage = read_usage_reconciled(manifest);
    let html = jcode_overnight_core::build_review_html_with_usage(
        manifest,
        &events,
        &notes,
        &preflight,
        &task_cards,
        usage.as_ref(),
    );
    write_text_file(&manifest.review_path, &html)
}

fn write_text_file(path: &Path, content: &str) -> Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(path, content)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    pub(in crate::overnight) fn test_manifest(root: &Path, run_id: &str) -> OvernightManifest {
        let run_dir = root.join("run");
        let now = Utc::now();
        OvernightManifest {
            task_attempt: None,
            task_admission: None,
            version: OVERNIGHT_VERSION,
            run_id: run_id.to_string(),
            parent_session_id: "parent".to_string(),
            coordinator_session_id: "coord".to_string(),
            coordinator_session_name: "coordinator".to_string(),
            started_at: now,
            target_wake_at: now + ChronoDuration::hours(7),
            handoff_ready_at: now + ChronoDuration::hours(6),
            post_wake_grace_until: now + ChronoDuration::hours(9),
            morning_report_posted_at: None,
            completed_at: None,
            cancel_requested_at: None,
            status: OvernightRunStatus::Running,
            mission: Some("verify things".to_string()),
            working_dir: Some("/tmp/project".to_string()),
            provider_name: "test-provider".to_string(),
            model: "test-model".to_string(),
            max_agents_guidance: 2,
            limits: OvernightLimits::default(),
            stop_settlement: None,
            limit_breach: None,
            process_id: 123,
            supervisor_start_time: None,
            heartbeat_at: None,
            run_dir: run_dir.clone(),
            events_path: run_dir.join("events.jsonl"),
            human_log_path: run_dir.join("run.log"),
            review_path: run_dir.join("review.html"),
            review_notes_path: run_dir.join("review-notes.md"),
            preflight_path: run_dir.join("preflight.json"),
            task_cards_dir: run_dir.join("task-cards"),
            issue_drafts_dir: run_dir.join("issue-drafts"),
            validation_dir: run_dir.join("validation"),
            last_activity_at: now,
        }
    }

    #[test]
    fn task_admission_native_manifest_revalidates_cancel_owner_and_policy() {
        let temp = tempfile::tempdir().unwrap();
        let mut manifest = test_manifest(temp.path(), "admission");
        manifest.process_id = std::process::id();
        manifest.working_dir = Some(temp.path().to_string_lossy().into());
        let mut policy = crate::config::TasksConfig {
            unattended: true,
            ..Default::default()
        };
        let snapshot = crate::config::TaskAdmissionSnapshot {
            scope: "bounded task".into(),
            authorization: Some(crate::config::TaskAuthorization {
                scope: "bounded task".into(),
                max_attempts: 2,
                max_runtime_seconds: 100,
                max_spend_microusd: 100,
            }),
            ownership_verified: true,
            dependencies_ready: true,
            workspace_ready: true,
            ..Default::default()
        };
        manifest.task_admission = Some(snapshot);
        assert!(
            ensure_task_dispatch(&manifest, &policy)
                .unwrap_err()
                .to_string()
                .contains("spend enforcer")
        );
        assert_eq!(
            revalidate_task_admission(&manifest, &policy).unwrap().state,
            crate::config::AdmissionState::Ready
        );
        policy.max_attempts = 100;
        assert_eq!(
            revalidate_task_admission(&manifest, &policy)
                .unwrap()
                .remaining_attempts,
            2
        );
        manifest.cancel_requested_at = Some(Utc::now());
        assert_eq!(
            revalidate_task_admission(&manifest, &policy).unwrap().state,
            crate::config::AdmissionState::Blocked
        );
        manifest.cancel_requested_at = None;
        manifest.process_id = u32::MAX;
        assert_eq!(
            revalidate_task_admission(&manifest, &policy).unwrap().state,
            crate::config::AdmissionState::NeedsHuman
        );
        manifest.process_id = std::process::id();
        manifest.task_admission.as_mut().unwrap().live_attempt = Some("other-owner".into());
        assert_eq!(
            revalidate_task_admission(&manifest, &policy).unwrap().state,
            crate::config::AdmissionState::Waiting
        );
        manifest.task_admission = None;
        assert!(revalidate_task_admission(&manifest, &policy).is_none());
    }

    #[test]
    fn task_admission_legacy_manifest_has_no_implicit_authority() {
        let temp = tempfile::tempdir().unwrap();
        let manifest = test_manifest(temp.path(), "legacy");
        let mut json = serde_json::to_value(&manifest).unwrap();
        json.as_object_mut().unwrap().remove("task_admission");
        let loaded: OvernightManifest = serde_json::from_value(json).unwrap();
        assert!(loaded.task_admission.is_none());
    }

    #[test]
    fn parse_duration_accepts_hours_minutes_and_decimals() {
        assert_eq!(parse_duration("7").unwrap().minutes, 420);
        assert_eq!(parse_duration("7h").unwrap().minutes, 420);
        assert_eq!(parse_duration("90m").unwrap().minutes, 90);
        assert_eq!(parse_duration("1.5").unwrap().minutes, 90);
    }

    #[test]
    fn parse_overnight_command_start_with_mission() {
        let parsed = parse_overnight_command("/overnight 7 fix verified bugs")
            .unwrap()
            .unwrap();
        match parsed {
            OvernightCommand::Start {
                duration, mission, ..
            } => {
                assert_eq!(duration.minutes, 420);
                assert_eq!(mission.as_deref(), Some("fix verified bugs"));
            }
            other => panic!("unexpected command: {:?}", other),
        }
    }

    #[test]
    fn parse_overnight_command_subcommands() {
        assert_eq!(
            parse_overnight_command("/overnight status")
                .unwrap()
                .unwrap(),
            OvernightCommand::Status
        );
        assert_eq!(
            parse_overnight_command("/overnight log").unwrap().unwrap(),
            OvernightCommand::Log
        );
        assert_eq!(
            parse_overnight_command("/overnight review")
                .unwrap()
                .unwrap(),
            OvernightCommand::Review
        );
        assert_eq!(
            parse_overnight_command("/overnight cancel")
                .unwrap()
                .unwrap(),
            OvernightCommand::Cancel
        );
    }

    #[test]
    fn html_escape_escapes_basic_entities() {
        assert_eq!(html_escape("<a&b>\"'"), "&lt;a&amp;b&gt;&quot;&#39;");
    }

    #[test]
    fn render_review_html_writes_required_sections() {
        let temp = tempfile::tempdir().expect("tempdir");
        let manifest = test_manifest(temp.path(), "overnight_test");
        write_initial_review_notes(&manifest).expect("write notes");
        render_review_html(&manifest).expect("render review");

        let html = std::fs::read_to_string(&manifest.review_path).expect("read review html");
        assert!(html.contains("Executive summary"));
        assert!(html.contains("Coordinator review notes"));
        assert!(html.contains("Timeline"));
        assert!(html.contains("Artifacts"));
        assert!(html.contains("Before"));
        assert!(html.contains("After"));
    }

    #[test]
    fn task_card_summary_reads_structured_json_cards() {
        let temp = tempfile::tempdir().expect("tempdir");
        let manifest = test_manifest(temp.path(), "overnight_cards");
        std::fs::create_dir_all(&manifest.task_cards_dir).expect("task card dir");
        std::fs::write(
            manifest.task_cards_dir.join("task-001.json"),
            r#"{
              "id": "task-001",
              "title": "Fix deterministic bug",
              "status": "completed",
              "risk": "low",
              "validation": { "commands": ["cargo test bug"], "result": "passed" },
              "updated_at": "2026-05-01T08:00:00Z"
            }"#,
        )
        .expect("write completed card");
        std::fs::write(
            manifest.task_cards_dir.join("task-002.json"),
            r#"{
              "id": "task-002",
              "title": "Investigate static-analysis finding",
              "status": "active",
              "risk": "high",
              "updated_at": "2026-05-01T08:10:00Z"
            }"#,
        )
        .expect("write active card");

        let cards = read_task_cards(&manifest).expect("read cards");
        assert_eq!(cards.len(), 2);
        let summary = summarize_task_cards_slice(&cards);
        assert_eq!(summary.total, 2);
        assert_eq!(summary.counts.completed, 1);
        assert_eq!(summary.counts.active, 1);
        assert_eq!(summary.validated, 1);
        assert_eq!(summary.high_risk, 1);
        assert_eq!(
            summary.latest_title.as_deref(),
            Some("Investigate static-analysis finding")
        );
    }

    #[test]
    fn progress_card_content_includes_task_summary_and_latest_event() {
        let temp = tempfile::tempdir().expect("tempdir");
        let manifest = test_manifest(temp.path(), "overnight_progress");
        std::fs::create_dir_all(&manifest.task_cards_dir).expect("task card dir");
        std::fs::create_dir_all(manifest.events_path.parent().unwrap()).expect("events dir");
        std::fs::write(
            manifest.task_cards_dir.join("task-001.json"),
            r#"{
              "id": "task-001",
              "title": "Verify reload race",
              "status": "completed",
              "validation": { "result": "passed" },
              "updated_at": "2026-05-01T08:00:00Z"
            }"#,
        )
        .expect("write card");
        let event = OvernightEvent {
            timestamp: Utc::now(),
            run_id: manifest.run_id.clone(),
            session_id: Some(manifest.coordinator_session_id.clone()),
            kind: "coordinator_turn_completed".to_string(),
            summary: "Coordinator turn completed".to_string(),
            details: json!({}),
            meaningful: true,
        };
        std::fs::write(
            &manifest.events_path,
            format!("{}\n", serde_json::to_string(&event).unwrap()),
        )
        .expect("write event");

        let card: OvernightProgressCard =
            serde_json::from_str(&format_progress_card_content(&manifest).expect("progress card"))
                .expect("parse card");
        assert_eq!(card.task_summary.counts.completed, 1);
        assert_eq!(card.task_summary.validated, 1);
        assert_eq!(
            card.latest_event_kind.as_deref(),
            Some("coordinator_turn_completed")
        );
        assert_eq!(
            card.active_task_title.as_deref(),
            Some("Verify reload race")
        );
    }

    fn write_run(
        home: &Path,
        run_id: &str,
        started_offset_min: i64,
        status: OvernightRunStatus,
    ) -> PathBuf {
        let mut manifest = test_manifest(home, run_id);
        manifest.started_at = Utc::now() + ChronoDuration::minutes(started_offset_min);
        manifest.status = status;
        let dir = home.join("overnight/runs").join(run_id);
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("manifest.json");
        std::fs::write(&path, serde_json::to_vec(&manifest).unwrap()).unwrap();
        path
    }

    fn with_home<T>(home: &Path, f: impl FnOnce() -> T) -> T {
        let _env = storage::lock_test_env();
        let original = std::env::var_os("JCODE_HOME");
        crate::env::set_var("JCODE_HOME", home);
        let out = f();
        match original {
            Some(value) => crate::env::set_var("JCODE_HOME", value),
            None => crate::env::remove_var("JCODE_HOME"),
        }
        out
    }

    #[test]
    fn latest_manifest_skips_corrupt_entries() {
        let temp = tempfile::tempdir().unwrap();
        write_run(temp.path(), "old", -30, OvernightRunStatus::Completed);
        let corrupt_dir = temp.path().join("overnight/runs/corrupt");
        std::fs::create_dir_all(&corrupt_dir).unwrap();
        std::fs::write(corrupt_dir.join("manifest.json"), b"{not json").unwrap();
        write_run(temp.path(), "newest", -10, OvernightRunStatus::Blocked);
        with_home(temp.path(), || {
            let latest = latest_manifest().unwrap().unwrap();
            assert_eq!(latest.run_id, "newest");
            assert!(load_manifest("corrupt").is_err());
        });
    }

    #[test]
    fn unknown_status_manifest_is_terminal_for_cancel() {
        let temp = tempfile::tempdir().unwrap();
        let path = write_run(temp.path(), "legacy", -10, OvernightRunStatus::Running);
        let text = std::fs::read_to_string(&path)
            .unwrap()
            .replace("\"status\":\"running\"", "\"status\":\"mystery\"");
        std::fs::write(&path, &text).unwrap();
        with_home(temp.path(), || {
            let outcome = cancel_latest_run().unwrap();
            assert!(matches!(outcome, CancelOutcome::Rejected(_)));
            let cancelled = outcome.manifest();
            assert_eq!(cancelled.status, OvernightRunStatus::Unknown);
            assert!(cancelled.cancel_requested_at.is_none());
            assert_eq!(std::fs::read_to_string(&path).unwrap(), text);
        });
    }

    #[test]
    fn cancelling_a_finished_run_is_rejected_and_leaves_the_manifest_unchanged() {
        for status in [
            OvernightRunStatus::Completed,
            OvernightRunStatus::Failed,
            OvernightRunStatus::Cancelled,
        ] {
            let temp = tempfile::tempdir().unwrap();
            let path = write_run(temp.path(), "finished", -10, status.clone());
            let before = std::fs::read(&path).unwrap();
            with_home(temp.path(), || {
                let outcome = cancel_latest_run().unwrap();
                let CancelOutcome::Rejected(manifest) = outcome else {
                    panic!("{status:?} must reject cancel");
                };
                assert_eq!(manifest.status, status);
                assert!(
                    CancelOutcome::rejection_message(&manifest)
                        .contains(&format!("already {}; unchanged", status.label()))
                );
            });
            assert_eq!(std::fs::read(&path).unwrap(), before, "{status:?}");
            assert!(
                !path.with_file_name("events.jsonl").exists(),
                "no cancel event for {status:?}"
            );
        }
    }

    #[test]
    fn old_manifest_without_limit_fields_loads_with_no_limits() {
        let temp = tempfile::tempdir().unwrap();
        let manifest = test_manifest(temp.path(), "old");
        let mut value = serde_json::to_value(&manifest).unwrap();
        let object = value.as_object_mut().unwrap();
        for key in ["limits", "stop_settlement", "limit_breach"] {
            object.remove(key);
        }
        let old: OvernightManifest = serde_json::from_value(value).unwrap();
        assert!(old.limits.is_unset());
        assert!(old.stop_settlement.is_none() && old.limit_breach.is_none());
    }

    #[test]
    fn cancelled_manifest_round_trips() {
        let temp = tempfile::tempdir().unwrap();
        let mut manifest = test_manifest(temp.path(), "cancelled-rt");
        manifest.status = OvernightRunStatus::Cancelled;
        manifest.stop_settlement = Some(StopSettlement::Settled { at: Utc::now() });
        let json = serde_json::to_string(&manifest).unwrap();
        assert!(json.contains("\"status\":\"cancelled\""), "{json}");
        let back: OvernightManifest = serde_json::from_str(&json).unwrap();
        assert_eq!(back.status, OvernightRunStatus::Cancelled);
        assert!(back.status.is_terminal());
        assert!(back.stop_settlement.unwrap().is_settled());
    }

    #[test]
    fn blocked_and_unknown_fail_admission_revalidation() {
        let temp = tempfile::tempdir().unwrap();
        for status in [OvernightRunStatus::Blocked, OvernightRunStatus::Unknown] {
            let mut manifest = test_manifest(temp.path(), "blocked");
            manifest.status = status;
            manifest.task_admission = Some(Default::default());
            let policy = crate::config::TasksConfig {
                unattended: true,
                ..Default::default()
            };
            let decision = revalidate_task_admission(&manifest, &policy).unwrap();
            assert_eq!(decision.state, crate::config::AdmissionState::Blocked);
            assert!(
                decision
                    .reasons
                    .iter()
                    .any(|reason| reason.contains("cancelled")),
                "{:?}",
                decision.reasons
            );
        }
    }

    #[test]
    fn stale_detection_uses_process_liveness_and_status() {
        let temp = tempfile::tempdir().unwrap();
        let mut manifest = test_manifest(temp.path(), "liveness");
        manifest.process_id = u32::MAX;
        assert!(!supervisor_is_alive(&manifest));
        assert!(is_stale(&manifest));
        assert_eq!(stale_pid(&manifest), Some(u32::MAX));
        manifest.status = OvernightRunStatus::CancelRequested;
        assert!(is_stale(&manifest));
        manifest.status = OvernightRunStatus::Failed;
        assert!(!is_stale(&manifest));
        manifest.status = OvernightRunStatus::Running;
        manifest.process_id = std::process::id();
        assert!(supervisor_is_alive(&manifest));
        assert!(!is_stale(&manifest));
        manifest.process_id = 0;
        assert!(is_stale(&manifest));
    }

    #[test]
    fn status_markdown_labels_stale_without_rewriting_manifest() {
        let temp = tempfile::tempdir().unwrap();
        let path = write_run(temp.path(), "dead_pid_md", -10, OvernightRunStatus::Running);
        let mut manifest: OvernightManifest =
            serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
        manifest.process_id = u32::MAX;
        std::fs::write(&path, serde_json::to_vec(&manifest).unwrap()).unwrap();
        let before = std::fs::read(&path).unwrap();
        let text = format_status_markdown(&manifest);
        assert!(
            text.contains("running (stale, supervisor pid 4294967295 gone)"),
            "{text}"
        );
        assert_eq!(build_progress_card(&manifest).stale_pid, Some(u32::MAX));
        assert_eq!(std::fs::read(&path).unwrap(), before);

        manifest.process_id = std::process::id();
        let live = format_status_markdown(&manifest);
        assert!(!live.contains("stale"), "{live}");
        assert_eq!(build_progress_card(&manifest).stale_pid, None);
    }

    #[test]
    fn render_review_html_includes_structured_task_cards() {
        let temp = tempfile::tempdir().expect("tempdir");
        let manifest = test_manifest(temp.path(), "overnight_review_cards");
        write_initial_review_notes(&manifest).expect("write notes");
        std::fs::create_dir_all(&manifest.task_cards_dir).expect("task card dir");
        std::fs::write(
            manifest.task_cards_dir.join("task-001.json"),
            r#"{
              "id": "task-001",
              "title": "Fix deterministic bug",
              "status": "completed",
              "why_selected": "Reproducible failure",
              "before": { "problem": "Test failed before the fix" },
              "after": { "change": "Test passes after the fix", "files_changed": ["src/example.rs"] },
              "validation": { "commands": ["cargo test deterministic_bug"], "result": "passed" },
              "updated_at": "2026-05-01T08:00:00Z"
            }"#,
        )
        .expect("write card");

        render_review_html(&manifest).expect("render review");
        let html = std::fs::read_to_string(&manifest.review_path).expect("read html");
        assert!(html.contains("Structured task cards"));
        assert!(html.contains("Fix deterministic bug"));
        assert!(html.contains("Reproducible failure"));
        assert!(html.contains("cargo test deterministic_bug"));
    }
}

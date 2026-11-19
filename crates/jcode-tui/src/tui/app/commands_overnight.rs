use super::{
    App, DisplayMessage, OvernightAutoPokeFingerprint, OvernightAutoPokeState, ProcessingStatus,
};
use crate::message::{ContentBlock, Message, Role};
use crate::overnight::{OvernightCommand, OvernightRunStatus, OvernightStartOptions};
use crate::provider::Provider;
use chrono::Utc;
use std::sync::Arc;
use std::time::{Duration, Instant};

const OVERNIGHT_STALL_LIMIT: u8 = 3;
const OVERNIGHT_ERROR_LIMIT: u8 = 2;
const OVERNIGHT_MAX_POKES: u16 = 48;

pub(super) fn handle_overnight_command(app: &mut App, trimmed: &str) -> bool {
    let Some(command) = crate::overnight::parse_overnight_command(trimmed) else {
        return false;
    };

    match command {
        Ok(OvernightCommand::Help) => show_overnight_help(app),
        Ok(OvernightCommand::Status) => show_overnight_status(app),
        Ok(OvernightCommand::Log) => show_overnight_log(app),
        Ok(OvernightCommand::Review) => open_overnight_review(app),
        Ok(OvernightCommand::Cancel) => cancel_overnight(app),
        Ok(OvernightCommand::Start {
            duration,
            mission,
            limits,
        }) => {
            let working_dir = app
                .session
                .working_dir
                .as_deref()
                .map(std::path::PathBuf::from)
                .filter(|path| path.is_dir())
                .or_else(|| std::env::current_dir().ok());
            if refuse_duplicate_overnight_start(app) {
                return true;
            }
            let provider = overnight_provider_for_app(app);
            let visible_provider = provider.clone();
            let options = OvernightStartOptions {
                duration,
                mission,
                parent_session: app.session.clone(),
                provider,
                registry: app.registry.clone(),
                working_dir,
                use_current_session: true,
                limits,
            };
            match crate::overnight::start_overnight_run(options) {
                Ok(launch) => {
                    let manifest = launch.manifest;
                    app.enable_overnight_auto_poke(&manifest);
                    app.upsert_overnight_display_card(&manifest);
                    if let Some(prompt) = launch.initial_prompt {
                        if !app.is_remote {
                            app.provider = visible_provider;
                        }
                        start_visible_overnight_turn(app, prompt);
                        app.set_status_notice("Overnight started in current session");
                    } else {
                        app.set_status_notice("Overnight started");
                    }
                }
                Err(error) => app.push_display_message(DisplayMessage::error(format!(
                    "Failed to start overnight run: {}",
                    crate::util::format_error_chain(&error)
                ))),
            }
        }
        Err(error) => app.push_display_message(DisplayMessage::error(error)),
    }

    true
}

/// Returns true (after showing an error) when this session already has a live
/// active run. A stale active run only produces a notice.
fn refuse_duplicate_overnight_start(app: &mut App) -> bool {
    let Ok(manifests) = crate::overnight::session_manifests_newest_first(&app.session.id) else {
        return false;
    };
    let mut active = manifests.into_iter().filter(|manifest| {
        matches!(
            manifest.status,
            OvernightRunStatus::Running | OvernightRunStatus::CancelRequested
        )
    });
    let mut newest_stale = None;
    for existing in active.by_ref() {
        if crate::overnight::supervisor_is_alive(&existing) {
            app.push_display_message(DisplayMessage::error(format!(
                "This session already has an active overnight run {}. Inspect status or cancel it before starting another.",
                existing.run_id
            )));
            return true;
        }
        newest_stale.get_or_insert(existing);
    }
    if let Some(existing) = newest_stale {
        app.push_display_message(DisplayMessage::system(format!(
            "Previous overnight run {} is stale (supervisor pid {} gone). Starting a new run.",
            existing.run_id, existing.process_id
        )));
    }
    false
}

fn start_visible_overnight_turn(app: &mut App, content: String) {
    if app.is_remote {
        app.commit_pending_streaming_assistant_message();
        app.queued_messages.push(content);
        app.set_status_notice("Overnight queued in current remote session");
        return;
    }

    app.commit_pending_streaming_assistant_message();
    app.add_provider_message(Message::user(&content));
    app.session.add_message(
        Role::User,
        vec![ContentBlock::Text {
            text: content,
            cache_control: None,
        }],
    );
    let _ = app.session.save();

    app.is_processing = true;
    app.status = ProcessingStatus::Sending;
    app.clear_streaming_render_state();
    app.stream_buffer.clear();
    app.thought_line_inserted = false;
    app.thinking_prefix_emitted = false;
    app.thinking_buffer.clear();
    app.streaming_tool_calls.clear();
    app.batch_progress = None;
    app.streaming.streaming_input_tokens = 0;
    app.streaming.streaming_output_tokens = 0;
    app.streaming.streaming_cache_read_tokens = None;
    app.streaming.streaming_cache_creation_tokens = None;
    app.kv_cache.current_api_usage_recorded = false;
    app.upstream_provider = None;
    app.status_detail = None;
    app.streaming.streaming_tps_start = None;
    app.streaming.streaming_tps_elapsed = Duration::ZERO;
    app.streaming.streaming_tps_collect_output = false;
    app.streaming.streaming_total_output_tokens = 0;
    app.streaming.streaming_tps_observed_output_tokens = 0;
    app.streaming.streaming_tps_observed_elapsed = Duration::ZERO;
    app.processing_started = Some(Instant::now());
    app.visible_turn_started = Some(Instant::now());
    app.pending_turn = true;
}

fn show_overnight_help(app: &mut App) {
    app.push_display_message(DisplayMessage::system(
        "/overnight <hours>[h|m] [--max-agents N] [--max-turn-minutes N] [--max-tokens N] [mission]\n  Start one visible overnight coordinator with guarded auto-poke follow-ups until the target wake/wrap time. The coordinator prioritizes verifiable, low-risk work, maintains logs, and updates a review HTML page. --max-agents caps live workers; --max-turn-minutes and --max-tokens need a supervised run (server /overnight) and are rejected here.\n\n/overnight status\n  Show the latest overnight run status.\n\n/overnight log\n  Show recent overnight events.\n\n/overnight review\n  Open the generated review page.\n\n/overnight cancel\n  Stop overnight auto-poke. A supervised run also refuses new workers, interrupts its running turn and drains owned work, and is marked cancelled only once that is confirmed. A visible run drains nothing and stays 'cancel requested'. A finished run is left unchanged.".to_string(),
    ));
}

fn overnight_provider_for_app(app: &mut App) -> Arc<dyn Provider> {
    if !app.is_remote {
        return app.provider.fork();
    }

    // Remote-attached TUIs intentionally use NullProvider because normal turns
    // execute in the remote backend process. `/overnight` is supervised by the
    // launching TUI process, so it needs a real local provider instead of the
    // remote placeholder. Restore the displayed session model when possible and
    // otherwise fall back to the local default provider.
    let provider: Arc<dyn Provider> = Arc::new(crate::provider::MultiProvider::new_fast());
    if let Some(model) = app
        .session
        .model
        .as_deref()
        .map(str::trim)
        .filter(|model| !model.is_empty() && *model != "unknown")
        && let Err(error) = provider.set_model(model)
    {
        app.push_display_message(DisplayMessage::system(format!(
            "Overnight could not restore remote model {} locally: {}. Using local default provider {} instead.",
            model,
            error,
            provider.name()
        )));
    }
    provider
}

fn show_overnight_status(app: &mut App) {
    match crate::overnight::latest_manifest() {
        Ok(Some(manifest)) => {
            if !app.upsert_overnight_display_card(&manifest) {
                app.push_display_message(DisplayMessage::system(
                    crate::overnight::format_status_markdown(&manifest),
                ));
            }
            app.set_status_notice("Overnight status");
        }
        Ok(None) => app.push_display_message(DisplayMessage::system(
            "No overnight runs found.".to_string(),
        )),
        Err(error) => app.push_display_message(DisplayMessage::error(format!(
            "Failed to read overnight status: {}",
            crate::util::format_error_chain(&error)
        ))),
    }
}

fn show_overnight_log(app: &mut App) {
    match crate::overnight::latest_manifest() {
        Ok(Some(manifest)) => {
            app.push_display_message(DisplayMessage::system(
                crate::overnight::format_log_markdown(&manifest, 30),
            ));
            app.set_status_notice("Overnight log");
        }
        Ok(None) => app.push_display_message(DisplayMessage::system(
            "No overnight runs found.".to_string(),
        )),
        Err(error) => app.push_display_message(DisplayMessage::error(format!(
            "Failed to read overnight log: {}",
            crate::util::format_error_chain(&error)
        ))),
    }
}

fn open_overnight_review(app: &mut App) {
    match crate::overnight::latest_manifest() {
        Ok(Some(manifest)) => {
            if let Err(error) = crate::overnight::render_review_html(&manifest) {
                app.push_display_message(DisplayMessage::error(format!(
                    "Failed to refresh overnight review page: {}",
                    crate::util::format_error_chain(&error)
                )));
                return;
            }
            match super::helpers::open_path_or_url_detached(&manifest.review_path) {
                Ok(()) => {
                    app.push_display_message(DisplayMessage::system(format!(
                        "Opened overnight review page: {}",
                        manifest.review_path.display()
                    )));
                    app.set_status_notice("Overnight review opened");
                }
                Err(error) => app.push_display_message(DisplayMessage::error(format!(
                    "Failed to open overnight review page {}: {}",
                    manifest.review_path.display(),
                    error
                ))),
            }
        }
        Ok(None) => app.push_display_message(DisplayMessage::system(
            "No overnight runs found.".to_string(),
        )),
        Err(error) => app.push_display_message(DisplayMessage::error(format!(
            "Failed to read overnight review: {}",
            crate::util::format_error_chain(&error)
        ))),
    }
}

fn cancel_overnight(app: &mut App) {
    match crate::overnight::cancel_latest_run() {
        Ok(crate::overnight::CancelOutcome::Rejected(manifest)) => {
            app.push_display_message(DisplayMessage::system(
                crate::overnight::CancelOutcome::rejection_message(&manifest),
            ));
            app.set_status_notice("Overnight run already finished");
        }
        Ok(crate::overnight::CancelOutcome::Requested(manifest)) => {
            app.finalize_cancelled_overnight_auto_poke(&manifest.run_id);
            if !app.upsert_overnight_display_card(&manifest) {
                app.push_display_message(DisplayMessage::system(cancel_requested_text(&manifest)));
            }
            app.set_status_notice("Overnight cancel requested");
        }
        Err(error) => app.push_display_message(DisplayMessage::error(format!(
            "Failed to cancel overnight run: {}",
            crate::util::format_error_chain(&error)
        ))),
    }
}

/// Honest cancel wording: only a supervised run interrupts its turn and drains
/// owned work. A visible current-session run has no supervisor.
fn cancel_requested_text(manifest: &crate::overnight::OvernightManifest) -> String {
    if crate::overnight::is_supervised(manifest) {
        format!(
            "Cancellation requested for overnight run {}. The running turn is being interrupted and owned work drained; the run is marked cancelled only once that is confirmed.",
            manifest.run_id
        )
    } else {
        format!(
            "Cancellation requested for visible overnight run {}. Auto-poke follow-ups stop, but this session has no supervisor: nothing is interrupted or drained, so interrupt the current turn and stop workers yourself. The run stays 'cancel requested' (not marked cancelled) and its worker cap no longer applies.",
            manifest.run_id
        )
    }
}

impl App {
    pub(super) fn cancel_overnight_for_interrupt(&mut self) -> bool {
        if self.overnight_auto_poke.is_none()
            && !self
                .queued_messages
                .iter()
                .any(|message| is_overnight_auto_poke_message(message))
        {
            return false;
        }

        let before = self.queued_messages.len();
        self.queued_messages
            .retain(|message| !is_overnight_auto_poke_message(message));
        if before != self.queued_messages.len() && !self.has_queued_followups() {
            self.pending_queued_dispatch = false;
        }

        match crate::overnight::cancel_latest_run() {
            Ok(crate::overnight::CancelOutcome::Requested(manifest)) => {
                self.finalize_cancelled_overnight_auto_poke(&manifest.run_id);
                let _ = self.upsert_overnight_display_card(&manifest);
                let note = if crate::overnight::is_supervised(&manifest) {
                    "owned work is draining"
                } else {
                    "visible run: no supervisor, nothing drained, not marked cancelled"
                };
                self.push_display_message(DisplayMessage::system(format!(
                    "🌙 Overnight run {} cancel requested by interrupt; {note}.",
                    manifest.run_id
                )));
            }
            Ok(crate::overnight::CancelOutcome::Rejected(manifest)) => {
                self.finalize_cancelled_overnight_auto_poke(&manifest.run_id);
                self.push_display_message(DisplayMessage::system(
                    crate::overnight::CancelOutcome::rejection_message(&manifest),
                ));
            }
            Err(error) => {
                self.push_display_message(DisplayMessage::error(format!(
                    "Interrupted, but failed to cancel overnight run: {}",
                    crate::util::format_error_chain(&error)
                )));
            }
        }
        true
    }

    pub(super) fn enable_overnight_auto_poke(
        &mut self,
        manifest: &crate::overnight::OvernightManifest,
    ) {
        let fingerprint = overnight_fingerprint_for_app(self, manifest);
        self.overnight_auto_poke = Some(OvernightAutoPokeState {
            run_id: manifest.run_id.clone(),
            last_fingerprint: fingerprint,
            stalled_turns: 0,
            error_turns: 0,
            total_pokes_sent: 0,
            diagnostic_sent: false,
            morning_report_poked: false,
            final_wrap_poked: false,
        });
    }

    pub(super) fn stop_overnight_auto_poke_for_non_retryable_error(&mut self, error: &str) -> bool {
        let Some(run_id) = self
            .overnight_auto_poke
            .as_ref()
            .map(|state| state.run_id.clone())
        else {
            return false;
        };
        record_overnight_poke_event(
            &run_id,
            "auto_poke_turn_error",
            "Auto-poke observed a turn error".to_string(),
            serde_json::json!({ "error": error }),
        );
        if !super::commands::is_non_retryable_auto_poke_error(error) {
            return false;
        }
        self.overnight_auto_poke = None;
        self.finalize_overnight_auto_poke(&run_id, "non_retryable_error");
        self.push_display_message(DisplayMessage::system(
            "🛑 The last request failed in a way that retrying won't fix, so we stopped the overnight run. Check /overnight status and continue manually if it looks right.".to_string(),
        ));
        self.set_status_notice("Overnight run stopped: this error won't fix itself");
        true
    }

    /// Clears tracked auto-poke after a successful cancel and finalizes the
    /// tracked run: `cancelled` if it is the run that was cancelled, otherwise
    /// `run_mismatch` so the tracked run does not stay `Running`.
    fn finalize_cancelled_overnight_auto_poke(&mut self, cancelled_run_id: &str) {
        let Some(state) = self.overnight_auto_poke.take() else {
            return;
        };
        let reason = if state.run_id == cancelled_run_id {
            "cancelled"
        } else {
            "run_mismatch"
        };
        self.finalize_overnight_auto_poke(&state.run_id, reason);
    }

    /// Clears tracked auto-poke for a non-cancel exit and finalizes its run as
    /// failed. Does nothing when no overnight auto-poke is tracked.
    pub(super) fn clear_overnight_auto_poke(&mut self, reason: &'static str) {
        if let Some(state) = self.overnight_auto_poke.take() {
            self.finalize_overnight_auto_poke(&state.run_id, reason);
        }
    }

    /// Records why auto-poke stopped and, if the run is still `Running`, closes
    /// it: `Completed` for `final_done`, `Failed` for every other reason except
    /// `cancelled`, which keeps the `CancelRequested` status already persisted.
    /// Best-effort: failures surface as a system message.
    fn finalize_overnight_auto_poke(&mut self, run_id: &str, reason: &'static str) {
        if let Err(error) = finalize_overnight_run(run_id, reason) {
            self.push_display_message(DisplayMessage::system(format!(
                "Could not finalize overnight run {run_id} after auto-poke stopped ({reason}): {}",
                crate::util::format_error_chain(&error)
            )));
        }
    }

    pub(super) fn schedule_overnight_poke_followup_if_needed(&mut self) -> bool {
        if self.overnight_auto_poke.is_none()
            || self.pending_queued_dispatch
            || self.pending_turn
            || self.has_queued_followups()
        {
            return false;
        }

        let Some(mut state) = self.overnight_auto_poke.take() else {
            return false;
        };
        let manifest = match crate::overnight::latest_manifest() {
            Ok(Some(manifest)) if manifest.run_id == state.run_id => manifest,
            Ok(Some(other)) => {
                self.push_display_message(DisplayMessage::system(format!(
                    "🛑 Overnight auto-poke stopped: latest run is {}, not tracked run {}.",
                    other.run_id, state.run_id
                )));
                self.finalize_overnight_auto_poke(&state.run_id, "run_mismatch");
                return false;
            }
            _ => {
                self.push_display_message(DisplayMessage::system(format!(
                    "🛑 Overnight auto-poke stopped: could not read the manifest for run {}.",
                    state.run_id
                )));
                self.finalize_overnight_auto_poke(&state.run_id, "manifest_unreadable");
                return false;
            }
        };

        if !matches!(
            manifest.status,
            OvernightRunStatus::Running | OvernightRunStatus::CancelRequested
        ) {
            self.push_display_message(DisplayMessage::system(format!(
                "✅ Overnight auto-poke finished: run {} is {}.",
                manifest.run_id,
                overnight_status_label(&manifest.status)
            )));
            self.set_status_notice("Overnight auto-poke finished");
            self.finalize_overnight_auto_poke(&state.run_id, "run_finished");
            return false;
        }
        if matches!(manifest.status, OvernightRunStatus::CancelRequested) {
            self.push_display_message(DisplayMessage::system(
                "🛑 Overnight auto-poke stopped: cancellation requested.".to_string(),
            ));
            self.set_status_notice("Overnight auto-poke stopped");
            self.finalize_overnight_auto_poke(&state.run_id, "cancelled");
            return false;
        }

        let fingerprint = overnight_fingerprint_for_app(self, &manifest);
        let progressed = fingerprint != state.last_fingerprint;
        if progressed {
            state.stalled_turns = 0;
        } else {
            state.stalled_turns = state.stalled_turns.saturating_add(1);
        }
        state.last_fingerprint = fingerprint;

        if state.stalled_turns >= OVERNIGHT_STALL_LIMIT {
            self.push_display_message(DisplayMessage::system(format!(
                "🛑 Overnight auto-poke stopped after {} consecutive no-progress turns. Review {} before continuing manually.",
                state.stalled_turns,
                manifest.review_path.display()
            )));
            self.set_status_notice("Overnight stopped: no progress");
            self.finalize_overnight_auto_poke(&state.run_id, "stalled");
            return false;
        }
        if state.error_turns >= OVERNIGHT_ERROR_LIMIT {
            self.push_display_message(DisplayMessage::system(
                "🛑 Overnight auto-poke stopped after repeated turn errors.".to_string(),
            ));
            self.set_status_notice("Overnight stopped: errors");
            self.finalize_overnight_auto_poke(&state.run_id, "non_retryable_error");
            return false;
        }
        if state.total_pokes_sent >= overnight_poke_budget(&manifest) {
            self.push_display_message(DisplayMessage::system(format!(
                "🛑 Overnight auto-poke stopped after reaching its safety budget of {} follow-up turns.",
                state.total_pokes_sent
            )));
            self.set_status_notice("Overnight stopped: poke budget");
            self.finalize_overnight_auto_poke(&state.run_id, "poke_budget");
            return false;
        }

        let phase = overnight_poke_phase(&manifest, &state);
        if matches!(phase, OvernightPokePhase::FinalDone) {
            self.push_display_message(DisplayMessage::system(format!(
                "✅ Overnight auto-poke finished after final wrap request. Review {}.",
                manifest.review_path.display()
            )));
            self.set_status_notice("Overnight auto-poke complete");
            self.finalize_overnight_auto_poke(&state.run_id, "final_done");
            return false;
        }

        if matches!(phase, OvernightPokePhase::MorningReport) {
            state.morning_report_poked = true;
        }
        if matches!(phase, OvernightPokePhase::FinalWrap) {
            state.final_wrap_poked = true;
        }
        if matches!(phase, OvernightPokePhase::Diagnostic) {
            state.diagnostic_sent = true;
        }
        state.total_pokes_sent = state.total_pokes_sent.saturating_add(1);

        let prompt = build_overnight_poke_message(&manifest, phase, state.stalled_turns);
        self.push_display_message(DisplayMessage::system(format!(
            "🌙 Overnight auto-poking: {}. /overnight cancel to stop.",
            overnight_phase_label(phase)
        )));
        let _ = crate::overnight::record_event(
            &manifest,
            "auto_poke_sent",
            format!("Auto-poke sent: {}", overnight_phase_label(phase)),
            serde_json::json!({
                "phase": overnight_phase_label(phase),
                "stalled_turns": state.stalled_turns,
            }),
            false,
        );
        self.queued_messages.push(prompt);
        self.pending_queued_dispatch = true;
        self.overnight_auto_poke = Some(state);
        true
    }
}

/// Best-effort event write for the poke path; the poke proceeds if it fails.
fn record_overnight_poke_event(
    run_id: &str,
    kind: &str,
    summary: String,
    details: serde_json::Value,
) {
    if let Ok(manifest) = crate::overnight::load_manifest(run_id) {
        let _ = crate::overnight::record_event(&manifest, kind, summary, details, false);
    }
}

fn finalize_overnight_run(run_id: &str, reason: &'static str) -> anyhow::Result<()> {
    let mut manifest = crate::overnight::load_manifest(run_id)?;
    let running = matches!(manifest.status, OvernightRunStatus::Running);
    let active = running || matches!(manifest.status, OvernightRunStatus::CancelRequested);
    // An already-terminal run was not completed by this exit.
    let reason = if !active && reason == "final_done" {
        "run_finished"
    } else {
        reason
    };
    // A failed event write must not keep the status from being persisted.
    let event = crate::overnight::record_event(
        &manifest,
        "auto_poke_stopped",
        format!("Auto-poke stopped: {reason}"),
        serde_json::json!({ "reason": reason }),
        false,
    );
    if !running || reason == "cancelled" {
        return event;
    }
    manifest.status = if reason == "final_done" {
        OvernightRunStatus::Completed
    } else {
        OvernightRunStatus::Failed
    };
    manifest.completed_at = Some(Utc::now());
    crate::overnight::save_manifest(&manifest)?;
    let review = crate::overnight::render_review_html(&manifest);
    event.and(review)
}

fn is_overnight_auto_poke_message(message: &str) -> bool {
    message.starts_with("Overnight auto-poke for run `")
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum OvernightPokePhase {
    Continue,
    Diagnostic,
    Handoff,
    MorningReport,
    PostWake,
    FinalWrap,
    FinalDone,
}

fn overnight_poke_phase(
    manifest: &crate::overnight::OvernightManifest,
    state: &OvernightAutoPokeState,
) -> OvernightPokePhase {
    let now = Utc::now();
    if state.stalled_turns > 0 && !state.diagnostic_sent {
        return OvernightPokePhase::Diagnostic;
    }
    if now >= manifest.post_wake_grace_until {
        return if state.final_wrap_poked {
            OvernightPokePhase::FinalDone
        } else {
            OvernightPokePhase::FinalWrap
        };
    }
    if now >= manifest.target_wake_at {
        return if !state.morning_report_poked && manifest.morning_report_posted_at.is_none() {
            OvernightPokePhase::MorningReport
        } else {
            OvernightPokePhase::PostWake
        };
    }
    if now >= manifest.handoff_ready_at {
        return OvernightPokePhase::Handoff;
    }
    OvernightPokePhase::Continue
}

fn overnight_phase_label(phase: OvernightPokePhase) -> &'static str {
    match phase {
        OvernightPokePhase::Continue => "continuation",
        OvernightPokePhase::Diagnostic => "diagnostic no-progress check",
        OvernightPokePhase::Handoff => "handoff-ready reminder",
        OvernightPokePhase::MorningReport => "morning report",
        OvernightPokePhase::PostWake => "post-wake continuation",
        OvernightPokePhase::FinalWrap => "final wrap-up",
        OvernightPokePhase::FinalDone => "complete",
    }
}

fn overnight_status_label(status: &OvernightRunStatus) -> &'static str {
    match status {
        OvernightRunStatus::Running => "running",
        OvernightRunStatus::CancelRequested => "cancel requested",
        OvernightRunStatus::Cancelled => "cancelled",
        OvernightRunStatus::Completed => "completed",
        OvernightRunStatus::Failed => "failed",
        OvernightRunStatus::Blocked => "blocked",
        OvernightRunStatus::Unknown => "unknown",
    }
}

fn build_overnight_poke_message(
    manifest: &crate::overnight::OvernightManifest,
    phase: OvernightPokePhase,
    stalled_turns: u8,
) -> String {
    let prefix = format!(
        "Overnight auto-poke for run `{}`. First inspect manifest `{}`, review notes `{}`, task cards `{}`, validation `{}`, and git/todo state. Keep artifacts current before stopping. ",
        manifest.run_id,
        manifest.run_dir.join("manifest.json").display(),
        manifest.review_notes_path.display(),
        manifest.task_cards_dir.display(),
        manifest.validation_dir.display(),
    );
    let body = match phase {
        OvernightPokePhase::Diagnostic => format!(
            "The auto-poke guard detected {} no-progress turn(s). Do not continue blindly. Diagnose why progress stalled: blocked task, missing credentials, failing tool, context/model issue, or unclear next step. Either recover with one small verifiable task, or mark the run/task blocked and stop.",
            stalled_turns
        ),
        OvernightPokePhase::Handoff => "Enter handoff-ready mode. Update review notes, task cards, validation evidence, dirty repo state, risks, skipped work, and next steps. Avoid starting large or risky new work.".to_string(),
        OvernightPokePhase::MorningReport => "Target wake time reached. Post the morning report now before starting any new work. Include completed work, current state, validation, files changed, risks, and next steps. Set `morning_report_posted_at` in the manifest when done.".to_string(),
        OvernightPokePhase::PostWake => "Post-wake continuation. Continue only bounded, safe, verifiable work that is in progress or clearly high-value. Do not start broad/risky new changes. Keep artifacts current.".to_string(),
        OvernightPokePhase::FinalWrap | OvernightPokePhase::FinalDone => "Final wrap-up. Stop starting new work. Finish immediate cleanup only, update review notes/task cards/review page with final evidence and risks, then mark the manifest completed.".to_string(),
        OvernightPokePhase::Continue => "Continue the overnight run. If the previous task is done, choose the next highest-confidence bounded task. If blocked, record why and switch to another useful task. Prove/reproduce before fixing, validate after, and update task cards/review notes.".to_string(),
    };
    format!("{}{}", prefix, body)
}

fn overnight_fingerprint_for_app(
    app: &App,
    manifest: &crate::overnight::OvernightManifest,
) -> OvernightAutoPokeFingerprint {
    let task_summary = crate::overnight::summarize_task_cards(manifest);
    OvernightAutoPokeFingerprint {
        run_id: manifest.run_id.clone(),
        status: overnight_status_label(&manifest.status).to_string(),
        last_activity_at: manifest.last_activity_at.to_rfc3339(),
        events_len: crate::overnight::read_events(manifest)
            .map(|events| {
                // Poke bookkeeping events must not look like run progress.
                events
                    .iter()
                    .filter(|event| !event.kind.starts_with("auto_poke_"))
                    .count()
            })
            .unwrap_or(0),
        task_total: task_summary.total,
        task_completed: task_summary.counts.completed,
        task_active: task_summary.counts.active,
        task_blocked: task_summary.counts.blocked,
        task_validated: task_summary.validated,
        session_message_count: app.session.messages.len(),
        review_notes_mtime: file_mtime_secs(&manifest.review_notes_path),
        validation_files: count_files(&manifest.validation_dir),
    }
}

fn overnight_poke_budget(manifest: &crate::overnight::OvernightManifest) -> u16 {
    let duration_hours = manifest
        .target_wake_at
        .signed_duration_since(manifest.started_at)
        .num_minutes()
        .max(1) as f32
        / 60.0;
    ((duration_hours.ceil() as u16).saturating_mul(4)).clamp(4, OVERNIGHT_MAX_POKES)
}

fn file_mtime_secs(path: &std::path::Path) -> Option<u64> {
    path.metadata()
        .ok()?
        .modified()
        .ok()?
        .duration_since(std::time::UNIX_EPOCH)
        .ok()
        .map(|duration| duration.as_secs())
}

fn count_files(path: &std::path::Path) -> usize {
    std::fs::read_dir(path)
        .ok()
        .into_iter()
        .flat_map(|entries| entries.filter_map(Result::ok))
        .filter(|entry| {
            entry
                .file_type()
                .map(|kind| kind.is_file())
                .unwrap_or(false)
        })
        .count()
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration as ChronoDuration;
    use std::path::PathBuf;

    fn test_manifest_with_times(
        started_at: chrono::DateTime<Utc>,
        target_wake_at: chrono::DateTime<Utc>,
        post_wake_grace_until: chrono::DateTime<Utc>,
    ) -> crate::overnight::OvernightManifest {
        crate::overnight::OvernightManifest {
            task_attempt: None,
            task_admission: None,
            version: 1,
            run_id: "overnight_test".to_string(),
            parent_session_id: "parent".to_string(),
            coordinator_session_id: "session".to_string(),
            coordinator_session_name: "Session".to_string(),
            started_at,
            target_wake_at,
            handoff_ready_at: target_wake_at - ChronoDuration::minutes(30),
            post_wake_grace_until,
            morning_report_posted_at: None,
            completed_at: None,
            cancel_requested_at: None,
            status: OvernightRunStatus::Running,
            mission: Some("test".to_string()),
            working_dir: None,
            provider_name: "mock".to_string(),
            model: "mock-model".to_string(),
            max_agents_guidance: 2,
            limits: Default::default(),
            stop_settlement: None,
            limit_breach: None,
            process_id: 1,
            run_dir: PathBuf::from("/tmp/overnight_test"),
            events_path: PathBuf::from("/tmp/overnight_test/events.jsonl"),
            human_log_path: PathBuf::from("/tmp/overnight_test/run.log"),
            review_path: PathBuf::from("/tmp/overnight_test/review.html"),
            review_notes_path: PathBuf::from("/tmp/overnight_test/review-notes.md"),
            preflight_path: PathBuf::from("/tmp/overnight_test/preflight.json"),
            task_cards_dir: PathBuf::from("/tmp/overnight_test/task-cards"),
            issue_drafts_dir: PathBuf::from("/tmp/overnight_test/issue-drafts"),
            validation_dir: PathBuf::from("/tmp/overnight_test/validation"),
            last_activity_at: started_at,
        }
    }

    fn test_state(manifest: &crate::overnight::OvernightManifest) -> OvernightAutoPokeState {
        OvernightAutoPokeState {
            run_id: manifest.run_id.clone(),
            last_fingerprint: OvernightAutoPokeFingerprint {
                run_id: manifest.run_id.clone(),
                status: "running".to_string(),
                last_activity_at: manifest.last_activity_at.to_rfc3339(),
                events_len: 0,
                task_total: 0,
                task_completed: 0,
                task_active: 0,
                task_blocked: 0,
                task_validated: 0,
                session_message_count: 0,
                review_notes_mtime: None,
                validation_files: 0,
            },
            stalled_turns: 0,
            error_turns: 0,
            total_pokes_sent: 0,
            diagnostic_sent: false,
            morning_report_poked: false,
            final_wrap_poked: false,
        }
    }

    #[test]
    fn overnight_poke_phase_requests_morning_report_at_target_once() {
        let now = Utc::now();
        let manifest = test_manifest_with_times(
            now - ChronoDuration::hours(2),
            now - ChronoDuration::minutes(1),
            now + ChronoDuration::hours(1),
        );
        let mut state = test_state(&manifest);
        assert_eq!(
            overnight_poke_phase(&manifest, &state),
            OvernightPokePhase::MorningReport
        );
        state.morning_report_poked = true;
        assert_eq!(
            overnight_poke_phase(&manifest, &state),
            OvernightPokePhase::PostWake
        );
    }

    #[test]
    fn overnight_poke_phase_stops_after_final_wrap_requested() {
        let now = Utc::now();
        let manifest = test_manifest_with_times(
            now - ChronoDuration::hours(4),
            now - ChronoDuration::hours(3),
            now - ChronoDuration::minutes(1),
        );
        let mut state = test_state(&manifest);
        assert_eq!(
            overnight_poke_phase(&manifest, &state),
            OvernightPokePhase::FinalWrap
        );
        state.final_wrap_poked = true;
        assert_eq!(
            overnight_poke_phase(&manifest, &state),
            OvernightPokePhase::FinalDone
        );
    }

    #[test]
    fn overnight_poke_phase_sends_one_diagnostic_on_stall() {
        let now = Utc::now();
        let manifest = test_manifest_with_times(
            now,
            now + ChronoDuration::hours(2),
            now + ChronoDuration::hours(4),
        );
        let mut state = test_state(&manifest);
        state.stalled_turns = 1;
        assert_eq!(
            overnight_poke_phase(&manifest, &state),
            OvernightPokePhase::Diagnostic
        );
        state.diagnostic_sent = true;
        assert_eq!(
            overnight_poke_phase(&manifest, &state),
            OvernightPokePhase::Continue
        );
    }

    #[test]
    fn overnight_poke_budget_is_bounded_by_duration_and_cap() {
        let now = Utc::now();
        let short = test_manifest_with_times(
            now,
            now + ChronoDuration::minutes(30),
            now + ChronoDuration::hours(2),
        );
        assert_eq!(overnight_poke_budget(&short), 4);
        let long = test_manifest_with_times(
            now,
            now + ChronoDuration::hours(72),
            now + ChronoDuration::hours(74),
        );
        assert_eq!(overnight_poke_budget(&long), OVERNIGHT_MAX_POKES);
    }

    struct HomeGuard {
        _lock: std::sync::MutexGuard<'static, ()>,
        original: Option<std::ffi::OsString>,
        _temp: tempfile::TempDir,
    }

    impl HomeGuard {
        fn new() -> Self {
            let lock = crate::storage::lock_test_env();
            let temp = tempfile::tempdir().expect("tempdir");
            let original = std::env::var_os("JCODE_HOME");
            crate::env::set_var("JCODE_HOME", temp.path());
            Self {
                _lock: lock,
                original,
                _temp: temp,
            }
        }
    }

    impl Drop for HomeGuard {
        fn drop(&mut self) {
            match self.original.take() {
                Some(value) => crate::env::set_var("JCODE_HOME", value),
                None => crate::env::remove_var("JCODE_HOME"),
            }
        }
    }

    /// Saved `Running` manifest under the temp `JCODE_HOME`.
    fn saved_manifest(
        run_id: &str,
        session_id: &str,
        process_id: u32,
    ) -> crate::overnight::OvernightManifest {
        let now = Utc::now();
        let mut manifest = test_manifest_with_times(
            now - ChronoDuration::hours(1),
            now + ChronoDuration::hours(6),
            now + ChronoDuration::hours(8),
        );
        let dir = crate::overnight::run_dir(run_id).unwrap();
        std::fs::create_dir_all(&dir).unwrap();
        manifest.run_id = run_id.to_string();
        manifest.coordinator_session_id = session_id.to_string();
        manifest.process_id = process_id;
        manifest.events_path = dir.join("events.jsonl");
        manifest.human_log_path = dir.join("run.log");
        manifest.review_path = dir.join("review.html");
        manifest.review_notes_path = dir.join("review-notes.md");
        manifest.preflight_path = dir.join("preflight.json");
        manifest.task_cards_dir = dir.join("task-cards");
        manifest.issue_drafts_dir = dir.join("issue-drafts");
        manifest.validation_dir = dir.join("validation");
        manifest.run_dir = dir;
        crate::overnight::save_manifest(&manifest).unwrap();
        manifest
    }

    fn event_reasons(manifest: &crate::overnight::OvernightManifest, kind: &str) -> Vec<String> {
        crate::overnight::read_events(manifest)
            .unwrap()
            .into_iter()
            .filter(|event| event.kind == kind)
            .map(|event| {
                event.details["reason"]
                    .as_str()
                    .unwrap_or_default()
                    .to_string()
            })
            .collect()
    }

    #[test]
    fn finalize_reasons_set_expected_status_and_record_event() {
        let _home = HomeGuard::new();
        let mut app = super::super::tests::create_test_app();
        let cases = [
            ("final_done", OvernightRunStatus::Completed),
            ("stalled", OvernightRunStatus::Failed),
            ("poke_budget", OvernightRunStatus::Failed),
            ("non_retryable_error", OvernightRunStatus::Failed),
            ("run_mismatch", OvernightRunStatus::Failed),
            ("manifest_unreadable", OvernightRunStatus::Failed),
            ("cancelled", OvernightRunStatus::Running),
        ];
        for (reason, expected) in cases {
            let run_id = format!("overnight_finalize_{reason}");
            let manifest = saved_manifest(&run_id, "session", std::process::id());
            app.finalize_overnight_auto_poke(&run_id, reason);
            let loaded = crate::overnight::load_manifest(&run_id).unwrap();
            assert_eq!(loaded.status, expected, "{reason}");
            assert_eq!(
                loaded.completed_at.is_some(),
                !matches!(expected, OvernightRunStatus::Running),
                "{reason}"
            );
            assert_eq!(
                event_reasons(&manifest, "auto_poke_stopped"),
                vec![reason.to_string()],
                "{reason}"
            );
        }
    }

    #[test]
    fn finalize_keeps_cancel_requested_status() {
        let _home = HomeGuard::new();
        let mut app = super::super::tests::create_test_app();
        let mut manifest = saved_manifest("overnight_cancel_kept", "session", 1);
        manifest.status = OvernightRunStatus::CancelRequested;
        crate::overnight::save_manifest(&manifest).unwrap();
        app.finalize_overnight_auto_poke("overnight_cancel_kept", "stalled");
        let loaded = crate::overnight::load_manifest("overnight_cancel_kept").unwrap();
        assert_eq!(loaded.status, OvernightRunStatus::CancelRequested);
    }

    #[test]
    fn finalize_missing_manifest_reports_system_message_without_panic() {
        let _home = HomeGuard::new();
        let mut app = super::super::tests::create_test_app();
        let before = app.display_messages.len();
        app.finalize_overnight_auto_poke("overnight_missing", "stalled");
        assert_eq!(app.display_messages.len(), before + 1);
    }

    #[test]
    fn poke_followup_records_auto_poke_sent() {
        let _home = HomeGuard::new();
        let mut app = super::super::tests::create_test_app();
        let manifest = saved_manifest("overnight_poke_sent", "session", std::process::id());
        app.enable_overnight_auto_poke(&manifest);
        assert!(app.schedule_overnight_poke_followup_if_needed());
        assert_eq!(app.queued_messages.len(), 1);
        let events = crate::overnight::read_events(&manifest).unwrap();
        let sent: Vec<_> = events
            .iter()
            .filter(|event| event.kind == "auto_poke_sent")
            .collect();
        assert_eq!(sent.len(), 1);
        assert!(sent[0].details["phase"].is_string());
        assert!(sent[0].details["stalled_turns"].is_number());
    }

    #[test]
    fn poke_followup_stops_and_fails_run_on_budget_exhaustion() {
        let _home = HomeGuard::new();
        let mut app = super::super::tests::create_test_app();
        let manifest = saved_manifest("overnight_budget", "session", std::process::id());
        app.enable_overnight_auto_poke(&manifest);
        app.overnight_auto_poke.as_mut().unwrap().total_pokes_sent = OVERNIGHT_MAX_POKES;
        assert!(!app.schedule_overnight_poke_followup_if_needed());
        assert!(app.overnight_auto_poke.is_none());
        let loaded = crate::overnight::load_manifest("overnight_budget").unwrap();
        assert_eq!(loaded.status, OvernightRunStatus::Failed);
        assert_eq!(
            event_reasons(&manifest, "auto_poke_stopped"),
            vec!["poke_budget".to_string()]
        );
    }

    #[test]
    fn poke_followup_run_mismatch_finalizes_tracked_run() {
        let _home = HomeGuard::new();
        let mut app = super::super::tests::create_test_app();
        let tracked = saved_manifest("overnight_tracked", "session", std::process::id());
        app.enable_overnight_auto_poke(&tracked);
        // A newer run becomes "latest", so the tracked one mismatches.
        let mut newer = saved_manifest("overnight_newer", "other", std::process::id());
        newer.started_at = Utc::now();
        crate::overnight::save_manifest(&newer).unwrap();
        assert!(!app.schedule_overnight_poke_followup_if_needed());
        let loaded = crate::overnight::load_manifest("overnight_tracked").unwrap();
        assert_eq!(loaded.status, OvernightRunStatus::Failed);
        assert_eq!(
            event_reasons(&tracked, "auto_poke_stopped"),
            vec!["run_mismatch".to_string()]
        );
    }

    #[test]
    fn non_retryable_error_records_turn_error_and_fails_run() {
        let _home = HomeGuard::new();
        let mut app = super::super::tests::create_test_app();
        let manifest = saved_manifest("overnight_nonretry", "session", std::process::id());
        app.enable_overnight_auto_poke(&manifest);
        let error = "400 Bad Request: invalid_request_error";
        assert!(app.stop_overnight_auto_poke_for_non_retryable_error(error));
        let loaded = crate::overnight::load_manifest("overnight_nonretry").unwrap();
        assert_eq!(loaded.status, OvernightRunStatus::Failed);
        assert_eq!(
            event_reasons(&manifest, "auto_poke_stopped"),
            vec!["non_retryable_error".to_string()]
        );
        let events = crate::overnight::read_events(&manifest).unwrap();
        assert!(
            events
                .iter()
                .any(|event| event.kind == "auto_poke_turn_error")
        );
    }

    #[test]
    fn start_guard_refuses_live_duplicate_in_same_session() {
        let _home = HomeGuard::new();
        let mut app = super::super::tests::create_test_app();
        saved_manifest(
            "overnight_live",
            &app.session.id.clone(),
            std::process::id(),
        );
        let before = app.display_messages.len();
        assert!(refuse_duplicate_overnight_start(&mut app));
        let message = app.display_messages.last().unwrap();
        assert_eq!(app.display_messages.len(), before + 1);
        assert_eq!(message.role, "error");
        assert!(message.content.contains(
            "This session already has an active overnight run overnight_live. Inspect status or cancel it before starting another."
        ));
    }

    #[test]
    fn start_guard_allows_stale_run_and_names_it() {
        let _home = HomeGuard::new();
        let mut app = super::super::tests::create_test_app();
        saved_manifest("overnight_stale", &app.session.id.clone(), u32::MAX);
        assert!(!refuse_duplicate_overnight_start(&mut app));
        let message = app.display_messages.last().unwrap();
        assert_eq!(message.role, "system");
        assert!(message.content.contains("overnight_stale"));
    }

    #[test]
    fn start_guard_ignores_other_sessions() {
        let _home = HomeGuard::new();
        let mut app = super::super::tests::create_test_app();
        saved_manifest("overnight_other", "someone-else", std::process::id());
        let before = app.display_messages.len();
        assert!(!refuse_duplicate_overnight_start(&mut app));
        assert_eq!(app.display_messages.len(), before);
    }

    #[test]
    fn start_guard_scans_past_other_sessions_and_corrupt_manifests() {
        let _home = HomeGuard::new();
        let mut app = super::super::tests::create_test_app();
        let session = app.session.id.clone();
        let mut mine = saved_manifest("overnight_mine_live", &session, std::process::id());
        mine.started_at = Utc::now() - ChronoDuration::hours(3);
        crate::overnight::save_manifest(&mine).unwrap();
        let mut other = saved_manifest("overnight_other_newer", "someone-else", 1);
        other.started_at = Utc::now();
        crate::overnight::save_manifest(&other).unwrap();
        let corrupt = crate::overnight::run_dir("overnight_corrupt").unwrap();
        std::fs::create_dir_all(&corrupt).unwrap();
        std::fs::write(corrupt.join("manifest.json"), "{not json").unwrap();
        assert!(refuse_duplicate_overnight_start(&mut app));
        assert!(
            app.display_messages
                .last()
                .unwrap()
                .content
                .contains("overnight_mine_live")
        );
    }

    #[test]
    fn start_guard_live_older_run_beats_newer_stale_run() {
        let _home = HomeGuard::new();
        let mut app = super::super::tests::create_test_app();
        let session = app.session.id.clone();
        let mut live = saved_manifest("overnight_live_old", &session, std::process::id());
        live.started_at = Utc::now() - ChronoDuration::hours(3);
        crate::overnight::save_manifest(&live).unwrap();
        saved_manifest("overnight_stale_new", &session, u32::MAX);
        assert!(refuse_duplicate_overnight_start(&mut app));
        assert_eq!(app.display_messages.last().unwrap().role, "error");
    }

    #[test]
    fn finalize_persists_status_when_event_write_fails() {
        let _home = HomeGuard::new();
        let mut app = super::super::tests::create_test_app();
        let mut manifest = saved_manifest("overnight_event_fail", "session", std::process::id());
        // A directory at the events path makes the append fail.
        manifest.events_path = manifest.run_dir.join("events-dir");
        std::fs::create_dir_all(&manifest.events_path).unwrap();
        crate::overnight::save_manifest(&manifest).unwrap();
        let before = app.display_messages.len();
        app.finalize_overnight_auto_poke("overnight_event_fail", "stalled");
        let loaded = crate::overnight::load_manifest("overnight_event_fail").unwrap();
        assert_eq!(loaded.status, OvernightRunStatus::Failed);
        assert!(loaded.completed_at.is_some());
        assert_eq!(app.display_messages.len(), before + 1, "failure surfaced");
    }

    #[test]
    fn finalize_does_not_claim_final_done_or_change_terminal_runs() {
        let _home = HomeGuard::new();
        let mut app = super::super::tests::create_test_app();
        for status in [
            OvernightRunStatus::Completed,
            OvernightRunStatus::Failed,
            OvernightRunStatus::Blocked,
            OvernightRunStatus::Unknown,
        ] {
            let run_id = format!("overnight_terminal_{}", status.label());
            let mut manifest = saved_manifest(&run_id, "session", 1);
            manifest.status = status.clone();
            manifest.completed_at = None;
            crate::overnight::save_manifest(&manifest).unwrap();
            app.finalize_overnight_auto_poke(&run_id, "final_done");
            app.finalize_overnight_auto_poke(&run_id, "stalled");
            let loaded = crate::overnight::load_manifest(&run_id).unwrap();
            assert_eq!(loaded.status, status);
            assert!(loaded.completed_at.is_none());
            assert_eq!(
                event_reasons(&manifest, "auto_poke_stopped"),
                vec!["run_finished".to_string(), "stalled".to_string()]
            );
        }
    }

    #[test]
    fn clear_overnight_auto_poke_fails_tracked_run_and_is_noop_when_untracked() {
        let _home = HomeGuard::new();
        let mut app = super::super::tests::create_test_app();
        let before = app.display_messages.len();
        app.clear_overnight_auto_poke("provider_failover");
        assert_eq!(app.display_messages.len(), before);
        let manifest = saved_manifest("overnight_clear", "session", std::process::id());
        app.enable_overnight_auto_poke(&manifest);
        app.clear_overnight_auto_poke("provider_failover");
        assert!(app.overnight_auto_poke.is_none());
        let loaded = crate::overnight::load_manifest("overnight_clear").unwrap();
        assert_eq!(loaded.status, OvernightRunStatus::Failed);
        assert_eq!(
            event_reasons(&manifest, "auto_poke_stopped"),
            vec!["provider_failover".to_string()]
        );
    }

    #[test]
    fn cancel_failure_keeps_tracked_auto_poke() {
        let _home = HomeGuard::new();
        let mut app = super::super::tests::create_test_app();
        let manifest = saved_manifest("overnight_cancel_fail", "session", std::process::id());
        app.enable_overnight_auto_poke(&manifest);
        // Corrupt the only manifest so cancel_latest_run finds no runs.
        std::fs::write(manifest.run_dir.join("manifest.json"), "{not json").unwrap();
        cancel_overnight(&mut app);
        assert!(app.overnight_auto_poke.is_some());
    }
}

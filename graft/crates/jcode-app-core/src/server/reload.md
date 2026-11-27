# crates/jcode-app-core/src/server/reload.rs

- RELOAD_GRACEFUL_SHUTDOWN_TIMEOUT · constant · L13-L13 — const RELOAD_GRACEFUL_SHUTDOWN_TIMEOUT: Duration = Duration::from_secs(2);
- prepare_server_exec · function · L15-L29 — fn prepare_server_exec(cmd: &mut std::process::Command, socket_path: &std::path::Path)
- inherited_serve_flags · function · L36-L61 — fn inherited_serve_flags(args: impl IntoIterator<Item = String>) -> Vec<String>
- VALUE_FLAGS · constant · L37-L44 — const VALUE_FLAGS: &[&str] = &[
- inherited_serve_flags_tests · module · L64-L120 — mod inherited_serve_flags_tests
- args · function · L66-L68 — fn args(list: &[&str]) -> Vec<String>
- keeps_systemd_provider_profile_and_server_name · function · L70-L92 — fn keeps_systemd_provider_profile_and_server_name()
- keeps_provider_and_model_flags · function · L95-L119 — fn keeps_provider_and_model_flags()
- receive_reload_signal · function · L122-L145 — async fn receive_reload_signal(
- await_reload_signal · function · L147-L354 — pub(super) async fn await_reload_signal(
- reload_recovery_candidates · function · L359-L385 — fn reload_recovery_candidates<'a>(
- recovery_candidate_tests · module · L388-L433 — mod recovery_candidate_tests
- includes_registered_ready_and_absent_members_preserving_headless_metadata · function · L394-L408 — fn includes_registered_ready_and_absent_members_preserving_headless_metadata()
- unions_running_active_and_triggering_sessions_in_sorted_unique_order · function · L411-L432 — fn unions_running_active_and_triggering_sessions_in_sorted_unique_order()
- persist_reload_recovery_intents · function · L435-L546 — async fn persist_reload_recovery_intents(
- graceful_shutdown_sessions · function · L548-L566 — pub(super) async fn graceful_shutdown_sessions(
- graceful_shutdown_sessions_with_timeout · function · L568-L728 — async fn graceful_shutdown_sessions_with_timeout(
- reload_tests · module · L732-L732 — mod reload_tests;

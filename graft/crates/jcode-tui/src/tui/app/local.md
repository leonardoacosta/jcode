# crates/jcode-tui/src/tui/app/local.rs

- BACKGROUND_PROGRESS_NOTICE_MIN_INTERVAL · constant · L19-L19 — const BACKGROUND_PROGRESS_NOTICE_MIN_INTERVAL: Duration = Duration::from_millis(400);
- BACKGROUND_PROGRESS_IDENTICAL_NOTICE_TTL · constant · L20-L20 — const BACKGROUND_PROGRESS_IDENTICAL_NOTICE_TTL: Duration = Duration::from_secs(2);
- process_turn_with_input · function · L22-L83 — pub(super) async fn process_turn_with_input(
- handle_tick · function · L85-L192 — pub(super) fn handle_tick(app: &mut App) -> bool
- handle_terminal_event · function · L194-L210 — pub(super) fn handle_terminal_event(
- MAX_DRAINED_EVENTS_PER_WAKE · constant · L201-L201 — const MAX_DRAINED_EVENTS_PER_WAKE: usize = 32;
- handle_bus_event · function · L212-L368 — pub(super) fn handle_bus_event(
- handle_ui_activity · function · L370-L404 — pub(super) fn handle_ui_activity(app: &mut App, activity: UiActivity) -> bool
- handle_manual_tool_completed · function · L406-L451 — fn handle_manual_tool_completed(app: &mut App, result: ManualToolCompleted)
- apply_terminal_event · function · L453-L500 — fn apply_terminal_event(
- handle_background_task_completed · function · L502-L554 — fn handle_background_task_completed(app: &mut App, task: BackgroundTaskCompleted)
- handle_background_task_stalled · function · L556-L616 — fn handle_background_task_stalled(app: &mut App, task: crate::bus::BackgroundTaskStalled)
- handle_background_task_progress · function · L618-L638 — fn handle_background_task_progress(app: &mut App, event: BackgroundTaskProgressEvent)
- maybe_set_background_progress_notice · function · L640-L654 — fn maybe_set_background_progress_notice(app: &mut App, notice: String)
- handle_input_shell_completed · function · L656-L665 — fn handle_input_shell_completed(app: &mut App, shell: InputShellCompleted)
- finish_turn · function · L667-L694 — pub(super) fn finish_turn(app: &mut App)

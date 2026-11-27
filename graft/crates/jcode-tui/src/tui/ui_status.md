# crates/jcode-tui/src/tui/ui_status.rs

- semver · function · L4-L7 — pub(super) fn semver() -> &'static str
- SEMVER · constant · L5-L5 — static SEMVER: OnceLock<String> = OnceLock::new();
- is_running_stable_release · function · L12-L46 — pub(super) fn is_running_stable_release() -> bool
- IS_STABLE · constant · L13-L13 — static IS_STABLE: OnceLock<bool> = OnceLock::new();
- calculate_input_lines · function · L49-L69 — pub(crate) fn calculate_input_lines(input: &str, line_width: usize) -> usize
- shorten_model_name · function · L71-L100 — pub(super) fn shorten_model_name(model: &str) -> String
- format_status_for_debug · function · L102-L154 — pub(super) fn format_status_for_debug(app: &dyn TuiState) -> String

# crates/jcode-tui/src/tui/focus_carryover.rs

- UNFOCUSED_ENV · constant · L13-L13 — const UNFOCUSED_ENV: &str = "JCODE_CLIENT_UNFOCUSED_PID";
- CONFIRMED_ENV · constant · L16-L16 — const CONFIRMED_ENV: &str = "JCODE_CLIENT_FOCUS_KNOWN_PID";
- marker_matches · function · L18-L28 — fn marker_matches(value: Option<&str>, pid: u32) -> bool
- environment_marker_matches · function · L30-L39 — fn environment_marker_matches(name: &str, pid: u32) -> bool
- record · function · L42-L56 — pub(crate) fn record(focused: bool)
- confirm · function · L59-L67 — pub(crate) fn confirm()
- initial_confirmed · function · L70-L73 — pub(crate) fn initial_confirmed() -> bool
- initial_focus · function · L77-L79 — pub(crate) fn initial_focus() -> bool
- tests · module · L82-L95 — mod tests
- only_this_process_marker_counts · function · L86-L94 — fn only_this_process_marker_counts()

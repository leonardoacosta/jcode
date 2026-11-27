# src/cli/output.rs

- QUIET_ENV · constant · L1-L1 — pub const QUIET_ENV: &str = "JCODE_QUIET";
- set_quiet_enabled · function · L3-L9 — pub fn set_quiet_enabled(enabled: bool)
- quiet_enabled · function · L11-L15 — pub fn quiet_enabled() -> bool
- stderr_info · function · L17-L21 — pub fn stderr_info(message: impl AsRef<str>)
- terminal_title · function · L23-L25 — pub fn terminal_title(title: impl AsRef<str>) -> String
- stderr_blank_line · function · L27-L31 — pub fn stderr_blank_line()
- print_centered · function · L34-L48 — pub fn print_centered(msg: &str)
- unicode_display_width · function · L50-L68 — fn unicode_display_width(s: &str) -> usize

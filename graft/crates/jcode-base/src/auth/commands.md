# crates/jcode-base/src/auth/commands.rs

- command_exists · function · L5-L46 — pub(crate) fn command_exists(command: &str) -> bool
- cache_command_result · function · L48-L52 — fn cache_command_result(command: &str, exists: bool)
- is_wsl2 · function · L56-L63 — fn is_wsl2() -> bool
- IS_WSL2 · constant · L57-L57 — static IS_WSL2: OnceLock<bool> = OnceLock::new();
- is_wsl2_windows_path · function · L67-L81 — pub(crate) fn is_wsl2_windows_path(dir: &std::path::Path) -> bool
- explicit_command_exists · function · L83-L109 — fn explicit_command_exists(path: &std::path::Path) -> bool
- command_candidates · function · L111-L151 — pub(crate) fn command_candidates(command: &str) -> Vec<std::ffi::OsString>
- contains_path_separator · function · L153-L157 — pub(crate) fn contains_path_separator(command: &str) -> bool
- has_extension · function · L159-L161 — pub(crate) fn has_extension(path: &std::path::Path) -> bool
- dedup_preserve_order · function · L163-L172 — pub(crate) fn dedup_preserve_order(mut values: Vec<std::ffi::OsString>) -> Vec<std::ffi::OsString>

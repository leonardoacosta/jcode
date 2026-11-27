# crates/jcode-tui/src/tui/app/commands_desktop.rs

- DESKTOP_BIN_ENV · constant · L12-L12 — const DESKTOP_BIN_ENV: &str = "JCODE_DESKTOP_BIN";
- desktop_binary_name · function · L14-L20 — fn desktop_binary_name() -> &'static str
- is_executable · function · L22-L24 — fn is_executable(path: &Path) -> bool
- desktop_binary_candidates · function · L27-L55 — fn desktop_binary_candidates(
- find_desktop_binary · function · L57-L66 — pub(super) fn find_desktop_binary() -> Option<PathBuf>
- desktop_launch_args · function · L68-L73 — pub(super) fn desktop_launch_args(session_id: &str) -> Vec<String>
- spawn_detached · function · L75-L108 — fn spawn_detached(
- DETACHED_PROCESS · constant · L98-L98 — const DETACHED_PROCESS: u32 = 0x0000_0008;
- CREATE_NEW_PROCESS_GROUP · constant · L99-L99 — const CREATE_NEW_PROCESS_GROUP: u32 = 0x0000_0200;
- handle_desktop_command · function · L110-L151 — pub(super) fn handle_desktop_command(app: &mut App, trimmed: &str) -> bool
- tests · module · L154-L180 — mod tests
- launch_args_open_session_in_single_panel_window · function · L158-L163 — fn launch_args_open_session_in_single_panel_window()
- candidates_prefer_override_then_sibling_then_path · function · L166-L179 — fn candidates_prefer_override_then_sibling_then_path()

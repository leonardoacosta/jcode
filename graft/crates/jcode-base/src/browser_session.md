# crates/jcode-base/src/browser_session.rs

- runtime_dir · function · L13-L18 — fn runtime_dir() -> PathBuf
- session_socket_path · function · L20-L22 — fn session_socket_path(name: &str) -> PathBuf
- session_pid_path · function · L24-L26 — fn session_pid_path(name: &str) -> PathBuf
- is_session_alive · function · L28-L40 — fn is_session_alive(name: &str) -> bool
- ensure_browser_session · function · L42-L44 — pub fn ensure_browser_session(session_id: &str) -> Option<String>
- ensure_browser_session_for · function · L48-L93 — pub fn ensure_browser_session_for(session_id: &str, browser: Option<&str>) -> Option<String>
- STARTUP_LOCKS · constant · L56-L57 — static STARTUP_LOCKS: LazyLock<Mutex<HashMap<String, Weak<Mutex<()>>>>> =
- ensure_browser_session_for_async · function · L96-L113 — pub async fn ensure_browser_session_for_async(
- reset_browser_session_for · function · L119-L135 — pub fn reset_browser_session_for(session_id: &str, browser: Option<&str>)
- browser_session_running · function · L139-L141 — pub fn browser_session_running(session_id: &str, browser: Option<&str>) -> bool
- is_stale_session_target_error · function · L144-L149 — pub fn is_stale_session_target_error(message: &str) -> bool
- browser_supports_bind_window · function · L151-L161 — fn browser_supports_bind_window(bin: &std::path::Path) -> bool
- spawn_browser_session · function · L163-L223 — fn spawn_browser_session(
- session_name_for · function · L227-L233 — pub(super) fn session_name_for(session_id: &str, browser: Option<&str>) -> String
- sanitize_session_name · function · L235-L241 — fn sanitize_session_name(session_id: &str) -> String
- tests · module · L245-L245 — mod tests;

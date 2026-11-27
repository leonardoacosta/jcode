# crates/jcode-base/src/client_focus.rs

- last_focused_session_write_cache · function · L12-L15 — fn last_focused_session_write_cache() -> &'static Mutex<Option<String>>
- CACHE · constant · L13-L13 — static CACHE: OnceLock<Mutex<Option<String>>> = OnceLock::new();
- remember_last_focused_session · function · L17-L47 — pub fn remember_last_focused_session(session_id: &str) -> Result<()>
- last_focused_session · function · L49-L68 — pub fn last_focused_session() -> Result<Option<String>>
- client_sessions_dir · function · L70-L72 — fn client_sessions_dir() -> Result<std::path::PathBuf>
- remember_client_session · function · L75-L107 — fn remember_client_session(pid: u32, session_id: &str) -> Result<()>
- registered_client_session · function · L109-L129 — pub fn registered_client_session(pid: u32) -> Option<String>
- clients_showing_session · function · L133-L140 — pub fn clients_showing_session(session_id: &str) -> Vec<u32>
- clients_showing_session_in · function · L142-L175 — fn clients_showing_session_in(
- is_live_jcode_client · function · L179-L193 — fn is_live_jcode_client(pid: u32) -> bool
- last_focused_session_path · function · L195-L197 — fn last_focused_session_path() -> Result<std::path::PathBuf>
- tests · module · L200-L218 — mod tests
- finds_other_live_clients_showing_the_session · function · L204-L217 — fn finds_other_live_clients_showing_the_session()

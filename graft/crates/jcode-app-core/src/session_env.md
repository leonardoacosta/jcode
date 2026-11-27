# crates/jcode-app-core/src/session_env.rs

- SESSION_ENV · constant · L11-L12 — static SESSION_ENV: LazyLock<RwLock<HashMap<String, HashMap<String, String>>>> =
- set_session_env · function · L17-L36 — pub fn set_session_env(session_id: &str, env: HashMap<String, String>) -> Result<()>
- with_session_env · function · L39-L46 — pub(crate) fn with_session_env<T>(
- session_env · function · L49-L55 — pub fn session_env(session_id: &str) -> HashMap<String, String>
- inherit_session_env · function · L58-L66 — pub fn inherit_session_env(parent: &str, child: &str)
- remove_session_env · function · L69-L73 — pub fn remove_session_env(session_id: &str)
- tests · module · L76-L123 — mod tests
- vars · function · L79-L81 — fn vars(value: &str) -> HashMap<String, String>
- invalid_updates_are_atomic_and_do_not_disclose_input · function · L84-L101 — fn invalid_updates_are_atomic_and_do_not_disclose_input()
- snapshots_and_children_are_independent_and_cleanup_removes_entries · function · L104-L122 — fn snapshots_and_children_are_independent_and_cleanup_removes_entries()

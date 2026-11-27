# crates/jcode-tui/src/tui/session_picker/cloud_presence.rs

- REFRESH_INTERVAL · constant · L9-L9 — const REFRESH_INTERVAL: Duration = Duration::from_secs(10);
- SCRIPT · constant · L12-L21 — const SCRIPT: &str = r#"d="$HOME/.jcode/streaming_pids"
- CloudPresence · struct · L24-L29 — pub(super) struct CloudPresence
- parse · function · L31-L57 — pub(super) fn parse(output: &str) -> HashMap<String, Option<std::time::SystemTime>>
- poll_host · function · L59-L86 — fn poll_host(host: &str) -> Option<HashMap<String, Option<std::time::SystemTime>>>
- presence_sessions · function · L89-L98 — fn presence_sessions(&self) -> impl Iterator<Item = &SessionInfo>
- cloud_hosts · function · L100-L115 — fn cloud_hosts(&self) -> Vec<String>
- maybe_refresh_cloud_presence · function · L119-L171 — pub(super) fn maybe_refresh_cloud_presence(&mut self) -> bool
- merge_cloud_presence · function · L174-L195 — pub(super) fn merge_cloud_presence(&mut self)
- tests · module · L199-L300 — mod tests
- parses_live_markers_and_rejects_unsafe_ids · function · L203-L211 — fn parses_live_markers_and_rejects_unsafe_ids()
- invalid_cloud_timestamps_keep_presence_without_age · function · L214-L219 — fn invalid_cloud_timestamps_keep_presence_without_age()
- cloud_running_session_shows_running_state · function · L222-L235 — fn cloud_running_session_shows_running_state()
- cloud_group · function · L237-L246 — fn cloud_group(sessions: Vec<SessionInfo>) -> ServerGroup
- grouped_cloud_sessions_are_polled_and_show_running_in_active_view · function · L249-L275 — fn grouped_cloud_sessions_are_polled_and_show_running_in_active_view()
- grouped_picker_orphans_and_reseed_keep_cloud_presence · function · L278-L292 — fn grouped_picker_orphans_and_reseed_keep_cloud_presence()
- make_cloud_session · function · L294-L299 — fn make_cloud_session() -> SessionInfo

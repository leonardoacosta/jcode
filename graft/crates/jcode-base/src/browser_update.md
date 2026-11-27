# crates/jcode-base/src/browser_update.rs

- CHECK_INTERVAL · constant · L17-L17 — const CHECK_INTERVAL: std::time::Duration = std::time::Duration::from_secs(6 * 60 * 60);
- CHECK_TIMEOUT · constant · L19-L19 — const CHECK_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(8);
- installed_version_path · function · L21-L23 — fn installed_version_path() -> PathBuf
- last_check_path · function · L25-L27 — fn last_check_path() -> PathBuf
- installed_bridge_version · function · L31-L37 — pub fn installed_bridge_version() -> Option<String>
- record_installed_version · function · L39-L43 — pub(super) fn record_installed_version(tag: &str)
- parse_version · function · L47-L59 — pub(super) fn parse_version(tag: &str) -> Option<(u64, u64, u64)>
- is_newer_release · function · L63-L71 — pub(super) fn is_newer_release(installed: Option<&str>, latest: &str) -> bool
- check_due · function · L75-L85 — pub(super) fn check_due(
- last_check_time · function · L87-L92 — fn last_check_time() -> Option<std::time::SystemTime>
- auto_update_disabled · function · L94-L99 — fn auto_update_disabled() -> bool
- fetch_latest_release · function · L102-L120 — pub(super) async fn fetch_latest_release() -> Result<serde_json::Value>
- BridgeUpdate · enum · L124-L131 — pub enum BridgeUpdate
- describe · function · L134-L145 — pub fn describe(&self) -> Option<String>
- update_bridge_if_newer · function · L152-L186 — pub async fn update_bridge_if_newer(kind: BrowserKind, force: bool) -> Result<BridgeUpdate>
- restart_running_hosts · function · L193-L202 — async fn restart_running_hosts()
- stop_hosts_running · function · L206-L251 — pub(super) fn stop_hosts_running(host: &std::path::Path)
- auto_update_before_action · function · L255-L267 — pub async fn auto_update_before_action(kind: BrowserKind) -> Option<String>

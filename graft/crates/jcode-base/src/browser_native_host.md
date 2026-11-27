# crates/jcode-base/src/browser_native_host.rs

- native_host_manifest_json · function · L3-L18 — pub(super) fn native_host_manifest_json(kind: BrowserKind, host_path: &str) -> serde_json::Value
- native_host_manifest_is_valid · function · L22-L43 — pub(super) fn native_host_manifest_is_valid(
- install_native_host_manifest_for · function · L45-L88 — pub(super) fn install_native_host_manifest_for(kind: BrowserKind) -> Result<bool>
- register_windows_native_host_manifest · function · L91-L126 — pub(super) fn register_windows_native_host_manifest(
- native_messaging_hosts_dirs_for · function · L128-L142 — pub(super) fn native_messaging_hosts_dirs_for(kind: BrowserKind) -> Result<Vec<PathBuf>>
- extra_native_messaging_dirs · function · L147-L154 — pub(super) fn extra_native_messaging_dirs(value: Option<&std::ffi::OsStr>) -> Vec<PathBuf>

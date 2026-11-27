# crates/jcode-app-core/src/server/runtime_upgrade.rs

- PROBE_TIMEOUT · constant · L12-L12 — const PROBE_TIMEOUT: Duration = Duration::from_secs(2);
- MAX_REPORT_BYTES · constant · L13-L13 — const MAX_REPORT_BYTES: u64 = 64 * 1024;
- FileIdentity · struct · L16-L21 — struct FileIdentity
- identity · function · L23-L50 — fn identity(path: &Path) -> Result<FileIdentity>
- validate_path · function · L52-L77 — fn validate_path(candidate: &Path) -> Result<(PathBuf, FileIdentity)>
- strict_version · function · L79-L86 — fn strict_version(value: &str) -> Result<(u64, u64, u64)>
- remove_runtime_identity · function · L88-L98 — fn remove_runtime_identity(command: &mut std::process::Command)
- probe_version · function · L100-L152 — async fn probe_version(path: &Path) -> Result<String>
- PreparedUpgrade · struct · L154-L158 — struct PreparedUpgrade
- UpgradeDecision · enum · L160-L164 — enum UpgradeDecision
- reserve_upgrade · function · L166-L188 — fn reserve_upgrade(
- prepare_upgrade · function · L190-L201 — async fn prepare_upgrade(candidate: &Path) -> Result<UpgradeDecision>
- status · function · L203-L210 — fn status(_id: u64, step: &str, message: impl Into<String>) -> ServerEvent
- handle · function · L214-L334 — pub(super) async fn handle(
- replace_after_delivery_checkpoint · function · L339-L355 — fn replace_after_delivery_checkpoint(
- SessionIndex · type · L357-L357 — type SessionIndex = std::collections::HashMap<String, Arc<Mutex<crate::agent::Agent>>>;
- checkpoint_sessions · function · L359-L388 — fn checkpoint_sessions(
- tests · module · L392-L392 — mod tests;

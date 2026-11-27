# crates/jcode-base/src/copilot_usage.rs

- TRACKER · constant · L10-L10 — static TRACKER: Mutex<Option<CopilotUsageTracker>> = Mutex::new(None);
- usage_path · function · L12-L16 — fn usage_path() -> PathBuf
- roll_if_needed · function · L20-L37 — fn roll_if_needed(tracker: &mut CopilotUsageTracker)
- record_usage · function · L39-L69 — fn record_usage(
- load_tracker · function · L70-L73 — fn load_tracker() -> CopilotUsageTracker
- save_tracker · function · L75-L78 — fn save_tracker(tracker: &CopilotUsageTracker)
- record_request · function · L81-L88 — pub fn record_request(input_tokens: u64, output_tokens: u64, is_premium: bool)
- get_usage · function · L91-L99 — pub fn get_usage() -> CopilotUsageTracker
- tests · module · L102-L189 — mod tests
- lock_env · function · L109-L111 — fn lock_env() -> std::sync::MutexGuard<'static, ()>
- EnvVarGuard · struct · L113-L116 — struct EnvVarGuard
- set · function · L119-L123 — fn set(key: &'static str, value: impl AsRef<std::ffi::OsStr>) -> Self
- drop · function · L127-L133 — fn drop(&mut self)
- clear_tracker · function · L136-L140 — fn clear_tracker()
- usage_path_respects_jcode_home · function · L143-L150 — fn usage_path_respects_jcode_home()
- save_and_load_roundtrip_under_jcode_home · function · L153-L188 — fn save_and_load_roundtrip_under_jcode_home()

# crates/jcode-base/src/browser_chromium_diag.rs

- ChromiumExtensionState · enum · L14-L23 — pub enum ChromiumExtensionState
- classify_record · function · L26-L55 — pub(super) fn classify_record(
- profile_dirs · function · L58-L73 — fn profile_dirs(user_data: &Path) -> Vec<String>
- read_profile_json · function · L77-L99 — fn read_profile_json(path: &Path) -> Option<Value>
- extension_record · function · L101-L109 — fn extension_record(profile_dir: &Path) -> Option<Value>
- inspect_user_data · function · L112-L133 — pub(super) fn inspect_user_data(
- user_data_dir · function · L136-L144 — fn user_data_dir(kind: BrowserKind) -> Option<PathBuf>
- diagnose_silent_chromium_bridge · function · L148-L171 — pub fn diagnose_silent_chromium_bridge(kind: BrowserKind) -> Option<String>
- tests · module · L176-L287 — mod tests
- write_profile · function · L180-L188 — fn write_profile(user_data: &Path, profile: &str, record: Option<Value>)
- local_state · function · L190-L200 — fn local_state(user_data: &Path, profiles: &[&str])
- reports_missing_disabled_other_folder_and_healthy_installs · function · L203-L251 — fn reports_missing_disabled_other_folder_and_healthy_installs()
- corrupt_optional_profile_files_still_inspect_default_preferences · function · L254-L266 — fn corrupt_optional_profile_files_still_inspect_default_preferences()
- missing_user_data_dir_is_not_a_diagnosis · function · L269-L275 — fn missing_user_data_dir_is_not_a_diagnosis()
- legacy_bitmask_and_state_zero_count_as_disabled · function · L278-L286 — fn legacy_bitmask_and_state_zero_count_as_disabled()

# crates/jcode-app-core/src/server/reload_exec_target.rs

- CANDIDATE_PROBE_TIMEOUT · constant · L8-L8 — const CANDIDATE_PROBE_TIMEOUT: Duration = Duration::from_secs(5);
- MAX_PROBE_BYTES · constant · L9-L9 — const MAX_PROBE_BYTES: u64 = 64 * 1024;
- probe_control_reader_version · function · L16-L56 — pub(crate) async fn probe_control_reader_version(candidate: &Path) -> Result<Option<u32>>
- ensure_candidate_control_floor · function · L63-L65 — pub(crate) async fn ensure_candidate_control_floor(candidate: &Path) -> Result<()>
- ensure_candidate_floor_in · function · L67-L77 — async fn ensure_candidate_floor_in(
- server_update_candidate · function · L79-L81 — pub(crate) fn server_update_candidate(is_selfdev_session: bool) -> Option<(PathBuf, &'static str)>
- reload_exec_target · function · L111-L186 — pub(crate) fn reload_exec_target(is_selfdev_session: bool) -> Option<(PathBuf, &'static str)>
- control_floor_tests · module · L189-L315 — mod control_floor_tests
- fake_candidate · function · L194-L203 — fn fake_candidate(dir: &Path, stdout: &str, exit: i32) -> PathBuf
- store_with_record · function · L205-L210 — fn store_with_record() -> (tempfile::TempDir, ControlStore)
- no_records_means_nothing_to_protect_and_no_probe · function · L213-L219 — async fn no_records_means_nothing_to_protect_and_no_probe()
- candidate_at_or_above_floor_is_accepted · function · L222-L233 — async fn candidate_at_or_above_floor_is_accepted()
- candidate_below_floor_or_without_version_is_refused · function · L236-L256 — async fn candidate_below_floor_or_without_version_is_refused()
- unprobeable_candidate_is_refused_fail_closed · function · L259-L271 — async fn unprobeable_candidate_is_refused_fail_closed()
- higher_record_floor_refuses_a_candidate_at_this_builds_version · function · L274-L293 — async fn higher_record_floor_refuses_a_candidate_at_this_builds_version()
- legacy_and_corrupt_records_set_the_floor_without_being_rewritten · function · L296-L314 — async fn legacy_and_corrupt_records_set_the_floor_without_being_rewritten()

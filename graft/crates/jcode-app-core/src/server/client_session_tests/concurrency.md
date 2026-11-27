# crates/jcode-app-core/src/server/client_session_tests/concurrency.rs

- IsolatedConcurrencyEnv · struct · L3-L6 — struct IsolatedConcurrencyEnv
- new · function · L9-L21 — fn new() -> Self
- drop · function · L25-L32 — fn drop(&mut self)
- restore_for_concurrency_test · function · L35-L114 — async fn restore_for_concurrency_test(
- failed_server_resume_keeps_original_concurrency_owner · function · L117-L144 — async fn failed_server_resume_keeps_original_concurrency_owner() -> Result<()>
- viewer_attach_reuses_live_owner_without_tracking_placeholder · function · L147-L183 — async fn viewer_attach_reuses_live_owner_without_tracking_placeholder() -> Result<()>
- concurrent_cold_resumes_share_one_restored_runtime · function · L186-L249 — async fn concurrent_cold_resumes_share_one_restored_runtime() -> Result<()>
- same_session_reattach_preserves_runtime_owner · function · L252-L272 — async fn same_session_reattach_preserves_runtime_owner() -> Result<()>
- resume_reservations_are_scoped_and_release_on_drop · function · L275-L293 — async fn resume_reservations_are_scoped_and_release_on_drop()
- comm_status_releases_member_guard_before_reading_connections · function · L296-L338 — async fn comm_status_releases_member_guard_before_reading_connections()

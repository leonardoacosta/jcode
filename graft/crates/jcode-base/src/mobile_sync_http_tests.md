# crates/jcode-base/src/mobile_sync_http_tests.rs

- KeepAliveFixture · struct · L6-L10 — struct KeepAliveFixture
- drop · function · L12-L14 — fn drop(&mut self)
- keep_alive_fixture · function · L17-L77 — async fn keep_alive_fixture() -> KeepAliveFixture
- body · function · L79-L90 — fn body() -> UploadBody<'static>
- benchmark_mobile_upload_keep_alive · function · L94-L110 — async fn benchmark_mobile_upload_keep_alive()
- mobile_upload_reuses_connection_after_separately_arriving_success_body · function · L113-L122 — async fn mobile_upload_reuses_connection_after_separately_arriving_success_body()
- body_fixture · function · L124-L167 — async fn body_fixture(
- mobile_upload_body_drain_is_time_bounded · function · L170-L178 — async fn mobile_upload_body_drain_is_time_bounded()
- mobile_upload_body_drain_bounds_unknown_chunked_body · function · L181-L191 — async fn mobile_upload_body_drain_bounds_unknown_chunked_body()
- mobile_upload_body_drain_preserves_exact_boundary_and_empty_success · function · L194-L202 — async fn mobile_upload_body_drain_preserves_exact_boundary_and_empty_success()
- mobile_upload_body_drain_classifies_truncation_without_error_text · function · L205-L210 — async fn mobile_upload_body_drain_classifies_truncation_without_error_text()
- mobile_upload_accepted_status_is_not_retried_for_bad_or_stalled_body · function · L213-L233 — async fn mobile_upload_accepted_status_is_not_retried_for_bad_or_stalled_body()

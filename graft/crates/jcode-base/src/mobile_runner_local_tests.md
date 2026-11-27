# crates/jcode-base/src/mobile_runner_local_tests.rs

- Request · struct · L7-L12 — struct Request
- api · function · L15-L69 — async fn api(
- FakeHost · struct · L72-L76 — struct FakeHost
- live_sessions · function · L79-L81 — fn live_sessions(&self) -> BoxFuture<'_, Vec<LiveSession>>
- deliver · function · L82-L93 — fn deliver<'a>(&'a self, session_id: &'a str, text: &'a str) -> BoxFuture<'a, Result<()>>
- live · function · L96-L101 — fn live(id: &str, busy: bool) -> LiveSession
- message · function · L103-L106 — fn message(session: &str, text: &str) -> serde_json::Value
- assert_legacy_claim_bytes · function · L108-L127 — async fn assert_legacy_claim_bytes(machine: &str, sessions: &[LiveSession])
- claim_bytes_match_legacy_empty_sessions · function · L130-L141 — async fn claim_bytes_match_legacy_empty_sessions()
- claim_bytes_match_legacy_unicode_and_escaping · function · L144-L150 — async fn claim_bytes_match_legacy_unicode_and_escaping()
- claim_bytes_match_legacy_ninety_sessions · function · L153-L159 — async fn claim_bytes_match_legacy_ninety_sessions()
- reports_live_sessions_and_delivers_the_claimed_message_once · function · L162-L206 — async fn reports_live_sessions_and_delivers_the_claimed_message_once()
- idle_heartbeat_claims_nothing_and_no_live_sessions_means_no_request · function · L209-L231 — async fn idle_heartbeat_claims_nothing_and_no_live_sessions_means_no_request()
- failed_delivery_is_reported_instead_of_dropped · function · L234-L255 — async fn failed_delivery_is_reported_instead_of_dropped()
- photo_validation_and_unsupported_host_fail_explicitly_without_text_delivery · function · L258-L287 — async fn photo_validation_and_unsupported_host_fail_explicitly_without_text_delivery()
- refuses_commands_for_other_sessions_and_other_kinds · function · L290-L319 — async fn refuses_commands_for_other_sessions_and_other_kinds()
- api_errors_surface_without_delivering · function · L322-L338 — async fn api_errors_surface_without_delivering()
- report_keeps_busy_sessions_under_the_cap_and_drops_invalid_ids · function · L341-L360 — fn report_keeps_busy_sessions_under_the_cap_and_drops_invalid_ids()

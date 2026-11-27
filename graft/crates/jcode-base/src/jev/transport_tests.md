# crates/jcode-base/src/jev/transport_tests.rs

- mock_server_does_not_count_local_discovery_probes_as_requests · function · L4-L50 — async fn mock_server_does_not_count_local_discovery_probes_as_requests()
- live_typesafe_jev_returns_expected_choice · function · L54-L90 — async fn live_typesafe_jev_returns_expected_choice() -> Result<()>
- browser_subscription_checks_browser_capability_and_posts_choice · function · L93-L129 — async fn browser_subscription_checks_browser_capability_and_posts_choice()
- browser_capability_denial_prevents_page_upload · function · L132-L151 — async fn browser_capability_denial_prevents_page_upload()
- browser_auth_billing_and_redirect_failures_never_retry_or_fallback · function · L154-L199 — async fn browser_auth_billing_and_redirect_failures_never_retry_or_fallback()
- subscription_requires_live_capability_then_sends_bound_bearer · function · L202-L225 — async fn subscription_requires_live_capability_then_sends_bound_bearer()
- missing_or_false_capability_never_posts_decisions · function · L228-L243 — async fn missing_or_false_capability_never_posts_decisions()
- transient_overload_is_retried_on_same_route_then_succeeds · function · L246-L268 — async fn transient_overload_is_retried_on_same_route_then_succeeds()
- gateway_plan_quota_is_not_retried_and_becomes_upgrade_prompt · function · L271-L302 — async fn gateway_plan_quota_is_not_retried_and_becomes_upgrade_prompt()
- gateway_quota_rejects_non_jcode_upgrade_links · function · L305-L326 — async fn gateway_quota_rejects_non_jcode_upgrade_links()
- persistent_overload_fails_after_bounded_retries_without_echo · function · L329-L358 — async fn persistent_overload_fails_after_bounded_retries_without_echo()
- auth_billing_and_redirect_errors_are_redacted_and_never_retried · function · L361-L381 — async fn auth_billing_and_redirect_errors_are_redacted_and_never_retried()
- oversized_and_invalid_json_responses_are_rejected_without_echo · function · L384-L398 — async fn oversized_and_invalid_json_responses_are_rejected_without_echo()
- streamed_body_limit_is_enforced_without_content_length · function · L401-L414 — async fn streamed_body_limit_is_enforced_without_content_length()
- failed_preflight_does_not_send_decision_request · function · L417-L438 — async fn failed_preflight_does_not_send_decision_request()
- laya_local_and_private_requests_bypass_http_proxy_including_voice_hedges · function · L441-L513 — async fn laya_local_and_private_requests_bypass_http_proxy_including_voice_hedges()

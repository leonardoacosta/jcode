# crates/jcode-base/src/subscription_api_tests.rs

- CannedResponse · type · L5-L5 — type CannedResponse = (u16, Vec<(&'static str, &'static str)>, String);
- spawn_server · function · L7-L27 — fn spawn_server(responses: Vec<CannedResponse>) -> String
- client · function · L29-L34 — fn client() -> reqwest::Client
- subscription_me_parses_expected_shape · function · L37-L50 — fn subscription_me_parses_expected_shape()
- metered_account_parses_spending_limit_and_tranche_state_without_a_tier · function · L53-L70 — fn metered_account_parses_spending_limit_and_tranche_state_without_a_tier()
- polling_backoff_is_deterministic_and_bounded · function · L73-L86 — fn polling_backoff_is_deterministic_and_bounded()
- device_request_uses_new_contract_and_parses_public_urls · function · L89-L101 — async fn device_request_uses_new_contract_and_parses_public_urls()
- metered_device_request_does_not_select_a_subscription_tier · function · L104-L129 — async fn metered_device_request_does_not_select_a_subscription_tier()
- legacy_device_response_is_explained_without_echoing_body · function · L132-L143 — async fn legacy_device_response_is_explained_without_echoing_body()
- token_poll_handles_pending_slow_down_success_denied_and_replay · function · L146-L187 — async fn token_poll_handles_pending_slow_down_success_denied_and_replay()
- me_and_revoke_classify_revoked_keys_without_leaking_them · function · L190-L209 — async fn me_and_revoke_classify_revoked_keys_without_leaking_them()
- billing_cap_and_portal_parse_and_reject_unsafe_urls · function · L212-L262 — async fn billing_cap_and_portal_parse_and_reject_unsafe_urls()
- checkout_and_plan_change_only_accept_safe_replies · function · L265-L307 — async fn checkout_and_plan_change_only_accept_safe_replies()

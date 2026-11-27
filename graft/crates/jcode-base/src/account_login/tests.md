# crates/jcode-base/src/account_login/tests.rs

- DEVICE · constant · L4-L4 — const DEVICE: &str = r#"{"device_code":"fixture-device-secret","flow_id":"public-flow","verification_uri":"https://jcode.sh/account","verification_uri_complete":"https://jcode.sh/account?flow=public-flow","expires_in":600,"interval":3}"#;
- APPROVED · constant · L5-L5 — const APPROVED: &str = r#"{"api_key":"fixture-account-secret","account_id":"acct_fixture","email":"fixture@example.invalid","tier":"none","status":"inactive"}"#;
- client · function · L7-L9 — fn client() -> reqwest::Client
- server · function · L11-L49 — fn server(
- account_only_start_and_approval_use_expected_wire_contract · function · L52-L82 — async fn account_only_start_and_approval_use_expected_wire_contract()
- pending_slowdown_denial_and_expiry_are_separate_results · function · L85-L120 — async fn pending_slowdown_denial_and_expiry_are_separate_results()
- rejects_unsafe_browser_urls · function · L123-L142 — fn rejects_unsafe_browser_urls()
- malicious_error_bodies_and_malformed_responses_are_redacted · function · L145-L163 — async fn malicious_error_bodies_and_malformed_responses_are_redacted()
- EnvGuard · struct · L167-L167 — struct EnvGuard(Vec<(&'static str, Option<std::ffi::OsString>)>);
- new · function · L169-L188 — fn new() -> Self
- drop · function · L191-L198 — fn drop(&mut self)
- explicit_save_and_current_account_are_sandboxed_without_paid_plan · function · L202-L263 — async fn explicit_save_and_current_account_are_sandboxed_without_paid_plan()
- in_flight_poll_is_cancellable_without_a_detached_worker · function · L266-L303 — async fn in_flight_poll_is_cancellable_without_a_detached_worker()
- deployed_account_start_and_pending_are_compatible · function · L309-L320 — async fn deployed_account_start_and_pending_are_compatible()
- email_code_sign_in_uses_expected_wire_contract · function · L323-L373 — async fn email_code_sign_in_uses_expected_wire_contract()
- gmail_link_searches_for_our_sender_including_spam · function · L376-L381 — fn gmail_link_searches_for_our_sender_including_spam()

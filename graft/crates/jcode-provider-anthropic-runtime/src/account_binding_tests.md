# crates/jcode-provider-anthropic-runtime/src/account_binding_tests.rs

- AccountSandbox · struct · L3-L8 — struct AccountSandbox
- new · function · L11-L25 — fn new() -> Self
- drop · function · L29-L40 — fn drop(&mut self)
- save_account · function · L43-L54 — fn save_account(label: &str, token: &str, refresh: &str) -> String
- accepted_refresh_commits_disk_and_memory_after_caller_cancellation · function · L58-L119 — async fn accepted_refresh_commits_disk_and_memory_after_caller_cancellation()
- cloud_import_replaces_live_daemon_cached_bearer · function · L123-L138 — async fn cloud_import_replaces_live_daemon_cached_bearer()
- auth_rejection_refreshes_once_without_route_or_identity_substitution · function · L141-L283 — fn auth_rejection_refreshes_once_without_route_or_identity_substitution()
- active_oauth_email_follows_cached_unbound_credentials_not_global_account · function · L287-L326 — async fn active_oauth_email_follows_cached_unbound_credentials_not_global_account()
- active_oauth_email_is_known_right_after_binding_an_account · function · L331-L351 — fn active_oauth_email_is_known_right_after_binding_an_account()
- oauth_account_binding_survives_global_switch_refresh_and_fork · function · L355-L436 — async fn oauth_account_binding_survives_global_switch_refresh_and_fork()

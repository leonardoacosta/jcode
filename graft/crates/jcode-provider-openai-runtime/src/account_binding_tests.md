# crates/jcode-provider-openai-runtime/src/account_binding_tests.rs

- AccountSandbox · struct · L3-L9 — struct AccountSandbox
- new · function · L12-L28 — fn new() -> Self
- drop · function · L32-L44 — fn drop(&mut self)
- save_account · function · L47-L58 — fn save_account(label: &str, token: &str, refresh: &str, identity: &str) -> String
- cloud_import_replaces_live_daemon_cached_bearer · function · L61-L77 — async fn cloud_import_replaces_live_daemon_cached_bearer()
- service_tier_catalog_is_scoped_across_account_mode_reload_and_fork · function · L80-L144 — async fn service_tier_catalog_is_scoped_across_account_mode_reload_and_fork()
- unbound_catalog_follows_loaded_credentials_across_global_switch_reload_and_fork · function · L147-L193 — async fn unbound_catalog_follows_loaded_credentials_across_global_switch_reload_and_fork()
- active_oauth_email_follows_cached_unbound_credentials_not_global_account · function · L196-L250 — async fn active_oauth_email_follows_cached_unbound_credentials_not_global_account()
- oauth_account_binding_survives_global_switch_refresh_and_fork · function · L253-L382 — async fn oauth_account_binding_survives_global_switch_refresh_and_fork()

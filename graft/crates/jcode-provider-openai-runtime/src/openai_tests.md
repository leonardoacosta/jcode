# crates/jcode-provider-openai-runtime/src/openai_tests.rs

- BRIGHT_PEARL_WRAPPED_TOOL_CALL_FIXTURE · constant · L13-L16 — const BRIGHT_PEARL_WRAPPED_TOOL_CALL_FIXTURE: &str = include_str!(concat!(
- EnvVarGuard · struct · L18-L21 — struct EnvVarGuard
- set · function · L24-L28 — fn set(key: &'static str, value: &str) -> Self
- set_path · function · L30-L34 — fn set_path(key: &'static str, value: &std::path::Path) -> Self
- remove · function · L36-L40 — fn remove(key: &'static str) -> Self
- drop · function · L44-L50 — fn drop(&mut self)
- test_persistent_ws_state · function · L53-L99 — pub(super) async fn test_persistent_ws_state() -> (PersistentWsState, tokio::task::JoinHandle<()>)
- test_persistent_ws_state_with_ping_notify · function · L101-L164 — async fn test_persistent_ws_state_with_ping_notify() -> (
- LiveOpenAITestEnv · struct · L166-L171 — struct LiveOpenAITestEnv
- new · function · L174-L204 — fn new() -> Result<Option<Self>>
- real_codex_auth_path · function · L207-L211 — fn real_codex_auth_path() -> Option<PathBuf>
- live_openai_catalog · function · L213-L226 — async fn live_openai_catalog() -> Result<Option<jcode_base::provider::OpenAIModelCatalog>>
- live_openai_smoke · function · L228-L243 — async fn live_openai_smoke(model: &str, sentinel: &str) -> Result<Option<String>>
- openai_credential_mode_runtime_provider_identity_round_trips · function · L257-L279 — fn openai_credential_mode_runtime_provider_identity_round_trips()
- openai_available_efforts_follow_active_model_catalog_metadata · function · L282-L354 — async fn openai_available_efforts_follow_active_model_catalog_metadata()
- catalog_credential_identity_survives_token_refresh_but_changes_accounts · function · L357-L373 — fn catalog_credential_identity_survives_token_refresh_but_changes_accounts()

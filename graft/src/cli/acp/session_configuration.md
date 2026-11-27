# src/cli/acp/session_configuration.rs

- CONFIG_ID_MODEL · constant · L5-L5 — pub(super) const CONFIG_ID_MODEL: &str = "model";
- CONFIG_ID_EFFORT · constant · L6-L6 — pub(super) const CONFIG_ID_EFFORT: &str = "reasoning_effort";
- acp_available_commands · function · L8-L25 — pub(super) fn acp_available_commands() -> Vec<Value>
- insert_session_configuration · function · L27-L38 — pub(super) fn insert_session_configuration(result: &mut Value, state: &SessionUiState)
- session_models · function · L40-L53 — pub(super) fn session_models(state: &SessionUiState) -> Option<Value>
- available_efforts · function · L55-L64 — pub(super) fn available_efforts(state: &SessionUiState) -> Vec<&'static str>
- session_config_options · function · L69-L119 — pub(super) fn session_config_options(state: &SessionUiState) -> Vec<Value>

# crates/jcode-base/src/auth/gemini_tests.rs

- GeminiOauthEnvReset · struct · L4-L7 — struct GeminiOauthEnvReset
- drop · function · L10-L22 — fn drop(&mut self)
- set_test_gemini_oauth_env · function · L25-L36 — fn set_test_gemini_oauth_env() -> GeminiOauthEnvReset
- parses_env_command_with_args · function · L39-L53 — fn parses_env_command_with_args()
- falls_back_to_gemini_binary_when_available · function · L56-L60 — fn falls_back_to_gemini_binary_when_available()
- falls_back_to_npx_when_gemini_binary_missing · function · L63-L67 — fn falls_back_to_npx_when_gemini_binary_missing()
- display_includes_args_when_present · function · L70-L76 — fn display_includes_args_when_present()
- build_manual_auth_url_contains_expected_redirect_uri · function · L79-L87 — fn build_manual_auth_url_contains_expected_redirect_uri()
- build_web_auth_url_includes_pkce_parameters · function · L90-L104 — fn build_web_auth_url_includes_pkce_parameters()
- resolve_callback_or_manual_code_accepts_manual_code_with_expected_state · function · L107-L111 — fn resolve_callback_or_manual_code_accepts_manual_code_with_expected_state()
- resolve_callback_or_manual_code_validates_state_for_callback_input · function · L114-L128 — fn resolve_callback_or_manual_code_validates_state_for_callback_input()
- uses_hardcoded_credentials_when_env_missing · function · L131-L145 — fn uses_hardcoded_credentials_when_env_missing()
- imports_cli_oauth_tokens_when_native_tokens_missing · function · L148-L177 — fn imports_cli_oauth_tokens_when_native_tokens_missing()
- imports_cli_oauth_tokens_without_changing_external_permissions · function · L181-L230 — fn imports_cli_oauth_tokens_without_changing_external_permissions()

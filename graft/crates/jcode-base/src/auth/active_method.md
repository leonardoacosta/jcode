# crates/jcode-base/src/auth/active_method.rs

- ActiveCredential · enum · L21-L26 — pub enum ActiveCredential
- from · function · L29-L34 — fn from(value: jcode_provider_core::ResolvedCredential) -> Self
- ResolvedProviderAuth · struct · L40-L51 — pub struct ResolvedProviderAuth
- has_both · function · L56-L58 — pub fn has_both(&self) -> bool
- resolve_dual_credential_auth · function · L70-L118 — pub fn resolve_dual_credential_auth(
- tests · module · L121-L197 — mod tests
- anthropic · function · L125-L134 — fn anthropic(has_oauth: bool, has_api_key: bool) -> AuthStatus
- anthropic_auto_prefers_oauth_when_both_present · function · L137-L142 — fn anthropic_auto_prefers_oauth_when_both_present()
- anthropic_auto_falls_back_to_api_key · function · L145-L150 — fn anthropic_auto_falls_back_to_api_key()
- anthropic_explicit_selection_wins_over_auto · function · L153-L164 — fn anthropic_explicit_selection_wins_over_auto()
- auto_resolution_is_not_marked_explicit · function · L167-L171 — fn auto_resolution_is_not_marked_explicit()
- anthropic_none_when_unconfigured · function · L174-L177 — fn anthropic_none_when_unconfigured()
- openai_auto_reports_both_and_prefers_oauth · function · L180-L189 — fn openai_auto_reports_both_and_prefers_oauth()
- non_dual_providers_return_none · function · L192-L196 — fn non_dual_providers_return_none()

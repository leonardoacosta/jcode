# crates/jcode-base/src/auth/login_diagnostics.rs

- AuthFailureReason · enum · L2-L16 — pub enum AuthFailureReason
- label · function · L19-L35 — pub fn label(self) -> &'static str
- classify_auth_failure_message · function · L38-L92 — pub fn classify_auth_failure_message(message: &str) -> AuthFailureReason
- auth_failure_recovery_hint · function · L94-L141 — pub fn auth_failure_recovery_hint(provider_id: &str, reason: AuthFailureReason) -> Option<String>
- augment_auth_error_message · function · L143-L151 — pub fn augment_auth_error_message(provider_id: &str, message: impl AsRef<str>) -> String
- tests · module · L154-L190 — mod tests
- classifies_callback_timeout · function · L158-L163 — fn classifies_callback_timeout()
- classifies_validation_failure · function · L166-L171 — fn classifies_validation_failure()
- classifies_oauth_rate_limit · function · L174-L181 — fn classifies_oauth_rate_limit()
- augments_message_with_next_step · function · L184-L189 — fn augments_message_with_next_step()

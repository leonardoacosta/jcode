# crates/jcode-tui/src/tui/app/support.rs

- SUPPORT_EMAIL · constant · L11-L11 — pub(super) const SUPPORT_EMAIL: &str = "support@jcode.sh";
- SUPPORT_SUBJECT · constant · L12-L12 — pub(super) const SUPPORT_SUBJECT: &str = "jcode support";
- SupportDiagnostics · struct · L17-L30 — pub(super) struct SupportDiagnostics
- build_support_body · function · L33-L58 — pub(super) fn build_support_body(d: &SupportDiagnostics) -> String
- build_mailto_url · function · L61-L68 — pub(super) fn build_mailto_url(body: &str) -> String
- build_channel · function · L70-L87 — fn build_channel() -> String
- read_telemetry_id · function · L91-L96 — fn read_telemetry_id() -> Option<String>
- gather_diagnostics · function · L98-L150 — fn gather_diagnostics(app: &App) -> SupportDiagnostics
- MAX_ERROR_LEN · constant · L125-L125 — const MAX_ERROR_LEN: usize = 500;
- handle_support_command · function · L152-L186 — pub(super) fn handle_support_command(app: &mut App, trimmed: &str) -> bool
- tests · module · L189-L270 — mod tests
- sample_diagnostics · function · L192-L207 — fn sample_diagnostics() -> SupportDiagnostics
- ssh_support_omits_laptop_identity · function · L210-L222 — fn ssh_support_omits_laptop_identity()
- support_body_includes_all_diagnostics · function · L225-L239 — fn support_body_includes_all_diagnostics()
- support_body_skips_optional_fields_when_absent · function · L242-L259 — fn support_body_skips_optional_fields_when_absent()
- support_mailto_url_is_correctly_encoded · function · L262-L269 — fn support_mailto_url_is_correctly_encoded()

# crates/jcode-app-core/src/tool/computer/osa.rs

- DEFAULT_TIMEOUT · constant · L19-L19 — pub const DEFAULT_TIMEOUT: Duration = Duration::from_secs(20);
- run_applescript · function · L23-L25 — pub fn run_applescript(script: &str) -> Result<String>
- run_applescript_timeout · function · L28-L30 — pub fn run_applescript_timeout(script: &str, timeout: Duration) -> Result<String>
- run_jxa · function · L33-L35 — pub fn run_jxa(script: &str) -> Result<String>
- run · function · L37-L45 — fn run(args: &[&str], lang: &str, timeout: Duration) -> Result<String>
- classify_osa_error · function · L51-L88 — fn classify_osa_error(trimmed: &str, lang: &str) -> String
- run_command_timed · function · L92-L133 — pub fn run_command_timed(
- as_quote · function · L138-L150 — pub fn as_quote(s: &str) -> String
- tests · module · L153-L225 — mod tests
- quotes_and_escapes · function · L157-L161 — fn quotes_and_escapes()
- timed_command_succeeds_fast · function · L164-L168 — fn timed_command_succeeds_fast()
- timed_command_times_out · function · L171-L176 — fn timed_command_times_out()
- classifies_permission_error · function · L179-L188 — fn classifies_permission_error()
- classifies_invalid_index_as_not_found_not_permission · function · L191-L200 — fn classifies_invalid_index_as_not_found_not_permission()
- classifies_no_such_object_as_not_found · function · L203-L209 — fn classifies_no_such_object_as_not_found()
- classifies_automation_error · function · L212-L218 — fn classifies_automation_error()
- classifies_generic_error · function · L221-L224 — fn classifies_generic_error()

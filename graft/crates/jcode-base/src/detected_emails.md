# crates/jcode-base/src/detected_emails.rs

- DetectedEmail · struct · L14-L17 — pub struct DetectedEmail
- detected_emails · function · L24-L73 — pub fn detected_emails() -> Vec<DetectedEmail>
- read_json · function · L75-L77 — fn read_json(path: &Path) -> Option<Value>
- codex_email · function · L79-L82 — fn codex_email(auth: &Value) -> Option<String>
- claude_code_email · function · L84-L91 — fn claude_code_email(config: &Value) -> Option<String>
- gemini_email · function · L93-L95 — fn gemini_email(accounts: &Value) -> Option<String>
- cursor_cli_email · function · L97-L99 — fn cursor_cli_email(config: &Value) -> Option<String>
- top_level_email · function · L101-L103 — fn top_level_email(tokens: &Value) -> Option<String>
- account_list_emails · function · L106-L112 — fn account_list_emails(store: &Value, list: &str) -> Vec<String>
- collect · function · L114-L136 — fn collect(found: impl IntoIterator<Item = (&'static str, Option<String>)>) -> Vec<DetectedEmail>
- plausible · function · L138-L143 — fn plausible(email: &str) -> bool
- tests · module · L146-L207 — mod tests
- jwt · function · L151-L155 — fn jwt(payload: Value) -> String
- reads_each_tools_layout · function · L158-L182 — fn reads_each_tools_layout()
- merges_case_insensitive_duplicates_and_drops_junk · function · L185-L206 — fn merges_case_insensitive_duplicates_and_drops_junk()

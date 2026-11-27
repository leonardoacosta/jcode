# crates/jcode-app-core/src/ambient/directives.rs

- UserDirective · struct · L15-L21 — pub struct UserDirective
- directives_path · function · L23-L25 — fn directives_path() -> Result<PathBuf>
- load_directives · function · L27-L38 — pub fn load_directives() -> Vec<UserDirective>
- save_directives · function · L40-L42 — fn save_directives(directives: &[UserDirective]) -> Result<()>
- add_directive · function · L45-L55 — pub fn add_directive(text: String, in_reply_to: String) -> Result<()>
- take_pending_directives · function · L58-L71 — pub fn take_pending_directives() -> Vec<UserDirective>
- has_pending_directives · function · L74-L76 — pub fn has_pending_directives() -> bool

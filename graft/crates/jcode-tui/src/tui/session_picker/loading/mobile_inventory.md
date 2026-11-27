# crates/jcode-tui/src/tui/session_picker/loading/mobile_inventory.rs

- VerifiedInventory · struct · L6-L9 — struct VerifiedInventory
- VerifiedSession · struct · L12-L18 — struct VerifiedSession
- available · function · L21-L25 — pub(super) fn available() -> bool
- refresh · function · L27-L92 — pub(super) fn refresh() -> Result<bool>
- valid_id · function · L94-L100 — fn valid_id(id: &str) -> bool
- should_verify · function · L102-L116 — fn should_verify(home: &Path, id: &str) -> bool
- prepare_ssh_config · function · L118-L145 — async fn prepare_ssh_config(config: &Path) -> Result<()>
- register · function · L147-L208 — fn register(
- title · function · L210-L216 — pub(super) fn title(id: &str) -> Option<String>
- tests · module · L220-L220 — mod tests;

# crates/jcode-todo-policy/src/await_dependency.rs

- ID_PREFIX · constant · L6-L6 — pub const ID_PREFIX: &str = "await-";
- STATE_DIR · constant · L7-L7 — pub const STATE_DIR: &str = "jcode-await-members";
- AwaitDependency · struct · L10-L17 — pub struct AwaitDependency
- id · function · L20-L22 — pub fn id(&self) -> String
- is_running_at · function · L24-L26 — pub fn is_running_at(&self, now_ms: u64) -> bool
- is_running · function · L28-L30 — pub fn is_running(&self) -> bool
- now_unix_ms · function · L33-L46 — pub fn now_unix_ms() -> u64
- WARNING · constant · L37-L37 — static WARNING: std::sync::Once = std::sync::Once::new();
- dependency_id · function · L50-L52 — pub fn dependency_id(key: &str, created_at_unix_ms: u64) -> String
- load · function · L54-L63 — pub fn load(runtime_dir: &Path, id: &str) -> Option<AwaitDependency>
- WARNING · constant · L58-L58 — static WARNING: std::sync::Once = std::sync::Once::new();
- load_checked · function · L67-L91 — pub fn load_checked(runtime_dir: &Path, id: &str) -> std::io::Result<Option<AwaitDependency>>
- running · function · L93-L95 — pub fn running(runtime_dir: &Path, id: &str, session_id: &str) -> bool
- tests · module · L98-L157 — mod tests
- strict_reads_distinguish_missing_invalid_and_unreadable_state · function · L102-L117 — fn strict_reads_distinguish_missing_invalid_and_unreadable_state()
- durable_dependency_validates_identity_generation_owner_and_deadline · function · L120-L156 — fn durable_dependency_validates_identity_generation_owner_and_deadline()

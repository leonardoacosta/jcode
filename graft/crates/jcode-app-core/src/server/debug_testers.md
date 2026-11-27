# crates/jcode-app-core/src/server/debug_testers.rs

- execute_tester_command · function · L5-L37 — pub(super) async fn execute_tester_command(command: &str) -> Result<String>
- load_testers · function · L39-L50 — fn load_testers() -> Result<Vec<serde_json::Value>>
- save_testers · function · L52-L56 — fn save_testers(testers: &[serde_json::Value]) -> Result<()>
- spawn_tester · function · L58-L202 — async fn spawn_tester(opts: serde_json::Value) -> Result<String>
- TesterPty · struct · L206-L209 — struct TesterPty
- allocate_pty · function · L219-L250 — fn allocate_pty(cols: u16, rows: u16) -> std::io::Result<TesterPty>
- execute_tester_subcommand · function · L252-L331 — async fn execute_tester_subcommand(
- debug_testers_tests · module · L335-L335 — mod debug_testers_tests;

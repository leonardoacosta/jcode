# crates/jcode-app-core/src/tool/computer/sys.rs

- get_clipboard · function · L11-L18 — pub fn get_clipboard() -> Result<ToolOutput>
- set_clipboard · function · L20-L38 — pub fn set_clipboard(text: &str) -> Result<ToolOutput>
- run_applescript · function · L40-L48 — pub fn run_applescript(script: &str) -> Result<ToolOutput>
- run_jxa · function · L50-L58 — pub fn run_jxa(script: &str) -> Result<ToolOutput>
- notify · function · L60-L68 — pub fn notify(text: &str, title: Option<&str>) -> Result<ToolOutput>
- wait_for · function · L72-L128 — pub fn wait_for(app: &str, contains: &str, timeout_ms: u64) -> Result<ToolOutput>
- system_state · function · L131-L146 — pub fn system_state() -> Result<ToolOutput>
- set_brightness · function · L151-L170 — pub fn set_brightness(level: f64) -> Result<ToolOutput>

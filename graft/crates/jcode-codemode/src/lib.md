# crates/jcode-codemode/src/lib.rs

- engine · module · L24-L24 — mod engine;
- AVAILABLE · constant · L28-L28 — pub const AVAILABLE: bool = cfg!(not(target_os = "freebsd"));
- ToolInfo · struct · L32-L36 — pub struct ToolInfo
- Host · interface · L39-L44 — pub trait Host: Send + Sync + 'static
- call_tool · function · L43-L43 — fn call_tool(&self, name: String, args: Value) -> BoxFuture<'static, Result<Value, String>>;
- Limits · struct · L48-L57 — pub struct Limits
- default · function · L60-L67 — fn default() -> Self
- RunOutput · struct · L72-L87 — pub struct RunOutput
- run · function · L94-L102 — pub async fn run(
- run · function · L107-L119 — pub async fn run(
- decode_tool_output · function · L123-L132 — pub fn decode_tool_output(text: &str) -> Value
- tests · module · L135-L135 — mod tests;

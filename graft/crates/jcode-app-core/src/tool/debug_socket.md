# crates/jcode-app-core/src/tool/debug_socket.rs

- next_debug_request_id · function · L18-L23 — fn next_debug_request_id() -> u64
- NEXT_ID · constant · L19-L19 — static NEXT_ID: OnceLock<AtomicU64> = OnceLock::new();
- DebugSocketInput · struct · L26-L32 — struct DebugSocketInput
- DebugSocketTool · struct · L34-L34 — pub struct DebugSocketTool;
- new · function · L37-L39 — pub fn new() -> Self
- name · function · L44-L46 — fn name(&self) -> &str
- description · function · L48-L50 — fn description(&self) -> &str
- parameters_schema · function · L52-L72 — fn parameters_schema(&self) -> Value
- execute · function · L74-L102 — async fn execute(&self, input: Value, _ctx: ToolContext) -> Result<ToolOutput>
- execute_debug_command · function · L106-L167 — async fn execute_debug_command(

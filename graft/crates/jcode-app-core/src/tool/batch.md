# crates/jcode-app-core/src/tool/batch.rs

- MAX_PARALLEL · constant · L10-L10 — const MAX_PARALLEL: usize = 10;
- BATCH_DESCRIPTION · constant · L12-L12 — const BATCH_DESCRIPTION: &str = "Run independent tool calls in parallel (1-10). Each tool_calls entry contains tool, intent, and the tool's normal arguments as sibling fields. Do not batch dependent calls or nest batch.";
- generic_batch_schema · function · L14-L39 — pub(crate) fn generic_batch_schema() -> Value
- ordered_batch_subcalls · function · L41-L73 — fn ordered_batch_subcalls(
- BatchTool · struct · L75-L77 — pub struct BatchTool
- new · function · L80-L82 — pub(super) fn new(registry: WeakRegistry) -> Self
- BatchInput · struct · L86-L88 — struct BatchInput
- ToolCallInput · struct · L91-L96 — struct ToolCallInput
- resolved_parameters · function · L99-L104 — fn resolved_parameters(self) -> (String, Value)
- normalize_batch_input · function · L111-L181 — fn normalize_batch_input(mut input: Value) -> Value
- name · function · L185-L187 — fn name(&self) -> &str
- description · function · L189-L191 — fn description(&self) -> &str
- parameters_schema · function · L193-L195 — fn parameters_schema(&self) -> Value
- execute · function · L197-L372 — async fn execute(&self, input: Value, ctx: ToolContext) -> Result<ToolOutput>
- batch_tests · module · L377-L377 — mod batch_tests;

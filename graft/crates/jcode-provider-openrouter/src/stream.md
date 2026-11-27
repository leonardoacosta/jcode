# crates/jcode-provider-openrouter/src/stream.rs

- truncated_stream_payload_context · function · L14-L16 — fn truncated_stream_payload_context(data: &str) -> String
- take_sse_event · function · L23-L35 — fn take_sse_event(buffer: &mut String) -> Option<String>
- sends_cumulative_reasoning_snapshots · function · L43-L45 — fn sends_cumulative_reasoning_snapshots(model: &str) -> bool
- completion_snapshot_delta · function · L48-L56 — fn completion_snapshot_delta<'a>(emitted: &str, snapshot: &'a str) -> Result<&'a str>
- OpenRouterStream · struct · L58-L81 — pub struct OpenRouterStream
- ToolCallAccumulator · struct · L84-L92 — struct ToolCallAccumulator
- new · function · L95-L118 — pub fn new(
- queue_message_end · function · L120-L130 — fn queue_message_end(&mut self)
- observe_provider · function · L132-L156 — fn observe_provider(&mut self, provider: &str)
- refresh_cache_pin · function · L158-L169 — fn refresh_cache_pin(&mut self, provider: &str)
- push_completed_tool_call · function · L171-L202 — fn push_completed_tool_call(&mut self, index: u64, mut tc: ToolCallAccumulator)
- queue_tool_progress · function · L204-L232 — fn queue_tool_progress(
- flush_tool_call_accumulators · function · L234-L239 — fn flush_tool_call_accumulators(&mut self)
- apply_tool_call_delta · function · L241-L291 — fn apply_tool_call_delta(
- parse_next_event · function · L293-L604 — fn parse_next_event(&mut self) -> Option<StreamEvent>
- Item · type · L608-L608 — type Item = Result<StreamEvent>;
- poll_next · function · L610-L655 — fn poll_next(mut self: Pin<&mut Self>, cx: &mut TaskContext<'_>) -> Poll<Option<Self::Item>>
- tests · module · L660-L660 — mod tests;

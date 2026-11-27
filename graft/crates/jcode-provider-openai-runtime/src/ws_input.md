# crates/jcode-provider-openai-runtime/src/ws_input.rs

- WsInputStats · struct · L6-L12 — pub(super) struct WsInputStats
- tool_callback_count · function · L15-L17 — pub(super) fn tool_callback_count(self) -> usize
- log_fields · function · L19-L28 — pub(super) fn log_fields(self) -> String
- summarize_ws_input · function · L31-L43 — pub(super) fn summarize_ws_input(items: &[Value]) -> WsInputStats
- persistent_ws_input_item_hashes · function · L45-L50 — pub(super) fn persistent_ws_input_item_hashes(input: &[Value]) -> Vec<u64>
- replay_identity · function · L54-L99 — fn replay_identity(item: &Value) -> Value
- log_ws_replayed_calls · function · L102-L133 — pub(super) fn log_ws_replayed_calls(items: &[Value], previous_output: &[Value])
- observe_ws_response_output · function · L141-L151 — pub(super) fn observe_ws_response_output(text: &str, output: &mut Vec<Value>)
- observe_ws_response_output_event · function · L153-L183 — pub(super) fn observe_ws_response_output_event(event: &Value, output: &mut Vec<Value>)
- persistent_ws_incremental_items · function · L185-L212 — pub(super) fn persistent_ws_incremental_items(

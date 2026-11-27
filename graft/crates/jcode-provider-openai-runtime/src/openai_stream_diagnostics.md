# crates/jcode-provider-openai-runtime/src/openai_stream_diagnostics.rs

- add_cache_comparison · function · L15-L35 — pub(super) fn add_cache_comparison(request: &mut Value, chatgpt: bool, baseline: &str)
- is_output_event · function · L38-L49 — fn is_output_event(event: &StreamEvent) -> bool
- DIAGNOSTIC_HEADERS · constant · L54-L60 — const DIAGNOSTIC_HEADERS: &[(&str, &str)] = &[
- MAX_HEADER_VALUE_LEN · constant · L64-L64 — const MAX_HEADER_VALUE_LEN: usize = 128;
- response_header_fields · function · L67-L90 — pub(super) fn response_header_fields(
- reqwest_header_fields · function · L92-L101 — pub(super) fn reqwest_header_fields(
- tungstenite_header_fields · function · L103-L113 — pub(super) fn tungstenite_header_fields(
- StreamProgress · struct · L117-L123 — pub(super) struct StreamProgress
- new · function · L126-L134 — pub(super) fn new() -> Self
- record_event · function · L136-L138 — pub(super) fn record_event(&mut self, event: &StreamEvent)
- record · function · L140-L148 — pub(super) fn record(&mut self, output: bool)
- fields · function · L152-L169 — pub(super) fn fields(&self) -> Vec<(&'static str, String)>
- tests · module · L173-L265 — mod tests
- cache_comparison_is_api_modern_model_only_and_preserves_options · function · L177-L214 — fn cache_comparison_is_api_modern_model_only_and_preserves_options()
- header_fields_are_allowlisted_and_sanitized · function · L217-L233 — fn header_fields_are_allowlisted_and_sanitized()
- header_fields_truncate_oversized_values · function · L236-L240 — fn header_fields_truncate_oversized_values()
- turn_state_header_value_is_never_logged · function · L243-L248 — fn turn_state_header_value_is_never_logged()
- progress_distinguishes_no_events_from_mid_stream_stall · function · L251-L264 — fn progress_distinguishes_no_events_from_mid_stream_stall()

# crates/jcode-tui/src/tui/app/response_output.rs

- ResponseOutput · struct · L4-L8 — pub(super) struct ResponseOutput
- reset · function · L11-L13 — pub(super) fn reset(&mut self)
- text · function · L15-L17 — pub(super) fn text(&mut self, text: &str)
- replace_text · function · L19-L21 — pub(super) fn replace_text(&mut self, text: &str)
- tool · function · L23-L25 — pub(super) fn tool(&mut self)
- finish · function · L27-L34 — pub(super) fn finish(&mut self, stop_reason: Option<&str>)
- has_final_response · function · L36-L38 — pub(super) fn has_final_response(&self) -> bool
- tests · module · L42-L84 — mod tests
- final_response_requires_text_without_tools_and_a_normal_end · function · L46-L74 — fn final_response_requires_text_without_tools_and_a_normal_end()
- replacement_can_remove_an_answer · function · L77-L83 — fn replacement_can_remove_an_answer()

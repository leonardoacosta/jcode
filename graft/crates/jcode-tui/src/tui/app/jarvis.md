# crates/jcode-tui/src/tui/app/jarvis.rs

- SpeechInput · struct · L8-L14 — struct SpeechInput
- speech_text · function · L16-L26 — fn speech_text(input: serde_json::Value) -> anyhow::Result<String>
- prepare_local_tools · function · L29-L51 — pub(super) async fn prepare_local_tools(
- poll_jarvis_speech_error · function · L53-L64 — pub(super) fn poll_jarvis_speech_error(&mut self) -> bool
- cancel_jarvis_speech · function · L66-L70 — pub(super) fn cancel_jarvis_speech(&self)
- jarvis_session_changed · function · L72-L77 — pub(super) fn jarvis_session_changed(&mut self, next: &str)
- jarvis_call · function · L79-L115 — pub(super) fn jarvis_call(
- tests · module · L119-L179 — mod tests
- speech_callback_rejects_other_sessions_without_playing_audio · function · L124-L158 — fn speech_callback_rejects_other_sessions_without_playing_audio()
- speech_input_is_bounded_and_strict · function · L161-L178 — fn speech_input_is_bounded_and_strict()

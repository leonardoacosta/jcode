# crates/jcode-session-types/src/transcription.rs

- TRANSCRIPTION_OPEN · constant · L7-L7 — pub const TRANSCRIPTION_OPEN: &str = "<transcription>";
- TRANSCRIPTION_CLOSE · constant · L8-L8 — pub const TRANSCRIPTION_CLOSE: &str = "</transcription>";
- wrap_transcription · function · L11-L16 — pub fn wrap_transcription(text: &str) -> String
- strip_transcription · function · L21-L43 — pub fn strip_transcription(text: &str) -> (Cow<'_, str>, bool)
- tests · module · L46-L84 — mod tests
- wrap_and_strip_round_trip · function · L50-L60 — fn wrap_and_strip_round_trip()
- strip_keeps_typed_text_and_multiple_segments · function · L63-L72 — fn strip_keeps_typed_text_and_multiple_segments()
- unbalanced_or_absent_tags_are_untouched · function · L75-L83 — fn unbalanced_or_absent_tags_are_untouched()

# crates/jcode-provider-openai/src/stream_citation_tests.rs

- MARKER · constant · L5-L5 — const MARKER: &str = "\u{e200}cite\u{e202}turn0search12\u{e201}";
- annotated_part · function · L7-L13 — fn annotated_part(prefix: &str, title: &str) -> Value
- done · function · L15-L19 — fn done(parts: Vec<Value>) -> Value
- http_events · function · L21-L32 — fn http_events(frames: Vec<Value>) -> Vec<StreamEvent>
- hosted_search_starts_before_completion_and_only_replays_final_item · function · L35-L71 — fn hosted_search_starts_before_completion_and_only_replays_final_item()
- streamed_citation_is_corrected_from_completed_item_before_text_done · function · L74-L94 — fn streamed_citation_is_corrected_from_completed_item_before_text_done()
- fallback_text_and_per_content_part_character_offsets_are_supported · function · L97-L106 — fn fallback_text_and_per_content_part_character_offsets_are_supported()
- completed_snapshot_without_annotations_does_not_duplicate_or_hide_raw_text · function · L109-L120 — fn completed_snapshot_without_annotations_does_not_duplicate_or_hide_raw_text()
- sequential_messages_apply_late_metadata_to_only_their_own_suffix · function · L123-L159 — fn sequential_messages_apply_late_metadata_to_only_their_own_suffix()

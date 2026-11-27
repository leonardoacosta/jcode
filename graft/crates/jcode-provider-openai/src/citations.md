# crates/jcode-provider-openai/src/citations.rs

- render_annotated_text · function · L8-L142 — pub(super) fn render_annotated_text(text: &str, annotations: Option<&Value>) -> String
- tests · module · L145-L262 — mod tests
- citation · function · L149-L153 — fn citation(text: &str, marker: &str, url: &str, title: &str) -> Value
- native_marker_uses_real_annotation_even_after_multibyte_prose · function · L156-L169 — fn native_marker_uses_real_annotation_even_after_multibyte_prose()
- shared_span_preserves_multiple_sources_and_deduplicates · function · L172-L180 — fn shared_span_preserves_multiple_sources_and_deduplicates()
- ordinary_annotated_prose_is_preserved · function · L183-L195 — fn ordinary_annotated_prose_is_preserved()
- provider_rendered_markdown_citations_are_not_duplicated · function · L198-L212 — fn provider_rendered_markdown_citations_are_not_duplicated()
- missing_or_invalid_metadata_never_hides_or_invents_sources · function · L215-L229 — fn missing_or_invalid_metadata_never_hides_or_invents_sources()
- source_title_and_destination_cannot_inject_markdown · function · L232-L244 — fn source_title_and_destination_cannot_inject_markdown()
- unordered_annotations_are_rendered_in_text_order · function · L247-L261 — fn unordered_annotations_are_rendered_in_text_order()

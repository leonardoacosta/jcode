# crates/jcode-tui/src/tui/ui/url.rs

- url_regex · function · L5-L10 — pub(crate) fn url_regex() -> Option<&'static Regex>
- URL_REGEX · constant · L6-L6 — static URL_REGEX: OnceLock<Option<Regex>> = OnceLock::new();
- markdown_link_regex · function · L12-L17 — fn markdown_link_regex() -> Option<&'static Regex>
- MARKDOWN_LINK_REGEX · constant · L13-L13 — static MARKDOWN_LINK_REGEX: OnceLock<Option<Regex>> = OnceLock::new();
- trim_url_candidate · function · L19-L40 — pub(crate) fn trim_url_candidate(candidate: &str) -> &str
- link_target_for_display_column · function · L42-L87 — pub(crate) fn link_target_for_display_column(raw_text: &str, column: usize) -> Option<String>
- RENDERED_MARKDOWN_PATH · constant · L72-L72 — static RENDERED_MARKDOWN_PATH: OnceLock<Regex> = OnceLock::new();
- tests · module · L90-L221 — mod tests
- url_regex_matches_supported_link_schemes · function · L94-L111 — fn url_regex_matches_supported_link_schemes()
- trim_url_candidate_removes_trailing_sentence_punctuation · function · L114-L127 — fn trim_url_candidate_removes_trailing_sentence_punctuation()
- trim_url_candidate_preserves_balanced_closing_delimiters · function · L130-L143 — fn trim_url_candidate_preserves_balanced_closing_delimiters()
- link_target_for_display_column_returns_trimmed_url_when_inside_url · function · L146-L161 — fn link_target_for_display_column_returns_trimmed_url_when_inside_url()
- link_target_for_display_column_uses_display_width_for_wide_prefixes · function · L164-L172 — fn link_target_for_display_column_uses_display_width_for_wide_prefixes()
- link_target_for_display_column_resolves_markdown_label · function · L175-L183 — fn link_target_for_display_column_resolves_markdown_label()
- rendered_markdown_destinations_are_cell_bounded · function · L186-L210 — fn rendered_markdown_destinations_are_cell_bounded()
- rendered_relative_path_does_not_override_enclosing_url · function · L213-L220 — fn rendered_relative_path_does_not_override_enclosing_url()

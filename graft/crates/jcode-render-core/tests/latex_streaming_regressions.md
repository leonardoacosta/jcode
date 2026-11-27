# crates/jcode-render-core/tests/latex_streaming_regressions.rs

- math_display_count · function · L3-L9 — fn math_display_count(markdown: &str) -> usize
- escaped_short_prose_lists_do_not_render_as_math · function · L12-L41 — fn escaped_short_prose_lists_do_not_render_as_math()
- parses_the_exact_multiline_equation_response_from_the_tui · function · L44-L62 — fn parses_the_exact_multiline_equation_response_from_the_tui()
- every_streaming_prefix_is_deterministic_and_the_complete_response_is_math · function · L65-L87 — fn every_streaming_prefix_is_deterministic_and_the_complete_response_is_math()
- display_math_survives_common_markdown_containers · function · L90-L104 — fn display_math_survives_common_markdown_containers()
- standalone_display_math_resists_markdown_block_interruptors · function · L107-L129 — fn standalone_display_math_resists_markdown_block_interruptors()
- crlf_and_adjacent_display_blocks_remain_balanced_and_idempotent · function · L132-L138 — fn crlf_and_adjacent_display_blocks_remain_balanced_and_idempotent()
- stabilization_preserves_newlines_comments_and_is_idempotent · function · L141-L151 — fn stabilization_preserves_newlines_comments_and_is_idempotent()
- nested_quote_list_display_math_keeps_its_container · function · L154-L160 — fn nested_quote_list_display_math_keeps_its_container()
- escaped_and_literal_delimiters_never_become_math · function · L163-L182 — fn escaped_and_literal_delimiters_never_become_math()
- standalone_multiline_inline_delimiters_promote_without_losing_content · function · L185-L218 — fn standalone_multiline_inline_delimiters_promote_without_losing_content()

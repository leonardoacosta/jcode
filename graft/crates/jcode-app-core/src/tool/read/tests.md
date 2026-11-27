# crates/jcode-app-core/src/tool/read/tests.rs

- make_ctx · function · L5-L15 — fn make_ctx(working_dir: std::path::PathBuf) -> ToolContext
- normalize_read_range_supports_start_and_end_lines · function · L18-L35 — fn normalize_read_range_supports_start_and_end_lines()
- normalize_read_range_supports_start_line_and_limit · function · L38-L55 — fn normalize_read_range_supports_start_line_and_limit()
- normalize_read_range_prefers_end_line_over_limit · function · L58-L76 — fn normalize_read_range_prefers_end_line_over_limit()
- normalize_read_range_rejects_start_line_and_offset · function · L79-L93 — fn normalize_read_range_rejects_start_line_and_offset()
- normalize_read_range_accepts_matching_start_line_and_offset · function · L96-L114 — fn normalize_read_range_accepts_matching_start_line_and_offset()
- normalize_read_range_accepts_end_line_with_zero_offset · function · L117-L134 — fn normalize_read_range_accepts_end_line_with_zero_offset()
- normalize_read_range_rejects_invalid_end_before_start · function · L137-L151 — fn normalize_read_range_rejects_invalid_end_before_start()
- read_tool_schema_avoids_openai_incompatible_combinators · function · L154-L160 — fn read_tool_schema_avoids_openai_incompatible_combinators()
- read_tool_schema_advertises_only_canonical_public_fields · function · L163-L174 — fn read_tool_schema_advertises_only_canonical_public_fields()
- read_tool_description_advertises_supported_file_types · function · L177-L189 — fn read_tool_description_advertises_supported_file_types()
- read_tool_supports_start_line_and_end_line · function · L192-L235 — async fn read_tool_supports_start_line_and_end_line()
- read_tool_continuation_hint_matches_start_line_style · function · L238-L261 — async fn read_tool_continuation_hint_matches_start_line_style()
- read_tool_supports_start_line_with_limit · function · L264-L302 — async fn read_tool_supports_start_line_with_limit()
- read_tool_prefers_end_line_over_limit · function · L305-L344 — async fn read_tool_prefers_end_line_over_limit()
- tiny_bmp_bytes · function · L346-L368 — fn tiny_bmp_bytes() -> Vec<u8>
- read_tool_converts_bmp_to_png_image_block · function · L373-L398 — async fn read_tool_converts_bmp_to_png_image_block()
- read_tool_refuses_undecodable_image_without_image_block · function · L402-L420 — async fn read_tool_refuses_undecodable_image_without_image_block()

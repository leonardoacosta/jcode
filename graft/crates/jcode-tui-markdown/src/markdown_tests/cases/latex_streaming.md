# crates/jcode-tui-markdown/src/markdown_tests/cases/latex_streaming.rs

- exact_multiline_latex_response · function · L1-L10 — fn exact_multiline_latex_response() -> &'static str
- empty_math_bypasses_native_and_image_rendering · function · L13-L27 — fn empty_math_bypasses_native_and_image_rendering()
- latex_foreground_is_white_and_styles_inline_math · function · L30-L42 — fn latex_foreground_is_white_and_styles_inline_math()
- exact_multiline_response_renders_all_five_equations · function · L45-L54 — fn exact_multiline_response_renders_all_five_equations()
- every_streaming_prefix_converges_to_the_full_math_render · function · L57-L73 — fn every_streaming_prefix_converges_to_the_full_math_render()
- math_rendering_never_runs_the_toolchain_on_the_render_thread · function · L79-L89 — fn math_rendering_never_runs_the_toolchain_on_the_render_thread()
- inline_math_stays_inline_in_image_mode_and_skips_the_image_toolchain · function · L92-L113 — fn inline_math_stays_inline_in_image_mode_and_skips_the_image_toolchain()
- display_math_nested_in_a_list_uses_graphical_rendering · function · L116-L131 — fn display_math_nested_in_a_list_uses_graphical_rendering()
- multiline_relations_survive_blockquotes_and_promoted_delimiters · function · L134-L148 — fn multiline_relations_survive_blockquotes_and_promoted_delimiters()

# crates/jcode-tui/src/tui/info_widget_stability_tests.rs

- sample_data · function · L5-L11 — fn sample_data() -> InfoWidgetData
- rich_data · function · L16-L37 — fn rich_data() -> InfoWidgetData
- flat_content_is_perfectly_stable · function · L40-L57 — fn flat_content_is_perfectly_stable()
- ragged_content_makes_widgets_move · function · L60-L76 — fn ragged_content_makes_widgets_move()
- analyze_frames_counts_travel_and_flicker · function · L79-L103 — fn analyze_frames_counts_travel_and_flicker()
- empty_input_is_safe · function · L106-L111 — fn empty_input_is_safe()
- occasional_long_lines_do_not_move_widgets · function · L118-L141 — fn occasional_long_lines_do_not_move_widgets()
- demo_quantify · function · L147-L182 — fn demo_quantify()
- profile · function · L148-L160 — fn profile(name: &str, content: &[u16])
- widgets_never_overlap_while_scrolling · function · L188-L200 — fn widgets_never_overlap_while_scrolling()
- content_anchoring_reduces_content_relative_travel · function · L208-L235 — fn content_anchoring_reduces_content_relative_travel()
- content_anchored_widgets_never_overlap_while_scrolling · function · L239-L251 — fn content_anchored_widgets_never_overlap_while_scrolling()
- demo_content_anchor · function · L257-L299 — fn demo_content_anchor()
- row · function · L258-L274 — fn row(name: &str, content: &[u16])
- demo_info_tradeoff · function · L306-L352 — fn demo_info_tradeoff()
- row · function · L307-L327 — fn row(name: &str, content: &[u16])
- demo_lookahead_sweep · function · L361-L418 — fn demo_lookahead_sweep()
- line · function · L362-L374 — fn line(label: &str, r: &super::StabilityReport)

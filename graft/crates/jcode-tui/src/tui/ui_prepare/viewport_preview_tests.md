# crates/jcode-tui/src/tui/ui_prepare/viewport_preview_tests.rs

- message · function · L3-L12 — fn message(role: &str, content: String) -> DisplayMessage
- transcript · function · L14-L19 — fn transcript(turns: usize) -> Vec<DisplayMessage>
- old · function · L21-L35 — fn old(
- assert_same · function · L37-L64 — fn assert_same(a: &PreparedMessages, b: &PreparedMessages)
- tool_fixture · function · L66-L80 — fn tool_fixture() -> Vec<DisplayMessage>
- viewport_preview_matches_old_algorithm_at_boundaries · function · L83-L125 — fn viewport_preview_matches_old_algorithm_at_boundaries()
- viewport_preview_stops_before_full_wide_render · function · L128-L133 — fn viewport_preview_stops_before_full_wide_render()
- viewport_preview_detects_overflow_after_wrapping · function · L136-L146 — fn viewport_preview_detects_overflow_after_wrapping()
- viewport_preview_cold_benchmark · function · L152-L193 — fn viewport_preview_cold_benchmark()
- cold · function · L153-L157 — fn cold()

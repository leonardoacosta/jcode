# crates/jcode-tui/src/tui/ui_tests/basic/idle_animation_partial.rs

- pin_full_tier · function · L26-L28 — fn pin_full_tier()
- IdleAnimationEnvGuard · struct · L39-L41 — struct IdleAnimationEnvGuard
- enable · function · L44-L51 — fn enable() -> Self
- drop · function · L55-L58 — fn drop(&mut self)
- idle_animation_state · function · L61-L68 — fn idle_animation_state(anim_elapsed: f32) -> TestState
- render_full · function · L70-L76 — fn render_full(state: &TestState, width: u16, height: u16) -> Terminal<TestBackend>
- draw_publishes_the_animated_rows_only_when_the_animation_rendered · function · L79-L109 — fn draw_publishes_the_animated_rows_only_when_the_animation_rendered()
- partial_repaint_matches_a_full_frame_at_the_same_animation_time · function · L112-L155 — fn partial_repaint_matches_a_full_frame_at_the_same_animation_time()
- advancing_the_animation_actually_changes_the_animated_rows · function · L158-L192 — fn advancing_the_animation_actually_changes_the_animated_rows()
- idle_animation_is_excluded_from_the_full_frame_redraw_signal · function · L195-L227 — fn idle_animation_is_excluded_from_the_full_frame_redraw_signal()
- partial_repaint_does_no_layout_or_transcript_work · function · L230-L268 — fn partial_repaint_does_no_layout_or_transcript_work()
- published_rectangle_round_trips_through_the_shared_slot · function · L271-L287 — fn published_rectangle_round_trips_through_the_shared_slot()
- a_full_screen_overlay_stops_the_decorative_animation_cadence · function · L310-L366 — fn a_full_screen_overlay_stops_the_decorative_animation_cadence()
- animation_cadence_implies_the_renderer_published_animated_rows · function · L376-L410 — fn animation_cadence_implies_the_renderer_published_animated_rows()
- the_animation_can_still_start_from_a_state_with_nothing_published · function · L422-L448 — fn the_animation_can_still_start_from_a_state_with_nothing_published()
- copying_only_the_animated_rows_matches_cloning_the_whole_frame · function · L466-L527 — fn copying_only_the_animated_rows_matches_cloning_the_whole_frame()
- copy_cells_in_copies_exactly_the_rectangle_and_nothing_else · function · L541-L598 — fn copy_cells_in_copies_exactly_the_rectangle_and_nothing_else()

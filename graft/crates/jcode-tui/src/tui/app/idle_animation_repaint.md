# crates/jcode-tui/src/tui/app/idle_animation_repaint.rs

- IDLE_ANIMATION_CHROME_FULL_FRAME_INTERVAL · constant · L15-L15 — pub(super) const IDLE_ANIMATION_CHROME_FULL_FRAME_INTERVAL: Duration = Duration::from_millis(250);
- idle_animation_partial_repaint_allowed · function · L27-L35 — pub(super) fn idle_animation_partial_repaint_allowed(
- copy_cells_in · function · L42-L49 — pub(crate) fn copy_cells_in(src: &Buffer, dst: &mut Buffer, area: Rect)
- tests · module · L52-L107 — mod tests
- quiet_idle_screen_serves_every_animation_tick_from_the_partial_repaint · function · L57-L64 — fn quiet_idle_screen_serves_every_animation_tick_from_the_partial_repaint()
- slow_chrome_gets_periodic_full_frames_without_pacing_the_animation · function · L67-L91 — fn slow_chrome_gets_periodic_full_frames_without_pacing_the_animation()
- chrome_full_frames_are_a_small_fraction_of_animation_ticks · function · L97-L106 — fn chrome_full_frames_are_a_small_fraction_of_animation_ticks()

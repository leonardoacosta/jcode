# crates/jcode-tui/src/tui/ui_tests/preview_redraw_schedule.rs

- policy · function · L9-L17 — fn policy(tier: crate::perf::PerformanceTier) -> crate::perf::TuiPerfPolicy
- state_with_picker · function · L19-L28 — fn state_with_picker(deep_idle: bool) -> TestState
- preview_work_started_during_draw_rearms_existing_timer_immediately · function · L31-L55 — async fn preview_work_started_during_draw_rearms_existing_timer_immediately()
- running_picker_keeps_existing_spinner_cadence_without_pending_work · function · L58-L125 — fn running_picker_keeps_existing_spinner_cadence_without_pending_work()
- focused_pending_preview_work_overrides_deep_idle_and_animation_policy · function · L128-L156 — fn focused_pending_preview_work_overrides_deep_idle_and_animation_policy()
- settled_or_closed_picker_has_no_permanent_repaint_and_restores_idle_cadence · function · L159-L183 — fn settled_or_closed_picker_has_no_permanent_repaint_and_restores_idle_cadence()
- rendering_settled_receivers_stops_the_fast_poll_loop · function · L186-L209 — fn rendering_settled_receivers_stops_the_fast_poll_loop()
- background_pending_preview_does_not_burn_frames_and_focus_resumes_polling · function · L212-L245 — fn background_pending_preview_does_not_burn_frames_and_focus_resumes_polling()

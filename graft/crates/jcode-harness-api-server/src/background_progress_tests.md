# crates/jcode-harness-api-server/src/background_progress_tests.rs

- parse · function · L3-L5 — fn parse(message: &str) -> BackgroundProgress
- progress_tick_carries_task_label_and_percent · function · L8-L17 — fn progress_tick_carries_task_label_and_percent()
- a_display_name_wins_over_the_tool_name · function · L20-L26 — fn a_display_name_wins_over_the_tool_name()
- indeterminate_progress_has_no_percent_but_still_reports · function · L29-L33 — fn indeterminate_progress_has_no_percent_but_still_reports()
- completion_is_reported_as_done · function · L36-L44 — fn completion_is_reported_as_done()
- other_notifications_are_not_progress · function · L47-L52 — fn other_notifications_are_not_progress()
- a_percent_over_a_hundred_is_clamped · function · L55-L58 — fn a_percent_over_a_hundred_is_clamped()
- the_daemons_own_formatter_round_trips · function · L65-L118 — fn the_daemons_own_formatter_round_trips()

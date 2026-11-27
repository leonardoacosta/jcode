# crates/jcode-tui/src/tui/ui_tests/pinned_todo_regressions.rs

- with_pinned_todos_config · function · L5-L32 — fn with_pinned_todos_config(f: impl FnOnce())
- RestoreHome · struct · L11-L11 — struct RestoreHome(Option<std::ffi::OsString>);
- drop · function · L13-L20 — fn drop(&mut self)
- todo_payload · function · L34-L40 — fn todo_payload() -> String
- pinned_config_and_app_reset_lock_order_does_not_deadlock · function · L46-L108 — fn pinned_config_and_app_reset_lock_order_does_not_deadlock()
- CHILD · constant · L49-L49 — const CHILD: &str = "JCODE_TEST_PINNED_CONFIG_LOCK_ORDER_CHILD";
- justrust_status_is_below_todos_and_never_wraps · function · L111-L180 — fn justrust_status_is_below_todos_and_never_wraps()
- pinned_todos_do_not_duplicate_passive_swarm_gallery_at_any_width · function · L183-L226 — fn pinned_todos_do_not_duplicate_passive_swarm_gallery_at_any_width()
- hidden_todo · function · L228-L243 — fn hidden_todo(payload: &str) -> DisplayMessage
- terminal_text · function · L245-L255 — fn terminal_text(terminal: &Terminal<TestBackend>) -> String
- pinned_todo_regression_packed_fresh_session_keeps_prompt_when_band_appears · function · L258-L316 — fn pinned_todo_regression_packed_fresh_session_keeps_prompt_when_band_appears()
- RestoreDebug · struct · L262-L262 — struct RestoreDebug(bool);
- drop · function · L264-L268 — fn drop(&mut self)
- pinned_todo_regression_hidden_message_has_zero_length_cache_boundary · function · L319-L385 — fn pinned_todo_regression_hidden_message_has_zero_length_cache_boundary()
- tiny_panes_draw_without_a_recovered_panic · function · L390-L416 — fn tiny_panes_draw_without_a_recovered_panic()

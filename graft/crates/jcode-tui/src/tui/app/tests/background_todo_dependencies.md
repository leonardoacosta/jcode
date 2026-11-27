# crates/jcode-tui/src/tui/app/tests/background_todo_dependencies.rs

- waiting_dependency_test_app · function · L2-L7 — fn waiting_dependency_test_app() -> App
- complete_blocker_pauses_auto_pokes_until_cleared_but_not_independent_work · function · L10-L43 — fn complete_blocker_pauses_auto_pokes_until_cleared_but_not_independent_work()
- goal_that_cannot_complete_pauses_its_pokes_survives_reload_and_resumes_when_cleared · function · L46-L78 — fn goal_that_cannot_complete_pauses_its_pokes_survives_reload_and_resumes_when_cleared()
- await_dependency_suppresses_pokes_and_survives_reload_until_terminal_or_timeout · function · L81-L141 — fn await_dependency_suppresses_pokes_and_survives_reload_until_terminal_or_timeout()
- dependency_todo · function · L143-L155 — fn dependency_todo(id: &str, task_id: &str) -> crate::todo::TodoItem
- DependencyTaskFixture · struct · L157-L157 — struct DependencyTaskFixture(std::path::PathBuf);
- drop · function · L160-L162 — fn drop(&mut self)
- live_dependency_fixture · function · L165-L179 — fn live_dependency_fixture(app: &App, task: &str) -> DependencyTaskFixture
- background_dependency_waiting_only_suppresses_review_and_resets_budget · function · L182-L214 — fn background_dependency_waiting_only_suppresses_review_and_resets_budget()
- background_dependency_mixed_poke_targets_only_actionable_names · function · L217-L238 — fn background_dependency_mixed_poke_targets_only_actionable_names()
- background_dependency_terminal_or_unknown_resumes_actionable_work · function · L241-L265 — fn background_dependency_terminal_or_unknown_resumes_actionable_work()
- background_dependency_stale_row_missing_or_malformed_state_is_actionable · function · L268-L285 — fn background_dependency_stale_row_missing_or_malformed_state_is_actionable()
- background_dependency_dead_owner_or_detached_child_is_actionable · function · L288-L312 — fn background_dependency_dead_owner_or_detached_child_is_actionable()
- background_dependency_card_refresh_tracks_only_live_task_progress · function · L315-L343 — fn background_dependency_card_refresh_tracks_only_live_task_progress()
- background_dependency_reload_reads_session_scoped_task_state · function · L346-L380 — fn background_dependency_reload_reads_session_scoped_task_state()

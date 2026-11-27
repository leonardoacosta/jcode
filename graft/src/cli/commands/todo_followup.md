# src/cli/commands/todo_followup.rs

- RunAutoPokeFollowUp · enum · L6-L21 — pub(super) enum RunAutoPokeFollowUp
- run_todos · function · L23-L33 — pub(super) fn run_todos(session_id: &str) -> Vec<crate::todo::TodoItem>
- run_task_is_waitable · function · L35-L48 — pub(super) fn run_task_is_waitable(
- run_actionable_todos · function · L50-L74 — pub(super) fn run_actionable_todos(
- run_actionable_poke_message · function · L76-L101 — pub(super) fn run_actionable_poke_message(
- run_goals · function · L103-L113 — pub(super) fn run_goals(session_id: &str) -> Vec<crate::todo::TodoGoal>
- run_user_auth_notice · function · L115-L126 — pub(super) fn run_user_auth_notice(todos: &[crate::todo::TodoItem]) -> Option<String>
- run_blocked_notice · function · L130-L142 — pub(super) fn run_blocked_notice(
- run_cannot_complete_notice · function · L144-L159 — pub(super) fn run_cannot_complete_notice(
- prepare_run_todo_follow_up · function · L164-L312 — pub(super) async fn prepare_run_todo_follow_up(
- take_run_gate_digest · function · L320-L361 — pub(super) fn take_run_gate_digest(session_id: &str, already_delivered: bool) -> Option<String>
- take_run_gate_digest_if_turn_ended · function · L369-L391 — pub(super) fn take_run_gate_digest_if_turn_ended(
- build_run_auto_poke_follow_up_from_todos · function · L393-L432 — pub(super) fn build_run_auto_poke_follow_up_from_todos(
- build_run_poke_message · function · L434-L436 — pub(super) fn build_run_poke_message(incomplete: &[crate::todo::TodoItem]) -> String
- build_run_todo_validation_message · function · L438-L475 — pub(super) fn build_run_todo_validation_message(

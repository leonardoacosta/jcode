# crates/jcode-app-core/src/server/comm_control_tests/assign_double.rs

- named_assignment_test_agent · function · L7-L16 — async fn named_assignment_test_agent(id: &str) -> Arc<Mutex<Agent>>
- active_assignment_conflict_detects_only_assigned_and_fresh_items · function · L21-L144 — fn active_assignment_conflict_detects_only_assigned_and_fresh_items()
- double_assign_fixture · function · L147-L202 — fn double_assign_fixture(
- unix_now_ms · function · L204-L209 — fn unix_now_ms() -> u64
- assign_task_rejects_double_assignment_of_actively_worked_task · function · L214-L295 — async fn assign_task_rejects_double_assignment_of_actively_worked_task()
- assign_task_allows_taking_over_stale_assignment · function · L300-L375 — async fn assign_task_allows_taking_over_stale_assignment()
- task_control_reassign_tells_displaced_worker_to_stand_down · function · L381-L506 — async fn task_control_reassign_tells_displaced_worker_to_stand_down()
- task_control_reassign_waits_for_unsettled_predecessor_then_hands_off · function · L509-L642 — async fn task_control_reassign_waits_for_unsettled_predecessor_then_hands_off()

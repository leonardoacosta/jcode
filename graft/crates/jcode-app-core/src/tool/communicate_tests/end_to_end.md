# crates/jcode-app-core/src/tool/communicate_tests/end_to_end.rs

- council_three_seat_run_is_read_only_bounded_and_stops_all_seats · function · L2-L274 — async fn council_three_seat_run_is_read_only_bounded_and_stops_all_seats()
- CouncilProvider · struct · L32-L37 — struct CouncilProvider(
- complete · function · L40-L80 — async fn complete(
- name · function · L81-L83 — fn name(&self) -> &str
- set_session_id · function · L84-L86 — fn set_session_id(&self, session_id: &str)
- fork · function · L87-L94 — fn fork(&self) -> Arc<dyn Provider>
- assert_stopped · function · L141-L152 — fn assert_stopped(id: &str)
- communicate_list_and_await_members_work_end_to_end · function · L277-L405 — async fn communicate_list_and_await_members_work_end_to_end()
- communicate_await_members_background_returns_immediately_and_notifies · function · L408-L513 — async fn communicate_await_members_background_returns_immediately_and_notifies()
- communicate_status_returns_busy_snapshot_for_running_member · function · L516-L600 — async fn communicate_status_returns_busy_snapshot_for_running_member()
- communicate_spawn_reports_completion_back_to_spawner · function · L603-L684 — async fn communicate_spawn_reports_completion_back_to_spawner()
- communicate_spawn_with_seeded_lane_and_summary_work_end_to_end · function · L687-L800 — async fn communicate_spawn_with_seeded_lane_and_summary_work_end_to_end()
- communicate_message_routes_as_dm_while_broadcast_targets_swarm · function · L808-L1005 — async fn communicate_message_routes_as_dm_while_broadcast_targets_swarm()
- communicate_retry_respawns_orphaned_worker_from_spawn_spec · function · L1008-L1088 — async fn communicate_retry_respawns_orphaned_worker_from_spawn_spec()

# tests/e2e/worker_stop.rs

- ACCEPTANCE · constant · L7-L703 — const ACCEPTANCE: &str = r#"
- refuses_shared_network_before_starting_daemon · function · L706-L721 — fn refuses_shared_network_before_starting_daemon()
- public_stop_prevents_post_stop_native_write · function · L725-L756 — fn public_stop_prevents_post_stop_native_write()
- public_stop_during_native_tool_never_reports_stopped_while_tool_runs · function · L760-L792 — fn public_stop_during_native_tool_never_reports_stopped_while_tool_runs()
- restart_after_unresolved_stop_keeps_worker_revoked · function · L796-L828 — fn restart_after_unresolved_stop_keeps_worker_revoked()
- alias_stop_is_swarm_local_over_the_public_socket · function · L832-L864 — fn alias_stop_is_swarm_local_over_the_public_socket()
- concurrent_wakes_after_settled_stop_admit_exactly_one_generation · function · L868-L900 — fn concurrent_wakes_after_settled_stop_admit_exactly_one_generation()
- owner_control_survives_churn_ambiguity_denial_and_force_over_the_public_socket · function · L904-L936 — fn owner_control_survives_churn_ambiguity_denial_and_force_over_the_public_socket()
- restart_after_settled_stop_stays_stopped_until_explicit_wake · function · L940-L972 — fn restart_after_settled_stop_stays_stopped_until_explicit_wake()
- restart_while_draining_keeps_worker_revoked · function · L976-L1008 — fn restart_while_draining_keeps_worker_revoked()
- concurrent_stop_and_wake_never_leave_a_runnable_revoked_generation · function · L1012-L1044 — fn concurrent_stop_and_wake_never_leave_a_runnable_revoked_generation()
- reassign_waits_for_stop_settlement_then_hands_the_task_off · function · L1048-L1080 — fn reassign_waits_for_stop_settlement_then_hands_the_task_off()
- graceful_reload_after_settled_stop_stays_stopped_until_explicit_wake · function · L1084-L1116 — fn graceful_reload_after_settled_stop_stays_stopped_until_explicit_wake()
- graceful_reload_while_draining_keeps_worker_revoked · function · L1120-L1152 — fn graceful_reload_while_draining_keeps_worker_revoked()
- reload_into_candidate_below_control_floor_is_refused_and_daemon_keeps_serving · function · L1156-L1188 — fn reload_into_candidate_below_control_floor_is_refused_and_daemon_keeps_serving()
- wait_bound_to_stopped_generation_never_completes_from_resumed_work · function · L1192-L1224 — fn wait_bound_to_stopped_generation_never_completes_from_resumed_work()

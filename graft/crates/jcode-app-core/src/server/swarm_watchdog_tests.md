# crates/jcode-app-core/src/server/swarm_watchdog_tests.rs

- member · function · L6-L32 — fn member(session_id: &str, role: &str, status: &str, status_age: Duration) -> SwarmMember
- inputs · function · L34-L42 — fn inputs(status: &'static str, activity: Option<u64>, status_age: u64) -> StallInputs<'static>
- WINDOW · constant · L44-L44 — const WINDOW: Duration = Duration::from_secs(600);
- silent_running_worker_is_stalled · function · L47-L56 — fn silent_running_worker_is_stalled()
- recent_activity_or_recent_turn_start_is_not_stalled · function · L59-L69 — fn recent_activity_or_recent_turn_start_is_not_stalled()
- running_tool_never_counts_as_stall · function · L72-L76 — fn running_tool_never_counts_as_stall()
- only_running_non_coordinator_members_can_stall · function · L79-L90 — fn only_running_non_coordinator_members_can_stall()
- sweep_interval_is_clamped · function · L93-L106 — fn sweep_interval_is_clamped()
- update_stalls_notifies_once_and_rearms_after_recovery · function · L109-L181 — fn update_stalls_notifies_once_and_rearms_after_recovery()
- messages_mention_recovery_actions · function · L184-L196 — fn messages_mention_recovery_actions()

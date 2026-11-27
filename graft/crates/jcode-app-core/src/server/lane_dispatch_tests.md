# crates/jcode-app-core/src/server/lane_dispatch_tests.rs

- item · function · L4-L15 — fn item(id: &str, stage: u32, lane: &str, owner: Option<&str>) -> TodoItem
- self_owned_lanes_never_dispatch · function · L18-L21 — fn self_owned_lanes_never_dispatch()
- ready_worker_lanes_in_one_phase_start_together · function · L24-L32 — fn ready_worker_lanes_in_one_phase_start_together()
- later_phase_waits_then_unlocks_on_completion · function · L35-L43 — fn later_phase_waits_then_unlocks_on_completion()
- explicit_dependency_unlocks_lane_early · function · L46-L57 — fn explicit_dependency_unlocks_lane_early()
- owned_lane_is_not_dispatched_again · function · L60-L76 — fn owned_lane_is_not_dispatched_again()
- completed_lane_is_not_dispatched · function · L79-L83 — fn completed_lane_is_not_dispatched()
- normalize_makes_owner_lane_wide · function · L86-L95 — fn normalize_makes_owner_lane_wide()

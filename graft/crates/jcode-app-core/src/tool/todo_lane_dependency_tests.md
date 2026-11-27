# crates/jcode-app-core/src/tool/todo_lane_dependency_tests.rs

- item · function · L3-L5 — fn item(id: &str, phase: u32, lane: &str) -> Value
- parsed · function · L7-L9 — fn parsed(value: Value) -> Vec<TodoItem>
- hierarchical_after_is_copied_to_every_member · function · L12-L24 — fn hierarchical_after_is_copied_to_every_member()
- incremental_edits_apply_to_whole_lane_and_new_members_inherit · function · L27-L56 — fn incremental_edits_apply_to_whole_lane_and_new_members_inherit()
- public_schema_exposes_lane_dependencies_only_at_correct_levels · function · L59-L76 — fn public_schema_exposes_lane_dependencies_only_at_correct_levels()
- named_dependencies_roundtrip_and_lane_updates · function · L79-L119 — fn named_dependencies_roundtrip_and_lane_updates()
- conflicting_dependency_aliases_are_rejected · function · L122-L137 — fn conflicting_dependency_aliases_are_rejected()
- phase_lane_owner_marks_every_member_and_update_flips_whole_lane · function · L140-L165 — fn phase_lane_owner_marks_every_member_and_update_flips_whole_lane()
- unknown_lane_owner_is_rejected · function · L168-L178 — fn unknown_lane_owner_is_rejected()

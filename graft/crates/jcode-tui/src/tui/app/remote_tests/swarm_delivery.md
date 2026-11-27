# crates/jcode-tui/src/tui/app/remote_tests/swarm_delivery.rs

- swarm_dm_notification · function · L1-L12 — fn swarm_dm_notification(sender: &str, body: &str, tldr: Option<&str>) -> ServerEvent
- soft_interrupt_injected · function · L14-L22 — fn soft_interrupt_injected(content: &str, display_role: Option<&str>) -> ServerEvent
- file_activity_delivery_uses_metadata_not_a_text_prefix · function · L25-L52 — fn file_activity_delivery_uses_metadata_not_a_text_prefix()
- every_system_and_background_delivery_adds_an_entry · function · L55-L73 — fn every_system_and_background_delivery_adds_an_entry()
- confirmed_memory_delivery_is_visible_with_local_memory_disabled · function · L76-L99 — fn confirmed_memory_delivery_is_visible_with_local_memory_disabled()
- background_delivery_survives_transcript_restore · function · L102-L115 — fn background_delivery_survives_transcript_restore()
- agent_completion_wakes_never_become_user_prompts · function · L118-L152 — fn agent_completion_wakes_never_become_user_prompts()
- lane_aware_completion_handoff_is_a_swarm_row · function · L155-L173 — fn lane_aware_completion_handoff_is_a_swarm_row()
- assert_no_long_rows · function · L175-L193 — fn assert_no_long_rows(app: &crate::tui::app::App)
- injected_swarm_dm_adds_delivery_entry_after_its_notification_card · function · L196-L221 — fn injected_swarm_dm_adds_delivery_entry_after_its_notification_card()
- injected_system_reminder_never_shows_raw_tags · function · L224-L262 — fn injected_system_reminder_never_shows_raw_tags()
- injected_swarm_dm_without_card_renders_one_collapsed_row · function · L265-L282 — fn injected_swarm_dm_without_card_renders_one_collapsed_row()
- restored_display_snapshot_normalizes_only_raw_swarm_deliveries · function · L285-L311 — fn restored_display_snapshot_normalizes_only_raw_swarm_deliveries()
- history_restore_renders_swarm_deliveries_as_one_row · function · L314-L370 — fn history_restore_renders_swarm_deliveries_as_one_row()

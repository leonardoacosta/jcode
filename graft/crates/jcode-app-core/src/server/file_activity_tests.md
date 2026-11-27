# crates/jcode-app-core/src/server/file_activity_tests.rs

- access · function · L6-L21 — fn access(session_id: &str, op: FileOp, age_ms: u64) -> FileAccess
- latest_peer_touches_excludes_previous_readers_from_modification_alerts · function · L24-L45 — fn latest_peer_touches_excludes_previous_readers_from_modification_alerts()
- latest_peer_touches_deduplicates_to_most_recent_touch_per_peer · function · L48-L61 — fn latest_peer_touches_deduplicates_to_most_recent_touch_per_peer()
- entry · function · L68-L81 — fn entry(agent: &str, path: &str, added: u64, removed: u64) -> FileActivityEntry
- notice_is_one_short_line_with_agent_path_counts_and_intent · function · L84-L104 — fn notice_is_one_short_line_with_agent_path_counts_and_intent()
- long_intents_are_truncated · function · L107-L113 — fn long_intents_are_truncated()
- repeated_edits_by_one_agent_to_one_file_merge_into_one_entry · function · L116-L138 — fn repeated_edits_by_one_agent_to_one_file_merge_into_one_entry()
- display_path_is_relative_to_the_recipient_working_dir · function · L141-L156 — fn display_path_is_relative_to_the_recipient_working_dir()

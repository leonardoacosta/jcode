# crates/jcode-tui/src/tui/app/state_ui_storage.rs

- compact_tool_input_for_display · function · L3-L463 — fn compact_tool_input_for_display(name: &str, input: &serde_json::Value) -> serde_json::Value
- compact_display_message_tool_data · function · L465-L470 — pub(crate) fn compact_display_message_tool_data(message: &mut DisplayMessage)
- compact_display_messages_for_storage · function · L472-L480 — pub(crate) fn compact_display_messages_for_storage(messages: &mut [DisplayMessage])
- infer_spawned_session_startup_hints · function · L482-L511 — pub(super) fn infer_spawned_session_startup_hints(
- tests · module · L514-L778 — mod tests
- codemode_preview_survives_transcript_compaction_and_restore · function · L518-L541 — fn codemode_preview_survives_transcript_compaction_and_restore()
- tool_message · function · L543-L558 — fn tool_message(name: &str, input: serde_json::Value) -> DisplayMessage
- compaction_keeps_webfetch_url_for_transcript_summary · function · L561-L581 — fn compaction_keeps_webfetch_url_for_transcript_summary()
- compaction_keeps_websearch_query_for_transcript_summary · function · L584-L600 — fn compaction_keeps_websearch_query_for_transcript_summary()
- compaction_preserves_model_provided_intent · function · L603-L614 — fn compaction_preserves_model_provided_intent()
- compaction_preserves_flat_and_nested_batch_subcall_intents · function · L617-L657 — fn compaction_preserves_flat_and_nested_batch_subcall_intents()
- compaction_keeps_swarm_action_and_intent_for_transcript_summary · function · L660-L686 — fn compaction_keeps_swarm_action_and_intent_for_transcript_summary()
- compaction_keeps_swarm_dm_target_for_transcript_summary · function · L689-L706 — fn compaction_keeps_swarm_dm_target_for_transcript_summary()
- compaction_keeps_gmail_action_and_intent_for_transcript_summary · function · L709-L727 — fn compaction_keeps_gmail_action_and_intent_for_transcript_summary()
- compaction_keeps_gmail_draft_body_and_attachments_for_draft_card · function · L730-L756 — fn compaction_keeps_gmail_draft_body_and_attachments_for_draft_card()
- compaction_keeps_browser_action_and_intent_for_transcript_summary · function · L759-L777 — fn compaction_keeps_browser_action_and_intent_for_transcript_summary()

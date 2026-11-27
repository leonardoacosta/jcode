# crates/jcode-app-core/src/server/jade_relay_tests.rs

- relay_listener_config_is_opt_in_and_requires_credentials · function · L4-L14 — fn relay_listener_config_is_opt_in_and_requires_credentials()
- relay_listener_config_accepts_complete_opt_in_config · function · L17-L34 — fn relay_listener_config_accepts_complete_opt_in_config()
- relay_launch_config_is_separately_opt_in · function · L37-L54 — fn relay_launch_config_is_separately_opt_in()
- launch_request_reads_structured_data · function · L57-L75 — fn launch_request_reads_structured_data()
- relay_session_listener_polls_prompt_and_cancel_commands · function · L78-L80 — fn relay_session_listener_polls_prompt_and_cancel_commands()
- relay_event_type_defaults_to_prompt_for_legacy_events · function · L83-L97 — fn relay_event_type_defaults_to_prompt_for_legacy_events()
- relay_url_encoding_matches_jade_api_expectations · function · L100-L104 — fn relay_url_encoding_matches_jade_api_expectations()
- truncation_preserves_short_text_and_marks_long_text · function · L107-L110 — fn truncation_preserves_short_text_and_marks_long_text()
- pending_queue_is_fifo_and_drains_once · function · L113-L119 — fn pending_queue_is_fifo_and_drains_once()
- pending_queue_drops_oldest_past_bound · function · L122-L132 — fn pending_queue_drops_oldest_past_bound()
- DrainTestProvider · struct · L134-L134 — struct DrainTestProvider;
- complete · function · L138-L149 — async fn complete(
- name · function · L150-L152 — fn name(&self) -> &str
- fork · function · L153-L155 — fn fork(&self) -> Arc<dyn crate::provider::Provider>
- pending_messages_drain_as_one_turn_after_busy_turn_ends · function · L159-L198 — async fn pending_messages_drain_as_one_turn_after_busy_turn_ends()

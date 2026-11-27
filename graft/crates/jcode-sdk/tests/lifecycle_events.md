# crates/jcode-sdk/tests/lifecycle_events.rs

- session · function · L20-L39 — fn session(id: &str) -> SessionInfo
- public_client_exposes_titles_from_list_and_attach · function · L42-L55 — fn public_client_exposes_titles_from_list_and_attach()
- public_client_exposes_swarm_metadata_from_list_and_attach · function · L58-L71 — fn public_client_exposes_swarm_metadata_from_list_and_attach()
- public_sdk_enrichment_helper_accepts_older_session_records · function · L74-L90 — fn public_sdk_enrichment_helper_accepts_older_session_records()
- UnixHarness · struct · L92-L99 — struct UnixHarness
- start · function · L102-L149 — fn start(events_per_attach: usize) -> Self
- connect · function · L151-L159 — fn connect(&self) -> JcodeClient
- wait_for_clients · function · L161-L173 — fn wait_for_clients(&self, expected: usize)
- drop · function · L177-L179 — fn drop(&mut self)
- serve_connection · function · L182-L240 — fn serve_connection(
- reply · function · L242-L252 — fn reply(id: u64, event: ApiEvent, writer: &mut UnixStream)
- push · function · L254-L264 — fn push(event: ApiEvent, writer: &mut UnixStream)
- global_events_discovers_existing_and_new_sessions_then_closes_children · function · L267-L308 — fn global_events_discovers_existing_and_new_sessions_then_closes_children()
- global_events_reports_bounded_queue_overflow · function · L311-L324 — fn global_events_reports_bounded_queue_overflow()
- missing_launch_binary_is_typed_and_cleans_up_its_temporary_home · function · L327-L360 — fn missing_launch_binary_is_typed_and_cleans_up_its_temporary_home()
- rotating_credentials_are_shared_but_mutable_config_is_copied · function · L363-L393 — fn rotating_credentials_are_shared_but_mutable_config_is_copied()
- credential_inheritance_rejects_the_users_own_home · function · L396-L400 — fn credential_inheritance_rejects_the_users_own_home()

# crates/jcode-app-core/src/server/client_lifecycle_metadata_tests.rs

- writer_buffer_retains_64kib_and_releases_larger_capacity · function · L5-L21 — fn writer_buffer_retains_64kib_and_releases_larger_capacity()
- writer_reused_buffer_preserves_sequential_wire_bytes · function · L24-L51 — async fn writer_reused_buffer_preserves_sequential_wire_bytes()
- fixtures · function · L53-L75 — fn fixtures() -> Vec<ServerEvent>
- wrapped · function · L77-L84 — fn wrapped(event: ServerEvent) -> ServerEvent
- old_transition · function · L88-L109 — fn old_transition(event: &ServerEvent, state: &mut (bool, Option<String>))
- new_transition · function · L111-L115 — fn new_transition(event: &ServerEvent, state: &mut (bool, Option<String>))
- metadata_transition_equivalence_all_states_wrappers_and_event_pairs · function · L118-L142 — fn metadata_transition_equivalence_all_states_wrappers_and_event_pairs()
- metadata_lock_eligibility_exact_transition_set_and_single_wrapper · function · L145-L159 — fn metadata_lock_eligibility_exact_transition_set_and_single_wrapper()
- metadata_lock_eligibility_for_1000_text_events_is_zero · function · L162-L180 — fn metadata_lock_eligibility_for_1000_text_events_is_zero()

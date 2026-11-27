# crates/jcode-app-core/src/server/state_fanout_tests.rs

- thousand_snapshots_avoid_single_recipient_heap_storage · function · L4-L29 — fn thousand_snapshots_avoid_single_recipient_heap_storage()
- original_session_fanout · function · L32-L51 — async fn original_session_fanout(members: &Members, session_id: &str, event: ServerEvent) -> usize
- original_vs_new_full_session_fanout_microprobe · function · L55-L124 — async fn original_vs_new_full_session_fanout_microprobe()
- EVENTS · constant · L56-L56 — const EVENTS: usize = 10_000;
- ROUNDS · constant · L57-L57 — const ROUNDS: usize = 8;
- Members · type · L126-L126 — type Members = Arc<RwLock<HashMap<String, SwarmMember>>>;
- swarm_member · function · L129-L156 — fn swarm_member(session_id: &str) -> (SwarmMember, mpsc::UnboundedReceiver<ServerEvent>)
- members · function · L158-L163 — fn members(member: SwarmMember) -> Members
- event · function · L165-L167 — fn event(text: &str) -> ServerEvent
- assert_text · function · L169-L174 — fn assert_text(rx: &mut mpsc::UnboundedReceiver<ServerEvent>, expected: &str)
- recipient_snapshot_shapes_preserve_vec_delivery_equivalence · function · L177-L216 — fn recipient_snapshot_shapes_preserve_vec_delivery_equivalence()
- all_fanouts_handle_missing_sessions_and_zero_attachments · function · L219-L254 — async fn all_fanouts_handle_missing_sessions_and_zero_attachments()
- fanouts_preserve_order_counts_and_peer_exclusion · function · L257-L311 — async fn fanouts_preserve_order_counts_and_peer_exclusion()
- maintenance_prunes_closed_targets_and_refreshes_stale_primary · function · L314-L363 — async fn maintenance_prunes_closed_targets_and_refreshes_stale_primary()
- all_closed_attachments_use_only_session_primary_fallback · function · L366-L391 — async fn all_closed_attachments_use_only_session_primary_fallback()
- session_fanout_updates_focus_batching_from_live_targets_without_delaying_delivery · function · L394-L460 — async fn session_fanout_updates_focus_batching_from_live_targets_without_delaying_delivery()

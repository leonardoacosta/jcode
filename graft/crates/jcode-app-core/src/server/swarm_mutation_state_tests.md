# crates/jcode-app-core/src/server/swarm_mutation_state_tests.rs

- RuntimeEnvGuard · struct · L7-L10 — struct RuntimeEnvGuard
- new · function · L13-L25 — fn new() -> (Self, tempfile::TempDir)
- drop · function · L29-L35 — fn drop(&mut self)
- swarm_mutation_replays_persisted_spawn_response · function · L39-L85 — async fn swarm_mutation_replays_persisted_spawn_response()
- swarm_mutation_concurrent_duplicates_share_final_done_response · function · L88-L123 — async fn swarm_mutation_concurrent_duplicates_share_final_done_response()
- control_driven_begin_re_dispatches_despite_persisted_final_state · function · L131-L172 — async fn control_driven_begin_re_dispatches_despite_persisted_final_state()
- control_driven_begin_coalesces_in_flight_duplicates · function · L177-L206 — async fn control_driven_begin_coalesces_in_flight_duplicates()
- swarm_mutation_redispatches_only_expired_pending_state · function · L209-L251 — async fn swarm_mutation_redispatches_only_expired_pending_state()

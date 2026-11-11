# Tasks

- [ ] Add `base_rev`, `acceptance_check`, `turn_budget` to plan items (serde-default).
- [ ] Record HEAD on spawn/assign; enforce turn budget as `blocked`/`budget`.
- [ ] Extend report schema with revisions, changed files and acceptance results; consume make-worker-stop-durable generation identity and add scope/check warnings. Verify a stale report cannot satisfy new work and stopped settlement is not successful acceptance.
- [ ] Tests in `crates/jcode-app-core/src/server/comm_plan_tests.rs` and swarm tests for success and warning paths.
- [ ] Run `cargo test -p jcode-plan` and `cargo test -p jcode-app-core swarm`.
- [ ] Run `openspec validate add-swarm-handoff-contracts --strict`.

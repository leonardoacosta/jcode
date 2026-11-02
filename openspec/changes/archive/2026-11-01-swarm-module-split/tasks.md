## 1. Split

- [x] 1.1 Record baseline: `cargo test -p jcode-app-core swarm` test count and pass list
- [x] 1.2 `git mv crates/jcode-app-core/src/server/swarm.rs crates/jcode-app-core/src/server/swarm/mod.rs`; move tests verbatim to `swarm/tests.rs`
- [x] 1.3 Extract `tree.rs`, `config.rs`, `tasks.rs`, `status.rs`, `plan.rs`, `lifecycle.rs`, `run.rs` with code moved verbatim; re-export from `mod.rs` preserving visibility
- [x] 1.4 Verify no file outside `server/swarm/` changed and no visibility widened (`git diff --stat`, grep `pub ` in new files)
- [x] 1.5 `cargo test -p jcode-app-core swarm` (same count as baseline) and `openspec validate swarm-module-split --strict`

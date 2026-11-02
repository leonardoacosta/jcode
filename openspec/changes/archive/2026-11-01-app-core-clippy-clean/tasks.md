## 1. Fix lints

- [x] 1.1 Release `MutexGuard`s before `.await` in `server/turn_failover.rs` (6 sites)
- [x] 1.2 Fix `items_after_test_module` in `agent/idle_hibernation.rs` and `server/swarm_todo_link.rs`
- [x] 1.3 Fix `field_reassign_with_default`, `collapsible_if`, `redundant_closure_call`, `type_complexity` in remaining app-core files
- [x] 1.4 Fix `collapsible_if` x2 in `crates/jcode-tui-mermaid/examples/image_lifecycle.rs`
- [x] 1.5 Run `cargo clippy --workspace --keep-going --all-targets --all-features -- -D warnings`; fix any newly surfaced crates

## 2. Guard and verify

- [x] 2.1 Confirm the CI clippy step exists; marking it a required check in GitHub branch protection is a manual step for the user (not done: remote setting)
- [x] 2.2 `cargo test -p jcode-app-core` (touched modules: 83 passed)
- [x] 2.3 `openspec validate app-core-clippy-clean --strict`

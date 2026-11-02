## 1. Implementation

- [ ] 1.1 Add cancellation select branch and abort handling in `run_turn_monitored`
- [ ] 1.2 Track and stop run-owned bg tasks and swarm workers on cancel
- [ ] 1.3 Add `max_agents`, `max_turn_minutes`, `max_run_tokens` manifest fields and enforcement
- [ ] 1.4 Tests: cancel aborts active turn; spawn beyond cap refused; token cap ends run

## 2. Verification

- [ ] 2.1 `cargo test -p jcode-app-core` passes for the new tests
- [ ] 2.2 `cargo clippy --workspace --all-targets -- -D warnings`
- [ ] 2.3 `openspec validate enforce-overnight-cancellation-and-limits --strict`

## 1. Implementation

- [ ] 1.1 Add admission event types to `jcode-overnight-core`
- [ ] 1.2 Gate the loop in `run_supervisor` on the admission decision
- [ ] 1.3 Implement wait-for-signal using bg/swarm completion notifications plus wake boundary
- [ ] 1.4 Add auth-blocked preflight check
- [ ] 1.5 Add tests: idle run issues no extra turns; completion signal admits one turn; all-auth-expired run blocks

## 2. Verification

- [ ] 2.1 `cargo test -p jcode-app-core` passes for the new tests
- [ ] 2.2 `cargo clippy --workspace --all-targets -- -D warnings`
- [ ] 2.3 `openspec validate admit-overnight-tasks-on-evidence --strict`

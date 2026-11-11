## 1. Implementation

- [ ] 1.1 Add admission event types to `jcode-overnight-core`
- [ ] 1.2 Gate the loop in `run_supervisor` on the admission decision
- [ ] 1.3 Implement wait-for-signal using bg/swarm completion notifications plus wake boundary
- [ ] 1.4 Use selected-route auth classification and existing strict deterministic task-admission policy; preserve original scope/owner/limits, cancellation and authoritative monetary reservations without probing other accounts or creating another admission engine.
- [ ] 1.5 Tests: idle run issues no extra turns; completion signal admits at most one otherwise-authorized turn; selected-auth failure, unknown authority, unsupported monetary capability and unresolved predecessor all block without effects.

## 2. Verification

- [ ] 2.1 `cargo test -p jcode-app-core` passes for the new tests
- [ ] 2.2 `cargo clippy --workspace --all-targets -- -D warnings`
- [ ] 2.3 `openspec validate admit-overnight-tasks-on-evidence --strict`

## 1. Implementation

- [ ] 1.1 Elect a coordinator from separate swarm-index and member-map snapshots; verify coordinator-election unit coverage passes.
- [ ] 1.2 Add stage events for subscribe lock acquisition and join-event persistence; verify formatting and compile checks pass.

## 2. Verification and rollout

- [ ] 2.1 Run focused `jcode-app-core` tests and the full crate test suite; record fresh results.
- [ ] 2.2 Build with the self-dev profile and verify a subscribe smoke test on an isolated socket.
- [ ] 2.3 Restart the shared daemon and verify it serves the newly built binary without changing session data.

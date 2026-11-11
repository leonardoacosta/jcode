## 1. Implementation

- [ ] 1.1 Add supervisor PID/heartbeat fields
- [ ] 1.2 Consume the configured admission and background-ownership contracts for startup reconciliation against persisted scope, reservations/remaining budgets, dependencies and current authorization. Verify unresolved predecessor/child work stays interrupted, cancellation never resumes and archive presence never substitutes for live monetary capability.
- [ ] 1.3 Add `Interrupted` terminal status handling in TUI views
- [ ] 1.4 Tests: eligible authorized task resumes independent of overnight clock; CancelRequested never resumes; live owner prevents duplication; policy rejection stays interrupted; broader config cannot widen prior authorization

## 2. Verification

- [ ] 2.1 `cargo test -p jcode-app-core` passes for the new tests
- [ ] 2.2 `cargo clippy --workspace --all-targets -- -D warnings`
- [ ] 2.3 `openspec validate reconcile-overnight-runs-after-restart --strict`

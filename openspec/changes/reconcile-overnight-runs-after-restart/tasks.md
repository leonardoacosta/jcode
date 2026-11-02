## 1. Implementation

- [ ] 1.1 Add supervisor PID/heartbeat fields
- [ ] 1.2 Implement startup reconciliation scan
- [ ] 1.3 Add `Interrupted` terminal status handling in TUI views
- [ ] 1.4 Tests: stale Running resumes; CancelRequested never resumes; past-wake marked interrupted

## 2. Verification

- [ ] 2.1 `cargo test -p jcode-app-core` passes for the new tests
- [ ] 2.2 `cargo clippy --workspace --all-targets -- -D warnings`
- [ ] 2.3 `openspec validate reconcile-overnight-runs-after-restart --strict`

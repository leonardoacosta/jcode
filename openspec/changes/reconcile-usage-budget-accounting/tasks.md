## 1. Implementation

- [ ] 1.1 Add per-run usage aggregator keyed by run id
- [ ] 1.2 Attribute worker/resumed-session usage to the run
- [ ] 1.3 Emit `usage_reconciled` and render with limit-bar
- [ ] 1.4 Tests: aggregation across worker sessions; missing usage not zero

## 2. Verification

- [ ] 2.1 `cargo test -p jcode-app-core` passes for the new tests
- [ ] 2.2 `cargo clippy --workspace --all-targets -- -D warnings`
- [ ] 2.3 `openspec validate reconcile-usage-budget-accounting --strict`

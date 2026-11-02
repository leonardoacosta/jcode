## 1. Implementation

- [ ] 1.1 Define scenario list and coverage model
- [ ] 1.2 Implement `--coverage` flag with table and JSON
- [ ] 1.3 Tests: not-covered and skipped rendering; JSON schema snapshot

## 2. Verification

- [ ] 2.1 `cargo test -p jcode-provider-doctor` passes for the new tests
- [ ] 2.2 `cargo clippy --workspace --all-targets -- -D warnings`
- [ ] 2.3 `openspec validate report-provider-doctor-coverage --strict`

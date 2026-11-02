## 1. Implementation
- [ ] 1.1 Define verdict type and JSON schema
- [ ] 1.2 Add `jcode review --json` running a read-only reviewer session and emitting one verdict
- [ ] 1.3 Map verdict to exit codes; invalid output to `inconclusive`
- [ ] 1.4 Update autoreview prompt in `commands_review.rs` to request and render the verdict

## 2. Verification
- [ ] 2.1 Unit tests for verdict parsing: valid pass/fail, malformed -> inconclusive, cancel -> cancelled
- [ ] 2.2 `cargo test -p jcode-tui commands_review` and `cargo test -p jcode review`
- [ ] 2.3 Runtime: `./target/selfdev/jcode review --json` on a dirty test repo; confirm revision and exit code
- [ ] 2.4 `openspec validate add-structured-headless-review --strict`

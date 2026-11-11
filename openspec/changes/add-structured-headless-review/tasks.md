## 1. Implementation
- [ ] 1.1 Define verdict type/schema with base/head and exact scoped diff/content digest including reviewed untracked files; test two different dirty diffs at identical HEAD never share a valid pass and changed checks invalidate the binding. Read-only reviewer cancellation reuses existing controls; add generation compatibility cases when make-worker-stop-durable lands without inventing a hard dependency for verdict formatting.
- [ ] 1.2 Add `jcode review --json` running a read-only reviewer session and emitting one verdict
- [ ] 1.3 Map verdict to exit codes; invalid output to `inconclusive`
- [ ] 1.4 Update autoreview prompt in `commands_review.rs` to request and render the verdict

## 2. Verification
- [ ] 2.1 Unit tests for verdict parsing: valid pass/fail, malformed -> inconclusive, cancel -> cancelled
- [ ] 2.2 `cargo test -p jcode-tui commands_review` and `cargo test -p jcode review`
- [ ] 2.3 Runtime: `./target/selfdev/jcode review --json` on a dirty test repo; confirm revision and exit code
- [ ] 2.4 `openspec validate add-structured-headless-review --strict`

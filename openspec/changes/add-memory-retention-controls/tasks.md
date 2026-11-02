## 1. Implementation
- [ ] 1.1 Add `jcode memory list|purge` CLI with scope/inactive/age filters; dry-run default
- [ ] 1.2 Implement filtered purge in `crates/jcode-base/src/memory.rs` reusing `forget`
- [ ] 1.3 Reject the entire secret-bearing remember request before persistence using existing detection helpers; safe error, sanitized resubmission test
- [ ] 1.4 Add default-off `[memory.retention]` auto-purge for inactive/superseded entries

## 2. Verification
- [ ] 2.1 Unit tests: dry-run deletes nothing; apply respects filters; secret redaction; ordinary content unchanged
- [ ] 2.2 `cargo test -p jcode-base memory`
- [ ] 2.3 Runtime: `./target/selfdev/jcode memory purge --inactive` on a test `JCODE_HOME`
- [ ] 2.4 `openspec validate add-memory-retention-controls --strict`

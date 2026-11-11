## 1. Implementation
- [ ] 1.1 Add `jcode memory list|purge` CLI with scope/inactive/age filters; dry-run default
- [ ] 1.2 Implement filtered purge in `crates/jcode-base/src/memory.rs` reusing `forget`
- [ ] 1.3 Reject the entire secret-bearing remember request before persistence using existing detection helpers; safe error, sanitized resubmission test
- [ ] 1.4 Add default-off `[memory.retention]` auto-purge for inactive/superseded entries

## 2. Verification
- [ ] 2.1 Unit and public-memory-interface tests: dry-run deletes nothing; apply respects filters; retention absent/off deletes nothing; explicit retention respects scope/inactive/age; concurrent recall/save cannot resurrect purged IDs; secret rejection does not echo content and ordinary content remains unchanged. Verify known-pattern detection is never labeled comprehensive confidentiality screening.
- [ ] 2.2 `cargo test -p jcode-base memory`
- [ ] 2.3 Runtime: `./target/selfdev/jcode memory purge --inactive` on a test `JCODE_HOME`
- [ ] 2.4 `openspec validate add-memory-retention-controls --strict`

### Visible overnight bounded secret-rejection slice, 2026-11-09 21:40 UTC

JIT preflight-check-v2-shadow: approved requirement1.3 only, synthetic known-pattern secrets/private storage, no purge/data migration/auto-retention/config changes. HEAD7abc9a0da main preserved/default dev mismatch. Clean targets coordinator-owned memory.rs,memory_tests.rs,app-core/tool/memory.rs serialized. Reuse existing message::redact_secrets detector and remember manager/tool boundaries, reject full request before activity/state/persistence, safe reason, sanitized resubmit. Public Tool::execute/private JCODE_HOME and manager APIs available; no provider call required. G1=2,G2=1(independent review absent),G3=1(branch uncertainty),G4=2(private test state),G5=2(clean exact targets),G6=1(full purge/retention not implemented):9/12 shadow. Do not mark whole proposal complete or treat known-pattern detection as comprehensive confidentiality screening.

### Known-pattern remember rejection verified, 2026-11-09 21:47 UTC

Manager and actual MemoryTool regressions first persisted synthetic known token and echoed Remembered; now existing redact_secrets detector rejects whole remember before activity/state/storage, fixed safe error, tags checked before content activity, manager additionally checks search text/source. Sanitized resubmission and ordinary code/Unicode content retained. Existing bare password assignment is not a supported detector pattern; test uses existing SERVICE_PASSWORD grammar, no new detection claim. Other graph import/update/tag action paths and legacy data are not covered by this remember-only gate.

50manager+6tool tests and known-pattern matrix pass; actual normal rebuilt private daemon public tool:memory rejects content/tag requests with safe reason, no rejected memory bytes, resubmit succeeds, binaryf902f29d33402be97ed98a668d2cef0ab0295e7247392e84c0b7679aa5935c18. Current run task-008 receipts contain red/green/public output. Workspace all-targetClippy passes with existing unrelated jev.rs732 deprecation warning; strict-D warning gate remains blocked. No inference/live credential/shared activation. Documentation states limited screening. Purge/default-off retention/concurrency requirements and mandatory review/commit/archive remain unfinished, native checkboxes unchanged.

## 1. Implementation
- [x] 1.1 Add `jcode memory list|purge` CLI with scope/inactive/age filters; dry-run default
- [x] 1.2 Implement filtered purge in `crates/jcode-base/src/memory.rs` reusing `forget`
- [x] 1.3 Reject the entire secret-bearing remember request before persistence using existing detection helpers; safe error, sanitized resubmission test
- [x] 1.4 Add default-off `[memory.retention]` auto-purge for inactive/superseded entries

## 2. Verification
- [x] 2.1 Unit and public-memory-interface tests: dry-run deletes nothing; apply respects filters; retention absent/off deletes nothing; explicit retention respects scope/inactive/age; concurrent recall/save cannot resurrect purged IDs; secret rejection does not echo content and ordinary content remains unchanged. Verify known-pattern detection is never labeled comprehensive confidentiality screening.
- [x] 2.2 `cargo test -p jcode-base memory`
- [x] 2.3 Runtime: `./target/selfdev/jcode memory purge --inactive` on a test `JCODE_HOME`
- [x] 2.4 `openspec validate add-memory-retention-controls --strict`

### Visible overnight bounded secret-rejection slice, 2026-11-09 21:40 UTC

JIT preflight-check-v2-shadow: approved requirement1.3 only, synthetic known-pattern secrets/private storage, no purge/data migration/auto-retention/config changes. HEAD7abc9a0da main preserved/default dev mismatch. Clean targets coordinator-owned memory.rs,memory_tests.rs,app-core/tool/memory.rs serialized. Reuse existing message::redact_secrets detector and remember manager/tool boundaries, reject full request before activity/state/persistence, safe reason, sanitized resubmit. Public Tool::execute/private JCODE_HOME and manager APIs available; no provider call required. G1=2,G2=1(independent review absent),G3=1(branch uncertainty),G4=2(private test state),G5=2(clean exact targets),G6=1(full purge/retention not implemented):9/12 shadow. Do not mark whole proposal complete or treat known-pattern detection as comprehensive confidentiality screening.

### Known-pattern remember rejection verified, 2026-11-09 21:47 UTC

Manager and actual MemoryTool regressions first persisted synthetic known token and echoed Remembered; now existing redact_secrets detector rejects whole remember before activity/state/storage, fixed safe error, tags checked before content activity, manager additionally checks search text/source. Sanitized resubmission and ordinary code/Unicode content retained. Existing bare password assignment is not a supported detector pattern; test uses existing SERVICE_PASSWORD grammar, no new detection claim. Other graph import/update/tag action paths and legacy data are not covered by this remember-only gate.

50manager+6tool tests and known-pattern matrix pass; actual normal rebuilt private daemon public tool:memory rejects content/tag requests with safe reason, no rejected memory bytes, resubmit succeeds, binaryf902f29d33402be97ed98a668d2cef0ab0295e7247392e84c0b7679aa5935c18. Current run task-008 receipts contain red/green/public output. Workspace all-targetClippy passes with existing unrelated jev.rs732 deprecation warning; strict-D warning gate remains blocked. No inference/live credential/shared activation. Documentation states limited screening. Purge/default-off retention/concurrency requirements and mandatory review/commit/archive remain unfinished, native checkboxes unchanged.

### Retention controls delivered, 2026-11-20

Purge: `jcode memory list|purge` (`--scope`, `--inactive`, `--older-than`), dry run by default, `--apply` required, no-filter purge refused. Core in `crates/jcode-base/src/memory/retention.rs` (a new module, not `forget`), under the store lock. Tombstones (`memory/purged-ids.json`) are per graph file and record the purged `updated_at`, so a stale snapshot cannot resurrect a purged entry while a newer write of the same id is kept. Legacy-note re-import honors them. Auto purge: `[memory.retention]` enabled + auto_purge + age > 0, global store only, inactive entries only, config read from the global `config.toml` only.

Independent review (sonnet 5.5, read-only) found 1 blocker, 4 major, 8 minor. Fixed with regression tests: B1 new files were git-ignored by `.git/info/exclude` (force-added); M1 cross-graph tombstone collision; M2 newer write after purge dropped; M3 `import` miscounted dropped entries; M4 `--older-than` overflow wiped everything (now matches nothing); m2 legacy-note reload; m3 corrupt tombstone names the file and logs; m4 save before tombstone; m5 auto purge scope is global, project scope without a directory errors; m6 preview redacted; m7 rustfmt order. Not fixed: m1 loaders still write on first load (normalize and legacy migration, pre-existing, idempotent); lock failure on save proceeds unlocked by design; a corrupt tombstone file fails open for saves and blocks purge until repaired.

Evidence: clippy `-D warnings` clean (jcode-config-types, jcode-base, jcode, all targets); jcode-base `memory::` 96 pass; jcode-config-types 47 pass; `jcode --lib memory_cli` 7 pass; app-core `tool::memory` 8 pass; `openspec validate --strict`. Runtime 2.3 ran earlier on a private `JCODE_HOME` with the selfdev binary: no-filter refused, dry run deleted nothing, apply removed 1 of 3, second apply idempotent. Real `~/.jcode/memory` never touched. The review fixes were verified by tests only, not a rebuilt binary. Known unrelated flake: 3 to 6 `hindsight_memory_scope` tests fail intermittently with `QueueFull`, also on a pristine tree. The documented known-pattern detector is not comprehensive confidentiality screening.

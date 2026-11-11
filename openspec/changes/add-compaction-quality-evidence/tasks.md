## 1. Implementation
- [ ] 1.1 Define fixture format (messages + must-retain facts) under `crates/jcode-compaction-core/tests/fixtures/`
- [ ] 1.2 Add 3+ fixtures: long coding task, resumed conversation, conflicting/updated requirement
- [ ] 1.3 Implement deterministic checker over `build_compaction_prompt` and scripted summaries
- [ ] 1.4 Add opt-in live mode with model/revision labelling and `inconclusive` on failure

## 2. Verification
- [ ] 2.1 `cargo test -p jcode-compaction-core`
- [ ] 2.2 Negative test: scripted summary missing a fact yields `failed`
- [ ] 2.3 `openspec validate add-compaction-quality-evidence --strict`
- [ ] 2.4 Exercise the actual public compaction interface with permitted fixtures and bind results to model/source/input digests; label checker-only evidence, missing access and unknown quality explicitly. Obtain separate authorization before real-session capture or provider disclosure; no private export or paid call is authorized by this task list alone.

### Visible overnight offline subset admission, 2026-11-09 21:35 UTC

JIT preflight-check-v2-shadow: HEAD7abc9a0da, current main preserved/default dev mismatch, empty prerequisites, synthetic fixtures only authorized by this run; no private export/provider calls/reviewer helper. Coordinator owns new compaction-core tests/fixtures and example checker, no existing source algorithm/prompt edits. Public build_compaction_prompt API and scripted-summary checker route available; actual provider-produced summary route excluded until scoped disclosure/access. G1=2,G2=1(no model/live quality/review),G3=1(default mismatch),G4=2(offline),G5=2(new scoped files),G6=1(actual provider acceptance pending):9/12 shadow. Implement fixture/checker coverage only, leave live/interface quality and native task completion unchecked. No claim of better compaction from scripted-summary success.

### Offline fixture/checker subset, 2026-11-09 21:38 UTC

Added synthetic coding/resumed/updated fixtures, real public build_compaction_prompt input assembly, literal must-retain ID checker, named utf8-bytes-div4-ceil-v1 approximation, positive/omitted-fact negative cases and no-summary inconclusive. Test-first missing evaluate E0425 then checker tests pass; full compaction-core suite passes. Input facts are checked against actual prompt and unique/nonempty IDs/text. No production compaction algorithm/prompt changed, no actual provider summary produced, no live mode/model quality claim. Fixture README documents exact limits (literal presence not semantic/contradiction detection; pre count includes summary instruction). Actual provider/model/revision/input-digest/live requirements and review/commit/archive remain unchecked. Receipts current run task-007.

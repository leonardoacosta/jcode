## 1. Implementation
- [ ] 1.1 Define fixture format (messages + must-retain facts) under `crates/jcode-compaction-core/tests/fixtures/`
- [ ] 1.2 Add 3+ fixtures: long coding task, resumed conversation, conflicting/updated requirement
- [ ] 1.3 Implement deterministic checker over `build_compaction_prompt` and scripted summaries
- [ ] 1.4 Add opt-in live mode with model/revision labelling and `inconclusive` on failure

## 2. Verification
- [ ] 2.1 `cargo test -p jcode-compaction-core`
- [ ] 2.2 Negative test: scripted summary missing a fact yields `failed`
- [ ] 2.3 `openspec validate add-compaction-quality-evidence --strict`

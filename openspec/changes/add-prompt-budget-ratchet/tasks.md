## 1. Measurement

- [ ] 1.1 Expose per-section prompt assembly (system, instructions, skills) from `crates/jcode-app-core/src/agent/prompting.rs`
- [ ] 1.2 Deterministic offline token counter and fixture workspace

## 2. Ratchet

- [ ] 2.1 `prompt-budget.toml` format and `jcode debug prompt-budget --check` with tolerance
- [ ] 2.2 `--accept --reason <text>` updates budget and appends ledger entry; reject empty reason
- [ ] 2.3 Add check to `.github/workflows/ci.yml`

## 3. Ledger

- [ ] 3.1 `prompt-budget-ledger.jsonl` entries: commit, section deltas, optional experiment score delta, decision KEEP/REVERT, reason
- [ ] 3.2 Wire `scripts/prepare_personal_prompt_eval.py` result into ledger entries

## 4. Tests and validation

- [ ] 4.1 Tests: under budget passes, over budget fails naming the section, accept requires reason, ledger append-only
- [ ] 4.2 `openspec validate add-prompt-budget-ratchet --strict`

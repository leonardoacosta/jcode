## 1. Setup

- [x] 1.1 Pin commit; write ~20 queries with ground-truth paths/symbols (2026-11-04: clean `git archive` snapshot of `6393b322b481453c058c504cec087d982abc74ea`; all 1,613 Rust file hashes match base Git blobs. All 20 answer paths and lexical definition/call anchors checked; eight answer-only symbol fields added without changing adapter priors. This is not semantic symbol resolution or complete caller ground truth. Historical pin conflict remains historical.)
- [x] 1.2 Harness script logging tool calls, tokens, recall@k, wall time (`harness.py` now records raw commands, stderr/exit/timeout, complete returned path ranks, true file recall against listed answers, explicit output-only character estimates, source/index fingerprints, and matched local query IDs. Regression and local CLI checks passed. This does not measure model tokens or agent turns.)

## 2. Runs

- [ ] 2.1 Run agentgrep, graft, ast-grep, search subagent, repo map; same model for agent runs (2026-11-04: four local CLI/heuristic arms run on clean pinned source with fresh deterministic Graft graph, network-isolated; 72 scored pairs, eight unsupported ast-grep cases, 20 explicit subagent skips, no errors/drift. No fixed-model agent trials or search-subagent results exist. Current owner scope forbids live calls/spend/spawning. Exact model, budget and disclosure recommendation is in README, not authorization.)
- [x] 2.2 Verify external claims from `scratch/grep.md` or mark unverified (all external performance figures remain unverified in this validation. The prior WarpGrep attribution was not rechecked. No live-web or network model research performed.)

## 3. Report

- [ ] 3.1 Results table and adopt/reject per strategy (historical table preserved with integrity caveats. Corrected JSON reports all ranks and the same 12 local query IDs, not five matched-model runs. README records provisional observations only; the prior superiority and new-adoption conclusions are withdrawn.)
- [ ] 3.2 File follow-up changes for adopted strategies (not started. Ast-grep product adoption or a new proposal requires separate approval and is outside this validation.)

## Validation checkpoint: 2026-11-02

This change remains incomplete and unarchived. Historical `results.json` is unchanged.
Scratch evidence and exact commands are listed in `docs/research/code-search-eval/README.md`.
Local validation does not substitute for the proposal's verification, independent review,
commit, or archive gates. No commit, push, or shared runtime reload performed by this validation.

## Offline preparation checkpoint: 2026-11-04

Evidence: `scratch/code-search-offline.3a80pt/final-local-results.json`,
`final-tests.log`, `graft-build.log`, `audit.log` and `final-audit.log`.
19 regression tests pass. Clean pinned-source/index run exits 0 with `partial` status;
local recall matches the corrected README table, not a matched-model acceptance result.
Historical `results.json` bytes and numeric `table.md` rows remain unchanged.
Commands, provenance limits and the proposed `openai` / `gpt-5.4`, USD 10 total ceiling,
source/snippet disclosure decision are in README. Tasks 2.1, 3.1 and 3.2 remain blocked
on live-run authorization/evidence and final decisions. No follow-up proposal, shared
Git mutation, commit, archive or worker spawning is authorized in this owner scope.

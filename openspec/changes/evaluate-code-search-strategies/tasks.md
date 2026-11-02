## 1. Setup

- [ ] 1.1 Pin commit; write ~20 queries with ground-truth paths/symbols (20 queries exist in `queries.json`, with answer paths present at both candidate revisions. README originally named `03a269d3`, but historical `results.json` records `ff20fae3`; the dirty source tree was not captured. No clean pinned-source run is established.)
- [x] 1.2 Harness script logging tool calls, tokens, recall@k, wall time (`harness.py` now records raw commands, stderr/exit/timeout, complete returned path ranks, true file recall against listed answers, explicit output-only character estimates, source/index fingerprints, and matched local query IDs. Regression and local CLI checks passed. This does not measure model tokens or agent turns.)

## 2. Runs

- [ ] 2.1 Run agentgrep, graft, ast-grep, search subagent, repo map; same model for agent runs (four local CLI/heuristic arms reproduced in isolated scratch output on 2026-11-02. No fixed-model agent trials or search-subagent results exist. Do not infer missing model/budget authorization from the old runner's hard-coded reason.)
- [x] 2.2 Verify external claims from `scratch/grep.md` or mark unverified (all external performance figures remain unverified in this validation. The prior WarpGrep attribution was not rechecked. No live-web or network model research performed.)

## 3. Report

- [ ] 3.1 Results table and adopt/reject per strategy (historical table preserved with integrity caveats. Corrected JSON reports all ranks and the same 12 local query IDs, not five matched-model runs. README records provisional observations only; the prior superiority and new-adoption conclusions are withdrawn.)
- [ ] 3.2 File follow-up changes for adopted strategies (not started. Ast-grep product adoption or a new proposal requires separate approval and is outside this validation.)

## Validation checkpoint: 2026-11-02

This change remains incomplete and unarchived. Historical `results.json` is unchanged.
Scratch evidence and exact commands are listed in `docs/research/code-search-eval/README.md`.
Local validation does not substitute for the proposal's verification, independent review,
commit, or archive gates. No commit, push, or shared runtime reload performed by this validation.

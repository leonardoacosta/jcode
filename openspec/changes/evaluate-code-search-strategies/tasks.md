## 1. Setup

- [x] 1.1 Pin commit; write ~20 queries with ground-truth paths/symbols (20 queries at 03a269d3, `docs/research/code-search-eval/queries.json`)
- [x] 1.2 Harness script logging tool calls, tokens, recall@k, wall time (`harness.py`; tokens = chars/4 estimate)

## 2. Runs

- [ ] 2.1 Run agentgrep, graft, ast-grep, search subagent, repo map; same model for agent runs (agentgrep, graft, ast-grep, repo map run; search subagent NOT run: no LLM budget authorized)
- [x] 2.2 Verify external claims from `scratch/grep.md` or mark unverified (WarpGrep claim source confirmed, number unreproduced; others marked unverified)

## 3. Report

- [x] 3.1 Results table and adopt/reject per strategy (`docs/research/code-search-eval/README.md`)
- [ ] 3.2 File follow-up changes for adopted strategies (pending: ast-grep complement integration not yet filed)

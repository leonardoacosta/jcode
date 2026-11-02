# Code search evaluation

Pinned commit: `03a269d3b9f82e6649fe2053f666ebcdb5d16dc0` (HEAD at run time, 2026-11-02; working tree had unrelated uncommitted edits).
Files: `queries.json` (20 queries, ground-truth files), `harness.py`, `results.json` (raw rows), `table.md` (per query: r@1/r@5, tokens, wall ms).

Re-run: `python3 docs/research/code-search-eval/harness.py --agentgrep <agentgrep-bin> > results.json`
(agentgrep v0.1.7 CLI built from the pinned git dep; graft 0.21.1 with `--no-refresh`; ast-grep from `~/.empryo/bin`).

## Method

- 20 queries: 7 exact-symbol, 7 concept, 2 caller, 4 cross-crate. Every answer path asserted to exist.
- Recall is file-level: a hit when a ground-truth file is in the strategy's top 1 / top 5 ranked paths.
- Tokens = output characters / 4 (estimate, not a tokenizer). Tool calls = CLI invocations per query.
- Each strategy gets one non-iterative query formulation. Agent-driven refinement is not modelled.
- agentgrep: `grep` for symbol/caller queries (definition files ranked first), `find` for concept queries.
- repo map: Empryo/aider-style map built from `git ls-files *.rs` + regex definitions (fn/struct/enum/trait), scored by query tokens vs path and symbol tokens. This is an approximation; Empryo's own `genome.db` was not queried.

## Results (n = queries scored)

| strategy | n | recall@1 | recall@5 | calls/query | median tokens | median wall ms |
|---|---|---|---|---|---|---|
| agentgrep | 20 | 0.90 | 1.00 | 1.00 | 106 | 11 |
| graft ask | 20 | 0.45 | 0.75 | 1.00 | 999 | 1114 |
| ast-grep | 12 | 1.00 | 1.00 | 1.92 | 146 | 477 |
| repo map | 20 | 0.80 | 0.95 | 1.00 | 111 | 43 |
| search subagent | 0 | not run | | | | |

- ast-grep: n/a on the 8 concept queries (no structural pattern exists). First run with a single pattern per query scored 0.25; it needs the exact shape (unit vs braced struct, `self.` method call), hence ~2 calls.
- graft misses: q03, q12, q17, q18, q19 (top-5); top-1 often a related doc/test node.
- repo map misses: q16 (caller query; map has no call edges).
- agentgrep top-1 misses: q11, q19.
- search subagent: not run. Requires ~20 paid LLM agent sessions on one pinned model; no budget authorized in this change.

## Decisions

| strategy | decision | rationale |
|---|---|---|
| agentgrep | adopt (keep as default) | best recall@5 (1.00) at lowest latency and low tokens across all query kinds |
| ast-grep | adopt (complement) | perfect on symbol/caller queries when the shape is known; useless for concept queries; ~2 calls |
| repo map | reject (as a separate tool) | 0.95 r@5 but no callers; agentgrep `find` already covers the same signal |
| graft ask | reject (for location) | 0.75 r@5, ~10x tokens and ~100x latency of agentgrep on this set |
| search subagent | reject (for now) | unmeasured; agentgrep reaches r@5 = 1.00 in one call, leaving no headroom to justify agent cost |

## External claims (`scratch/grep.md`)

- WarpGrep "0.73 F1 in ~4 steps": verified that letta-ai/skills `tools/morph-warpgrep/SKILL.md` states it (fetched 2026-11-02). The underlying benchmark was not reproduced: unverified as a measurement.
- Cursor semantic search, zilliz claude-context, zoekt, serena, probe: unverified (not checked in this change).

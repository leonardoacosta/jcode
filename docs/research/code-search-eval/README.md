# Code search evaluation: partial, not a completed comparison

The approved proposal requires five strategies and a fixed model for agent runs.
Only four local CLI/heuristic adapters have run. No matched-model agent trials,
search-subagent measurements, or final adoption decision are established.
`tasks.md` remains open. No product integration or new proposal is authorized here.

## Historical evidence and integrity findings

`results.json` and the numeric rows in `table.md` are preserved historical measurements,
not output from the corrected harness. Their limitations are material:

- **Source pin conflict:** the original README named `03a269d3b9f82e6649fe2053f666ebcdb5d16dc0`,
  but `results.json.commit` is `ff20fae3870b0d5104bccbfaa867cdcbdbf80f8a`.
  The README disclosed a dirty tree, but neither source contents nor graph fingerprints
  were recorded. A HEAD value is not a pinned input snapshot. All 20 answer paths exist
  at both revisions, which does not prove source or index equivalence.
- **Top-five audit gap:** the old runner saved only three candidate paths per row,
  with no stdout, stderr, exit code, or exact invocation. In particular, the reported
  top-five successes for agentgrep `q19` and graft `q10` lack their target in the stored
  top-three paths. Those claims cannot be independently rescored from committed data.
  They are not proven false, but a matching rerun does not repair historical provenance.
- **Agentgrep was an assisted adapter:** symbol results were reranked after reading
  source definitions in Python. Those reads were absent from call/token counts and
  command timings. Its reported `0.90/1.00` cannot be attributed to native returned
  order or used to claim the product's default tool is superior.
- **Unequal queries and priors:** ast-grep scored 12 queries, the other local arms 20.
  Its eight unsupported cases are seven concept queries plus cross-crate `q19`, not
  eight concept queries. It received hand-authored structural shapes, sometimes two
  per query. The earlier single-pattern `0.25` claim has no saved trial artifact here
  and is withdrawn. No tuning split or held-out test set was recorded.
- **Metrics and missing arm:** the old `r1/r5` scorer measured any-listed-target hits,
  not general recall over every relevant file. The current fixture lists one positive
  file per query, so those definitions coincide only for this fixture. Tokens were
  stdout characters divided by four, not model usage. The skipped subagent row's
  hard-coded assertion about absent authorization is not authorization evidence.

Original aggregates, recomputed from preserved rows:

| Historical adapter | Scored queries | Reported hit@1 | Reported hit@5 |
|---|---:|---:|---:|
| agentgrep + definition reranking | 20 | 0.90 | 1.00 |
| graft text-path extraction | 20 | 0.45 | 0.75 |
| ast-grep + curated patterns | 12 | 1.00 | 1.00 |
| regex repo map | 20 | 0.80 | 0.95 |
| search subagent | 0 | not run | not run |

These rows do not form a matched five-strategy benchmark. No precision, full caller
recall, model efficiency, or production workload advantage follows from them.

## Corrected harness contract

- `queries.json` retains the original 20 queries: seven symbols, seven concepts,
  two callers, four cross-crate. File existence is checked, not exhaustive relevance
  or symbol-level correctness. One curated target per query favors location tasks.
- All local adapters target Rust paths under `crates/` and `src/`. Agentgrep uses
  JSON file records with an explicit source glob, without Python definition reranking.
  Graft uses only JSON `hits[].pointer`, never paths inside snippets, titles, or scope
  hints. ast-grep merges JSON match files in pattern/occurrence order. The repo map
  retains the approximate path/symbol heuristic and exact-symbol bonus, not Empryo
  itself. None of these is a controlled model-driven search-agent trial.
- Candidate order means **first distinct returned files**, not a promise of semantic
  relevance ranking. Graft returns at most eight nodes before file deduplication.
  Every returned path is retained, including ranks four and five. Missing source paths,
  failed commands, malformed output, and timeouts produce errors, not scored misses.
- `recall_at_k = |listed answers intersect first k files| / |listed answers|`.
  The JSON separately reports each adapter's available-query denominator and the
  identical query IDs successful across all four local arms. Unsupported/missing arms
  are not silently scored as zero or removed from the coverage report.
- Per-query `cli_calls` counts search subprocesses only. The repo map uses zero search
  subprocesses and records its one-time build time/file reads separately. Metadata and
  binary-version calls are outside query counts. Row wall time includes adapter parsing
  and scoring. `output_tokens_estimate` is captured stdout plus stderr characters / 4
  (repo map: returned path text / 4). It excludes model input, generation, and reasoning.
  JSON and path-only output sizes are not comparable model token budgets.
- JSON records exact argv, cwd, stdout, stderr, return code, elapsed time, binary paths,
  versions/entrypoint hashes, harness/query hashes, and before/after Rust source hashes,
  Git state, and graph fingerprint. Drift invalidates the run and returns exit 1.
  A stable dirty tree is labeled dirty, not clean or pinned. Fingerprints detect drift,
  not a recoverable historical snapshot or proof that the existing graph is fresh.
- `--output` creates a new file exclusively. Reusing a path fails before trials start.
  Missing executable arms and the absent subagent remain explicit. A successful local
  run is still `partial`; command errors or source drift also return a nonzero exit.

## Local validation, 2026-11-02

Artifacts are isolated in `scratch/code-search-validation.p5fYaY/`, not committed
measurements. The full legacy rerun (`legacy-all-results.json` and
`legacy-all-commands.jsonl`) reproduced all four historical hit-rate aggregates:
72 scored query/strategy pairs, 66 subprocess calls including preparation, zero
nonzero exits. An earlier legacy rerun omitted ast-grep because the background PATH
lacked its directory. This is preserved rather than hidden.

The corrected run is `corrected-results-2.json`: 72 scored pairs, eight unsupported
ast-grep cases, 20 explicit subagent skips, and zero command errors. Source, graph,
and harness/query fingerprints stayed stable during the run. Its source HEAD was
`2c8ad32600dde2dbd9900918573c0032d0ccae8a` with a **dirty tree**, 1,605 Rust source
hashes and 1,859 graph files, not a clean pinned snapshot. Versions were agentgrep
0.1.7, graft 0.21.1, and ast-grep 0.40.0. Its 12-query intersection is
`q01-q07`, `q15-q18`, `q20`. This excludes every concept query and `q19`.

| Corrected local adapter | Coverage / 20 | Available recall@1 | Available recall@5 | Same 12 recall@1 | Same 12 recall@5 |
|---|---:|---:|---:|---:|---:|
| agentgrep, native file order | 20 | 0.45 | 0.75 | 0.25 | 0.5833 |
| graft, existing graph | 20 | 0.45 | 0.75 | 0.50 | 0.75 |
| ast-grep, curated shapes | 12 | 1.00 | 1.00 | 1.00 | 1.00 |
| approximate regex repo map | 20 | 0.80 | 0.95 | 0.9167 | 0.9167 |

These are descriptive results against listed singleton targets, not a new universal
ranking. The corrected agentgrep arm intentionally differs from the historical
assisted adapter, so the score change is not evidence of a product regression.
The missing subagent is not included in this four-arm intersection.

Commands from the repository root:

```bash
# Regression suite, including real subprocess failure/timeout checks:
python3 -m unittest discover -s docs/research/code-search-eval -p test_harness.py -v

# Actual corrected local run, exclusive output creation:
python3 docs/research/code-search-eval/harness.py \
  --agentgrep /home/nyaptor/.jcode/scratch/agtarget/release/agentgrep \
  --graft /home/nyaptor/.local/share/mise/installs/node/25.0.0/bin/graft \
  --ast-grep /home/nyaptor/.empryo/bin/ast-grep \
  --output scratch/code-search-validation.p5fYaY/corrected-results-2.json

# Independent saved-evidence rescore, not the harness's scorer:
python3 scratch/code-search-validation.p5fYaY/audit.py

# For another run, use a fresh output path, not results.json or the path above.
git diff --check -- docs/research/code-search-eval \
  openspec/changes/evaluate-code-search-strategies/tasks.md
```

The first regression deliberately failed on the old runner because `--help` executed
the benchmark instead of showing usage. The repaired suite covers rank-five cutoffs,
multi-answer recall, structured-only ranking, missing/unsupported arms, stale paths,
command failures, timeouts, matched-query denominators, source drift, and overwrite
protection. All 17 tests passed. The independent audit rescored all 72 successful rows,
reconstructed the complete ranks from all 63 search-command stdout records, verified
matched-query IDs/aggregates, and confirmed preserved historical JSON bytes, numeric
table rows, and original task titles. `git diff --check` passed. Validation logs and
raw subprocess evidence remain in the scratch directory.

## Decision state and limits

Keep existing product behavior unchanged. The local smoke test provides no basis for
new ast-grep adoption, replacing graft, rejecting a search subagent, or claiming that
agentgrep is the best default across workloads. ast-grep's curated structural coverage
is promising but incomplete. The repo map is only an approximation. Final adopt/reject
conclusions remain pending clean pinned-source/index evidence and the missing fixed-model
agent trials. No follow-up implementation proposal was created.

External performance figures, including WarpGrep's reported ~0.73 F1 / ~4 steps,
remain unverified here. The notes attribute that number to a third-party skill, not a
reproduced Jcode measurement. No live-web or network model research was performed.

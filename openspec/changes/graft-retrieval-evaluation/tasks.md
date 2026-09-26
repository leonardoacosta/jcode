## 1. Benchmark contract and fixtures

- [ ] 1.1 Define versioned question and gold-evidence schema; validate required fields, allowed task classes, and paths against a pinned scratch repository using a self-check.
- [ ] 1.2 Select and manually label 25–40 real repository questions across conceptual location, exact symbol, exhaustive occurrence, caller/impact, and edit-target discovery; run an independent label review and record agreement/ambiguities.

## 2. Retrieval runner

- [ ] 2.1 Implement shell-free bounded execution for `graft ask`, `graft grep`, and `rg`, with explicit Graft freshness/version preflight; verify missing, stale, failing, timeout, malformed-output, and successful cases using fixtures.
- [ ] 2.2 Invoke Jcode `agentgrep` through an existing callable harness if present; otherwise record it as unavailable and add no production API solely for benchmarking. Verify this disposition from test/runner discovery.
- [ ] 2.3 Add scratch-only graph/repository setup with explicit opt-in, pinned source revision, cleanup, and proof tracked files/config stay unchanged after the default run.

## 3. Scoring and report

- [ ] 3.1 Score path precision/recall at documented K, reciprocal rank, optional symbol/span hits, output size, median latency/spread, failures, and freshness; verify metrics against hand-calculated fixture cases.
- [ ] 3.2 Emit versioned JSONL raw results and a Markdown summary containing source/fixture/tool versions, task mix, missing backends, and limitations; validate output schema and deterministic scoring on repeat runs.
- [ ] 3.3 Run the full evaluation twice on the same pinned scratch snapshot; confirm identical quality scores and metadata, bounded latency variance reporting, zero unintended tracked-file changes, and that conclusions do not include AST/LSP or agent task-success claims.

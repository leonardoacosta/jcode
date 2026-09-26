## Context

See `proposal.md` for motivation and `specs/repository-retrieval-evaluation/spec.md` for acceptance behavior. This is an offline developer evaluation, not a production retrieval change. Graft 0.20.0 exposes `ask --source`, `ask --no-graph-rank`, `grep`, freshness checking, and JSON output. The installed `agentgrep` Rust crate is a Cargo dependency, not an executable. The checkout has an empty `graft/` directory, so Graft queries currently fail until a graph is built. `graft build` may change repository ignore/config state, and `--dir` selects graph location.

## Goals / Non-Goals

**Goals:**
- Measure retrieval quality independently from model reasoning, on a fixed query set and source revision.
- Compare Graft's graph-ranked and lexical-only retrieval, its exhaustive structural grep, and Jcode's actual `agentgrep` implementation where a callable evaluation harness is available.
- Make stale or unavailable strategies visible; never contaminate source or silently skip failed tasks.
- Produce repeatable machine-readable output and a human-readable summary.

**Non-Goals:**
- Change Jcode runtime tool choice, prompts, MCP behavior, or default navigation.
- Implement AST/LSP support or use `graft build --lsp` as an AST/LSP substitute.
- Score end-to-end patch success in the initial feature; model and harness variance would confound retrieval attribution.
- Add hosted APIs, embedding services, telemetry, or new dependencies.
- Automatically create a Graft graph in the project root.

## Decisions

### Use fixed, manually judged retrieval questions

Create a modest initial set (target 25–40) from real repository tasks across conceptual location, exact symbol, exhaustive occurrence, caller/impact, and edit-target discovery. Each record contains a stable ID, query text, type, expected paths and optional expected symbol/line spans, plus grading notes. Labels must be established by reading source and independently checked before scoring. Keep the set out of agent-visible input and prevent answers or expected paths from leaking into query text.

Alternative: use only synthetic fixtures. Rejected as the sole benchmark because it misses real repository topology; a few tiny synthetic parser fixtures can remain test inputs, not scored tasks.

### Separate retrieval quality from agent task success

Initial scoring consumes raw retrieval outputs and gold evidence. Report file-path precision/recall at documented cutoffs, reciprocal rank of the first expected file, optional exact-symbol/span hit rate, output size in bytes (and token estimates only when a fixed tokenizer is available), latency, errors, and freshness. Do not call these coding success. An optional later feature can run fixed agents/prompts and score task completion after retrieval metrics are stable.

Alternative: compare complete agent coding runs immediately. Rejected for the first slice because model/harness randomness would obscure whether retrieval itself improved.

### Strategy set and fair execution

Run, when each backend can be invoked safely:
1. `graft ask --source -n K <query>` (graph ranking on).
2. Same Graft query with `--no-graph-rank` (ranking ablation).
3. `graft grep --fixed <symbol-or-query>` only for exact/exhaustive query classes; do not treat regex grep as semantic retrieval.
4. Jcode `agentgrep` mode(s), invoked through the existing tool implementation/test harness rather than inventing a nonexistent standalone binary. Include plain `rg` as an explicit lexical control when direct invocation is useful.

Use identical query text, repository snapshot, relevant scope, top-k/output budget where comparable. Record non-comparable limits rather than pretending the methods have identical APIs. Randomize strategy order and run multiple repetitions for latency; quality scores are deterministic where outputs are stable. Keep each backend's natural query semantics apparent in the report. If `agentgrep` cannot be driven externally without adding a product hook, mark it unavailable in version one and do not block the other comparisons.

### Use isolated data and avoid destructive setup

Build or copy a pinned repository snapshot into scratch. Place Graft state in an isolated `--dir` path if verified supported; otherwise stage the snapshot in scratch and build there. Do not permit implicit graph refresh during scored queries: check freshness first, capture graph/source revision, then run with refresh disabled where supported. Do not touch the checkout's `.gitignore`, `.ignore`, or source files. No deep LLM pass.

### CLI and result format

Use one small runner and a versioned JSONL result format plus a generated Markdown summary. Fail setup clearly if `graft` or `rg` is missing. Record Jcode `agentgrep` as unavailable, not silently omitted, if it cannot be driven from the runner without adding production hooks. Use shell-free process invocation, fixed timeouts, bounded stdout/stderr capture, and cleanup of temporary data.

## Risks / Trade-offs

- [Gold labels are incomplete or subjective] → define expected paths and evidence before running; review labels separately and retain grading notes.
- [Graft and agentgrep outputs differ in granularity] → report path metrics separately from symbol/span metrics and preserve raw bounded output.
- [Empty/stale Graft graph] → explicit preflight; no implicit build or auto-refresh.
- [Latency noise from cache and machine load] → warm-up and repeated runs; publish median and spread, not one measurement.
- [Dirty, shared checkout] → run only against an immutable scratch snapshot; never stage benchmark artifacts or normalize unrelated working-tree state.
- [Evaluation overstates generality] → conclusions apply to this repository/question set only; include question composition and unavailable rows in report.

## Migration Plan

No migration. Run the evaluator manually in a scratch checkout. A later CI integration requires separate evidence and approval.

---
execution:
  version: 1
  depends_on: []
---
# Proposal

Roadmap owner: [P02: Discoverable native tools, MCP and skills provenance](../../../docs/plans/PRODUCT_ROADMAP.md#p02-discoverable-native-tools-mcp-and-skills-provenance).

Selected outcome: P02.O4, all `code-search-evaluation` requirements and scenarios, tasks 1.1–3.2. Research only; existing partial local results do not satisfy matched-model acceptance.

## Why

Agents spend many tool calls and tokens locating code. jcode has several options (`agentgrep` in `crates/jcode-app-core/src/tool/agentgrep.rs`, `graft` index with callers graph, `ast-grep`, a search subagent) and external claims (WarpGrep ~0.73 F1 in ~4 steps) but no measured comparison on this repo. Notes: `scratch/grep.md`; several external claims there are unverified.

## What Changes

- Build a fixed set of ~20 code-location queries over the jcode repo with known answer files/symbols.
- Run each strategy: agentgrep, graft, ast-grep, search subagent, and an Empryo-style repo map.
- Measure tool calls, tokens, recall@k (k=1,5), and wall time.
- Publish a results table and an adopt/reject decision per approach.
- Research only: no product code changes in this change.

## Capabilities

### New Capabilities

- `code-search-evaluation`: Reproducible benchmark and decision record for code search strategies.

## Impact

New `docs/research/code-search-eval/` (queries, harness script, results). Follow-up implementation changes per adopted approach.

## Open decisions

1. Model for subagent runs. Recommended: one fixed model for all agent-driven strategies to isolate tool effect.

## Approval

Approved 2026-11-01 with recommended defaults for all open decisions.

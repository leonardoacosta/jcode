---
execution:
  version: 1
  depends_on: []
---
# Proposal

Roadmap owner: [P07: Memory/context quality and privacy](../../../docs/plans/PRODUCT_ROADMAP.md#p07-memorycontext-quality-and-privacy). Supporting: P01.

## Why

Prompt context (system prompt, instruction files, overlay, skill list) grows silently. The available-skills block alone is several hundred lines per session, and instruction loading keeps expanding (`openspec/changes/native-instruction-file-loading`). Assembly lives in `crates/jcode-app-core/src/agent/prompting.rs`. Nothing fails when it grows, so cost and attention dilution go unnoticed. The prompt-experiment harness (`scripts/prepare_personal_prompt_eval.py`, `openspec/specs/personal-prompt-experiment-adapters/`, archived `2026-10-27-evaluate-personalized-jcode-prompt`, `2026-10-28-automate-personal-prompt-experiments`) already evaluates prompt variants but records no keep/revert decision. Reference pattern: t3code agent-context-budget workflow; local notes `~/.jcode/scratch/selfimprove.md`.

## What Changes

- Committed budget file (`prompt-budget.toml`) with token counts per section: system, instructions, skills, total.
- `jcode debug prompt-budget --check` measures assembled sections for a fixed fixture workspace and fails when any section exceeds budget plus tolerance (default 2%).
- Explicit acceptance: `--accept` rewrites the budget with a required reason, appended to the ledger.
- KEEP/REVERT ledger (`prompt-budget-ledger.jsonl`): each prompt change records budget delta and, when run, the prompt-experiment harness score delta, with decision KEEP or REVERT.
- CI step in `.github/workflows/ci.yml` runs the check.

## Capabilities

### New Capabilities

- `prompt-budget-ratchet`: Measured prompt context budget with ratchet check and decision ledger.

## Impact

`crates/jcode-app-core/src/agent/prompting.rs` (expose section measurement), new debug subcommand, `scripts/prepare_personal_prompt_eval.py` (emit score for ledger), `.github/workflows/ci.yml`. No runtime prompt change.

## Open decisions

1. Token counting: use a deterministic offline approximation with method/version labels. Do not label bytes/4 as exact provider tokens or compare unlike counters.
2. Ratchet direction: a drop greater than 5% computes a proposed tighter budget during explicit acceptance. `--check` is read-only; neither CI nor ordinary measurement writes budgets or silently records KEEP/REVERT. Apply tightening only through the existing explicit accept command and reason.
3. Personal overlay/skills. Recommended: CI measures a fixture with no personal files; personal skills measured only in local mode. Alternative: include them (not reproducible in CI).

## Approval

Approved 2026-11-01 with recommended defaults for all open decisions.

Reconciliation 2026-11-07: size measures are not task correctness or cost guarantees. `add-compaction-quality-evidence` is a recommended independent evidence lane, not a hard dependency for measuring size. CI integration is blocked while the user-owned `.github/workflows/ci.yml` deletion is unresolved; this proposal does not authorize restoring or replacing that file.

---
execution:
  version: 1
  depends_on: []
---
Roadmap owner: [P07: Memory/context quality and privacy](../../../docs/plans/PRODUCT_ROADMAP.md#p07-memorycontext-quality-and-privacy)

## Why

Compaction is measured only by size. `CompactionEvent` records tokens, dropped/compacted message counts and `summary_chars` (`crates/jcode-compaction-core/src/lib.rs:97-108`), and the summary is a free-form block (`crates/jcode-compaction-core/src/lib.rs:134-135`). Nothing checks whether facts the user still needs (open requirements, file paths, decisions) survive compaction, so a smaller summary can look like success while dropping task-critical state. The roadmap gate requires task correctness alongside context cost, not structural scores.

## What Changes

- Add deterministic synthetic fixture/checker tests and a separate actual compaction-interface evaluation that checks per-fixture must-retain facts. Scripted summaries validate the checker, not summarization quality.
- Report per fixture: retained/missing facts, pre/post tokens and verdict `passed`/`failed`/`inconclusive` (inconclusive when no provider summary is available).
- Add an opt-in live mode that uses the configured provider and labels results with model and revision.

## Non-scope

- Changing the compaction algorithm or prompt (later slices act on this evidence).
- Prompt-size ratchets (add-prompt-budget-ratchet).

## Capabilities

### New Capabilities
- `compaction-quality-evidence`: fact-retention evidence for compaction.

### Modified Capabilities
- None.

## Impact

New test fixtures under `crates/jcode-compaction-core/tests/`, a small eval binary or test target; live mode sends fixture text (no user data) to the configured provider.

## Resolved defaults

- Use synthetic fixtures by default. A real-session export requires explicit corpus/content-class authorization even after `Session::redacted_for_export`: secret redaction does not make private source, messages or reasoning public. Actual provider evaluation is opt-in under its own route/disclosure/spend bounds; missing access is inconclusive, not checker success promoted to quality evidence.

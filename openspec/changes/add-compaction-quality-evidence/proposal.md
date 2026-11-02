---
execution:
  version: 1
  depends_on: []
---
Roadmap owner: [P07: Memory/context quality and privacy](../../../docs/plans/PRODUCT_ROADMAP.md#p07-memorycontext-quality-and-privacy)

## Why

Compaction is measured only by size. `CompactionEvent` records tokens, dropped/compacted message counts and `summary_chars` (`crates/jcode-compaction-core/src/lib.rs:97-108`), and the summary is a free-form block (`crates/jcode-compaction-core/src/lib.rs:134-135`). Nothing checks whether facts the user still needs (open requirements, file paths, decisions) survive compaction, so a smaller summary can look like success while dropping task-critical state. The roadmap gate requires task correctness alongside context cost, not structural scores.

## What Changes

- Add a deterministic fixture harness that replays recorded long conversations through `build_compaction_prompt` plus a scripted summary and checks a per-fixture list of must-retain facts.
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

## Open decisions

- Fixture source: synthetic conversations or redacted real sessions? Recommended: synthetic plus one redacted export via `Session::redacted_for_export` (`crates/jcode-base/src/session.rs:1219`), because synthetic alone misses real failure modes and raw sessions leak private data.

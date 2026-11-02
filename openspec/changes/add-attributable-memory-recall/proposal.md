---
execution:
  version: 1
  depends_on: []
---
Roadmap owner: [P07: Memory/context quality and privacy](../../../docs/plans/PRODUCT_ROADMAP.md#p07-memorycontext-quality-and-privacy)

## Why

Injected memory cannot be traced back to its stored entry. `InjectedMemoryItem` carries only `section` and `content` (`crates/jcode-memory-types/src/lib.rs:137-141`), while `MemoryEntry` already has `id`, `source`, `trust`, `active` and `superseded_by` (`crates/jcode-memory-types/src/lib.rs:233-256`). The relevance context is bounded by chars/messages (`crates/jcode-base/src/memory_prompt.rs:1-3`), but the injected payload has no per-item provenance, so a stale or wrong recall cannot be attributed, corrected or forgotten from the event log.

## What Changes

- Add `id`, `source`, `trust` and `updated_at` to `InjectedMemoryItem` and populate them where `MemoryInjected` is emitted.
- Render a compact per-item reference (memory id) in the injected memory block so the agent and user can call `memory forget <id>`.
- Never inject entries with `active = false` or a `superseded_by`; enforce an explicit item-count and char cap on the injected payload and record truncation in the event.

## Non-scope

- New retrieval/ranking algorithms or a replacement memory service.
- Workspace-scoped recall (owned by workspace-aware-context).
- Retention/deletion policy (add-memory-retention-controls).

## Capabilities

### New Capabilities
- `memory-recall-attribution`: provenance and bounds for injected memory.

### Modified Capabilities
- None.

## Impact

`crates/jcode-memory-types`, `crates/jcode-base/src/memory*.rs`, memory event log JSONL schema (additive fields, serde defaults keep old logs readable). Content crossing the provider boundary is unchanged except the short id reference.

## Open decisions

- Show ids in model-visible text or only in event logs? Options: model-visible; log-only. Recommended: model-visible short id, because the agent must be able to forget/correct a recalled item without a second lookup.

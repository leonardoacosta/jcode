---
execution:
  version: 1
  depends_on: []
---
Roadmap owner: [P07: Memory/context quality and privacy](../../../docs/plans/PRODUCT_ROADMAP.md#p07-memorycontext-quality-and-privacy)

## Why

Memory has single-id deletion (`MemoryManager::forget`, `crates/jcode-base/src/memory.rs:942`) and a fixed 14-day prune of memory event logs (`crates/jcode-base/src/memory_log.rs:403-420`), but no way to see or bulk-remove what is stored per scope, no age policy for inactive/superseded entries, and no secret check before storing. Global memory lives in `~/.jcode/memory/global.json` (`crates/jcode-base/src/memory.rs:354`), so a stored secret or a stale fact persists indefinitely and is eligible for recall across projects.

## What Changes

- Add `jcode memory list|purge` with filters (`--scope project|global`, `--inactive`, `--older-than <days>`); purge is dry-run by default and requires `--apply`.
- Reject or redact memory content matching existing secret patterns before storing, reporting the redaction to the caller.
- Optional `[memory.retention]` config (default off) to auto-purge inactive/superseded entries older than N days.

## Non-scope

- Session file retention (session-store-retention).
- Changing memory event log retention (already 14 days).
- Encryption at rest.

## Capabilities

### New Capabilities
- `memory-retention-controls`: inspection, purge and secret-safe storage for memory.

### Modified Capabilities
- None.

## Impact

`src/cli/args.rs` (new subcommand), `crates/jcode-base/src/memory.rs`, config types. Deletion is local and irreversible only with `--apply`.

## Pending user decision

- Secret match on store: reject or redact? Recommended: redact and report, because rejecting loses the non-secret part of an otherwise useful memory.

The recommendation above is not a selected default. The dependent behavior remains blocked until this choice is settled; this proposal does not authorize it.

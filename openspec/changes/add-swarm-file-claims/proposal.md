# Proposal

## Why

Swarm workers share one working tree and nothing prevents two workers from editing the same file; the coordinator relies on prompt-level "file ownership". Conflicting edits silently overwrite each other. Swarm server state lives in `crates/jcode-app-core/src/server/swarm.rs`, which already tracks members and orphan state that claims can hang off.

## What Changes

- Workers claim a file path before an edit tool writes it; the claim is held by the worker's session.
- A second worker editing a claimed path is warned by default, or blocked when `swarm.file_claims = "block"`.
- Claims are released when the worker finishes, is stopped, or is detected as orphaned.
- `swarm status` lists active claims.

## Capabilities

### New Capabilities

- `swarm-file-claims`: Per-file edit claims among swarm workers.

## Impact

`crates/jcode-app-core/src/server/swarm.rs`, edit/write/patch tool paths, swarm status rendering. Extends existing specs `swarm-state-persistence` (claims persisted with swarm state) without modifying it. **Depends on** `recover-swarm-workers-after-reload` (not yet present in `openspec/changes/`) so claims survive reload and orphan detection is reliable.

## Open decisions

1. Default mode. Recommended: `warn`, so existing workflows keep working; `block` opt-in.
2. Coordinator edits. Recommended: coordinator is exempt from blocking but still warned.

## Approval

Approved 2026-11-01 with recommended defaults for all open decisions.

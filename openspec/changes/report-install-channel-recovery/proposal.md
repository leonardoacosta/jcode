---
execution:
  version: 1
  depends_on: [make-worker-stop-durable]
---
# Proposal

Roadmap owner: [P10: Distribution/platform support/performance/diagnostics](../../../docs/plans/PRODUCT_ROADMAP.md#p10-distributionplatform-supportperformancediagnostics)

## Why

Install state is split across `~/.local/bin/jcode`, `builds/{stable,current,canary,shared-server}` and `builds/versions/<version>` (AGENTS.md Install Notes). Update only repairs a stale shared-server channel silently into the log (`crates/jcode-app-core/src/update.rs:1281`); there is no user-facing view of which binary the launcher resolves to, and no rollback to a prior immutable version.

## What Changes

- `jcode version --channels` prints launcher target, each channel's resolved version and the shared-server target.
- `jcode update --rollback` repoints the active channel to the previous compatible immutable version in `builds/versions` and reports both versions. Check persisted session-control compatibility floor and composed-config support before any swap. Refuse unsupported downgrade without deleting state, flattening user config or weakening stop barriers.
- A failed update leaves the prior channel target unchanged and says so.

## Non-scope

New channels, Windows installer rewrite, release pipeline changes.

Config-composition is a conditional compatibility boundary, not a hard prerequisite: single-file installs remain supported, but a composed install cannot silently downgrade to a loader that ignores fragments. Channel reporting is independent preparation; complete rollback acceptance consumes durable control's compatibility floor. No symlink activation occurs during proposal reconciliation.

## Capabilities

### New Capabilities

- `install-channel-recovery`: report channel identity and roll back to a prior immutable version.

## Impact

`update.rs`, `jcode-build-support` channel helpers, CLI args. Linux, macOS and Windows path variants must be covered.

## Resolved defaults

1. Rollback target: most recent other version, or explicit `--to <version>`? Selected default: default to previous and accept optional `--to`; covers the common case with one flag.

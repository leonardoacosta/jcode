---
execution:
  version: 1
  depends_on: []
---
# Proposal

Roadmap owner: [P10: Distribution/platform support/performance/diagnostics](../../../docs/plans/PRODUCT_ROADMAP.md#p10-distributionplatform-supportperformancediagnostics)

## Why

Install state is split across `~/.local/bin/jcode`, `builds/{stable,current,canary,shared-server}` and `builds/versions/<version>` (AGENTS.md Install Notes). Update only repairs a stale shared-server channel silently into the log (`crates/jcode-app-core/src/update.rs:1281`); there is no user-facing view of which binary the launcher resolves to, and no rollback to a prior immutable version.

## What Changes

- `jcode version --channels` prints launcher target, each channel's resolved version and the shared-server target.
- `jcode update --rollback` repoints the active channel to the previous immutable version in `builds/versions` and reports both versions.
- A failed update leaves the prior channel target unchanged and says so.

## Non-scope

New channels, Windows installer rewrite, release pipeline changes.

## Capabilities

### New Capabilities

- `install-channel-recovery`: report channel identity and roll back to a prior immutable version.

## Impact

`update.rs`, `jcode-build-support` channel helpers, CLI args. Linux, macOS and Windows path variants must be covered.

## Open decisions

1. Rollback target: most recent other version, or explicit `--to <version>`? Recommended: default to previous and accept optional `--to`; covers the common case with one flag.

---
execution:
  version: 1
  depends_on: []
---
# Proposal

Roadmap owner: [P10: Distribution/platform support/performance/diagnostics](../../../docs/plans/PRODUCT_ROADMAP.md#p10-distributionplatform-supportperformancediagnostics)

## Why

AGENTS.md documents that a fresh build is inert until `builds/shared-server` is repointed, and that runtime checks otherwise silently measure old code. Reload activation has rollback support (`crates/jcode-build-support/src/lib.rs:189`) but no post-reload assertion that the serving daemon runs the built revision.

## What Changes

- After `selfdev reload`, query the daemon's build hash and compare it with the built binary; report match or mismatch.
- A mismatch is reported as failed, never as success.

## Non-scope

Build profile changes, remote builds, bridge reload.

## Capabilities

### New Capabilities

- `selfdev-reload-verification`: verify that the serving daemon runs the just-built revision after self-dev reload.

## Impact

`src/cli/selfdev.rs`, `jcode-build-support`, selfdev tool result text. Complements shared-server-crash-resilience without changing its scope.

## Open decisions

None

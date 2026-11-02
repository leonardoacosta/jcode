---
execution:
  version: 1
  depends_on: []
---
# Proposal

Roadmap owner: [P10: Distribution/platform support/performance/diagnostics](../../../docs/plans/PRODUCT_ROADMAP.md#p10-distributionplatform-supportperformancediagnostics)

## Why

Terminal capability detection is scattered: rounded pill caps (`crates/jcode-tui/src/tui/ui_onboarding.rs:44`), image protocols in `crates/jcode-terminal-image`, Windows VT mouse handling (`src/cli/terminal.rs:28`). No report states which capabilities were detected or which fallback is active, so platform bugs cannot be triaged from user evidence.

## What Changes

- `jcode debug terminal` prints detected terminal, color depth, image protocol, mouse, Unicode width mode and the chosen fallback for each.
- A test matrix asserts fallbacks when each capability is forced off.

## Non-scope

New protocols, changes to default detection heuristics.

## Capabilities

### New Capabilities

- `terminal-capability-acceptance`: report detected terminal capabilities and active fallbacks.

## Impact

New read-only CLI subcommand calling existing detectors.

## Open decisions

None

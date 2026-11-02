# Proposal

Roadmap owner: [P04: Swarm collaboration and handoffs](../../../docs/plans/PRODUCT_ROADMAP.md#p04-swarm-collaboration-and-handoffs). Supporting: P09.

## Why

`agents.swarm_model=omnipi:DeepSeek-V4.1-Flash` silently overrode an Opus coordinator; spawn output did not say which model a worker got or why. Resolution context is only shown by `list_models` (`crates/jcode-app-core/src/tool/communicate.rs:1606-1620`).

## What Changes

- Spawn output and `swarm list` show each worker's resolved model and source: `explicit|config|env|inherit|slot`.

## Capabilities

### Modified Capabilities

- `swarm-communicate-reliability`: worker model visibility.

## Impact

`crates/jcode-app-core/src/tool/communicate.rs` spawn/list formatting; model resolution returns source.

## Open decisions

1. Warn when config overrides coordinator model? Recommended: yes, one line on spawn. Alternative: source label only.

## Approval

Approved 2026-11-01 with recommended defaults for all open decisions.

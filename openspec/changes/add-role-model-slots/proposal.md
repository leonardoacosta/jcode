# Proposal

## Why

Workers have a single default model: `agents.swarm_model` (`crates/jcode-config-types/src/lib.rs:571`), listed by `format_swarm_model_list` (`crates/jcode-app-core/src/tool/communicate.rs:1604`). Read-only research, editing, compaction, web search, and council work have very different cost/quality needs, so users either overpay or pass `model` on every spawn.

## What Changes

- Add per-role model slots under `[agents.roles]`: `research` (read-only), `edit`, `compaction`, `websearch`, `council`.
- A spawn with a role and no explicit `model` resolves to that role's slot, then `swarm_model`, then coordinator inheritance.
- Report the resolved source as `slot:<role>` through the resolved-model reporting added by `show-resolved-worker-model`.
- `swarm list_models` shows configured slots.

## Capabilities

### New Capabilities

- `role-model-slots`: Role-based default model resolution for spawned agents and internal tasks.

## Impact

`crates/jcode-config-types/src/lib.rs`, `crates/jcode-app-core/src/tool/communicate.rs`, compaction and websearch model selection. **Depends on** `show-resolved-worker-model` (not yet present in `openspec/changes/`; must land first or be drafted).

## Open decisions

1. How a spawn declares its role. Recommended: optional `role` parameter on `swarm spawn`; no inference from prompt text.
2. Should `research` enforce read-only tools? Recommended: yes, the slot implies a read-only tool profile; otherwise it is only a model hint.

## Approval

Approved 2026-11-01 with recommended defaults for all open decisions.

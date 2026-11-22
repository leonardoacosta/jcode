---
execution:
  version: 1
  depends_on: []
---
# Proposal: Overnight model chains and coordinator-only main thread

Roadmap owner: [P05: Durable overnight, scheduling and recovery](../../../docs/plans/PRODUCT_ROADMAP.md#p05-durable-overnight-scheduling-and-recovery). Supporting: P04, P06.

## Why

An overnight run stops at the first model failure on its route, so a single unavailable or quota-limited model ends unattended work for the night. The coordinator also has full edit, build and review tools, so it does implementation work itself instead of delegating it to workers. That mixes coordination with execution and makes the run hard to supervise.

## What Changes

- Overnight gets two ordered model chains on the session's own provider route: one for the coordinator and one for its workers. On a turn failure classified as `Transient`, `RateLimited`, `ModelUnavailable` or `QuotaExhausted`, the run switches to the next chain model on the same route and retries the turn. Auth failures (`AuthExpired`, `AuthRevoked`) and `Permanent` failures never advance the chain.
- Defaults: coordinator `claude-opus-5.5 → gpt-6-sol → claude-sonnet-5.5 → gpt-6-luna → DeepSeek-V4.1-Flash → qwen3.5-35b`; worker `claude-sonnet-5.5 → gpt-6-luna → DeepSeek-V4.1-Flash → qwen3.5-35b`. Configurable as `[overnight] coordinator_model_chain` and `[overnight] worker_model_chain`.
- An exhausted coordinator chain ends the run `blocked` and lists every model tried. An exhausted worker chain fails only that worker and notifies the coordinator.
- The provider, account and route never change. This is an overnight-only exception: the retirement of the global `model_fallback` chain (`fallback-on-transient-model-errors`) stays in force for every other session.
- The overnight coordinator session gets a hard tool allowlist: `swarm`, `todo`, `schedule`, `bg`, `read`, `agentgrep`, `conversation_search`, `session_search`, `batch` (members still checked), MCP tools, `edit`/`write` only for `*.md` paths, and `bash` only for read-only commands. Denied calls return a message telling the coordinator to delegate to a worker, and record a `coordinator_tool_denied` event.
- The coordinator prompt states the role: coordinate, facilitate, nudge, manage todos. Workers do edits, builds, reviews and judging.

## Capabilities

### New Capabilities

- `overnight-model-chain`: per-role ordered model fallback for overnight runs on the selected route.
- `overnight-coordinator-role`: coordinator tool allowlist and delegation guidance.

### Modified Capabilities

- `overnight-admission`: preflight checks that each chain has at least one model the route serves.

## Impact

`crates/jcode-app-core/src/overnight*` (supervisor loop, admission gate, enforce), `crates/jcode-app-core/src/tool/mod.rs` (session tool policy path/command checks), the swarm spawn path for workers owned by an overnight run, `crates/jcode-overnight-core` (prompts), and config types for the `[overnight]` section. No provider, account or gateway substitution. No change to normal sessions.

## Non-scope

- Restoring the global `model_fallback` chain.
- OmniRoute family combos (`gpt-sol` and similar) and switching the chains to family names. Tracked separately.
- Cross-provider failover.

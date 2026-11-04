---
execution:
  version: 1
  depends_on: []
---
# Warn when a conversation leaves its agreed goal

## Why

Semantic compaction detects subject changes, not whether work still serves the user's goal. The user wants an advisory warning for both agent and user, potentially using Jev per turn, without treating legitimate testing, investigation or explicitly changed goals as mistakes.

## What Changes

- Add opt-in goal-alignment monitoring attached to the existing session initiative/goal, never another task ledger. Use its current description and success criteria plus explicitly recorded exclusions; do not infer authorization from a classifier.
- After each completed user-visible agent turn, evaluate bounded recent work against that versioned goal through an explicitly selected Jev MCP classifier. Include aligned, supporting-work, possible-drift and insufficient-evidence outcomes. No automatic provider/tool discovery or fallback.
- Warn after repeated high-confidence drift, with a concise reason, goal reference and choices to refocus, dismiss or explicitly revise the goal. Show the same warning to the agent's next turn and the user without silently starting a new model turn.
- Bound payload, latency and calls; skip stale results and expose unavailable/uncertain checks honestly. No blocking, tool veto, forced continuation or automatic goal mutation.

## Capabilities

### New Capabilities

- `goal-drift-warning`: opt-in, source-bound goal alignment checks and advisory warnings.

### Modified Capabilities

None. Existing continuation and compaction contracts remain unchanged.

## Impact

Reuse `crates/jcode-base/src/goal.rs`, session metadata, existing MCP manager and protocol/bus events. Integrate at shared completed-turn boundaries in `crates/jcode-app-core/src/agent`, covering TUI and headless paths, with warning rendering in `crates/jcode-tui`. Existing `initiative` actions own goal creation/update. Reuse installed dependencies. No live config, provider, MCP connection, permission, external message or deployment changes during authoring.

This is a standalone requested feature. The product roadmap was inspected but no new outcome binding or phase completion is claimed. `route-jev-supervision-explicitly` owns a separate direct-HTTP Jev route; this change does not depend on or implement it. Shared classifier semantics can inform tests, but direct routing remains out of scope. No issue binding is requested.

## Non-goals

No task-source integrations, scheduler cleanup, memory migration, Firecrawl implementation, provider catalog repairs, direct Jev transport, mandatory remote judging, hidden transcript disclosure, auto-created goals or automatic action on warnings. Full per-turn monitoring is opt-in and remains off until the user selects a trusted MCP tool and permits its bounded data disclosure.

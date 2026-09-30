---
status: draft
execution:
  version: 1
  depends_on: []
---

# Proposal

## Why

Simple requests such as “open the selected code in the sidebar” currently enter the full chat loop. This adds delay and can select an incompatible surface: the observed panel tool accepted Markdown/PDF rather than the requested source file. Jcode needs a fast path that recognizes one supported action, resolves its target from trusted current-session context, and falls through to chat when interpretation is uncertain.

The user selected a bounded classifier and authorized the first-version catalog to include interface actions, read-only actions, and bounded local actions such as running a named test target or formatting the selected file. This authorizes planning only, not implementation or unrestricted tool execution.

## What Changes

- Add a pre-chat System One classification path returning one registered action with typed arguments, or a chat disposition. Do not add a multi-step planner.
- Define an explicit catalog of supported semantic actions with closed argument schemas, target-resolution requirements, risk class, tool binding, and execution/confirmation policy.
- Include native Jcode actions and explicitly registered MCP actions. MCP discovery, descriptions, and read-only hints do not confer execution authority.
- Reuse existing tool dispatch and preserve tool availability, deny lists, session scope, permission checks, confirmation, cancellation, and execution evidence. A classifier result cannot bypass these checks.
- Resolve references such as “the code” only from trusted current-session selection/recent artifact context. Unsupported file types, multiple plausible targets, uncertainty, multi-step requests, unavailable classification, and invalid output go to chat without side effects.
- Expose visible execution results and a chat-continuation path. Never automatically replay a possibly executed action through chat after timeout or error.
- Keep initial interface execution within this repository's existing supported surfaces. Jcode Desktop is a separate repository; do not implement Desktop code-view support here or advertise an unsupported source viewer.

## Capabilities

### New Capabilities

- `system-one-command-routing`: Bounded, pre-chat intent classification with explicit abstention, trusted target resolution, and safe fallback.
- `registered-command-execution`: Native/MCP action catalog, bounded local actions, authorization-preserving dispatch, and observable once-only execution.

### Modified Capabilities

None. Existing `custom-tool-routing` planning concerns capability-aware chat prompt guidance; this change adds a distinct pre-chat path and must coordinate shared dispatch files without silently absorbing that proposal.

## Impact

The owning code project and planning root are `/home/nyaptor/.jcode`. Likely integration surfaces include the user-message submission path, `crates/jcode-base/src/systemone.rs` / `jev.rs`, the native tool registry, and `crates/jcode-app-core/src/agent/turn_execution.rs`. Existing MCP tools enter the registry through `crates/jcode-base/src/mcp/tool.rs`.

A small command-routing crate may own catalog-independent decision DTOs and validation if the current workspace conventions support it; this is a proposed addition, not an assertion that a command classifier crate already exists. Extend existing System One routing rather than duplicating endpoint or credential resolution. Do not log full user messages, credentials, or private tool payloads for classifier telemetry. Do not enable external/destructive commands automatically, arbitrary shell generation, provider actions, or source-file conversion merely to disguise an unsupported viewer.

---
execution:
  version: 1
  depends_on: [add-typed-skill-selector-metadata]
---
# Proposal

Roadmap owner: [P02: Discoverable native tools, MCP and skills provenance](../../../docs/plans/PRODUCT_ROADMAP.md#p02-discoverable-native-tools-mcp-and-skills-provenance).
Supporting: [mcp-management-experience](../archive/2026-11-02-mcp-management-experience/proposal.md) (server table; not duplicated here).

## Why

Native tools and MCP tools are discoverable only by the model. MCP `search` returns tool names and schemas matching a query (`crates/jcode-app-core/src/tool/mcp.rs:194-198,616-623`), but there is no user-facing view, and native tools never appear in it. The TUI selector lists only native commands and skills (`crates/jcode-tui/src/tui/app/state_ui_input_helpers.rs:378-415`). Users cannot see which native tools and MCP tools exist, which namespace they belong to, or what they do, locally or in a remote session.

## What Changes

- Add a read-only `/tools [query]` view listing native tools and connected MCP tools as typed rows (kind `tool` or `mcp`, name, server for MCP, description), reusing the typed entry from `add-typed-skill-selector-metadata`.
- Filter by case-insensitive name/description query.
- Remote sessions request the list from the server; an older server yields a notice "tool list unavailable from this server".
- Native tool names take precedence over same-named MCP tools in display, matching existing collision handling (`crates/jcode-app-core/src/tool/tests/mcp_collision.rs`).
- Listing never invokes a tool or starts a lazy MCP server.

## Non-scope

- MCP server state, connect/login actions, exposure modes (`mcp-management-experience`).
- Failure backoff (`mcp-failure-backoff`).
- Provenance/trust fields (`show-skill-and-mcp-provenance`).

## Capabilities

### New Capabilities

- `capability-discovery`: User-visible listing of native and MCP tools.

## Impact

TUI command registry, MCP manager read API, remote protocol (additive request).

## Resolved defaults

1. Entry point. Options: new `/tools` command; tab inside `/mcp`. Selected default: `/tools`, because native tools are not MCP and `/mcp` is owned by `mcp-management-experience`.

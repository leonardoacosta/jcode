---
execution:
  version: 1
  depends_on: []
---
# Proposal

Roadmap owner: [P02: Discoverable native tools, MCP and skills provenance](../../../docs/plans/PRODUCT_ROADMAP.md#p02-discoverable-native-tools-mcp-and-skills-provenance).

## Why

The slash-command selector stores candidates as untyped `(String, &'static str)` pairs (`crates/jcode-tui/src/tui/app/state_ui_input_helpers.rs:352,378`). Every skill row gets the fixed help text "Activate skill" (`state_ui_input_helpers.rs:391,413`) even though each `Skill` carries a real `description` (`crates/jcode-base/src/skill.rs:16-18`). Remote sessions receive only skill names (`app.remote_skills`, `crates/jcode-tui/src/tui/app/remote/server_events.rs:2165`), so a remote selector cannot show descriptions at all. Users cannot tell a native command from a skill in the same list, or what a skill does, before invoking it.

## What Changes

- Replace the tuple candidate with a typed entry: `kind` (native command, skill), name, description.
- Skill rows show the skill's frontmatter description; native commands keep their registered help.
- Render a short kind label per row with distinct styling, truncated with an ellipsis to the available cell width (no wrapping, Unicode display cells).
- Native command names keep precedence on same-name collisions (existing `seen` set behavior).
- Extend the remote skill payload with an optional description; legacy peers that send names only render description "unknown" rather than the fixed placeholder.
- Sanitize control characters in descriptions before rendering.

## Non-scope

- MCP and native-tool rows (see `discover-native-and-mcp-capabilities`).
- Source/trust details (see `show-skill-and-mcp-provenance`).
- Skill folder/name lint (`lint-skill-name-and-folder`).

## Capabilities

### New Capabilities

- `skill-selector-metadata`: Typed selector entries with kind labels and real descriptions, local and remote.

## Impact

`crates/jcode-tui/src/tui/app/state_ui_input_helpers.rs`, selector rendering, remote skill event payload (protocol additive, backward compatible).

## Resolved defaults

1. Kind label width. Options: fixed ten-cell cap; abbreviate (`cmd`, `skill`); glyph only. Selected default: short words (`cmd`, `skill`) under the proposed ten-cell cap, since glyph-only is unreadable over some terminals and the cap is the selected design default.

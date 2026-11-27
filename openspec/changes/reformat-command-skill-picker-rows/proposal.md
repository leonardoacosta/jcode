---
execution:
  version: 1
  depends_on: []
---
# Reformat command and skill picker rows

Roadmap owner: [P09: TUI/onboarding everyday UX](../../../docs/plans/PRODUCT_ROADMAP.md#p09-tuionboarding-everyday-ux), outcome P09.O1. This change refines the existing `skill-selector-metadata` capability.

## Why

The slash-command picker currently prefixes rows with `cmd` or `skill[source]`, which repeats the item's type before its slash name and makes the list harder to scan. Fixed name and source columns will align names and provenance, keep the command-versus-skill distinction in color, and leave the remaining row width for the description.

## What Changes

- Remove the textual `cmd` and `skill[...]` prefixes from command and skill suggestions. Keep the leading slash and color each name using its existing command or skill kind color.
- Reserve a 25-terminal-cell name field, including the slash, and an 8-terminal-cell source-label field. Pad short values even when the row has unused width. Truncate longer values with one `…` character included within the field limit. Brackets around the source label are delimiters and do not count toward the 8-cell label field.
- Show the declared upstream skill author when known; otherwise show a verified owning workspace label (`brown`, `priceless`, or `leo`) when available. Native commands show `leo`. Show `unknown` when the source cannot be established. Do not treat a discovery root such as `global` or `project` as the author.
- Keep detailed skill discovery-root, path, symlink, and shadowed-root provenance available in `/skill info`; keep it distinct from the compact author/owner label in the picker.
- Use the remaining terminal width for the description. Keep each row to one visual line, sanitize control/escape text, and preserve explicit unknowns for legacy remote peers.

## Scope and non-goals

This change formats slash-command and skill suggestions only. It does not change invocation, command precedence, matching, or selection behavior. It does not change the detailed `/skill info` provenance contract, the MCP/native-tool listing, or other pickers.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `skill-selector-metadata`: Define aligned name and source columns, source identity semantics, existing kind colors, truncation and no-wrap behavior for local and remote suggestions.

## Impact

Likely surfaces include `crates/jcode-protocol/src/wire.rs` for additive remote skill author/owner metadata, `crates/jcode-base/src/skill.rs` for local author metadata, `crates/jcode-tui/src/tui/app/state_ui_input_helpers.rs` for candidate source identity, `crates/jcode-tui/src/tui/ui_input/command_suggestions.rs` for rendering, and their existing tests. Preserve `SkillMetadata.source` as the discovery-root field used by `/skill info`; use a distinct optional field for the picker label so the two meanings cannot be confused.

## Resolved decisions

- The name field is 25 terminal cells, including `/`; the source label itself is 8 terminal cells, with visible brackets outside that width. Both fields stay at their specified width when values are shorter and space is available.
- The Unicode ellipsis `…` is one display character and consumes one cell of the field's truncation budget.
- Source precedence is declared upstream author, then verified owning workspace label, then `unknown`; native commands use `leo`.
- P02.O3's delivered discovery-root provenance remains unchanged. P09.O1 covers the new compact selector source display, while `/skill info` remains the detailed discovery-root view.
- No new runtime credential, network call, provider, or external service is required.

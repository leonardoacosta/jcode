---
execution:
  version: 1
  depends_on: []
---
# Proposal

Roadmap owner: [P09: TUI/onboarding everyday UX](../../../docs/plans/PRODUCT_ROADMAP.md#p09-tuionboarding-everyday-ux)

## Why

`NO_COLOR`/`JCODE_NO_COLOR` are honoured only by CLI paths (`src/cli/provider_doctor.rs:91`, `src/cli/dispatch.rs:568`); no match exists under `crates/jcode-tui/src`. Mouse capture is a single toggle (`src/cli/terminal.rs:8`) and there is no check that mouse-only actions have keyboard equivalents.

## What Changes

- TUI honours `NO_COLOR`/`JCODE_NO_COLOR`: selected, error and status state is conveyed by glyph or text, not color alone.
- Inventory mouse-only actions (click to copy, open, select) and add a keyboard path for each.
- Frame tests in monochrome mode.

## Non-scope

Screen-reader protocol integration, theme redesign, new keybinding system.

## Capabilities

### New Capabilities

- `terminal-accessibility-fallbacks`: honour NO_COLOR in the TUI and provide a keyboard path for every mouse-only action.

## Impact

TUI style resolution, onboarding pills, info widget, input handling.

## Resolved defaults

1. Monochrome via env only or also a config key? Options: env only (standard, zero config); env plus a `[ui]` config key. Selected default: env only; `NO_COLOR` is the established convention and avoids new config.

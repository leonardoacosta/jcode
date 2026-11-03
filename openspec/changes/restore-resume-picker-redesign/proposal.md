---
execution:
  version: 1
  depends_on: []
---
# Proposal

Roadmap owner: [P09: TUI/onboarding everyday UX](../../../docs/plans/PRODUCT_ROADMAP.md#p09-tuionboarding-everyday-ux).

## Why

A resume picker redesign was implemented on 2026-09-22 (commits `02d618ada`..`dc235d289`) in a separate checkout, `~/dev/jcode/source/jcode`, that was later deleted. The commits never reached this repository; only the design docs survived. Finding a session among hundreds is still hard: rows are tall and clipped, sorting is fixed, PgUp/PgDn moves 3 items, and the split ignores terminal width.

## What Changes

Re-implement on current main, preserving the newer transcript preview (`22b3eda84`) and project scoping (`0975e7e4d`, `6346b24cd`):

- Compact 3-line rows: title and activity age; first-prompt excerpt; project, source, saved, here and status badges.
- Sort model: `o` cycles recency, title, saved-first, creation; `O` reverses; search sorts by relevance (title > directory > body).
- Search bar shows match count (`15/261`).
- Adaptive list/preview split by terminal width.
- PgUp/PgDn step derived from visible rows.
- Display-cell truncation for all row text.

## Capabilities

### New Capabilities

- `session-picker-browsing`: sorting, compact rows, paging and layout for the resume picker.

## Impact

`crates/jcode-tui/src/tui/session_picker*`, `crates/jcode-tui-session-picker/src/lib.rs`.

## Evidence

Recovery bundle: [plans/recovery/resume-picker-2026-09-22/](../../../plans/recovery/resume-picker-2026-09-22/) (closeout, exploration, sort research, task cards, review notes, and `edits.json` with the 39 original edits extracted from session `session_dog_1790022984976_52b6d91f4bf4d2a8`).

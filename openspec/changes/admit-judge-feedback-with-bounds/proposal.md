---
execution:
  version: 1
  depends_on: []
---
# Proposal

Roadmap owner: [P01: Trusted core and acceptance/evidence baseline](../../../docs/plans/PRODUCT_ROADMAP.md#p01-trusted-core-and-acceptanceevidence-baseline)

## Why

Judge and review sessions return free-form DMs to the parent. The autojudge prompt requires a `CONTINUE:` or `STOP:` prefix (`crates/jcode-tui/src/tui/app/commands_review.rs:514-545`), asserted only as prompt text (`crates/jcode-tui/src/tui/app/tests/remote_startup_input_01/part_01.rs:164-165`), and the session notice promises exactly one DM (`crates/jcode-tui/src/tui/app/state_ui_storage.rs:485`). No parent-side code validates the prefix, counts DMs or caps continuations, so a judge can drive repeated continuation and its text arrives with user-like authority. The roadmap contract states judge feedback does not authorize external actions.

## What Changes

- Parse judge DMs: act only on the first message per judge session with a `CONTINUE:` or `STOP:` prefix; others are shown but not acted on.
- Cap automatic `CONTINUE:` admissions per user turn; at the cap, stop and show `judge cap reached`.
- Wrap admitted feedback as advisory reviewer content, never as authorization for external actions.
- `/review` results are displayed only, never auto-continued.

## Non-scope

Changing judge prompts or models; council tool (`add-council-tool`); scoring judge quality.

## Capabilities

### New Capabilities

- `judge-feedback-admission`: see specs/judge-feedback-admission/spec.md.

## Impact

`crates/jcode-tui/src/tui/app/commands_review.rs`, judge DM handling in the TUI turn flow, config for the cap.

## Open decisions

1. Default continuation cap. Options: 1; 3; unbounded with user stop. Recommended: 3, because autojudge defaults to CONTINUE and a small bound keeps usefulness while preventing loops.
2. Malformed DM. Options: drop; show as note. Recommended: show as a note without continuation so the user still sees it.

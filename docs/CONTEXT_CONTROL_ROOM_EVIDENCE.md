# Context Control Room evidence

This document records the integration and regression evidence for `add-context-control-room`.

## Implemented behavior

- `Alt+O` opens the temporary Context Control Room overlay from normal prompt input.
- `Alt+O` closes the overlay when it is already open.
- `Esc` closes the overlay.
- While open, ordinary keys are owned by the overlay and do not leak into the prompt draft.
- Higher-priority overlays keep ownership. The session picker receives `Alt+O` without opening the Control Room.
- The overlay renders semantic context separately from execution substrate context.
- Rows display provenance, confidence, and action hints.
- Unavailable semantic and Herdr rows are explicit.
- Copy only operates on copyable rows.
- Focus feedback only applies to existing focusable surfaces and does not spawn panes or sessions.

## Degraded Herdr behavior

When Herdr metadata is unavailable, the execution substrate section includes `Herdr pane` with `unavailable` provenance and a reason such as `Herdr not detected`. This preserves visibility of the missing integration without failing the UI.

When Herdr environment metadata exists, it is shown as execution substrate with `herdr` provenance and inferred confidence. It does not replace durable project or workspace identity.

## Validation commands

The following commands are part of the final gate for this change:

```bash
cargo test -p jcode-app-core context
cargo test -p jcode-app-core client_session
cargo test -p jcode-tui control_room
cargo test -p jcode-tui session_picker usage_overlay keybinding
cargo fmt --all -- --check
openspec validate add-context-control-room --strict --no-interactive
git diff --check
```

## Observed results

- `cargo test -p jcode-app-core context`: passed, 33 tests.
- `cargo test -p jcode-app-core client_session`: passed, 29 tests.
- `cargo test -p jcode-tui control_room`: initially found two issues, then passed, 7 tests.
  - Fixed open-overlay `Alt+O` so it closes instead of being swallowed as an ordinary overlay key.
  - Made the Herdr rendering assertion environment-flexible because a live Herdr pane can be present.
- `cargo test -p jcode-tui session_picker`: passed, 94 tests, 9 ignored developer benchmarks.
- `cargo test -p jcode-tui usage_overlay`: passed, no matching unit tests in this package.
- `cargo test -p jcode-tui keybinding`: passed, 2 tests.
- `cargo fmt --all -- --check`: passed.
- `openspec validate add-context-control-room --strict --no-interactive`: passed.
- `git diff --check`: passed.

## Known limitations

- Organization, initiative, and task/run rows render as explicit unavailable rows until durable selection surfaces are wired.
- The TUI snapshot currently uses current session state and path fallback for immediate display. The app-core context module provides durable snapshot/persistence helpers for server-owned identity.
- Focus action is intentionally conservative. It reports focus feedback only for existing focusable rows and does not allocate new execution resources.

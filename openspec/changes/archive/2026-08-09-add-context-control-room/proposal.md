# Add Context Control Room Overlay

## Why

Jcode is becoming the durable homelab orchestration/control plane, with Herdr providing local terminal and agent execution. Today, the live system exposes sessions, workspaces, panes, memories, and agents through separate surfaces, but it does not give the user one authoritative context view that explains what organization, project, workspace, initiative, task/run, Jcode session, and Herdr pane they are operating in.

This causes three problems:

- Semantic context is inferred from shell paths or live panes rather than represented as durable first-class objects.
- Reconnecting from another client makes it hard to verify which project/run/session is authoritative before acting.
- Herdr can show execution substrates, but Jcode cannot yet present the control-plane hierarchy that should govern them.

The approved direction is the temporary Control Room overlay toggled by `Alt+O`.

## What Changes

- Add a TUI Context Control Room overlay opened and closed with `Alt+O`.
- Display the durable hierarchy: organization -> project -> workspace/worktree -> initiative -> task/run -> Jcode session -> Herdr pane.
- Show context provenance and confidence so users can distinguish persisted identity from inferred or unavailable data.
- Keep Jcode authoritative for durable semantic identity while treating Herdr as the local execution substrate.
- Provide keyboard navigation and safe actions for inspection, focusing existing sessions/panes, and copying context identifiers.
- Persist enough context identity to survive reconnects and server reloads without relying on daily shell state.
- Document the context model and the Herdr/Jcode integration boundary separately from implementation code.

## Capabilities

### New Capabilities

- `context-control-room`: A first-class Control Room overlay for viewing and navigating the active context hierarchy and execution substrate.

### Modified Capabilities

None. The repository has no existing checked-in context-control-room specification.

## Impact

- Adds TUI state, rendering, and input handling for one new overlay.
- Adds app-core context snapshot structures and server/client event plumbing where needed for persisted context identity and Herdr pane metadata.
- Adds or extends config for the overlay hotkey only if existing keybinding configuration requires a named binding rather than hard-coded `Alt+O` handling.
- Adds docs for context architecture and integration/test evidence as separate files.
- Does not expose Herdr directly over the network, create organization/project administration CRUD, or replace existing session picker/resume behavior.

- touches: `crates/jcode-tui/src/tui/app.rs`
- touches: `crates/jcode-tui/src/tui/app/input.rs`
- touches: `crates/jcode-tui/src/tui/app/local.rs`
- touches: `crates/jcode-tui/src/tui/app/remote.rs`
- touches: `crates/jcode-tui/src/tui/control_room.rs (new)`
- touches: `crates/jcode-tui/src/tui/app/control_room.rs (new)`
- touches: `crates/jcode-tui/src/tui/app/tests/control_room.rs (new)`
- touches: `crates/jcode-app-core/src/server/context.rs (new)`
- touches: `crates/jcode-app-core/src/server/client_session.rs`
- touches: `crates/jcode-app-core/src/server/state.rs`
- touches: `crates/jcode-app-core/src/server/client_session_tests.rs`
- touches: `crates/jcode-app-core/src/protocol_tests/context.rs (new)`
- touches: `crates/jcode-session-types/src/context.rs (new)`
- touches: `crates/jcode-session-types/src/lib.rs`
- touches: `docs/CONTEXT_ARCHITECTURE.md (new)`
- touches: `docs/CONTEXT_CONTROL_ROOM_EVIDENCE.md (new)`
- base-commit: jcode@9941e1c6d660136762d7da3c8ab50224cd0e9127

## Preconditions

- Existing session restore, remote client subscribe/resume, and grouped session behavior remain working.
- Existing session picker, account/login overlays, usage overlay, side panel, and copy/scroll hotkeys retain priority and behavior except for the new `Alt+O` binding.
- Herdr metadata is consumed only from local trusted environment/socket/API surfaces already available to the Jcode host.
- The overlay must degrade cleanly when Herdr is absent, disconnected, or returns incomplete metadata.

## Decisions

- Choose the temporary overlay rather than side-panel reuse or a persistent rail. The overlay is lower risk, inspectable on demand, and does not permanently consume terminal width.
- Bind `Alt+O` as the primary toggle. If a platform cannot deliver `Alt+O`, expose the same action through the existing command/debug path or documented keybinding configuration.
- Treat organization/project/workspace/initiative/task/run as semantic Jcode context and Herdr pane/process as execution substrate context.
- Persist semantic context in Jcode-owned storage using durable IDs, not only path hashes or shell cwd.
- Use explicit provenance labels: persisted, current-client, Herdr, inferred-from-path, unavailable.
- Keep the first implementation read-mostly. Destructive or mutating context administration, such as deleting projects or reassigning organization membership, is excluded.
- Prefer focus/navigation actions over spawning new work. Opening a new agent/pane from the overlay is deferred unless the implementation can do it through existing safe session/resume flows.
- Keep context architecture documentation and integration/test evidence documentation as separate docs.

## Done Means

- `Alt+O` opens and closes a centered Control Room overlay from connected and reconnecting sessions without disrupting input drafts.
- The overlay shows the active organization, project, workspace/worktree, initiative, task/run, Jcode session, and Herdr pane when known.
- Missing or inferred context is clearly labeled and never displayed as authoritative.
- The user can navigate overlay sections, dismiss with Esc or `Alt+O`, copy context IDs, and focus existing sessions/panes where supported.
- Jcode persists durable context identity across reload and client reconnect.
- Herdr absence or failure produces a degraded execution-substrate panel rather than blocking the overlay.
- Existing overlays and hotkeys continue to pass their tests.
- Context architecture and integration/test evidence docs are written as separate files.

## Testing

- Run `cargo test -p jcode-app-core context`; expected result: context snapshot, persistence, protocol, reload, and Herdr-degraded cases pass.
- Run `cargo test -p jcode-tui control_room`; expected result: overlay rendering, `Alt+O` toggle, dismissal, navigation, copy action, and collision with existing overlays/hotkeys pass.
- Run existing focused gates around sessions and overlays: `cargo test -p jcode-tui session_picker usage_overlay keybinding` and `cargo test -p jcode-app-core client_session`; expected result: existing behavior remains green.
- Run `openspec validate add-context-control-room --strict --no-interactive`; expected result: the feature artifacts validate.
- Run `git diff --check`; expected result: no whitespace errors.

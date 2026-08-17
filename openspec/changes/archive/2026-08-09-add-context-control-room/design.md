# Design

## Context

The current Jcode TUI already has overlay primitives for session, login, account, and usage views. App state keeps overlay options inside `tui/app.rs`; input dispatch runs through `tui/app/input.rs`, local/remote terminal event handlers, and overlay-specific handlers. Server/client subscribe and resume flows already preserve session IDs and working directories, while Herdr integration today is limited to terminal launch/routing and environment signals such as `HERDR_ENV` and `HERDR_PANE_ID`.

Memory and session state are currently path/session oriented. Project memory is path-hash based, Herdr can expose live panes, and session restore can recover groups, but there is no user-facing semantic context object that states: this is the organization, project, workspace/worktree, initiative, task/run, Jcode session, and Herdr pane for the current interaction.

The approved UI direction is a temporary Control Room overlay toggled with `Alt+O` rather than reusing the side panel or adding a persistent rail.

## Goals / Non-Goals

**Goals:**

- Add one discoverable overlay that gives the user a quick authoritative view of current context.
- Represent the context hierarchy explicitly and label every field's provenance.
- Persist durable semantic identity across reconnects and reloads where Jcode owns the identity.
- Include Herdr pane/process metadata only as local execution-substrate context.
- Provide safe read-mostly actions: dismiss, section navigation, copy IDs, and focus existing sessions/panes when supported.
- Document architecture and integration/test evidence separately.

**Non-Goals:**

- Full organization/project administration CRUD.
- Network exposure of Herdr or direct remote Herdr control.
- Replacing `/resume`, the session picker, side panel, task tools, or memory tools.
- Creating new agent panes from the overlay unless it can be delegated to existing safe resume/focus flows.
- Solving the memory visibility split in this change.
- Implementing the persistent project rail or side-panel variant.

## Architecture

### 1. Context snapshot model

Add a compact context snapshot model in app-core or shared session types. The snapshot is an immutable view assembled for a TUI render frame and contains:

- `organization`: id, name, provenance, confidence.
- `project`: id, name, root/canonical path, memory key/path hash where available, provenance, confidence.
- `workspace`: id, worktree path, branch/revision where available, provenance, confidence.
- `initiative`: id/name/status where available.
- `task_run`: id/name/status/current validation or background task summary where available.
- `jcode_session`: session ID, display title, connected client, model/provider, cwd, resume/group info.
- `herdr`: pane ID, workspace/window name, local harness state, focusability, unavailable reason.

Every field uses a small provenance enum: `Persisted`, `CurrentClient`, `Herdr`, `InferredFromPath`, `Unavailable`. Confidence is descriptive: `authoritative`, `inferred`, or `unavailable`.

**Why:** The UI should not imply that a path hash or shell cwd is the same as a durable project record.

### 2. Context assembly service

Add a context assembly function in app-core that accepts the known server/session state plus current client subscription details. It should gather only cheap local data synchronously or through bounded calls. Herdr lookup is optional and timeout-bounded.

Resolution order:

1. Persisted Jcode semantic IDs if present.
2. Current subscribed/resumed session metadata.
3. Herdr pane metadata from trusted local env/socket/API.
4. Path-derived fallback labels.
5. Explicit unavailable placeholders.

The assembler never mutates semantic records during render. If it discovers a useful inferred project label, it reports it as inferred; a later explicit promote/persist flow can be a separate feature.

### 3. TUI overlay state and rendering

Add `control_room_overlay: Option<ControlRoomOverlay>` to `App`. The overlay follows existing overlay patterns: centered modal, title, bordered sections, footer hotkey hints, scrollable body, and deterministic tests from rendered buffers.

Initial layout:

```text
Context Control Room                                                Esc close
┌ Semantic context ────────────────────────────────────────────────┐
│ Org        personal/homelab                         persisted   │
│ Project    jcode                                    persisted   │
│ Workspace  /home/nyaptor/dev/jcode/source/jcode     current     │
│ Initiative durable context control plane            unavailable │
│ Task/run   add-context-control-room                 current     │
└─────────────────────────────────────────────────────────────────┘
┌ Execution substrate ─────────────────────────────────────────────┐
│ Jcode session  jaguar / e757fcb / provider model                 │
│ Herdr pane     pane-id or unavailable reason                     │
└─────────────────────────────────────────────────────────────────┘
[↑/↓ section] [c copy] [f focus existing] [Alt+O/Esc close]
```

The implementation can adjust exact styling to existing TUI conventions, but it must preserve the information architecture and provenance labels.

### 4. Input behavior

`Alt+O` toggles the overlay globally when no higher-priority overlay owns the key. When the Control Room is open:

- `Esc` and `Alt+O` close it.
- `↑/↓`, `j/k`, or existing navigation conventions move between sections/rows.
- `c` copies the selected context ID/value using the existing clipboard helper path.
- `f` focuses an existing Jcode session or Herdr pane only when a known safe focus mechanism exists; otherwise it shows a non-destructive unsupported message.
- Other keys do not leak into the draft input.

Existing overlays keep precedence. If session picker/account/login/usage overlay is open, `Alt+O` either no-ops with feedback or waits until those overlays close, matching existing overlay priority style.

### 5. Persistence and reload/reconnect

Persist durable semantic IDs in Jcode-owned storage associated with the session/workspace. This can start as an additive record keyed by session ID and canonical project root. It must not overwrite project identity solely because a reconnecting client reports `$HOME` or a different transient cwd.

On server reload or client reconnect, the assembler rebuilds the snapshot from persisted IDs plus the current client/session state. The overlay should be available even when disconnected from Herdr, with execution-substrate fields degraded.

### 6. Documentation

Add two separate docs:

- `docs/CONTEXT_ARCHITECTURE.md`: hierarchy, identity ownership, persistence boundaries, Herdr/Jcode authority split.
- `docs/CONTEXT_CONTROL_ROOM_EVIDENCE.md`: implementation evidence, test plan, edge cases, and live/degraded validation notes.

## Risks / Trade-offs

- **Context false authority:** mitigated by provenance and confidence labels.
- **Hotkey conflicts:** mitigated by overlay precedence tests and keybinding documentation.
- **Herdr failures block UI:** mitigated by timeout-bounded optional lookup and degraded panel.
- **Scope creep into project CRUD:** explicitly excluded from this feature.
- **Path-derived identity drift:** path-derived fields are labeled inferred and do not overwrite persisted IDs.
- **Terminal space constraints:** temporary overlay avoids a persistent rail and can scroll.

## Migration Plan

1. Add snapshot types and assembler with tests for persisted, inferred, and unavailable fields.
2. Add TUI overlay state/rendering and key handling behind `Alt+O`.
3. Add copy/focus-safe actions and degraded Herdr display.
4. Add docs and evidence notes.
5. Run focused app-core/TUI gates and existing overlay/keybinding regressions.

## Open Questions

None blocking. The user approved the temporary Control Room overlay (`Alt+O`) as the implementation direction and previously approved Jcode as authoritative with Herdr as local substrate.

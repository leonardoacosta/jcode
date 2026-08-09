# Context architecture

Jcode's durable context model separates semantic identity from local execution details. The homelab Jcode daemon is the source of truth for durable identity, while Herdr is treated as the local terminal and agent execution substrate.

## Hierarchy

The Control Room presents context in this order:

1. Organization
2. Project
3. Workspace or worktree
4. Initiative
5. Task or run
6. Jcode session
7. Herdr pane

The first five rows are semantic context. They describe what the user is working on. The Jcode session and Herdr pane rows are execution substrate context. They describe where the work is currently running.

## Authority and provenance

Every row carries a value, optional stable ID, provenance, confidence, copyability, focusability, and an explicit unavailable reason when missing.

Provenance labels:

- `persisted`: loaded from durable Jcode-owned context identity.
- `current-client`: observed from the currently connected TUI client or session state.
- `herdr`: observed from Herdr-provided execution metadata.
- `inferred`: derived from a path fallback such as the current working directory.
- `unavailable`: intentionally absent or not yet selected.

Confidence labels:

- `authoritative`: durable Jcode identity.
- `inferred`: useful but not authoritative.
- `unavailable`: no safe value exists.

Persisted semantic identity wins over refreshed client state. A reconnect from `$HOME`, a temporary shell, or a Herdr pane update must not clobber the selected project or workspace. Current-client and path-derived values are additive fallbacks until the durable identity is selected.

## Herdr boundary

Herdr is not the project system of record. It supplies local execution context such as pane ID, workspace label, or harness state when available. If Herdr metadata is absent, the Control Room renders an explicit unavailable row rather than hiding the missing integration.

The overlay never spawns panes, agents, sessions, or browser contexts. It may copy a selected context value or focus a surface that already exists.

## UI decision

The implemented UI is the temporary Control Room overlay toggled with `Alt+O`.

Rejected alternatives:

- Reusing the side panel would overload memory/document browsing with context inspection.
- A persistent project rail would consume too much terminal width and imply always-on navigation state before the semantic model is fully durable.

The overlay is intentionally transient. Higher-priority modals, such as the session picker, keep input ownership over the Control Room toggle.

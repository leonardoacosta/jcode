# Command Center Migration Ledger

Status: active ledger for OpenSpec change `add-solidstart-command-center-vertical-slice`.

This ledger maps older architecture and topology documents into the approved Command Center direction. It does not archive or delete any source document.

| Source | Decision | Ledger entry |
|---|---|---|
| `docs/WORKFLOW_AUTOMATION_ROADMAP.md` | Absorbed in part | Server authority, snapshots, events, commands, attention semantics, approvals, schedules, and handoff visibility feed the command-center model. The earlier HTMX/control-room UI recommendation is superseded by the approved daemon-hosted SolidStart client. Portfolio-wide lanes remain later work. |
| `docs/CONTEXT_ARCHITECTURE.md` | Retained | The semantic/execution hierarchy and provenance rules remain valid. The terminal Control Room remains a transient inspector, while the browser command center becomes the interactive initiative supervision surface. |
| `docs/CONTEXT_CONTROL_ROOM_EVIDENCE.md` | Completed foundation | Evidence for the existing overlay remains the compatibility baseline. This child change must not expand the overlay into duplicate editing, graph manipulation, schedule administration, or approval workflows. |
| `docs/DESKTOP_SUPERAPP_WORKSPACE.md` | Retained with extension | The future desktop workspace should include a `CommandCenter` surface that hosts the same SolidStart routes and generated client contract rather than reimplementing command-center domain behavior. |
| `docs/DESKTOP_APP_ARCHITECTURE.md` | Partially superseded | The blanket no-WebView stance is superseded only for a later dedicated `CommandCenter` surface. Native/custom rendering remains valid for the rest of desktop2. Desktop transport, sandboxing, and focus integration are deferred. |
| `docs/MAC_HOMELAB_SSH_TOPOLOGY.md` | Retained | The Mac initiates access to homelab runtime. Remote browser access must use an explicit authenticated tunnel/bridge and must not expose an unauthenticated command-center listener. |
| `docs/AMBIENT_MODE.md` | Retained | Ambient scheduling remains the scheduling foundation. This slice projects linked schedule state only and does not implement global schedule administration. |
| `docs/COMMAND_CENTER.md` | New operational reference | Documents enablement, security posture, repository-local gates, managed topology post gate, thresholds, operations, and rollback for the vertical slice. |

## Roll-forward requirements

Before the command center can be enabled by default, the implementation record must link evidence for:

1. Generated contract drift gate.
2. Security gate.
3. Repository-local browser acceptance with isolated daemon state.
4. Orca-unavailable acceptance.
5. Loopback/tunnel fixture.
6. Managed Mac/homelab terminal post gate.
7. Compatibility gates with the web feature disabled.
8. Threshold measurements for startup, idle resources, event latency, reconnect, timeline virtualization, and shutdown cleanup.

## Rollback requirements

Rollback must be a configuration rollback first. Disable the experimental web host and verify no listener or daemon-supervised child process survives shutdown. Preserve durable initiative, schedule, session, and Orca reference data. Existing TUI, daemon socket, ambient, browser automation, and desktop2 clients must remain usable with the feature disabled.

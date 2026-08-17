# Workflow Automation Roadmap

Status: Research captured; architecture recommendation pending explicit product approval.
Updated: 2026-08-11

> Command Center ledger: OpenSpec change `add-solidstart-command-center-vertical-slice` supersedes the earlier HTMX/control-room UI recommendation for the flagship interactive surface. The canonical vertical-slice UI is daemon-hosted SolidStart, while this roadmap remains a retained source for inbox, schedule, approval, handoff, snapshot, event, and attention semantics. See [`COMMAND_CENTER.md`](./COMMAND_CENTER.md) and [`COMMAND_CENTER_MIGRATION_LEDGER.md`](./COMMAND_CENTER_MIGRATION_LEDGER.md).

This roadmap records the research into jcode's ambient inbox, scheduling, approvals, handoffs, and the control-room UI. It also records the architecture options evaluated before implementation begins.

## Product direction

Jcode should evolve from a collection of reliable background primitives into a supervision surface for delegated work:

```text
signal → work item → policy gate → execution run → evidence → outcome → next action
```

The user should supervise many pieces of work through an inbox rather than manually maintain many chat sessions.

## Current foundation

Jcode already provides:

- Durable scheduled items in `~/.jcode/ambient/queue.json`.
- Ambient, session-resume, and child-session-spawn schedule targets.
- Persisted working directory, branch, relevant files, task context, and success criteria.
- Email-reply directives in `~/.jcode/ambient/directives.json`.
- Permission requests and approval decisions.
- Ambient runner, adaptive scheduling, debug commands, and TUI status visibility.
- Tested queue, scheduler, public schedule-tool, startup, and child-session paths.

The missing layer is a unified work model and a UI that explains why work needs attention, who owns it, what the agent proposes, and what happens next.

## Research findings

### Agent inbox

The agent inbox pattern replaces a list of conversations with a queue of actionable work. Perea's research identifies four interaction types:

- **Notify:** something happened; no decision is required.
- **Question:** the agent needs information.
- **Review:** the agent proposes an action for approval, editing, rejection, or rerouting.
- **Revisit:** a prior run needs correction, retry, or checkpoint resumption.

The inbox depends on durable pause/resume state. It should support many concurrent agent tasks and make the human an on-the-loop supervisor rather than a synchronous participant in every step.

### Scheduled work

AI UX Playground's Scheduled Tasks pattern establishes a minimum schedule contract:

- cadence
- timezone
- next run
- last result
- pause/cancel
- run history

The design must make missed schedules, repeated failures, retry counts, and stale schedules visible. A schedule without a last-result link is not trustworthy.

### Approval queues

Approval queues should contain human judgment, not every failure. Separate these lanes:

- approval-required
- needs-information
- tool-failed
- data-conflict
- manual-investigation
- scheduled

An approval item needs a decision packet:

- proposed action
- reason for escalation
- source evidence
- risk summary
- affected files/tools
- deadline or SLA
- approve, edit, reject, ask, assign, or manual-handle actions

### Handoffs

Handoff views should show ownership transitions rather than only session IDs:

```text
ambient → verifier → implementer → reviewer → human approval
```

Each transition should preserve the reason, context, current owner, and expected next outcome.

## Control-room UI direction

The proposed control room is a three-pane supervision surface:

```text
┌──────────────┬──────────────────────────────┬────────────────────┐
│ Work lanes   │ Attention queue              │ Decision / evidence│
│              │                              │                    │
│ Inbox     07 │ Auth migration needs review  │ Proposed action    │
│ Approvals 02 │ CI check due now             │ Why it surfaced    │
│ Schedules 11 │ Directive from email         │ Evidence           │
│ Runs         │                              │ Risk + actions     │
│ Handoffs     │                              │                    │
└──────────────┴──────────────────────────────┴────────────────────┘
```

Primary lanes:

- Work Inbox
- Approvals
- Schedules
- Runs
- Handoffs

Work Inbox filters:

- Review
- Question
- Notify
- Revisit
- Tool Failed
- Data Conflict
- Scheduled

The visual hierarchy is:

- left: where work is
- center: what needs attention
- right: why it matters and what happens next

## Architecture options evaluated

### Option A: native Rust control room

A renderer-agnostic Rust UI model, native renderer, and typed client over the existing server/runtime.

**Pros:** aligns with the desktop superapp direction, avoids a second runtime and state model, supports keyboard-first spatial navigation, shares Rust types, and preserves native macOS/Linux goals.

**Cons:** higher initial UI cost and slower visual iteration.

**Assessment:** recommended long-term direction.

### Option B: React + Vite + local server functions

A React client communicating with Rust or Node server functions over JSON, SSE, or WebSockets.

**Pros:** fastest visual prototyping and a strong ecosystem for timelines, filters, forms, and tables.

**Cons:** introduces a second state model and build/runtime pipeline. Server functions risk duplicating Jcode domain behavior. Desktop packaging requires a webview or a separate browser surface.

**Assessment:** useful for a disposable research prototype, not the primary product architecture.

### Option C: Tauri + React

Rust application logic with HTML rendered in an OS WebView and an IPC bridge.

**Pros:** smaller than Electron, mature packaging, React ecosystem, Rust system layer.

**Cons:** conflicts with Jcode's documented avoidance of WebView UI shells and introduces WebView, IPC, and frontend synchronization behavior.

**Assessment:** technically viable but strategically misaligned unless the desktop thesis changes.

### Option D: Axum + server-rendered HTML + HTMX

A Rust/Axum server returning HTML fragments, with HTMX interactions and SSE for live updates.

**Pros:** minimal client runtime, no React build, server-authoritative state, excellent for queues, detail panels, filters, and approvals.

**Cons:** best suited to a browser companion rather than the native spatial desktop. High-frequency event views and rich client layout become less natural.

**Assessment:** recommended as an optional local web companion over the same server contract.

### Option E: egui/eframe

A pure-Rust immediate-mode GUI for rapid native prototyping.

**Pros:** fast to prototype, no JavaScript or WebView, straightforward panels and forms.

**Cons:** different from the planned retained/spatial desktop architecture; native polish, accessibility, and text behavior need validation.

**Assessment:** useful for a fake-data prototype or internal operations surface, but not yet the committed product renderer.

## Proposed architecture

The recommended direction is a shared server contract with multiple clients:

```text
                    ┌─────────────────────┐
                    │ Jcode server truth  │
                    │ sessions            │
                    │ schedules           │
                    │ directives          │
                    │ approvals           │
                    │ run history         │
                    └──────────┬──────────┘
                               │
              typed snapshots + events + commands
                               │
          ┌────────────────────┼────────────────────┐
          ▼                    ▼                    ▼
   Native desktop       Existing TUI          Local web companion
   control room         surfaces               HTML/HTMX
```

The server owns durable workflow truth. Clients own layout, focus, filters, and transient input state.

### API categories

**Queries** provide snapshots for initial render and reconnect:

```text
get_work_inbox
get_approvals
get_schedules
get_run_history
get_handoff_timeline
get_context
```

**Commands** perform explicit mutations:

```text
approve_item
reject_item
edit_and_approve
answer_directive
pause_schedule
resume_schedule
trigger_schedule
cancel_schedule
retry_run
handoff_task
focus_session
```

Commands should be idempotent where possible and return the resulting entity state.

**Events** provide live updates:

```text
TaskCreated / TaskUpdated / TaskCompleted / TaskBlocked
ApprovalRequested / ApprovalResolved
ScheduleDue / SchedulePaused
RunStarted / RunProgressed / RunFinished / RunFailed
HandoffCreated / SessionUpdated
```

### Unified read model

The UI should consume a `WorkItem` projection rather than independently coupling itself to schedules, directives, permission requests, and sessions:

```rust
struct WorkItem {
    id: WorkItemId,
    kind: WorkItemKind,
    title: String,
    status: WorkItemStatus,
    priority: Priority,
    owner: OwnerRef,
    reason: Option<String>,
    due_at: Option<DateTime<Utc>>,
    risk: RiskLevel,
    source_run_id: Option<RunId>,
    available_actions: Vec<ActionKind>,
    evidence_preview: Vec<EvidenceRef>,
}
```

### Ownership boundary

**Server-owned:** durable task, schedule, approval, session, run, handoff, permission, context, and provenance state.

**Client-owned:** pane layout, selected lane, focused item, filters, expanded details, keyboard mode, local drafts, window geometry, and temporary optimistic state.

## Recommended sequence

1. Define shared read-model types and event vocabulary.
2. Add a server-side `WorkInbox` projection over schedules, directives, permissions, and runs.
3. Expose query, command, and event-stream boundaries.
4. Build a fake-data control-room prototype against the contract.
5. Render the prototype in the native desktop direction.
6. Optionally add an HTMX local web companion.
7. Defer React until a concrete interaction proves it is necessary.

## Decision status

The native Rust client plus typed query/command/event protocol is the recommended direction, but this remains a proposal until explicitly approved. The roadmap intentionally records the alternatives and tradeoffs rather than silently treating the recommendation as a final architecture decision.

## Sources

- [AI UX Playground: Scheduled Tasks & Recurring Actions](https://www.aiuxplayground.com/pattern/scheduled-tasks/)
- [Perea: The Agent Inbox](https://www.perea.ai/research/agent-inbox-ux)
- [Stackwell: AI Agent Approval Queue](https://iamstackwell.com/posts/ai-agent-approval-queue/)
- [Tauri Architecture](https://v2.tauri.app/concept/architecture/)
- [HTMX: Hypermedia-Driven Applications](https://htmx.org/essays/hypermedia-driven-applications/)
- [Axum SSE](https://docs.rs/axum/latest/axum/response/sse/index.html)
- [egui documentation](https://docs.rs/egui/latest/egui/)
- [Jcode Desktop Superapp Workspace](./DESKTOP_SUPERAPP_WORKSPACE.md)
- [Jcode Desktop App Architecture](./DESKTOP_APP_ARCHITECTURE.md)
- [Jcode Context Architecture](./CONTEXT_ARCHITECTURE.md)
- [Jcode Ambient Mode](./AMBIENT_MODE.md)

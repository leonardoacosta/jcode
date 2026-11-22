# Design

## Context

See proposal.md, Why. Evidence gathered on 2026-11-22 with Herdr 0.9.5 and Jcode `774344983` (dirty tree):

- Live pane `wA6:p1` ran Jcode session `session_chipmunk_…` at idle, but `herdr pane get` showed `agent_status: working` and `agent_session.source: herdr:codex`, value `01a20114-…`. That Codex session is a `codex exec` run (`originator: codex_exec`, cwd `~/.jcode/scratch`) that the session started through the bash tool. It failed on an `op` authentication error.
- The bash tool calls `apply_scoped_client_terminal_env` (`crates/jcode-app-core/src/tool/bash.rs` foreground and detached paths) or `apply_client_terminal_env` (background path). Both forward all `CLIENT_TERMINAL_ENV_VARS`, including `HERDR_ENV`, `HERDR_PANE_ID` and `HERDR_SOCKET_PATH` (`crates/jcode-terminal-launch/src/lib.rs`). Every installed Herdr agent integration that was checked (Codex, Claude, Copilot, Cursor, OpenCode, Pi, OMP, Hermes) gates on `HERDR_ENV=1`.
- A disposable Herdr session (`herdr --session jcode-herdr-repro`, since deleted) reproduced the lock-out. After one `herdr:codex` `report-agent-session`, the pane showed `agent: None`, `unknown`, and Herdr ignored later Jcode `report-agent`, `report-agent-session` and `release-agent` calls (exit 0, no effect). Reports with a resume command failed with `resume_not_accepted`. A Codex release did not clear it. So Jcode cannot recover a claimed pane; prevention is the only producer-side fix.
- The same session accepted `--state blocked --message …`. `pane get` does not return the message, so Herdr's display of it is unverified.
- The `herdr` CLI works without `HERDR_ENV`. It needs only the socket and pane variables.
- `herdr::sync` records `last_submitted` before delivery. A failed or timed-out report is never retried until the state changes.
- `sync_herdr_agent_state` (`crates/jcode-tui/src/tui/app/terminal_title.rs`) uses only `is_processing()`.

## Goals / Non-Goals

Goals: the five requirements in `specs/jcode-herdr-pane-status/spec.md`.

Non-goals: Herdr receiver changes; `done` state; a new wire protocol field; changing hook or terminal-launch environments; reconciling a pane that is already claimed.

## Decisions

### 1. Strip `HERDR_ENV` only at the bash tool boundary

Add a bash-tool helper in `jcode-app-core` that applies the client terminal environment and then removes `HERDR_ENV`. Use it at all three bash spawn sites. `apply_client_terminal_env` already removes every forwarded key before it sets the client values, so the server's own `HERDR_ENV` is also cleared.

Alternatives: change `apply_client_terminal_env` itself (rejected: hooks and terminal launch share it, and `detected_resume_terminal_with_client_env` needs `HERDR_ENV` from the client snapshot, not the child environment); remove `HERDR_PANE_ID` (rejected by D1: breaks `herdr-handoff` and pane splits).

### 2. Carry the wake flag to the client in the started notification body

`publish_task_started_activity` already receives `notify`. Pass `wake` as well, and add a fixed second-paragraph line, for example `Wakes this session when it finishes.`, only for wake tasks. Extend `ParsedBackgroundTaskStartedNotification` with `wake: bool`, and store it on `BackgroundTaskRow`. The header line stays byte-identical, so older clients still parse it.

Alternatives: a new `UiActivity` field (rejected: changes the server-client wire for one bit); count all running tasks (rejected by D3).

Rows rebuilt without a started notification, for example after reload, default to `wake: false`. The pane then shows `idle`, which is the conservative failure.

### 3. One pure state function in the client

Add a pure function that takes three inputs: the first `requires_user_auth` reason from `pinned_todos_payload`; whether any member of `filter_inline_swarm_subtree(remote_swarm_members, self_id)` has status `running`, `streaming` or `thinking`; and whether any `BackgroundTaskRow` is `Running` with `wake`. It also takes `is_processing()`, and returns `(AgentState, Option<message>)`. `sync_herdr_agent_state` calls it on each tick. Parse the todo payload through the existing cached parser that `todo_blocker_notes` uses, so the tick stays cheap. Add `AgentState::Blocked` (`"blocked"`) and an optional message on `StateReport`. `report_args` appends `--message` when set.

### 4. Worker-owned retry

Replace `last_submitted` with two values: `desired` (set by `sync`, used to skip unchanged requests without waking the worker) and `delivered` (set by the worker on success). The worker loop keeps the current job after a failure and waits on the condition variable with a timeout: 1 s, doubling to a 60 s cap, reset on success or on a new desired report. A new job replaces the pending one, and release ends the loop. Each attempt calls `next_seq()`. The existing clear-before-replace branch keeps `attempted_id` across retries, so the clear is retried before the new session is reported.

Alternative: periodic re-assertion (rejected by D5: it cannot recover a claimed pane, and it adds load when nothing failed).

### 5. Bounded error capture

`run_herdr` pipes stderr and reads at most 512 bytes after exit or kill. Herdr errors are one JSON line, so a full pipe is not a practical risk, and the 2 s timeout still kills a blocked child. The log line names the command kind, state, sequence and status.

## Risks / Trade-offs

- [A user script in a bash tool command checks `HERDR_ENV`] → Document the change and the `JCODE_CLIENT_HERDR_ENV` fallback in `docs/HERDR.md`. Update the five local Herdr skills in task group 6.
- [Upstream `herdr --skill` text still says `HERDR_ENV`] → Inside Jcode, the agent can read the documented fallback. Outside Jcode nothing changes.
- [Swarm status strings drift] → The state function and its tests use the same status set as the dock rank in `info_widget_swarm_background.rs`.
- [A wake task hangs for hours] → The pane stays `working`, which is accurate. Stall projection belongs to `surface-worker-stall-and-block-reasons`.
- [Retry traffic while Herdr is down] → At most one attempt every 60 s per client.

## Migration Plan

No data migration. Roll back by reverting the commits. This change prevents new claims; it does not repair a pane that a nested agent already claimed. Herdr documents that it clears an unreleased agent once the pane returns to its shell prompt, but that clean-up was not observed for a claimed pane, so the operator fallback is to restart Jcode in a new pane.

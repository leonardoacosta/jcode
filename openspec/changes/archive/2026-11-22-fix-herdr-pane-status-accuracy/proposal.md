---
execution:
  version: 1
  depends_on: []
---
# Proposal

Roadmap owner: [P04: Swarm collaboration and handoffs](../../../docs/plans/PRODUCT_ROADMAP.md#p04-swarm-collaboration-and-handoffs). Supporting: P09.

## Why

Herdr shows a Jcode pane as `working` when nothing runs, as `idle` while the session waits on its own workers or background tasks, and never as `blocked` when a todo needs the user. The first fault makes a pane permanently wrong: a command that a Jcode tool runs can claim the Jcode pane for another agent, and Herdr 0.9.5 then ignores every later Jcode report for that pane.

## What Changes

- Commands that the bash tool starts (foreground, background and detached) no longer receive the bare `HERDR_ENV` marker. They keep every other forwarded client terminal variable and the `JCODE_CLIENT_HERDR_ENV` copy, so routing, pane splits and Herdr skills still work. Nested agents with Herdr integrations then stop reporting on the Jcode pane. Lifecycle hooks keep their current environment.
- The Jcode client reports `blocked`, with the blocker reason as the Herdr message, while an incomplete todo has a `requires_user_auth` reason.
- The client reports `working` while it processes a turn, while a running swarm worker in its own spawn subtree exists, or while a running background task that will wake this session exists. Precedence: blocked > working > idle.
- A report that Herdr rejects, or that times out, is submitted again with bounded backoff until Herdr accepts it. Duplicate suppression applies only to accepted reports.
- A failed Herdr command logs its exit status, the reported state, the sequence number and bounded Herdr error output.
- `docs/HERDR.md` describes the shipped states, the environment rule for tool commands and the takeover limitation. The local Herdr skills accept `JCODE_CLIENT_HERDR_ENV=1` as their inside-Herdr check.

## Capabilities

### New Capabilities

- `jcode-herdr-pane-status`: truthful Jcode pane state in Herdr, protection of the pane from nested agents, and report delivery.

### Modified Capabilities

None. `jcode-herdr-session-binding` keeps its identity, ownership and ordering requirements unchanged.

## Impact

- Jcode: `crates/jcode-app-core/src/tool/bash.rs` and its terminal environment tests, `crates/jcode-base/src/background.rs` and `crates/jcode-base/src/message/notifications.rs` (wake marker in the started notification), `crates/jcode-tui/src/tui/herdr.rs`, `crates/jcode-tui/src/tui/app/terminal_title.rs`, background task rows in `crates/jcode-tui/src/tui/mod.rs` and `app/state_ui_messages.rs`, `docs/HERDR.md`.
- Other repositories (task group 6 only): `~/.agents` (`skills/herdr-handoff`, `skills/herdr-open-git`, `skills/herdr-open-markdown`) and `~/dev/personal/.agents` (`skills/herdr`, `skills/herdr-jcode`).
- `report-agent-lifecycle-to-herdr` now depends on this change. That change keeps `done`, terminal notifications, worker stall states and the shared presentation projection.
- No Herdr changes, no new protocol fields for the server-client wire, no new dependencies or services.

## Non-scope

- The Herdr receiver bug in which a foreign session report locks out the pane's owner. No upstream issue is drafted (user decision, 2026-11-22).
- `done`, focus-cleared completion, OSC notifications and stall projection (owned by `report-agent-lifecycle-to-herdr`).
- Blocking on swarm `Blocked` members, because that status often means a dependency wait, not a user decision.
- Environment inherited by processes other than bash tool commands, such as MCP servers started from a shared server that itself runs in a Herdr pane.

## Resolved decisions (2026-11-22)

1. D1 A: remove only bare `HERDR_ENV` from bash tool commands; update the Herdr skill gates.
2. D2 A: `blocked` comes only from incomplete `requires_user_auth` todos.
3. D3 A: owned running swarm workers and running `wake=true` background tasks count as `working`.
4. D4 A: standalone change; `report-agent-lifecycle-to-herdr` points here for the overlapping slices.
5. D5 A: retry until accepted with bounded backoff; no periodic re-assertion.

## Evidence

See [design.md](design.md) Context: live pane `wA6:p1` analysis and a disposable Herdr 0.9.5 session repro on 2026-11-22.

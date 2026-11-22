# Herdr integration

Jcode integrates with the [Herdr](https://herdr.dev) terminal multiplexer in two ways: it opens new sessions in Herdr panes, and it reports its own pane state to Herdr.

## Pane launches

When a headed session launch is requested from a client with `HERDR_ENV=1` and `HERDR_PANE_ID`, Jcode splits the calling pane to the right, focuses the new pane, and starts the resumed Jcode session there. `HERDR_BIN_PATH` is honored when present. This covers visible swarm spawns, resume-in-new-terminal, self-development launches, and restart restores, because they all use the shared terminal launcher. A configured `[terminal].spawn_hook` still takes precedence.

`pane run` starts the session inside the new pane's interactive shell, so the launcher runs `<jcode command> && exit`: the pane closes when Jcode exits cleanly (quit, `swarm stop`), and stays open on a non-zero exit so the error remains readable.

## Pane state reporting

Inside a Herdr pane (`HERDR_ENV=1`, with `HERDR_PANE_ID`, `HERDR_SOCKET_PATH` and `HERDR_BIN_PATH` set), the TUI reports through `"$HERDR_BIN_PATH" pane report-agent --source jcode --agent jcode`, and calls `pane release-agent` when the user quits. The code is in `crates/jcode-tui/src/tui/herdr.rs`. The state comes from `App::herdr_state` in `crates/jcode-tui/src/tui/app/terminal_title.rs`.

| State | When |
| --- | --- |
| `blocked` | An incomplete todo has a `requires_user_auth` reason, for example a login prompt. The first such reason is sent as `--message=<reason>` (one line, at most 200 bytes; the `=` form keeps a reason such as `--` a value). |
| `working` | Not blocked, and one of these is true: a turn is processing; a swarm worker in this session's own spawn subtree is `running`, `streaming` or `thinking`; or a background task of this session that was started with `wake=true` is running. |
| `idle` | Anything else. Dependency waits (`blocked_by`, `waiting_on`), blocked workers, and background tasks without `wake` do not change the state. |

Each report also carries the session id and its `jcode --resume <id>` command. Herdr older than 0.9.2 rejects the `--` resume separator as a usage error (exit 2) and drops the whole report. On that specific error Jcode retries once without the resume command, and keeps reporting state only for the rest of the process.

Delivery:

- One background thread sends reports. A newer state replaces one that has not been delivered.
- A report counts as delivered only when `herdr` exits 0. After a failure or the 2 second timeout, Jcode retries the latest state with a new `--seq`. The wait is 1 second, doubling to at most 60 seconds. Release on quit stops the retries.
- A session change clears the old session identity before it reports the new one.
- A failed call is logged to the Jcode log with the command, pane, state, sequence, exit status or timeout, and the first 512 bytes of Herdr's error output.

Background task rows learn about `wake` from the "Background task started" notification. Rows restored without that notification count as not waking, so the pane can show `idle` while such a task runs.

## Environment of tool commands

Bash tool commands (foreground, background and detached) get the client's terminal environment, without `HERDR_ENV`. Other agents' Herdr integrations (Codex, Claude Code, OpenCode, Pi and others) report only when `HERDR_ENV=1`. If a nested agent reports to the Jcode pane, Herdr 0.9.5 gives the pane to that agent and ignores Jcode's later reports and releases, so the pane shows a stale state.

Tool commands still get `HERDR_PANE_ID`, `HERDR_SOCKET_PATH`, `HERDR_BIN_PATH`, `HERDR_TAB_ID`, `HERDR_WORKSPACE_ID` and the `JCODE_CLIENT_HERDR_*` copies, so `herdr pane split` and similar commands keep working. A script that must detect a Herdr pane from a tool command should test:

```sh
[ "${HERDR_ENV:-${JCODE_CLIENT_HERDR_ENV:-}}" = 1 ]
```

Lifecycle hooks keep the full environment, including `HERDR_ENV`.

If a pane was taken by a nested agent before this rule existed, Jcode cannot take it back. Quit Jcode and start it again in a new pane.

## Forwarded variables

Jcode forwards `HERDR_ENV`, `HERDR_SOCKET_PATH`, `HERDR_PANE_ID`, `HERDR_TAB_ID`, `HERDR_WORKSPACE_ID`, `HERDR_BIN_PATH`, `HERDR_SESSION`, and `HERDR_AGENT` from the requesting client to server-side spawn, focus and hook paths, also as `JCODE_CLIENT_<NAME>`. It recognizes Herdr as a masking terminal multiplexer for Mermaid graphics capability detection.

## Not reported yet

- `done` (turn finished and not yet seen) and `blocked` for pending approval or ask-user prompts: see `openspec/changes/report-agent-lifecycle-to-herdr`.
- Stalled or blocked worker projection: see `openspec/changes/surface-worker-stall-and-block-reasons`.

References:

- <https://herdr.dev/docs/add-herdr-support/>
- <https://herdr.dev/docs/socket-api/>
- <https://herdr.dev/docs/session-state/>

---
name: orca-cli
description: >-
  Use the public `orca` CLI to operate Orca-managed worktrees, folder contexts,
  terminals, repos, automations, worktree comments, and the browser embedded
  inside the Orca app. Use when the user says "$orca-cli", "use orca cli",
  "Orca worktree", "child worktree", "cardStatus", "spawn codex/claude in a worktree",
  "read/wait/send Orca terminal", "terminal send", "full handoff", "handover",
  "give this to another agent", "another worktree", "Orca browser", or
  "control the browser inside Orca". Prefer this over raw `git worktree`, ad hoc
  PTYs, Playwright, or Computer Use when the task touches Orca-managed state.
  Use Computer Use for browser windows, webviews, or desktop UI outside Orca's
  embedded browser.
---

# Orca CLI

This file is a discovery stub, not the usage guide. The full, version-matched Orca CLI
reference is served by the `orca` binary itself — kept out of this file on purpose so it
can never drift from the binary that will actually run your commands.

Engage Orca whenever its running editor/runtime is the source of truth: Orca-managed
worktrees, folder contexts, terminals, repos, automations, worktree comments, and the
browser embedded inside the Orca app. Triggers include "$orca-cli", "Orca worktree",
"child worktree", "spawn codex/claude in a worktree", "read/wait/send Orca terminal",
"full handoff" / "handover" / "give this to another agent", and "control the browser
inside Orca". Use plain shell tools when Orca state does not matter.

## Resolve the CLI for this session

Choose the executable once and reuse it for every later command:

- If the `ORCA_CLI_COMMAND` environment variable is set, use its value. Orca exports this
  for managed WSL sessions.
- Otherwise, in a dev checkout whose session exposes `ORCA_DEV_REPO_ROOT`, use `orca-dev`.
- Otherwise, on Linux outside an Orca-managed terminal, use `orca-ide`. Never run bare
  `orca` there — outside Orca's terminals it normally resolves to the
  GNOME Orca screen reader (`/usr/bin/orca`) and starts speech on the user's machine.
- Otherwise, use `orca`.

Below, `ORCA` is a placeholder for the executable you resolved. Substitute it before
running anything; do not create a shell variable or run `ORCA` literally. This works the
same way in POSIX shells, PowerShell, and cmd.exe.

If the selected executable cannot run, report its exact error and stop. Do not fall through
to another executable, which could silently target a different Orca build.

## Headless runtime recovery

On Linux without a usable desktop window, use Orca's supported headless server instead of repeatedly retrying `open`:

```text
ORCA serve --no-pairing --project-root <absolute-repository-path> --json
```

Run it in a Herdr-managed pane when Herdr is the available terminal supervisor. The command is foregrounded and must remain alive. Verify readiness from a separate terminal:

```text
ORCA status --json
ORCA orchestration run-list --json
ORCA repo list --json
ORCA worktree current --json
```

A valid recovery requires `runtime.state: ready`, `reachable: true`, `orchestration.contract.v1` in capabilities, the repository registered, and the current worktree resolved. `ORCA orchestration run-current` requires a live Orca terminal sender, so treat `no_active_sender_terminal` as a terminal-binding issue, not a runtime failure. Keep the headless server running until the supervised run is complete; stop it through the owning Herdr pane afterward.

## Load the full guide before running Orca commands

```text
ORCA skills get orca-cli
```

That prints the complete, version-matched guide for the exact binary that will handle your
next commands — worktrees, handoffs, terminals, automations, and the built-in browser.
Read it first, then run the specific command you need.

Don't guess subcommands or flags from memory or from a cached copy of this stub. They
change between Orca releases, and this file deliberately no longer lists them. Confirm the
app is up with `ORCA status --json` (start it with `ORCA open --json` if needed), and
prefer `--json` for agent-driven calls.

## If an older Orca does not recognize `skills get`

Use this fallback only when the selected binary explicitly reports that `skills get` is an
unknown command. Another failure is not proof of an older binary; report it rather than
guessing or changing executables. For a confirmed pre-guide binary, use only this bounded,
read-only bootstrap to orient. Do not dead-end and do not invent commands:

```text
ORCA status --json
ORCA worktree ps --json
ORCA terminal list --json
```

Then tell the user that updating Orca restores the full, version-matched guide via
`ORCA skills get orca-cli`. Beyond these commands, ask the user rather than guessing a
command surface this older binary may not support.

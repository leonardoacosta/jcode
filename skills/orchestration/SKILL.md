---
name: orchestration
description: >-
  Use Orca orchestration for structured multi-agent coordination: threaded
  messages, blocking ask/reply flows, task dispatch, worker_done/escalation
  waits, task DAGs, decision gates, coordinator loops, or decomposing work
  across agents. Use `orca-cli` instead for full ownership handoffs, including
  requests phrased as "hand off", "handoff", "handover", "give this to another
  agent", or "another worktree" when the user did not explicitly ask to
  supervise, monitor, wait for results, or coordinate a DAG. Use `orca-cli` for
  ordinary terminal control, lightweight terminal prompts, shell commands, Orca
  worktree management, reading or waiting on terminals, and automation of the
  browser embedded inside Orca. Use Computer Use for browser windows, webviews,
  Orca app UI, or desktop UI outside Orca's embedded browser.
---

# Orca Orchestration

This file is a discovery stub, not the usage guide. The full, version-matched Orca
orchestration reference is served by the `orca` binary itself — kept out of this file on
purpose so it can never drift from the binary that will actually run your commands.

Engage Orca orchestration whenever you need structured multi-agent coordination: threaded
messages, blocking ask/reply flows, task dispatch, worker_done/escalation waits, task DAGs,
decision gates, coordinator loops, or decomposing work across agents. Use the orca-cli skill
instead for full ownership handoffs ("hand off", "handoff", "handover", "give this to
another agent", "another worktree") when the user did not ask to supervise, monitor, wait
for results, or coordinate a DAG — and for ordinary terminal control, shell commands,
worktree management, and the built-in browser. Coordination requires real Orca runtime
state; never substitute a non-Orca subagent tool.

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

## llmtrim and Codex launcher compatibility

The project policy is baremetal agent access with a guard layer, not a Codex sandbox. Use Codex's explicit no-sandbox mode while retaining the harness/llmtrim/Shepherd guard path:

```text
--sandbox danger-full-access --ask-for-approval never
```

Do not use `--dangerously-bypass-approvals-and-sandbox` for normal workers. That is a separate escape hatch that bypasses the guard composition and conflicts with the approval flag. The failure mode is:

```text
--ask-for-approval <policy> cannot be used with --dangerously-bypass-approvals-and-sandbox
```

`llmtrim wrap` forwards Codex arguments and is compatible with `--sandbox danger-full-access --ask-for-approval never`. Keep the guard layer authoritative for blocked commands, secrets, destructive operations, and protected paths. `llmtrim doctor` also reports when the current shell predates setup; use a new shell or explicitly export the values from `llmtrim setup --env` before testing proxy routing.

## Direct Jcode launch for model-specific workers

When the user specifies a model/provider/effort, or when Orca's built-in agent launcher conflicts with a local wrapper (for example, `llmtrim` injecting incompatible Codex flags), do not route the work through a coordinator prompt just to make the coordinator delegate it. Launch the requested Jcode CLI directly in the isolated worktree instead:

1. Create the Orca child worktree first.
2. Start the worker with the requested model at process startup, not in the task prompt. For Codex through Herdr, pass CLI arguments after `--`, for example:
   `herdr agent start <name> --kind codex --pane <pane> --timeout 120000 -- -m gpt-5.5 -c model_reasoning_effort="low"`
   Use the configured OAuth profile/provider for the session, or pass the explicit Codex `-p <profile>` when the provider/profile is not the default. Verify the rendered status line shows the requested model before sending work. A prompt saying “use GPT-5.5” does not change the running Jcode model. It only instructs the current model to delegate, which is a different and usually needless hop.
3. Send the full task contract directly to that instance, including scope, checklist rules, validation commands, commit/no-merge policy, and reporting fields.
4. Keep the Orca task graph as the coordination and audit record. If the Herdr terminal handle is not addressable by Orca's supervised-worker API, record the terminal/worktree IDs and update the Orca task status/result manually from the coordinator.
5. Launch a separate direct Jcode reviewer in another terminal for review. Never use a Claude model when the user has prohibited it.

This avoids a needless coordinator delegation hop while preserving isolated worktrees, model fidelity, dependency gates, verification evidence, and merge decisions. Treat the direct Jcode instance as the worker and the main thread as the supervisor, not as an intermediate prompt relay.

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
ORCA skills get orchestration
```

That prints the complete, version-matched guide for the exact binary that will handle your
next commands — task creation and dispatch, injected lifecycle preambles, worker_done
authority, decision gates, and coordinator loops. Read it first, then run the specific
command you need.

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
ORCA orchestration task-list --json
ORCA terminal list --json
```

Then tell the user that updating Orca restores the full, version-matched guide via
`ORCA skills get orchestration`. Beyond these commands, ask the user rather than guessing a
command surface this older binary may not support.

---
execution:
  version: 1
  depends_on: []
---
# Proposal

## Why

On 2026-11-19 one bad daemon build (`8a2141d07`, selfdev profile) took down `jcode.service` 39 times in 36 minutes:

1. **Crash on resume.** Tokio's default 2 MiB worker stack overflowed inside `handle_client`'s resume path (~2 MB future at opt-level=0). Baseline repro exits 134 (SIGABRT).
2. **Client-driven crash loop.** Every reconnecting TUI ran `systemctl --user start jcode.service`, re-arming a unit systemd had already given up on (`start-limit-hit`), and re-requested the same poisoned resume.
3. **Garbled TUIs.** `systemctl` diagnostics and `stderr_info` lines were written over live raw-mode TUI frames.

Earlier (2026-10-31) the same unit looped 744 times on "already running" lock contention, and idle-timeout exits under `Restart=always` add churn. Nothing gates promotion of a binary that cannot resume a real session.

## What Changes

| Fix | Change | State at authoring |
|---|---|---|
| 1 | 8 MiB tokio worker stack; `Box::pin` the client future in `ServerRuntime` | written, verified on private socket |
| 2 | Per-session `~/.jcode/resume-in-progress/<sanitized id>` marker; a marker left by a dead pid refuses that resume | written, not compiled/tested |
| 3 | Clients refuse `systemctl start` when the unit reports `Result=start-limit-hit`; `systemctl` stdio nulled; `stderr_info` silent in raw mode | written, not compiled/tested |
| 4 | Supervised `serve` waits up to 10 s (250 ms poll) for the daemon lock; "already running" exits 78 | written, verified |
| 5 | No idle-timeout exit when `JCODE_SERVER_SYSTEMD_UNIT` is set | written, verified |
| 6 | `jcode server promote` boots the candidate on a private socket and isolated `JCODE_HOME`, resumes a copy of the newest real session, and refuses promotion on error/exit; `--skip-resume-check` opts out | not started |
| 7 | `await_members` startup scan ignores non-`.json` files (orphaned `.bak`) | written, verified |

Decisions (user, 2026-11-19): Fix 6 is an in-command smoke check (1a); the uncommitted Fix 2 and Fix 3 code is the starting point (2keep); the Fix 2 marker lives in its own `resume-in-progress/` directory, not `exit_records/` (3sep); the homelab unit drop-in already has `RestartPreventExitStatus=78` and needs no change (4ok).

## Non-scope

Homelab unit drop-in edits, token rotation, the old daemon's CPU/RAM anomaly, core cleanup, and any other uncommitted work in the tree (for example the `background_tasks::reattach_background_tasks_on_startup` hunk in `server.rs`, `client_lifecycle.rs`, `swarm/`, `src/cli/terminal.rs`).

## Capabilities

### Modified Capabilities

- `shared-server-crash-resilience`: adds resume-poison quarantine, supervised start-limit respect, lock-wait/exit-78, supervised residency, worker stack headroom, raw-mode stderr discipline and promotion resume gating.

## Impact

`src/main.rs`, `src/cli/{dispatch.rs,dispatch_tests.rs,output.rs,commands.rs}`, `crates/jcode-app-core/src/server.rs`, `crates/jcode-app-core/src/server/{runtime.rs,socket.rs,util.rs,await_members_state.rs,session_manager.rs,session_manager_tests.rs}`. Harnesses `scratch/rc-core/{repro.sh,lock_exit.sh}` are local evidence tools and are not committed. The shared daemon is never reloaded during this work; `builds/current` stays at 1.0.1.

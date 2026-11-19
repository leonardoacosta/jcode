# Design

## Context

Incident chain on 2026-11-19 (build `8a2141d07`): a stack overflow on resume aborted the daemon. Then each reconnecting client re-ran `systemctl start` and re-requested the same resume, so systemd's start limit was bypassed and the crash repeated 39 times. Client stderr was printed over raw-mode TUIs. The fixes are layered so that any single layer stops the loop:

```
bad binary ──► Fix 6 promote gate (never reaches shared-server)
   │ (skipped / missed)
   ▼
resume crash ──► Fix 1 stack headroom (root cause)
   │ (new overflow / other panic)
   ▼
same resume again ──► Fix 2 dead-pid marker refuses that session only
   │
   ▼
restart storm ──► Fix 3 clients respect start-limit-hit · Fix 4 exit 78 · Fix 5 no idle churn
   │
   ▼
visible damage ──► Fix 3 raw-mode stderr discipline · Fix 7 quiet startup
```

## Decisions

### Fix 1: stack headroom, not just boxing
`thread_stack_size(8 MiB)` matches the existing Windows main-thread reserve. `Box::pin(client)` moves the largest future off the worker stack. Both are cheap. Shrinking the future itself is out of scope.

### Fix 2: separate `resume-in-progress/` directory (decision 3sep)
`exit_records/` describes *process* exits keyed by pid and is consumed by crash classification. A resume marker is *session*-keyed and short-lived, so mixing them would make classification parse non-exit files. The marker holds only the pid. Session ids pass through `durable_state::sanitize_session_id`. The marker is removed whether the restore succeeds or fails; only a process death leaves it behind. Liveness uses `lifecycle::process_alive`. Accepted risk: pid reuse can make a dead writer look alive, which fails open (the resume proceeds), never closed.

### Fix 3: `Result=start-limit-hit` as the signal
`Result=start-limit-hit` persists until `reset-failed` or a successful start, so it is durable. `NRestarts` counts restarts over the unit's lifetime rather than within a time window, so it would refuse healthy long-lived units. Per-client backoff already exists (`DEAD_SERVER_RESPAWN_INTERVAL`, 60 s, in `reconnect.rs`), so the TUI needs no change. `stderr_info` checks `crossterm::terminal::is_raw_mode_enabled()` and drops the line in raw mode; the TUI shows its own reconnect status.

### Fix 4: exit 78 (EX_CONFIG)
78 exits ensure `RestartPreventExitStatus=78` units stop instead of looping. The homelab drop-in already sets this (decision 4ok). The 10 s wait applies only under supervision, where stop/start overlap and exec reloads release the lock late.

### Fix 5: residency keyed on `JCODE_SERVER_SYSTEMD_UNIT` only
This is the same explicit opt-in that disables client spawning. Inferring supervision from cgroups was rejected because it is ambiguous for user scopes.

### Fix 6: in-command promotion smoke (decision 1a)
Inside `run_server_promote_command` (`src/cli/commands.rs`), before `promote_version_to_shared_server`:
1. Pick the newest session file under the real sessions directory by mtime, skipping test or debug sessions (`JCODE_TEST_SESSION` markers).
2. Create a temporary `JCODE_HOME` with `sessions/<id>.json` copied in, plus the minimum config needed to boot. Do not copy credentials or provider auth: the check needs no provider call.
3. Spawn `<candidate> --no-update --socket <tmp>/s.sock serve` with `JCODE_TEST_SESSION=1` and `JCODE_NON_INTERACTIVE=1`. Remove `JCODE_SERVER_SYSTEMD_UNIT` from the child environment.
4. Connect with `Client`, call `resume_session_with_options`, and wait for `ServerEvent::Done` or an error. Total timeout is 60 s (constant). Watch the child; an exit before Done counts as failure and reports the exit status or signal.
5. Kill and reap the child, then delete the temporary directory, on every path.
6. Add `promote --skip-resume-check`, and add `resume_check: {status: passed|failed|skipped_flag|skipped_no_session, detail}` to `ServerPromoteReport`.

Alternatives rejected: a script under `scripts/` (1b) is easy to skip, and the bad binary shipped through exactly this command. Dropping the check (1c) leaves the shipped failure mode open.

## Risks

- **Fix 6 latency.** The check adds up to about 60 s to promotion. This is acceptable for a deploy step; the opt-out flag exists for emergencies.
- **Fix 6 large session copy.** Copy only the single session file. Never copy the whole home.
- **Shared-tree commits.** Other sessions have uncommitted hunks in `server.rs`, `socket.rs`-adjacent files and `dispatch.rs`. Stage by hunk with `git add -p` or patch files, and verify with `git diff --cached` before each commit.

# Tern daemon resilience techniques: fit for the Jcode daemon

2026-11-19 · Explore · research only, no implementation or proposal.

## Recommendation

**No change.** None of the ten ranked Tern techniques justifies Jcode work now. Six already exist in Jcode in an equivalent form. Two target failure windows that Jcode's logs show it handles today. Two conflict with Jcode's lifecycle direction. Two premises in the findings doc do not hold for Jcode:

- Jcode's reload is already a same-PID `execve`, so systemd's MainPID never changes on reload.
- `jcode server promote` repoints a channel link after a resume smoke check. It does not restart the unit.

One optional decision about the homelab unit is recorded below. It lives outside this repository and does not block anything.

## Scope and evidence limits

- Input: `scratch/tern-resilience-findings.md` (Tern `0.9.18 (6f1fa0f)`, reverse-engineered on isolated sandbox daemons). I took Tern claims as reported. I did not re-run any Tern experiment.
- Jcode source: `main` with 31 unrelated modified files, which I did not touch. Paths cited below are at that tree.
- Runtime evidence came from read-only checks:
  - `/proc` of the live shared daemon: release `1.0.1`, run under `jcode.service` through a secret-injecting `op run` wrapper.
  - `systemctl --user show jcode.service`, the unit's journal since 2026-11-01, and the daily logs in `~/.jcode/logs/`.
  - Two local probes: `flock` semantics in Python, and `op run` exit-status propagation using an empty env file.
- No daemon was signaled, reloaded or restarted. No build or test ran. Reload-gap timing comes from 3 logged reloads, which is a small sample.

## What Jcode already does

| Lifecycle point | Jcode mechanism | Where |
|---|---|---|
| Single instance | `flock(LOCK_EX\|LOCK_NB)` on `<socket>.daemon.lock`. When supervised, waits 10 s for the lock, then exits 78. | `server/socket.rs` `acquire_daemon_lock_for`, `src/cli/dispatch.rs` serve arm (commit `84b3f4efc`) |
| In-place upgrade | Same-PID `execve` of `serve --socket S` plus inherited provider and identity flags. The listener and the lock are `FD_CLOEXEC`. The socket is unlinked and rebound. | `server/reload.rs`, `server.rs` `run` |
| Pre-exec gates | Control-reader floor probe, a no-downgrade guard and a binary-exists check, all run before sessions shut down. Same binary means a fresh re-exec. | `server/reload_exec_target.rs`, `server/runtime_upgrade.rs` (`UpgradeDecision::Unchanged`) |
| State across exec | Recovery intents persisted, prewarm of owned sessions, reaping of inherited children, and reattachment of detached background tasks by PID and start time. | `server/reload.rs`, `inherited_children`, `server/background_tasks.rs` (commit `6d48d1de9`) |
| Poisoned resume | A per-session `resume-in-progress` marker left by a dead PID refuses that one resume. | `refuse_poisoned_resume` (`harden-rc-crash-loop` Fix 2) |
| Promotion gate | `server promote` resumes a copy of the newest real session on a private socket and refuses on failure. | `src/cli/commands.rs` (Fix 6) |
| Client during reload | Reads the reload state and waits up to 30 s for the handoff. Backoff starts at 100 ms and grows exponentially to a 30 s cap. Dead-server respawn needs at least 3 failed attempts and is throttled to once per 60 s. Clients will not start a unit that has hit its start limit. | `tui/app/remote/reconnect.rs`, `src/cli/dispatch.rs` |
| Stack headroom | 8 MiB tokio worker stack and a boxed client future. | `src/main.rs` |

Observed numbers:

- **Exec-to-listen gap:** 38 ms, 71 ms and 227 ms across the 3 logged reloads (2026-11-18 and 2026-11-19).
- **Live daemon state:** the lock fd has flags `02100001`, so `O_CLOEXEC` is set. `NRestarts=0`. 36–37 threads, of which 29 are `tokio-rt-worker`. All the rest have `jcode-*` names except one `reqwest` internal thread.

## Technique-by-technique scrutiny

| # | Tern technique | Jcode status | Verdict and reason |
|---|---|---|---|
| 1 | Keep the lock fd across exec, so there is no unlocked window | Gap, but mitigated | **Reject for now.** The window is the exec-to-bind gap of 38–227 ms. A competing starter would have to be either systemd, which cannot start a second instance of an active unit, or a client, which sees reload state `Starting` and waits. The 17 "already running" refusals on 2026-11-18 came from an unsupervised holder colliding with the unit. Inheriting the lock does not fix that case. The shipped lock-wait plus exit-78 fix does. Inheriting also has a cost: `flock` is per open file description, and a probe confirmed that reopening the same file in the same process gets `EWOULDBLOCK`. So the new image would have to adopt a numbered fd passed through env. That adds a contract and a leak risk to close a window no incident has used. |
| 2 | Keep the listener fd across exec, plus a handoff file | Partially present | **Reject.** The premise does not hold, because MainPID already survives reload. Inheriting the listener would only turn sub-250 ms `ENOENT` connects into queued connects, and the reconnect path already absorbs those. Jcode's handoff is spread across disk records (intents, sessions, background tasks) instead of one versioned blob. That avoids Tern's own failure mode, where a decode error drops all handed-over state at once. |
| 3 | Unchanged exe means no exec | Present | `UpgradeDecision::Unchanged`, plus the same-binary fast path. |
| 4 | Client-driven upgrade on a protocol-level mismatch | Present, in a different form | A newer client triggers reload through `has_newer_binary` and `server_has_update`. Durable stop compatibility is fenced by `control_reader_version`, which fails closed. No incident shows wire-protocol drift, so an integer protocol level is not justified. |
| 5 | Degraded restore and a "fresh tab" fallback | Present | The poisoned-resume marker refuses one session and leaves the daemon running. That is the fix for the 39-restart class. |
| 6 | Periodic 10 s snapshot thread | Not applicable | Tern needs it for PTY scrollback, which only exists in memory. Jcode persists session state as messages arrive. |
| 7 | Client wait with a deadline and a plain message | Present | 30 s handoff wait, capped backoff and a respawn throttle. A Tern-style local "ephemeral" mode does not map to agent turns, which need the daemon's provider runtime. |
| 8 | Unit hygiene (`Type=exec`, `on-failure`, `RestartPreventExitStatus`, `KillMode=process`) | Mostly present | `RestartPreventExitStatus=78`, `StartLimitBurst=5/120s` and `RestartSec=5` are live. A probe showed that `op run` passes exit 78 through unchanged, but turns a child killed by a signal into **255**. That is why the 2026-11-19 SIGABRT loop shows in the journal as 39× `255/EXCEPTION` instead of 134. It is harmless for restart policy and only costs diagnostic detail. `KillMode=process` is a real tradeoff: see Decision 1. |
| 9 | Idle exit | Conflicts with Jcode's direction | **Reject.** `keep-shared-daemon-resident` and Fix 5 deliberately keep the supervised daemon resident so that schedule, ambient and background work survives. The stale-image concern Tern addresses with idle exit is covered by update-driven reload. |
| 10 | Named threads | Present enough | Non-runtime threads already have `jcode-*` names. A stack overflow still names its thread (`tokio-rt-worker`). Renaming tokio workers would not have shortened the 2026-11-19 diagnosis. |

Tern techniques that Jcode does **not** need to borrow: Tern's 2 MiB default stacks and its lack of a recursion guard. Jcode's 8 MiB stack and boxed future already go beyond Tern here.

## Decision (resolved 2026-11-19: keep `control-group`)

**1. Should the homelab unit drop-in set `KillMode=process`?**

- **Why it matters:** with `KillMode=control-group`, any systemd stop or crash restart kills detached background tasks and MCP children. Reattachment (`6d48d1de9`) only works across exec reload, not across a unit restart.
- **Choice A: keep `control-group` (recommended).** Children die with the daemon. That is predictable, and repeated restarts cannot pile up orphans. A crash already loses in-flight turns, so keeping only their background children buys little.
- **Choice B: `process`.** Background tasks would survive a crash or restart and could reattach. The costs: orphaned MCP servers and the `op` wrapper's children outlive the unit, a crash loop accumulates leaked processes, and `systemctl stop` no longer stops everything.
- **Unlocks:** nothing in this repo. The drop-in is outside the repo and outside the scope of both `harden-rc-crash-loop` and `keep-shared-daemon-resident`.

## Unknowns that would change the verdict

- If reloads start logging connects that fail during the exec gap, or "already running" refusals while a reload is in flight, revisit techniques 1 and 2. Today's sample of 3 gaps is small.
- If a future incident shows clients and daemon disagreeing on message shapes, revisit technique 4.
- Tern internals marked unknown in the findings doc (backoff numbers, how exe changes are detected, how a rejected handoff is handled) cannot change this verdict. Each of them has a Jcode equivalent that is already chosen.

## Route

No change. Leave `keep-shared-daemon-resident` as the active lifecycle vehicle. Reopen this analysis only on the triggers listed under unknowns.

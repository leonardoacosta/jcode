## Why

Forensics (60d) show shared server PIDs dying with 50-377 attached sessions each, all recorded as "Process N exited unexpectedly (no shutdown signal captured)" (`crates/jcode-base/src/session.rs::detect_crash`, `session/crash.rs`). The cause (signal, OOM kill, panic) is never captured, so crashes cannot be diagnosed, and mass-crashed sessions need manual recovery. A WIP supervised-server gate (`enforce_supervised_shared_server` in `src/cli/dispatch.rs`, marker `~/.jcode/require-supervised-server` = `jcode.service`) is uncommitted and unspecified.

## What Changes

- Server writes a durable exit record (clean shutdown, caught signal, panic message and location) before exiting; panic hook added for `serve`.
- On crash detection, classify cause from the exit record, else from supervisor data (systemd `Result=oom-kill`, `ExecMainStatus`), else `unknown`; store it as the session crash reason.
- Specify the supervised-server gate: non-temporary `serve` outside the marker unit's cgroup delegates to `systemctl --user start <unit>` and exits; no marker keeps current behavior.
- Crash recovery: sessions orphaned by one server PID are grouped into one crash event, recoverable in bulk, with one summary notice instead of N.

## Capabilities

### New Capabilities
- `shared-server-crash-resilience`: exit-cause capture, supervised-start gate, grouped session recovery.

## Impact

`src/cli/dispatch.rs`, `src/cli/terminal.rs` (panic hook), `crates/jcode-app-core/src/server.rs` (signal/exit record), `crates/jcode-base/src/session.rs`, `crates/jcode-base/src/session/crash.rs`.

## Open decisions

1. **Auto-resume vs. prompt after crash.** Recommended default: mark recoverable and show one grouped prompt; never auto-resume turns (avoids repeating side-effecting tool calls). Alt: auto-resume idle sessions only (faster, risk of duplicate work).
2. **OOM evidence source.** Recommended: systemd unit properties when supervised, else `unknown`; no kernel log scraping. Alt: parse `journalctl -k` (better coverage, privileged and slow).
3. **Gate failure when `systemctl` is unavailable.** Recommended: refuse to start with an error naming the marker file. Alt: warn and run unsupervised (keeps working, loses unit-injected secrets).

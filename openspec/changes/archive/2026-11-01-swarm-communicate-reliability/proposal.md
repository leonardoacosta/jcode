# Proposal

## Why

The swarm `communicate` tool reports false failures and gives misleading errors. Every socket request uses a fixed 30s deadline (`send_request_with_timeout`, `crates/jcode-app-core/src/tool/communicate/transport.rs:85-86`), including `spawn` (`communicate.rs:1425`) and `launch`/`run_plan` setup calls. On a busy server the spawn can complete after the client gives up, the agent sees "Timed out waiting for response", retries, and creates a duplicate worker. Errors are wrapped uniformly as `Failed to <verb>: {}` (e.g. `communicate.rs:3141`), so a client-side timeout reads the same as a server-side rejection. The unknown-action error (`communicate.rs:3147-3151`) lists internal actions and omits advertised `launch`, `list_models` and `list_swarms`. `await_members` forces `background = true` (`communicate.rs:3090`) and ignores `background=false`, leaving the blocking socket-timeout branch (`communicate.rs:3108-3112`) dead.

## What Changes

- Per-action socket deadlines: short for reads, longer for `spawn`, `launch`, `retry`, `resume` and plan mutations.
- Typed request outcome distinguishing client timeout, connection closed and server error. Timeouts on mutating actions say the action may have succeeded and point to `status`/`list` before retrying.
- Unknown-action error derived from `ADVERTISED_SWARM_ACTIONS`.
- `await_members` honors `background=false` (blocking wait with `timeout_secs + 30` deadline).
- Tests: schema enum vs dispatch parity, timeout classification, unknown-action message, `await_swarm_progress` loop.

## Capabilities

### New Capabilities

- `swarm-communicate-reliability`: Deadlines, error classification and action discoverability for the swarm communicate tool.

## Impact

`crates/jcode-app-core/src/tool/communicate.rs`, `crates/jcode-app-core/src/tool/communicate/transport.rs`, `crates/jcode-app-core/src/tool/communicate_tests.rs`. No protocol or config change.

## Evidence

Checked against current `~/.jcode` source on 2026-11-01:

- Confirmed: fixed 30s default deadline in `send_request_with_timeout` (`transport.rs:76-86`); `send_request` passes `None` (`transport.rs:72-74`); spawn uses `send_request` (`communicate.rs:1425`), as do about 20 other call sites.
- Confirmed: timeout surfaces as `anyhow::bail!("Timed out waiting for response")` in `transport.rs`, then is re-wrapped as `Failed to await members: {}` and similar (`communicate.rs:3141`); no type distinguishes it from server `Error` events.
- Confirmed: unknown-action list at `communicate.rs:3147-3151` includes internal actions and lacks `launch`, `list_models`, `list_swarms`; `ADVERTISED_SWARM_ACTIONS` exists near `communicate.rs:1683`.
- Confirmed: `let background = true;` beside `blocking_was_requested = params.background == Some(false)` (`communicate.rs:3089-3090`); the non-background deadline branch at `3108-3112` is unreachable.
- Partly confirmed: `schema_advertises_a_lean_lane_first_action_set` (`communicate_tests.rs:1050`) checks the enum list, but no test asserts every advertised action dispatches. `await_should_wake_for_new_ready` is tested (`communicate_tests.rs:285-293`), but `await_swarm_progress` (`communicate.rs:333-400`) has no loop test.
- Dropped: none. Line numbers corrected from the fork audit (unknown-action at 3147, not 3146).

## Resolved decisions

1. `await_members background=false`: honor it (blocking wait with `timeout_secs + 30` deadline), since the schema advertises it.
2. Mutating-action deadline: 120s for `spawn`/`launch`/`retry`/`resume`, 30s for reads. Idempotency keys stay out of scope.

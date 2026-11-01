# Proposal

## Why

CI runs `cargo clippy --locked --all-targets --all-features -- -D warnings` (`.github/workflows/ci.yml`, "Run clippy with warnings denied"), but the tree at `8e3f1b459` fails it: the `jcode-app-core` lib test target has 13 clippy errors and the `jcode-tui-mermaid` example `image_lifecycle` has 2. The CI clippy step is therefore red, so new lint regressions land unnoticed. Six errors are `await_holding_lock` in `turn_failover.rs`, a real deadlock and latency risk, not just style.

## What Changes

- Fix all 13 `jcode-app-core` clippy errors (mostly test code; `turn_failover.rs` guards dropped before `.await`).
- Fix the 2 `collapsible_if` errors in `crates/jcode-tui-mermaid/examples/image_lifecycle.rs`.
- Keep the existing CI clippy step as the guard and make it a required check so the workspace stays clean.

## Capabilities

### New Capabilities

- `clippy-clean-workspace`: Workspace passes clippy with `-D warnings` on all targets, enforced by CI.

## Impact

`crates/jcode-app-core/src/{agent/idle_hibernation.rs,agent/streaming.rs,agent_tests.rs,agent_tests/tool_streaming.rs,server/{client_state_tests.rs,idle_hibernation.rs,runtime_upgrade_tests.rs,swarm_todo_link.rs,turn_failover.rs}}`, `crates/jcode-tui-mermaid/examples/image_lifecycle.rs`. No behavior change except lock scope in `turn_failover.rs`.

## Evidence

Checked 2026-11-01 at `8e3f1b459`.

- `cargo clippy -p jcode-app-core --all-targets -- -D warnings`: 13 errors.
  - `await_holding_lock` x6: `server/turn_failover.rs` (guards near lines 356, 438, 486).
  - `field_reassign_with_default` x2: test files (`server/runtime_upgrade_tests.rs:113`, `server/client_state_tests.rs:68` area).
  - `items_after_test_module` x2: `agent/idle_hibernation.rs:13`, `server/swarm_todo_link.rs:305`.
  - `collapsible_if` x1, `redundant_closure_call` x1, `type_complexity` x1, across `agent/streaming.rs:203`, `agent_tests.rs:2709`, `agent_tests/tool_streaming.rs:114`, `server/client_state_tests.rs:98`, `server/idle_hibernation.rs:235/240`, `server/runtime_upgrade_tests.rs:119`. Exact lint-to-site mapping to confirm at fix time.
- Corrected: "turn_failover.rs x10" counted `-->` locations (errors plus notes); actual errors there are 6.
- `cargo clippy --workspace --all-targets -- -D warnings`: also fails in `jcode-tui-mermaid` example `image_lifecycle` (`collapsible_if` x2, lines 208/209). Other crates may be masked by early stop; re-run with `--keep-going` during the fix.
- CI already enforces clippy (`ci.yml`) with `--all-features`; the local runs above omitted `--all-features`.

## Open decisions

1. `await_holding_lock` fix. Recommended: scope or drop guards before `.await`, using `tokio::sync::Mutex` only where the guard must span the await. Alternative: `#[allow]` (hides a real risk).
2. Guard. Recommended: rely on the existing CI step and mark it required. Alternative: add a pre-commit clippy hook (slower local commits).

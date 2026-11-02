# Proposal

## Why

`crates/jcode-app-core/src/server/swarm.rs` is 3518 lines: ~1974 lines of production code plus an inline `mod tests` of ~1540 lines (31 tests, from line 1975). It mixes unrelated concerns (spawn-tree walking, status broadcast debouncing, task staleness/heartbeat, plan broadcast, member lifecycle/salvage, idle-worker reaping, `run_swarm_task`/`run_swarm_message`). Reviews and merges touch the same file for unrelated changes.

## What Changes

- Convert `server/swarm.rs` into `server/swarm/mod.rs` plus concern submodules; `mod.rs` re-exports every item currently visible outside the file so all `super::swarm::*` call sites compile unchanged.
- Proposed seams (from measured function boundaries):
  - `tree.rs`: `swarm_ancestors`, `swarm_spawn_depth`, `swarm_is_self_or_ancestor`, `swarm_descendants`, `cancel_spawned_descendant_turns` (lines 39-183).
  - `config.rs`: env/config timing helpers (`swarm_status_debounce_*`, `configured_positive_u64`, `swarm_task_heartbeat_interval`, `swarm_task_stale_after`, `swarm_task_sweep_interval`, terminal retention/GC intervals, `now_unix_ms`, `log_swarm_lifecycle`) (lines ~184-300).
  - `lifecycle.rs`: idle reap candidates, salvage (`salvage_plan_assignments_of`, `salvage_assignments_of_dead_member`, `notify_coordinator_of_salvage`), `remove_session_from_swarm`, unrestored-idle notices, member status updates and event recording (lines ~355-570, 1111-1776).
  - `tasks.rs`: `touch_swarm_task_progress`, `refresh_swarm_task_staleness` (lines 571-760).
  - `status.rs`: `broadcast_swarm_status`, `broadcast_swarm_status_now`, pending broadcast debounce (lines 761-911).
  - `plan.rs`: `broadcast_swarm_plan*`, `send_swarm_plan_to_session` (lines 912-1110).
  - `run.rs`: `run_swarm_task`, `run_swarm_message`, `SwarmTaskSpec`, `parse_swarm_tasks` (lines 1777-1974).
  - Tests move to `swarm/tests.rs` with bodies unchanged.
- No logic, signature, visibility-widening, or behavior changes. Functions are not split.

## Capabilities

### New Capabilities

- `swarm-server-modules`: Behavior-preserving module structure for the server swarm coordinator.

## Impact

`crates/jcode-app-core/src/server/swarm.rs` becomes `crates/jcode-app-core/src/server/swarm/`. `swarm_cascade_tests.rs` (138 lines) keeps its path. No public API, protocol, or persisted-state change.

## Evidence

Checked against the current tree in `/home/nyaptor/.jcode`:
- `wc -l` confirms 3518 lines.
- `#[cfg(test)] mod tests` starts at line 1975; 31 `#[test]`/`#[tokio::test]` attributes.
- Largest functions (span to next fn): `remove_session_from_swarm` 230, `update_member_status_with_report_tldr` 179, `refresh_swarm_task_staleness` 119, `broadcast_swarm_plan_with_previous` 112, `notify_owner_of_unrestored_idle_workers` 108, `run_swarm_message` 96, `run_swarm_task` 87, `cancel_spawned_descendant_turns` 77, `broadcast_swarm_status` 76, `broadcast_swarm_status_now` 75. No function exceeds 230 lines, so the problem is file cohesion, not giant functions.
- Production items are mostly `pub(super)`; moving them one level deeper requires `pub(in crate::server)` or re-exports to keep identical reach.
- Corrected: the brief suggested `spawn`/`message` modules. This file has no spawn handler (only spawn-tree helpers at lines 67-183 and reap at 355), so no `spawn.rs` is proposed; messaging lives in `run_swarm_message` and goes to `run.rs`.

## Resolved decisions

1. Visibility mechanism. Chosen: submodules declare `pub(in crate::server)` and `mod.rs` re-exports them, keeping reach identical. Alternative: `pub(crate)` everywhere (simpler, widens API).
2. Tests layout. Chosen: one `swarm/tests.rs` moved verbatim (minimal, reviewable diff). Alternative: per-module test files (better locality, larger diff).

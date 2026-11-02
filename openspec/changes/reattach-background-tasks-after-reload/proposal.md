# Proposal

Roadmap owner: [P05: Durable overnight, scheduling and recovery](../../../docs/plans/PRODUCT_ROADMAP.md#p05-durable-overnight-scheduling-and-recovery). Supporting: P04.

## Why

`recover-swarm-workers-after-reload` covers swarm members only. Background tasks started with `bash run_in_background` are not reattached after a server reload, so running builds and tests disappear from `bg list` and their completions never wake the session.

## What Changes

- Persist the background task registry (task ID, owning session, PID, start time, output path, wake/notify flags).
- On reload, reattach tasks whose PID is alive and matches the start time; keep tailing output and deliver completion as before.
- Mark tasks whose process is gone as `lost`, keeping the output file reachable.

## Capabilities

### New Capabilities

- `background-task-recovery`: background tasks survive server reload.

## Impact

Background task manager and server reload path in `crates/jcode-app-core`. Exit status of reattached non-child processes may be unknown; report it as unknown rather than success.

## Open decisions

1. Exit status of reattached processes? Recommended: report `exited (status unknown)` unless written by a wrapper.

## Evidence

Survey: [plans/2026-11-01-background-agent-ux-gaps.md](../../../plans/2026-11-01-background-agent-ux-gaps.md) gap 6.

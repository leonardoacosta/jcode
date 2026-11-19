---
execution:
  version: 1
  depends_on: [make-worker-stop-durable]
---
# Proposal

Roadmap owner: [P05: Durable overnight, scheduling and recovery](../../../docs/plans/PRODUCT_ROADMAP.md#p05-durable-overnight-scheduling-and-recovery). Supporting: P04.

## Why

The original discovery predates current background status files, detached-task registration, orphan reconciliation and persisted session recovery notes in `crates/jcode-base/src/background.rs`. Reuse that manager and characterize remaining process reattachment, ownership and completion-delivery gaps instead of adding another registry or claiming all persistence is absent.

## What Changes

- Extend existing persisted background task records only where needed: task ID, owning session/execution generation, PID/start identity, output path and delivery flags.
- On reload, reattach tasks whose PID is alive and matches the start time; keep tailing output and deliver completion as before.
- Mark tasks whose process is gone as `lost`, keeping the output file reachable.
- Preserve durable stop/revocation when reconciling owned tasks. Completion delivery can update history but must not wake a revoked generation; uncertain liveness remains unresolved for worker settlement.

## Capabilities

### New Capabilities

- `background-task-recovery`: background tasks survive server reload.

## Impact

Background task manager and server reload path in `crates/jcode-app-core`. Exit status of reattached non-child processes may be unknown; report it as unknown rather than success.

## Resolved defaults

1. Report reattached processes as `exited (status unknown)` unless the exit status is written by a wrapper.

## Evidence

Survey: [plans/2026-11-01-background-agent-ux-gaps.md](../../../plans/2026-11-01-background-agent-ux-gaps.md) gap 6.

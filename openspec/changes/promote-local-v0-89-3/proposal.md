## Why

The merged v0.89.3 source passed regression checks but the shared daemon still runs v0.88.176. Activate the validated local source without losing rollback or disrupting active work unexpectedly.

## What Changes

- Freeze and record main's promotion source revision and build fingerprint.
- Build with coordinated selfdev tooling and validate the candidate in an isolated runtime before activation.
- Coordinate active sessions, verify pending activation ownership, and promote both launcher and shared daemon through the supported selfdev path.
- Retain the old immutable binary and record a rollback procedure.

## Capabilities

### New Capabilities
- `validated-local-promotion`: Evidence-gated local launcher and shared-daemon activation with rollback.

### Modified Capabilities
None.

## Impact

Local build channels, launcher, shared daemon and session reload lifecycle. No remote push, stable release publication, credential changes, MCP configuration edits, or unrelated feature work.

## Approval status

User approved this promotion design on 2026-09-30, explicitly conditioned on validation, session coordination and rollback. Written spec awaits review.

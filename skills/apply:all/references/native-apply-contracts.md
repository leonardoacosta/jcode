# Native apply workflow contracts

These contracts are shared by `/apply` and `/apply:all`. They are intentionally runner-neutral and describe the durable state Jcode must freeze before mutation, update during execution, and use for recovery or closeout.

## Schedule contract v1

Required fields:

- `contract_version`: `native-apply.schedule.v1`.
- `schedule_id`: stable id for the frozen schedule revision.
- `schedule_revision`: monotonically increasing integer scoped to the selected feature or queue.
- `created_at` and `created_by`: provenance for the schedule producer.
- `repository`: repository authority, root, branch, and current commit.
- `selection`: exact user-selected feature or ordered queue arguments. `/apply:all` must not add features not present here.
- `features[]`: one record per selected feature with id, authority path or issue id, authority revision, approval state, dependencies, claims, touched paths, repositories, workspaces, external systems, schemas, deployment targets, mutable resources, verification contract, review contract, risk score, orchestration floor, lineage, idempotency scope, and extension fields.
- `preflight`: setup preference, consent receipts, telemetry availability, available runtime capabilities, missing runtime capabilities, and degraded-path decision.
- `execution_path`: frozen `jcode-native` or `orca-supervised` path plus rationale. This cannot silently change after mutation begins.
- `wave_plan`: dependency and conflict-safe waves derived from this schedule.
- `recovery`: attempt id, retry lineage, durable evidence locations, cleanup obligations, and stale-input invalidation fields.

## Wave-plan contract v1

Required fields:

- `contract_version`: `native-apply.wave-plan.v1`.
- `schedule_id` and `schedule_revision`: bind the wave plan to one frozen schedule.
- `waves[]`: ordered list of feature ids that may execute concurrently.
- `frontier`: currently ready feature ids with blocker-free preconditions.
- `serialized_edges[]`: dependency or conflict edges with source, target, kind, resource, and rationale.
- `paused[]`, `blocked[]`, `failed[]`, `completed[]`, `skipped[]`, and `remaining[]`: queue-state buckets used for truthful partial settlement.
- `integration_gates[]`: queue-level checks, their required branches, and gate status.

## Validation and rejection rules

Fail closed before mutation when any of these hold:

- Input is missing, ambiguous, unsupported, stale, legacy-shaped, or resolves to more than one authority.
- Selected features are not approved, current, or implementation-ready.
- Repository revision, feature authority revision, dependency set, or verification contract changed after scheduling.
- Hard dependencies contain a cycle.
- Required Orca or external capability is unavailable and the selected work cannot satisfy isolation, supervision, recovery, and validation on the Jcode-native path.
- A requested lower orchestration tier is below the computed policy floor and lacks an explicit authorized approval gate.

## Conflict analysis

Serialize features when they overlap any of these mutable resources:

- Touched paths, generated artifacts, package boundaries, repositories, workspaces, or named claims.
- Database schemas, migrations, seed data, destructive operations, deployments, infrastructure, CI, auth, permissions, secrets, external APIs, browser sessions, devices, or other live runtime state.
- Any declared extension resource in a feature contract.

## Degraded-path policy

Telemetry, side-pane projection, or optional runtime observation failures are degraded capabilities. They must be reported but must not weaken scheduling, review, verification, persistence, or settlement policy.

Orca, repository authority, consent, approval, verification, persistence, archive, or issue-settlement failures are execution capabilities. If required, they pause before mutation or before the affected lifecycle phase with exact blockers.

## Review and settlement receipts

Review receipts must include reviewer identity or provider family, immutable requirements input, final diff identity, verification evidence input, findings, severity, affected requirement, and reproducible check. Relevant mutation invalidates the receipt.

Settlement requires fresh correlated evidence for implementation, verification, review, persistence, and authority update. Worker completion, terminal success, browser state, and runtime observation are evidence only, never settlement authority by themselves.

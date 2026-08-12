---
name: apply
description: Native Jcode single-feature execution workflow. Use to execute one approved, current feature through preflight, implementation, verification, review, persistence, recovery, and truthful settlement without invoking Codex or Claude-owned apply workflows.
---

# Native Apply

Treat `/apply` as Jcode's native lifecycle for one approved feature. It resolves exactly one authoritative input, freezes an execution path before mutation, executes the full contract, and settles only from fresh evidence. It does not activate `codex-apply` or Claude-owned workflows.

## Invocation and input resolution

- Preserve the selected feature argument exactly.
- Run the shared workflow preflight from `explore` for repository identity, setup preferences, telemetry, and degraded routes.
- Resolve one approved, current, implementation-ready feature from exactly one repository authority.
- Reject stale, ambiguous, unsupported, invalid, legacy, or multi-feature inputs before mutation.
- Reconstruct or reject schedules when authoritative artifacts, dependencies, repository revision, or verification contracts changed.

## Scheduling and execution path

Compile the selected feature into the shared canonical schedule used by `/apply:all`. The schedule records revision, provenance, dependency, conflict, touched path, mutable resource, risk, verification, lineage, idempotency, and extension fields.

Before editing, freeze and report one execution path:

- Orca-supervised when required runtime capabilities are available and necessary.
- Jcode-native when isolation, supervision, validation, and recovery needs can be satisfied without Orca.

Never silently downgrade after mutation begins. If a required capability is missing, pause before mutation with the exact missing capability and recovery option.

## Risk-selected review

Select rigor from observable risk and topology:

- Low risk may execute directly with deterministic checks.
- Normal risk uses independent review when warranted and may use the same provider family.
- High risk requires adversarial review from a different provider family plus separate acceptance verification.
- Critical risk requires multiple independent reviewers, cross-provider coverage, acceptance verification, and a human approval gate.

Users may request higher rigor. Lowering below policy requires an explicit authorized approval gate.

## Complete lifecycle

For the selected feature, execute and preserve one lifecycle:

1. Preflight authority, claims, dependencies, conflicts, mutable resources, workspaces, external systems, validation tools, and user gates.
2. Implement bounded tasks in dependency order, using delegation only for isolated work with no overlapping paths or mutable resources.
3. Run required targeted, regression, build, security, migration, deployment, or acceptance checks from the feature contract.
4. Review against requirements, final diff, verification evidence, and risks. Findings must include evidence, severity, affected requirement, and a reproducible check.
5. Persist owned changes with path-scoped staging and commits where repository policy allows.
6. Settle, archive, close issues, or update authorities only when fresh correlated evidence satisfies the declared contract.

Worker completion messages, terminal state, browser state, or runtime observations are evidence only. They are not durable settlement authority by themselves.

## Recovery and idempotency

On resume or retry:

- Reconstruct state from the authoritative feature, frozen schedule, Git state, Jcode initiative checkpoints, Orca receipts when present, and fresh verification.
- Do not use conversation memory as execution state.
- Assign every retry a new attempt identity linked to the original.
- Prevent duplicate mutation through attempt-scoped idempotency and current artifact checks.
- Preserve unresolved cleanup, recovery, or validation obligations in durable state.

## Projection, telemetry, and efficiency

- Project compact state, execution mode, rationale, receipts, missing capabilities, recovery obligations, and authorized actions in `side_panel` when useful.
- Keep terminal output compact and event-driven. Link to durable evidence instead of ingesting full transcripts.
- Check telemetry every invocation and emit best-effort scheduling, risk, orchestration, review, verification, degradation, recovery, and settlement observations when supported.
- Prefer typed Jcode tools, structured output, batching, timeouts, and source-side caps. Use shell only when no typed surface exists.

## Output contract

Report implementation scope, execution path, schedule revision, validation evidence, review evidence, persistence result, durable outcome, blockers, recovery obligations, and any follow-up work. Never report completion from code changes or partial tests alone.

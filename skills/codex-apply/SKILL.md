---
name: codex-apply
description: Execute one approved OpenSpec feature through implementation, verification, archive, issue closure, and required persistence. Use for a named change, a bead that resolves to one change, or resuming an interrupted single-feature implementation.
---

# Codex Apply

Treat `apply` as one feature's full execution and finalization contract. Do not hand
archive, spec synchronization, issue closure, or required persistence to a separate
top-level workflow.

## Native task execution

Use Codex-native delegation only for tasks whose dependencies, write paths, mutable resources, and
validation resources are genuinely independent. Give each delegate bounded scope and current
artifact context. The parent retains claims, cross-repository authorization, task and Beads state,
integration, archive, persistence, and finalization. Execute inline whenever isolation is unclear or
delegation is unavailable; do not create a custom clone/import dispatcher.

## Resolve and claim work

1. Resolve one concrete change from the supplied name or bead. If multiple candidates
   remain, stop and ask the user with a recommended choice.
2. Read repository instructions, OpenSpec status/instructions, every required context
   file, task artifact, proposal, design, and relevant main specification.
3. Claim or lock work using the repository's supported mechanism. Treat an active
   conflicting claim as a blocker, not an invitation to race another session.

## Preflight before editing

1. Run `openspec validate <change> --strict --no-interactive` before the first edit.
2. Re-check dependencies, touched paths, base drift, preconditions, runtime tools,
   workspace boundaries, integrations, and configured validation commands.
3. Classify resolved edit roots as proposal-local, authorizable external, or
   independently blocked. For external scope, follow
   [the cross-repository authorization protocol](references/cross-repo-authorization.md)
   and persist its repository-specific turn-boundary gate before the first external
   edit.
4. Resolve other required pre-execution user gates at a turn boundary and persist the
   answer in the feature's durable decision record.
5. Revalidation classifies repository changes as unchanged, reconcilable drift,
   conflict, or authority expansion. Drift updates evidence without revoking the same
   executing owner's grant; conflicts pause execution; expansions require only a
   supplemental decision for the added authority.
6. Stop before edits for invalid artifacts, missing prerequisites, unresolved
   repository identity, denied required scope, active claim conflicts, governing-policy
   conflicts, or another policy decision that cannot be made locally. External scope
   alone is not an unsafe-workspace blocker when the protocol can authorize it.

<!-- codex-protected-mutation-boundary:v1 -->

## Execute and verify tasks

1. Work tasks in their declared dependency and phase order. Preserve required TDD,
   migration, review, security, and operational checks for each task.
2. Update task and beads state only after fresh evidence proves the task complete.
3. Delegate only isolated work with no overlapping paths, mutable resources, or hidden
   dependency. A delegate may implement and validate its bounded task; the orchestrator
   retains finalization ownership.
4. On failure, collect the relevant logs, group failures by likely root cause, investigate
   the lead, and remediate when authorized. Persist a real remaining blocker rather than
   marking incomplete work done.
5. Track every authorized external repository from its immutable baseline. All
   authorized external edits must follow the protocol's `run_owned_paths` ownership
   rules, pass repository gates, and end in a verified complete local commit containing
   all and only the run-owned paths. Apply the runtime `commit + push` or `local commit
   only` verdict and persist a separate repository outcome. An immutable baseline is
   attribution evidence, not an authorization-expiry trigger; preserve the active
   owner-bound grant through reconciliation and recovery.

## Finalize the feature

1. Run all required targeted and regression validation, plus project type/build/security
   and deployment checks where applicable.
2. Resolve valid post-execution user gates and persist the result.
3. Revalidate every active owner-bound cross-repository grant and outcome. Reconcile
   attributable drift without revocation, pause on unresolved conflicts, and obtain
   supplemental authority only for added scope, another owner, or a new remote
   mutation. Incomplete external validation, commit closure, or repository-required
   persistence blocks archive and issue closure; resume from durable per-repository
   outcomes and append an explicit completion, abandonment, or revocation terminal
   event when the grant actually ends.
4. Run strict validation again, synchronize required delta specs, and archive only after
   all completion conditions pass.
5. Close feature/task beads as required, then complete the repository's required session
   closeout and remote persistence.
6. Report implementation scope, every fresh validation result, archive location, issue
   state, persistence result, and any follow-up bead created for remaining work.

Never report success merely because code was written or a subset of tests passed.

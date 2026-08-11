## Context

Jcode Command Center persists initiatives, milestones, schedules, permissions, commands, and outcomes. Orca exposes canonical repositories and worktrees plus live Runs, Tasks, Dispatches, workers, terminals, messages, gates, and runtime health. The existing integration is incomplete: generic Orca skills contain policy that belongs to Jcode, obsolete `llmtrim` guidance remains installed, and the app-core adapter currently projects Orca's runtime ID as a project ID.

The approved architecture is a layered projection bridge. Jcode remains the durable authority that decides what work means. Orca remains the runtime authority that proves where and how work is executing. Command Center composes both through typed, idempotent commands and authorization-scoped runtime observations.

Constraints:

- The canonical Jcode source repository remains on `dev`; this change does not introduce a feature branch.
- The user explicitly approved this initiative as cross-repository work spanning `/home/nyaptor/dev/jcode/source/jcode` and the canonical skill repository `/home/nyaptor/dev/agents`. Each repository retains independent ownership, verification, staging, and commits; the OpenSpec initiative correlates their evidence.
- Generic Orca skills must remain version-matched and reusable outside Jcode.
- Unsupported Orca capabilities must fail closed. The implementation must not invent CLI calls.
- Runtime evidence can inform durable state but cannot directly become durable authority.
- Skills must use progressive disclosure and remain testable through realistic routing and policy prompts.
- Final authoring verification depends on `/home/nyaptor/dev/codex/scripts/verify-codex-feature-artifacts.sh`; if it is unavailable, readiness is blocked rather than silently replaced.

## Goals / Non-Goals

**Goals:**

- Give Command Center one policy skill that selects the correct Orca orchestration pattern.
- Preserve canonical Jcode and Orca identifiers across launch, observation, retry, cancellation, approval, settlement, and cleanup.
- Separate full ownership handoff from supervised orchestration.
- Correlate scheduled work with the initiative, Jcode run, Orca run, task, dispatch, worktree, terminal, and idempotency envelope.
- Keep generic `orca-cli` and `orchestration` skills free of Jcode-specific authority rules.
- Remove all remaining `llmtrim` guidance from the relevant installed skill surface.
- Provide representative evaluations and deterministic checks for accepted behavior and likely failure modes.

**Non-Goals:**

- Making Orca the system of record for Jcode initiatives or schedules.
- Replacing Orca's version-matched CLI and orchestration guides.
- Adding unsupported start, retry, or cancellation CLI verbs to the Rust adapter.
- Allowing the browser or desktop UI to settle durable outcomes independently.
- Publicly exposing Command Center or ntfy infrastructure.

## Decisions

### 1. Use a three-skill architecture

- `orca-cli` owns generic runtime mechanics: project and worktree discovery, terminals, comments, browser control, automation, mobile emulation, and full handoffs.
- `orchestration` owns generic supervised coordination: Run/Task/Dispatch, messaging, ask/reply, decision gates, worker retention and release, and recovery.
- `jcode-command-center-orchestration` owns Jcode policy: pattern selection, authority, identifier mapping, scheduling correlation, lifecycle projection, safe mutation boundaries, degraded states, and acceptance evidence.

**Why:** This preserves version-matched Orca mechanics while keeping Jcode-specific policy discoverable and testable in one place.

**Rejected alternatives:**

- Patch only generic Orca skills. Rejected because Jcode policy would leak into reusable guidance.
- Make Command Center an Orca-native control surface. Rejected because it would invert approved durable authority.

### 2. Select one explicit ownership pattern per action

The policy skill SHALL classify every operation as exactly one of:

1. Full handoff for true ownership transfer.
2. Supervised Run/Task/Dispatch when Jcode must monitor dependencies, gates, retries, or outcomes.
3. Direct terminal action for narrow operator-driven work without a durable DAG.
4. Observation-only projection when no mutation is authorized.
5. Decision gate when an authorized approval must precede execution.

The skill must not silently downgrade one pattern to another.

### 3. Preserve separate identity domains

The projection envelope carries distinct fields for:

- Jcode initiative and run IDs.
- Orca canonical repository or project ID.
- Orca Run ID.
- Task and Dispatch IDs.
- Worktree and terminal handles.
- Correlation and idempotency IDs.

Orca runtime IDs are runtime-health identifiers only and must never populate canonical project fields. Canonical project identity must come from a repository or project lookup.

### 4. Project ordered evidence, then settle durable state

Jcode issues a typed command and records correlation before Orca mutation. Orca observations are normalized into ordered evidence. Jcode settles durable state only after a verified receipt satisfies the command's preconditions and expected terminal state.

Unknown events remain visible but cannot mutate durable state. Sequence gaps trigger replay within the same authorization and initiative scope. Orca outages produce degraded or unavailable states without fabricated completion.

Replay cursors are scoped by authenticated principal, initiative, and Orca run. A cursor becomes invalid when authorization changes or retained evidence expires; the client must obtain a fresh authorized snapshot instead of replaying across that boundary.

### 5. Fail closed at mutation boundaries

Each mutating capability declares its owner, preconditions, idempotency behavior, expected receipt, and unavailable-Orca result. If the installed Orca interface does not expose a verified operation, the adapter returns `UnsupportedCapability`. Skills must never synthesize undocumented command shapes.

The idempotency envelope is durably recorded before dispatch. Recovery after a process crash reconciles that envelope against Orca observations before issuing another mutation. Partial launch or cleanup failures remain visible as recovery-required evidence and never imply that resources were released.

Capability availability is discovered from the selected version-matched Orca runtime and projected to Command Center. The UI and policy skill may offer only capabilities that the adapter has verified for that runtime.

### 6. Treat scheduling as durable intent, not an alternate executor

A schedule records when Jcode intends an initiative action to become eligible. When triggered, the same policy skill selects the Orca pattern and creates the same correlation envelope used by interactive commands. Retries retain the original durable intent while creating a new dispatch attempt with explicit causality.

### 7. Evaluate policy behavior against realistic prompts

The new skill ships eval prompts for handoff, supervised DAG work, observation, approval, scheduled retry, identity ambiguity, Orca unavailability, and unsupported mutation. Comparisons against the generic-skill baseline must demonstrate that the focused skill preserves authority and identifiers without degrading generic Orca mechanics.

## Risks / Trade-offs

- **Policy drift from Orca versions** → Keep mechanics in version-matched generic skills and make the Jcode skill consume their discovered capabilities rather than copying command syntax.
- **Duplicate or conflicting authority** → Encode the authority table and require verified receipts before durable transitions.
- **Identifier confusion** → Use explicit typed fields and regression coverage that rejects runtime-ID substitution.
- **Skill over-triggering** → Limit the new skill description to Jcode Command Center, initiatives, schedules, and Orca-backed lifecycle work; keep generic Orca requests routed to existing skills.
- **Skill under-triggering** → Include concrete trigger phrases for Command Center launch, retry, cancel, handoff, scheduled work, initiative execution, and Orca projection.
- **Evaluation cost and variance** → Use a small deterministic core plus representative paired skill/baseline prompts.

## Migration Plan

1. Persist this initiative and linked OpenSpec change.
2. Remove obsolete `llmtrim` text from `orchestration` without changing its generic contract.
3. Clarify the generic `orca-cli` and `orchestration` routing boundary.
4. Add `jcode-command-center-orchestration` with progressive-disclosure references for pattern selection, authority, identifiers, lifecycle projection, and acceptance.
5. Correct app-core canonical project projection without adding unsupported mutations.
6. Add skill evals and deterministic policy checks.
7. Run OpenSpec, skill, Rust, and representative acceptance verification.
8. Roll back skill routing by restoring the prior generic skill files and removing the focused skill. Do not restore the known runtime-ID-as-project-ID defect; if canonical lookup cannot be retained, leave the association unresolved and fail closed. Durable initiative history remains as an audit record.

## Open Questions

None. Pattern selection, lifecycle projection, authority, skill contracts, and acceptance coverage were approved by the user on 2026-08-11.

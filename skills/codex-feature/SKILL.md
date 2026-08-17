---
name: codex-feature
description: Create or update a refined, final-state-verified OpenSpec feature. Use when a behavior change, multi-step task, interface change, architecture decision, or proposal-lane request needs discovery, requirements refinement, artifact verification, task planning, and authoring closeout.
---

# Codex Feature

Treat `feature` as the proposal-authoring completion surface. Complete all apply-required
OpenSpec artifacts and required authoring closeout before reporting the feature ready.

## Refine before authoring

Build a decision-complete model of the requested outcomes, scope, assumptions, affected surfaces,
dependencies, material edge cases, tasks, done conditions, verification recipes, and expected
results. Resolve discoverable facts locally, record safe reversible defaults and rejected
alternatives, and ask one focused user-only judgment at a turn boundary when necessary. Use
Codex-native read-only reviewers for ambiguity, surface, or executability checks when helpful; the
parent verifies and integrates their findings. Native review is an aid, not a separate clearance
runtime, and inline refinement remains valid.

## Discover and refine

1. Read repository guidance, existing OpenSpec material, relevant code, archived
   attempts, plans, and active work. When present, consume the structured exploration handoff
   and freshness-check its evidence, confirmed paths, and repository revisions;
   persist durable decisions rather than repeating settled research.
2. For a direct feature invocation without an exploration handoff, perform equivalent
   intake and gather or resolve every required handoff field before artifact authoring.
3. In a beads repository, run `bd prime`; find related epic/feature/task beads and
   identify active claims, dependencies, priorities, and conflicts. Report a clean
   degradation when beads are unavailable.
4. Inspect capabilities and runtime prerequisites before authoring work that depends on
   them. Either resolve a missing prerequisite, create an explicit prerequisite feature,
   or stop for a real user decision.
5. Classify every material uncertainty in scope, behavior, compatibility, security,
   data, integration, UI/UX, testing, operations, ownership, dependencies, touched
   paths, and completion criteria as a discoverable fact, safe workflow default,
   user-only judgment, or later-evidence-dependent human action.
6. Give each class its required disposition before authoring:
   - For a discoverable fact, investigate it and cite or record the evidence; do not ask
     the user for locally discoverable information.
   - For a safe workflow default, choose it and record the chosen and rejected
     alternatives as `decided-by: default`, exposed for correction.
   - For a user-only judgment, ask one focused question at a turn boundary, record the
     answer as `decided-by: user` before authoring, and do not defer it to apply.
   - For a later-evidence-dependent human action, place the acceptance decision in a
     terminal post gate or make dependent work a separate prerequisite or follow-on
     feature.
7. Inventory every discovered relevant surface and consumer before selecting final
   scope: callers, routes, components, schemas, integrations, operational surfaces, and
   workflows. Then declare the in-scope subset and explicit exclusions.
8. For every requirement, evaluate other consumers, excluded-path behavior, free-form
   "Other" or off-menu answers, and interacting state combinations.
   Map every material case to its own requirement scenario or an explicit exclusion with defined behavior, and give it an exact verification recipe.

Do not begin artifact authoring until every critical uncertainty has a disposition and
the surface and material-case model is complete. Persist durable answers in the proposal
or design; do not leave decisions in chat-only state.

<!-- codex-protected-mutation-boundary:v1 -->

## Author the feature

1. Create or update the change through `openspec` and always read the CLI's resolved artifact
   instructions before writing an artifact. Persist material decisions and rejected alternatives
   in the proposal or design rather than in a second execution ledger.
2. Build every artifact required for apply, in dependency order. Include requirements
   with scenarios, design decisions and alternatives, tasks grouped for execution,
   preconditions, tests, done means, dependencies, touched paths, and valid user gates.
3. Keep tasks independently executable and explicit about validation. Do not hide
   deferred, speculative, or unowned work in prose.
4. Detect conflicts with active changes and existing requirements. Add a real dependency
   or revise scope; do not silently schedule overlapping work.

## Verify the feature

Load and follow the Codex-owned gate catalog in
[references/verification.md](references/verification.md). Keep deterministic helper results
separate from semantic judgment, and bind every evidence row to the same artifact digest.

1. Run `bash scripts/verify-codex-feature-artifacts.sh --root "$PWD" --change <slug> --phase authored`. Stop on any deterministic failure.
2. Complete the initial semantic evidence matrix: traceability and consistency, edge-cases, executability and freshness, and a completed cold-review of the authored artifacts.
3. After both initial passes succeed, perform required issue linkage or record the objective reason it is inapplicable. Every artifact mutation invalidates every verification layer and all prior evidence, including repairs and issue-linkage writes.
4. Run `bash scripts/verify-codex-feature-artifacts.sh --root "$PWD" --change <slug> --phase final`, then rerun every semantic matrix row against that final digest. This is a complete rerun, not an affected-layer subset.
   Complete a separate final cold review against those unchanged bytes.
5. A timeout, skipped check, partial result, stale result, or nonzero exit blocks readiness and fails closed unless an equivalent current check completes successfully.
6. Recheck active claims and repository baselines, including touched paths, base revisions, and dirty baselines, immediately before handoff.
7. Persist only the reviewed bytes with path-scoped staging and persistence. Resolve the
   containing commit, then run the helper again with `--phase final --containing-commit
   <commit>` and require its exact `PASS containing-commit` row for the unchanged reviewed
   digest before reporting ready. Verify commit containment binds the artifact digest before
   ready status; any later artifact change revokes readiness.

The persistence sequence is normative: bind the final semantic evidence matrix to the
current artifact digest and freeze those bytes; stage and commit only the exact artifact
path set, never unrelated worktree state; name the resulting containing commit; rerun the
deterministic verifier against that commit and the same digest; and only after `PASS
containing-commit` report ready-for-apply. A missing proof blocks before readiness. Any
artifact mutation after semantic review or commit proof revokes readiness and restarts
the final semantic review, exact path-scoped persistence, and containing-commit proof.

If review exposes a hidden user-only judgment, return to refinement and record the answer.
If any finding changes an authoritative artifact, restart this entire phase from the
authored helper pass. Report the final deterministic and semantic evidence matrix, issue
linkage result, artifact digest, containing commit, and any truthful external blocker.

## Gate and close out authoring

1. Treat the completed final verification phase as the authoring gate; a missing or stale
   evidence row blocks closeout.
2. Complete required proposal-authoring persistence without changing the reviewed
   artifacts. If persistence changes them, restart verification.
3. Report the feature name, artifacts, unresolved external blockers, final verification
   evidence, and the exact `apply` handoff.

Do not implement application code in this skill. A feature is ready only when its artifact
set and required authoring closeout have succeeded.

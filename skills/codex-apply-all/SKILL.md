---
name: codex-apply-all
description: Execute the ready OpenSpec feature queue autonomously with dependency-aware scheduling, conflict isolation, full apply-level verification, and durable-state recovery. Use when multiple approved features are ready or when resuming an interrupted queue run.
---

# Codex Apply All

Treat `apply:all` as the queue-level form of `apply`. Every selected feature must satisfy
the same implementation, verification, archive, issue-closure, and persistence contract.

## Native feature execution

After each selected feature passes its own preflight, use Codex-native delegation only for proven
independent features. Serialize dependency, path, claim, external-repository, mutable-resource, and
validation conflicts. Every delegated feature still completes the full parent `apply` contract;
the parent retains authorization, integration, archive, issue closure, persistence, and queue
reconstruction. Sequential execution remains valid and requires no fallback protocol.

## Derive the queue

1. Read current repository guidance and enumerate active OpenSpec changes from the CLI.
2. For each change, inspect apply readiness, strict-validation eligibility, task state,
   declared dependencies and relative `after:` hints, beads state and priority when
   available, current git/worktree state, and declared touched paths.
3. Construct a deterministic ready set: hard dependencies precede dependents; relative
   hints and priority select among ready work; stable change-name ordering breaks ties.
4. Report excluded work with its exact blocker. Do not infer readiness from conversation
   history, stale summaries, or an earlier scheduling result.

## Preflight the execution set

1. Detect overlapping paths, shared mutable resources, dependency cycles, workspace
   boundaries, capability gaps, ambiguity, and user gates across the candidate set.
2. Resolve each feature's external repository and mutable-resource sets. Include them
   in conflict detection and serialize any overlap, even when proposal-repository paths
   do not overlap.
3. Form safe waves: dependent or conflicting work belongs in later waves; only proven
   independent work may run together.
4. Resolve batch-level decisions before dispatch and persist durable decisions in the
   affected proposal or bead. Keep transient lock state only for coordination.
5. Run each feature's `apply` preflight before its first implementation edit. For every
   external repository, use the shared
   [cross-repository authorization protocol](../codex-apply/references/cross-repo-authorization.md)
   and obtain that feature's current authorization before the first external edit.

<!-- codex-protected-mutation-boundary:v1 -->

## Execute waves

1. Preserve the repository's phase model, including DB/API/UI/E2E sequencing, TDD,
   reviews, type/build/security gates, and deployment monitoring where applicable.
2. Delegate only independent units. Give each delegate explicit scope, current artifact
   context, and validation requirements. Delegates cannot archive, push, or close beads.
3. The orchestrator owns cross-repository authorization, claim ordering, exact
   run-owned external commit closure, authorized push handling, and finalization;
   delegates cannot make those landing decisions.
4. Finalize each feature through the complete `apply` contract. Record only confirmed
   completion in tasks and beads, and do not finalize while any required repository has
   incomplete validation, commit closure, or persistence.
5. After every completed feature, paused feature, wave, or failure, re-read OpenSpec,
   beads, tasks, and git before choosing further work. This makes interruption recovery
   a reconstruction from durable workflow facts.

## Stop and report

Continue through already authorized ready work without ceremonial pauses. Stop only for a
real blocker, policy conflict, destructive decision outside the request, or a user gate
that cannot be resolved locally. Return completed, archived, blocked, failed, deferred,
and newly-ready features with fresh evidence and follow-up beads for real remaining work.

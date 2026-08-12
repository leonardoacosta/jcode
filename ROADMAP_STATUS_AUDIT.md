# Roadmap Audit Snapshot

**Audited:** 2026-08-12 00:50 UTC  
**Primary evidence:** `scratch/command-center-commit-check/` planning docs, OpenSpec queue, archived changes, and git history.  
**Confidence:** medium-high for OpenSpec and handoff status; medium for broad plan documents because several are proposals without machine-readable completion state.

## Executive read

The project is not blocked on ideas. It is blocked on roadmap convergence and delivery focus.

- The original P0-P14 handoff explicitly records **P1, P2, and P3 closed**.
- The current repository has **four active OpenSpec changes** with **29 of 89 checked tasks complete** when counting only explicit checklist items.
- The largest active effort is the SolidStart Command Center vertical slice: **17/45 tasks complete**. This is the clearest bridge from the old roadmap's P6/P12 control-plane goals toward a durable product surface.
- The active queue also contains two mostly-unstarted capability branches: Mac Browser Fleet **0/16** and Rendered Artifact Cards **0/15**.
- The planning corpus is broad and partially stale. It contains active planning notes for client-core splitting, server-service splitting, code quality, compile performance, memory graph, desktop build-out, workflow automation, and provider runtime migration, but no single current source of truth ranks them.
- The latest top-level commit moved the skills tree into the fork and deleted `ROADMAP_HANDOFF.md` plus the prior visual artifact. That makes roadmap continuity worse even though the source project still retains the handoff in history.

## Status by original roadmap phase

| Phase | Assessment | Evidence |
|---|---|---|
| P0 Provider/session foundation | Partial / not closed | Handoff says full session profile remains future work; prompt portion shipped under P1. |
| P1 Prompt compatibility | Closed | Handoff records applied and archived `add-prompt-assembly-contract`. |
| P2 Basic Zentui port | Closed for initial surface | Handoff records footer, composer, message framing, and existing TUI surfaces. |
| P3 Motion/frame scheduler | Closed with explicit follow-up gap | Handoff records `add-frame-clock`; reduced-motion/non-color follow-up deferred. |
| P4 Tools/execution safety | In progress / not evidenced as a single milestone | Active auth-safety work and existing tool surface, but no closed phase gate found. |
| P5 Herdr hooks/telemetry | Partial | Recent docs record liveness caveat and turn-hook probe; no phase closure found. |
| P6 Basic orchestration control plane | In progress | Command-center vertical slice is the strongest active implementation. |
| P7 Claude adapter | Unclear | No current active or closed phase record found in the audited corpus. |
| P8 Cursor Cloud adapter | Unclear / likely future | No current active or closed phase record found. |
| P9 Worktree/workspace manager | Unclear | Planning exists elsewhere, but no current phase gate found. |
| P10 Remote environment manager | Unclear | No current phase closure found. |
| P11 GitHub/ADO integrations | Unclear | Not represented in the four active changes. |
| P12 OpenSpec/Beads workflow integration | In progress | Command-center change explicitly includes durable initiative and workflow surfaces. |
| P13 Advanced orchestration | Not started as a gated milestone | The active command-center work is prerequisite-level, not advanced orchestration completion. |
| P14 Advanced workflow integration | Not started | No active change or closure evidence found. |

## Active queue health

| Change | Task progress | Interpretation |
|---|---:|---|
| `add-solidstart-command-center-vertical-slice` | 17/45, 38% | Strategic critical path. Needs contract generation, hosting lifecycle, security script, query/command services, and end-to-end proof. |
| `harden-config-auth-safety` | 12/13, 92% | Near-term finish. Best candidate for immediate closure if the remaining task has acceptance evidence. |
| `add-mac-browser-fleet` | 0/16, 0% | Large new branch. Should not compete with the command-center critical path without an explicit product decision. |
| `add-rendered-artifact-cards` | 0/15, 0% | Valuable UX branch, but currently disconnected from the primary control-plane completion gate. |

## Main risks

1. **Roadmap fragmentation.** The handoff, broad planning notes, active OpenSpec queue, and recent visual work describe related but different centers of gravity.
2. **Unclosed critical path.** The command-center vertical slice is only 38% checked off and contains the largest number of unfinished contract, lifecycle, security, and service tasks.
3. **Premature breadth.** Browser fleet and artifact cards add surface area before the control plane and workflow evidence model are visibly complete.
4. **Loss of canonical history.** `ROADMAP_HANDOFF.md` was deleted in the latest commit, so the most useful phase-level status is now only recoverable from git history.
5. **Weak machine-readable status.** Most planning docs use prose such as “proposed,” “active planning note,” or inline “done” statements rather than a consistent status schema.

## Recommended next move

Adopt a single near-term objective: **finish and archive the Command Center vertical slice, with auth-safety closure as a prerequisite/parallel fast path.** Treat Mac Browser Fleet and Rendered Artifact Cards as explicitly parked unless they directly unblock that objective.

Then restore a canonical roadmap handoff or status index that contains:

- one row per P0-P14 phase;
- owner, status, evidence commit, and next gate;
- links to active OpenSpec changes;
- explicit `deferred`, `superseded`, and `not-started` states;
- a rule that a phase cannot be called closed without validation evidence.

## Suggested sequence

1. Close `harden-config-auth-safety` and record its final evidence.
2. Finish the command-center contract, lifecycle, security, query/command, and end-to-end verification slices.
3. Reconcile the broad plans against the command-center architecture and mark absorbed or deferred work.
4. Reintroduce the roadmap handoff as a maintained status index.
5. Re-rank Browser Fleet and Artifact Cards after the command-center gate, not before.

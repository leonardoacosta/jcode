---
name: codex-explore
description: Assess an idea, problem, change request, or bead before implementation. Use for discovery, prior-art retrieval, trade-off analysis, lane selection, dependency mapping, and deciding whether to attach to work, create a feature, or take a bounded ad-hoc task.
---

# Codex Explore

Treat `explore` as the shared workflow's intake and decision surface. It assesses and
routes work; it does not create an alternate task ledger or implement product changes.

## Native evidence gathering

When evidence separates into independent domains, use Codex-native read-only agents when that
improves coverage or latency. Give each agent a bounded question and source scope. The parent owns
verification, synthesis, questions, lane selection, and the final recommendation. Inline evidence
gathering is equally valid when delegation is unavailable or unnecessary; never create a custom
dispatcher or alternate report protocol for this workflow.

## Gather evidence

1. Read applicable `AGENTS.md` files and identify the working repository.
2. Inspect current OpenSpec changes, committed specs, archived changes, project decision
   records, plans, and relevant code. Read indexes first; load only matched material.
3. When beads are available, run `bd prime`, inspect the ready frontier, and search for
   related issues, epics, dependencies, and active claims.
4. Verify every path and repository revision cited in the outcome. Distinguish verified
   facts from assumptions and external claims.
5. Surface conflicts with existing requirements, current work, scope locks, or explicit
   exclusions before recommending a path.

## Refine intake before routing

Clarify the destination, user-observable success criteria, material scope boundaries,
assumptions, and expectations. Investigate discoverable facts before asking the user,
and distinguish verified conclusions from assumptions.

For a user-only judgment, ask one focused, highest-impact question at a turn boundary and lead with a recommendation and its evidence.
Record the answer, then resume and repeat the refinement loop until no critical user-only ambiguities remain.
You must not route work to the proposal lane or a feature handoff while critical user-only judgments remain unresolved.
When the destination is known but material decisions remain, route to continued research or a decision map instead of a feature-ready handoff.

## Decide the lane

Choose exactly one default route, while explaining alternatives:

- **Attach** when an active feature or bead already owns the outcome.
- **Feature** when work changes behavior, spans multiple files or steps, changes an
  interface/schema/architecture, or benefits from explicit decomposition.
- **Ad-hoc bead** only when work is bounded, independently verifiable, and does not
  need proposal ceremony.
- **Research or decision map** when the destination is known but material decisions
  remain unresolved. Track those decisions as a beads hierarchy when beads are present.

Use the repository's ceremony threshold. Do not create a markdown TODO list or a second
execution tracker.

<!-- codex-protected-mutation-boundary:v1 -->

## Handle decisions and capture

Ask one focused turn-boundary question only when an answer cannot be discovered locally.
Lead with a recommendation and explain its evidence. If the user explicitly asks to
capture a decision, write it to its durable home: proposal/design artifacts for feature
decisions, or beads for tracker state. Do not auto-capture merely because discovery
found something useful.

## Output contract

End with an ordered execution queue, even when there is one item:

| # | Item | Evidence | Why this position |
| --- | --- | --- | --- |
| 1 | recommended action | verified paths/issues | rationale |

Include the prior art consulted, conflicts or blockers, the selected lane, and the
single default action. For a feature recommendation, preserve enough provenance that
`codex-feature` can continue without rediscovering the same material.

For a proposal-lane recommendation, pass a structured handoff containing:

- the clarified destination, success criteria, prior art, findings, and trade-offs;
- selected and rejected alternatives, in-scope and out-of-scope boundaries, resolved user decisions, and recorded defaults;
- verified assumptions and preconditions, the surface inventory, and material edge cases;
- confirmed paths, repository revision stamps, dependencies and conflicts, and blockers;
- done means and the single default feature action.

The handoff is session context, not a second durable ledger. Persist durable decisions
in the proposal or design while retaining the ordered execution queue above.

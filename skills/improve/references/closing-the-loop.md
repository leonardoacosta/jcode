# Canonical Handoffs and Review

The advisor identifies and specifies work. The repository's canonical workflow owns creation,
implementation, review state, and completion. Do not add parallel lifecycle commands or ledgers.

## Attach to existing work

Prefer an existing owner when an active OpenSpec change or Beads issue already describes the same
outcome. Provide:

- the existing identifier;
- new evidence and how it changes scope or priority;
- dependencies or conflicts discovered by the audit;
- the verification gate affected by the finding.

Do not create a duplicate merely because the existing item uses different wording.

## Hand off feature-sized work

Send feature-sized work to the harness binding for `feature`. The handoff must include:

- a proposed kebab-case slug and concise outcome;
- repository instructions, prior art, and current owner search;
- the complete suggestion contract and base commit;
- explicit in-scope and out-of-scope paths;
- decided choices, unresolved questions, dependencies, and STOP conditions;
- structural and behavioral verification with expected results;
- an exemplar `path:line` for the implementation shape.

The `feature` workflow decides how to author or update OpenSpec artifacts and link Beads. The
advisor does not write a substitute proposal format.

## Hand off ad-hoc work

Use one canonical tracker task only when the change is bounded, low-risk, and requires no design
decision. Its description must carry the full suggestion contract, base commit, paths, tests,
dependencies, and completion condition. Parent or label it according to repository conventions.

If a supposedly ad-hoc item grows into a capability change or needs several coordinated tasks,
stop and reroute it through `feature`.

## Hand off execution

- Use `apply` for one named, ready feature.
- Use `apply:all` for an explicitly ordered dependency-safe queue.

Before the handoff, confirm the selected artifact exists, prerequisites are complete, cited code
has not drifted beyond the contract, and the repository's execution surface is available. Pass
the artifact identifier, scope boundaries, expected gates, and known blockers. The apply workflow
owns implementation; the advisor does not dispatch a private executor or edit the result.

## Review checkpoints

When asked to review a proposal or completed change, stay read-only and check:

1. Every requirement traces to concrete tasks and verification.
2. Scope boundaries and STOP conditions are explicit.
3. Tests prove behavior rather than merely exercising lines.
4. Changed files trace to the selected artifact.
5. Deviations are documented and still serve the intended outcome.
6. Completion includes validation, Beads state where present, normal OpenSpec archival, and the
   repository's required commit or push persistence.

Return `ready`, `revise`, or `blocked` with evidence. Do not repair the implementation during the
review.

## Refresh stale work

For open work, compare cited evidence and the base stamp to current code. Refresh the existing
artifact if the outcome is unchanged. If the work was fixed independently, close or retire it
with the evidence. If the desired outcome changed materially, return to `feature` rather than
silently rewriting an execution contract.

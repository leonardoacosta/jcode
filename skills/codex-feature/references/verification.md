# Codex Feature Verification

Use this catalog after artifact authoring and before persistence or a ready-for-apply
claim. It separates deterministic checks from semantic judgment: the repository-owned
helper proves structural and repository-state gates, while the author completes the
semantic rows against the request, exploration handoff, and final artifacts.

## Evidence matrix

Every row is required for the same artifact digest. Record the command or review method,
the result, concrete evidence, findings and repairs, and the digest reviewed.

| Evidence row | Mechanism | Required evidence |
| --- | --- | --- |
| mechanical | Deterministic helper | Strict validation, parent compatibility, required artifact shape, task syntax, linkage markers, and named gate results. |
| traceability | Semantic review | Request and handoff outcomes map through requirements, scenarios, tasks, and exact verification recipes without contradiction. |
| edge-cases | Semantic review | Each applicable other consumer, excluded path, off-menu answer, and interacting state combination maps to a scenario and recipe or a defined exclusion. |
| executability | Mixed | Dependencies, tools, workspace roots, idempotent preconditions, legal user gates, and runnable recipes are current. |
| freshness | Mixed | Touched paths, base stamps, dirty baselines, active claims, and the reviewed artifact digest remain current. |
| cold-review | Semantic review | A completed reread of the final artifacts finds no omitted outcome, hidden assumption, scope growth, or unprovable completion claim. |

The semantic evidence matrix cannot be satisfied by the helper's exit code. An available
independent reviewer may perform cold review, but a completed local cold review is valid
when no reviewer transport exists.

## Deterministic artifact gates

- Every verification task names an exact verification recipe and its expected result or expected output. A bare “verify X” instruction is not executable evidence.
- Use parser-visible `- touches:` and `- depends on:` declarations for touched paths and dependencies; prose mentions do not sequence or fence work.
- A touched path must exist at the checked revision unless it carries the explicit `(new)` path suffix. The suffix is only for a path the feature will create.
- Recheck each touched repository's base-revision stamp and dirty baseline immediately before final handoff; drift invalidates freshness evidence.
- Use repository-supported active-claim fencing. An overlapping active claim or conflicting touch blocks readiness until ownership or scope is resolved.
- Enforce user-gate legality: answerable judgments are resolved during authoring, while a later human action is only a terminal post gate or a separate feature boundary.
- The helper also rejects missing artifacts, incompatible delta parents, malformed tasks,
  forbidden deferral tokens, consumer-invisible declarations, and missing final linkage
  markers when beads apply.

The helper reports deterministic structure only; it never declares semantic completeness.

## Result semantics

`PASS` means the named gate completed for the reported digest; `FAIL` means a blocking defect was found; `SKIP` means objective evidence proves the gate is inapplicable.
A missing tool, unavailable required check, timeout, partial result, or merely convenient
omission is not an objective `SKIP` and fails closed.

## Ordered passes and invalidation

The `authored` phase is the pre-link pass; only after it and the initial semantic pass
succeed may issue linkage run. The `final` phase is the post-link pass and covers the
artifact bytes that may have been changed by linkage.

Every authoritative artifact mutation triggers full-stack invalidation and restart of all verification layers. This includes a repair, formatter rewrite, issue-linkage write,
or decision recorded after review. The next attempt must rerun both the deterministic
helper and all semantic rows; do not select only the layer that found the defect.

Readiness requires one uninterrupted final pass over one digest. A skipped, timed-out,
partial, inconclusive, stale, or nonzero required result is a blocker unless an equivalent
current check completes successfully.

## Digest and persistence binding

After the final pass, recheck active claims, touched paths, repository base revisions, and
dirty baselines without changing artifacts. Persist only the reviewed bytes. Recompute
the artifact digest from the containing commit and require it to match the reviewed artifact digest; bind the digest, commit, evidence matrix, linkage result, and remaining
external blockers in the final handoff. Any later byte change revokes readiness.

The order is exact: final worktree verification and final semantic review bind one digest;
path-scoped staging and commit persist only that artifact set; the verifier receives the
named containing commit; and readiness remains blocked until it emits `PASS
containing-commit` for the same path set and digest. Do not stage unrelated paths or treat
a commit hash alone as proof. Recompute after persistence, and treat any subsequent
artifact mutation as revocation requiring the complete final review and proof again.

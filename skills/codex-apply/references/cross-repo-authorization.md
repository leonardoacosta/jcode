# Cross-Repository Apply Authorization

Use this protocol whenever an approved feature requires an edit in a Git repository
other than the repository containing its OpenSpec change. External scope is
authorizable, but authorization never overrides an invalid feature, a missing
prerequisite, governing repository instructions, an active claim, or another policy
conflict. This protocol is feature state, not a second task ledger.

## Discover and classify repositories

1. Resolve every proposed implementation path to its canonical Git repository root.
   Classify each root as proposal-local, authorizable external, or independently
   blocked. An unresolved repository identity is independently blocked.
2. For every external repository, read its complete applicable instruction chain and
   discover its supported claim or lock, required validation and closeout commands,
   current branch, `HEAD`, upstream, normalized primary remote, push remote, and dirty
   worktree state.
3. Capture the complete dirty baseline before edits. Its deterministic fingerprint
   covers porcelain-v2 status records and the content and mode digests of dirty or
   untracked entries intersecting proposed paths. Identify every proposed path already
   present in that baseline as an overlap.
4. Resolve the full external-repository roster before the first external edit. Acquire
   supported claims or locks in stable canonical-repository-root order after
   authorization. A live conflicting claim is a blocker and cannot be authorized away.

## Present the turn-boundary gate

Present one row for every authorizable external repository in a single user gate. Do
not edit any external file until all required rows have a persisted verdict.

| Field | Gate value |
| --- | --- |
| Repository | stable identity and canonical root |
| Revision | branch, base SHA, upstream, and push remote |
| Scope | proposed paths and any exact dirty overlaps |
| Governance | instruction chain, claim/lock, executing owner, and required gates |
| Baseline | dirty summary and deterministic fingerprint |
| Verdict | `commit + push`, `local commit only`, or `deny` |

`deny` grants no edit authority. When a denied repository is required by the approved
feature, keep the feature active and make no edit in that repository. `local commit
only` and `commit + push` both authorize the listed paths and require a complete local
commit; only `commit + push` authorizes remote mutation.

A dirty overlap is excluded and blocked by default. The user must give separate,
explicit adoption for the exact overlap before its baseline diff becomes agent-owned.
Never infer overlap adoption from permission to edit the repository or a neighboring
path.

## Persist authorization

Append one JSON object per repository to `<changeRoot>/decisions.jsonl`. Use this
authorization schema (additional audit fields are allowed):

```json
{
  "kind": "cross_repo_authorization",
  "repository_id": "normalized remote or stable local identity",
  "repository_root": "/canonical/repository/root",
  "branch": "current branch",
  "base_sha": "HEAD before edits",
  "upstream": "branch upstream or null",
  "push_remote": "normalized push destination or null",
  "authorized_paths": ["repo/relative/path"],
  "dirty_baseline": {
    "fingerprint": "deterministic digest",
    "entries": ["porcelain-v2 records with content/mode digests"]
  },
  "adopted_overlaps": ["exact explicitly adopted path or empty"],
  "governing_instructions": ["ordered instruction sources"],
  "executing_owner": "agent and session identity",
  "required_gates": ["repository validation and closeout commands"],
  "landing_mode": "commit + push | local commit only | deny",
  "grantor": "user identity",
  "authorized_at": "timestamp"
}
```

Chat history alone is not authorization. The durable record must exist before the
first external edit.

The authorization is an owner-bound execution grant. It MUST remain active for its exact `executing_owner`, repository, authorized paths, and landing mode until a terminal lifecycle event closes it. Another agent cannot inherit or use the grant; a different executing owner needs a separate authorization. Repository drift and an execution conflict are not terminal events.

Append lifecycle changes to the same `decisions.jsonl` instead of rewriting prior
records:

```json
{
  "kind": "cross_repo_authorization_event",
  "repository_id": "same stable identity",
  "executing_owner": "same owner or the owner of a separate grant",
  "event": "activated | reconciled | conflict_detected | conflict_resolved | supplemental_grant | completed | abandoned | revoked",
  "authorization_changes": {
    "authorized_paths": ["only paths added by a supplemental grant"],
    "landing_mode": "newly authorized remote mutation or null"
  },
  "evidence": "changed facts, attribution, conflict, or terminal outcome",
  "recorded_at": "timestamp"
}
```

`completed`, `abandoned`, and `revoked` are the only terminal events. `completed` is
recorded only when that executing owner's required repository work and persistence are
complete. `abandoned` requires the owner to stop the execution explicitly. `revoked`
requires an explicit user decision. An activation, reconciliation, conflict, or
`supplemental_grant` event leaves the grant active.

## Revalidate and acquire ownership

Immediately before the first edit, and again before commit or push, recompute and
compare repository identity/root, branch, `base_sha`, upstream, push remote,
`authorized_paths`, governing instructions, required gates, and dirty-baseline
fingerprint. Classify the result without changing authorization lifetime:

- **unchanged** — all persisted facts still match; continue;
- **reconcilable drift** — the repository and executing owner still match, the work
  remains within authorized authority, and changed branch, base, dirty, instruction,
  gate, or path contents can be attributed safely; append a `reconciled` event with the
  new evidence and continue under the same grant;
- **conflict** — a live claim, prohibiting instruction, ambiguous ownership, unsafe
  branch state, or other incompatibility pauses execution without revoking or expiring
  the authorization; append conflict events when the conflict is detected and resolved;
- **authority expansion** — another repository, a different executing owner, a path
  outside authorized scope, or a remote mutation not covered by the landing mode needs
  a separate supplemental decision before that authority is used.

For authority expansion, present a gate containing only the added authority and append
a `supplemental_grant` event when approved. Existing grants remain active for their
existing scope regardless of the supplemental verdict. Never silently widen a grant,
and never treat a supplemental decision as transfer of the original grant.

After the first successful comparison, acquire claims or locks in stable root order.
When no repository mechanism exists, the persisted `executing_owner` is the ownership
attribution and a last-moment `HEAD` and baseline check supplies conflict detection.

Define `run_owned_paths` as exactly:

- post-baseline path changes created by this authorized execution; plus
- baseline paths whose exact overlaps the user explicitly adopted.

Pre-existing changes outside `run_owned_paths` remain user-owned and must remain
byte-for-byte and mode-for-mode unchanged. If an edit, formatter, generator, or gate
changes an undeclared path, classify it as authority expansion and request supplemental
scope naming that path before further use of that path or landing. The original
authorization remains active for its existing scope. Do not discard, absorb, or commit
the additional path implicitly.

## Validate and land each repository

For each authorized external repository:

1. Run its required gates before landing and capture the validation result. Failed
   validation means no commit or push in that repository.
2. Revalidate authorization facts and ownership, then stage only `run_owned_paths`.
   Never use a whole-worktree add when unrelated baseline changes exist.
3. Create a local commit and verify that the commit path set exactly equals `run_owned_paths`.
   Also verify that unrelated baseline entries are unchanged and no
   run-owned staged, modified, untracked, or other residue remains outside the commit.
4. For `commit + push`, run the required remote persistence and record its result. For
   `local commit only`, stop after the verified local commit. If repository policy
   requires a push, that remaining action blocks feature archive and closure until a
   supplemental grant and successful persistence.

Append a separate outcome object for each repository to the same `decisions.jsonl`:

```json
{
  "kind": "cross_repo_outcome",
  "repository_id": "same stable identity",
  "executing_owner": "same authorized owner",
  "run_owned_paths": ["all and only committed paths"],
  "validation_result": "passed | failed",
  "commit_sha": "verified local commit SHA or null",
  "landing_mode": "authorized verdict",
  "push_result": "succeeded | failed | not-authorized | not-attempted",
  "persistence_state": "complete | remaining",
  "remaining_action": "none or exact recovery action",
  "recorded_at": "timestamp"
}
```

The feature may archive, close beads, and report success only after every required
repository has passed validation, has a verified complete local commit, and has met its
repository-required persistence contract. After recording that repository outcome,
append a `completed` authorization event for the same executing owner. A nonterminal
outcome leaves its grant active for recovery.

## Failure recovery and resume

Treat multi-repository landing as per-repository durable progress, not an atomic Git
transaction. If a later repository fails, retain every earlier repository outcome,
keep the feature active, and do not automatically reset, rewrite, force-push, or revert
an already landed repository.

On resume, reconstruct state from the per-repository authorization and outcome records.
Fold authorization events in append order for the same repository and executing owner.
A legacy expiry record caused only by repository drift is nonterminal and MUST be
superseded by a `reconciled` event rather than interpreted as user revocation. Revalidate
a recorded commit SHA, branch/remote facts, and persistence state before accepting that
repository as complete. Continue only the unfinished repository after repair,
reconciliation, or supplemental authorization. Before any commit, run-owned edits may
be removed only with an exact reverse patch or equivalent path-scoped restoration to
the captured baseline; broad reset, checkout, and clean operations remain forbidden.
Rollback after a commit or push is separate explicitly authorized follow-up work.

## Worked rows

| Situation | Gate/outcome | Required behavior |
| --- | --- | --- |
| Denied repository | `deny` | Make no edit; keep a required feature active. |
| Local landing | `local commit only` | Validate, commit all `run_owned_paths`, record `commit_sha`; pause finalization if push is required. |
| Remote landing | `commit + push` | Validate, make the exact local commit, push to the authorized remote, and record successful persistence. |
| Reconcilable drift | The base or an authorized path changes without an ownership conflict | Append reconciliation evidence and continue under the same owner grant. |
| Conflict | A live claim owns an authorized path | Pause execution without revoking the grant; resume after coordination resolves the claim. |
| Scope expansion | A generator creates an undeclared path | Keep the original grant active and request supplemental scope before using or landing the path. |
| Different owner | Another agent resumes unfinished work | Do not transfer the grant; obtain a separate owner-specific authorization. |
| Overlapping dirt | Proposed path is already dirty | Block by default; proceed only after separate explicit adoption of that exact overlap. |
| Later validation failure | Repository A committed; repository B fails a gate | Record A's completed outcome, do not commit B, keep the feature active, and resume from the per-repository outcomes. |

# Nomi completion handoff

## Objective

Resume the previously approved Nomi roadmap and external `linear-report` consumer work. Deliver verified integration and truthful OpenSpec closeout, not just more helpers or passing synthetic tests. The worker/session-manager investigation is a separate task. Do not turn this session into that side quest.

This handoff was refreshed with read-only Git/task/hash observations at approximately **2026-11-07 17:27–17:28 UTC**. No tests were rerun while writing it.

## Optimized fresh-session prompt

Copy this into a fresh Jcode session:

> Read `~/.jcode/nomi-completion-handoff.md`. Resume the existing approved Nomi completion work, starting with the uncommitted feed-staging slice in `/home/nyaptor/dev/personal/.agents`, then continue eligible remaining tasks from authoritative OpenSpec state in that repository and `/home/nyaptor/dev/personal/nomi`.
>
> Use the existing designs and `tasks.md` as the source of truth. First recover current repository state and exclusive write ownership. Do not trust old worker status, completion messages, hashes or test results as current evidence. Preserve all unrelated changes. Resolve ordinary implementation choices yourself. Do not ask for routine permission already granted.
>
> Finish one integrated, reviewable unit at a time: understand the actual caller/storage flow, add a regression before fixing defects, implement the smallest correct change, run fresh behavior-relevant gates on stable bytes, obtain independent read-only review, repair actionable findings, rerun affected gates and review, promptly check only independently verified bounded children, and commit only owned changes. Keep parent tasks open until their full acceptance criteria pass. After each unit, select the next eligible task and continue without a ceremonial pause.
>
> Prefer bounded swarm implementation and independent review using explicit `omnipi:gpt-6-luna` when that route is available. Do not switch model/provider routes silently. Establish one writer per overlapping path. A reported stop is not enough after this incident: verify execution quiescence and ownership before replacement. If native control remains unreliable, do useful coordinator-owned work only after excluding other writers. Do not create another uncontrolled worker loop.
>
> The immediate goal is to recover, verify, independently approve and selectively commit the pending seven-path feed-stage candidate. It must remain staging-only: no successful cursor advancement, work-completion claim or provider ACK. Then implement actual durable affected-work completion, transactional invalidation/journal effects, guarded cursor acknowledgement, rebootstrap and remaining report-state behavior as required by the existing proposal. Avoid unused abstractions and checkbox-driven micro-slices that do not advance an actual entrypoint.
>
> Continue all authorized local work until complete or genuinely blocked. For live DB/provider/schema/permission/deployment/cutover/rollback/publication effects, first bind exact targets and existing authorization. Do not invent targets, policies or acceptance evidence. Consolidate any remaining material decisions into one brief only when useful unblocked work is exhausted. Keep acceptance, review, commits, archive and remote persistence as distinct evidence-backed states. Do not archive or claim whole-feature completion while mandatory gates remain open.

## Suggested skills

Load through the native Skill tool as needed:

- `recovery`: recover authoritative state and interrupted work.
- `preflight-check`: required just-in-time execution/handoff eligibility and full terminal-path assessment.
- `apply` / `openspec-apply-change`: continue the existing proposal, not a new parallel plan.
- `jcode-orchestration`: bounded implementation/review workers and ownership.
- `test-driven-development`, `systematic-debugging`, `verification-before-completion`: regression, repair and fresh verification discipline.
- `review` or `adversarial-code-review`: independent final-diff review.
- `linear-openspec` / `linear-management`: only for verified Personal/Priceless issue linkage. Never invent an issue or cross-link Brown work.
- `close`: terminal reconciliation/archive only when actual prerequisites pass.

## Repositories and verified checkpoints

| Repository | Freshly observed HEAD | Relevant state |
| --- | --- | --- |
| `/home/nyaptor/dev/personal/.agents` | `7829a30` | Dirty consumer implementation plus substantial unrelated Brown/report/journal/resource/graph work and unrelated untracked context-status work |
| `/home/nyaptor/dev/personal/nomi` | `e732f17` | Unrelated dirty `.gitignore`, `README.md`, `openspec/ROADMAP.md`; untracked `.ignore` and two documentation files |

The consumer repository is **not** `~/.agents`, which is an installed consumer copy. Do not edit the installed copy instead of the canonical source.

Read `~/.agents/AGENTS.md`, the verified Personal workspace policy and each target repository/nested `AGENTS.md`. Keep the primary checkout on its current branch. Do not switch branches/worktrees or publish to bypass policy. No worktree creation is included in this handoff.

### Last independently approved commits

- Consumer `c13148d`: public explicit-client staged per-work Nomi commit gate. Child **3.4.5** checked, committed-range review approved.
- Consumer `7829a30`: conservative saved-report projection suppresses Nomi-bound mixed-work content after known local observation drift, including matching Brown journal overrides in direct Markdown/HTML rendering. Child **4.4.1** checked, committed-range review approved.
- Earlier bounded producer/consumer slices are already recorded in Git and authoritative research/tasks. Read those artifacts instead of recreating their history here.

These commits do not complete parent 3.4/4.1/4.2/4.4, real acceptance, controlled cutover or archive.

## Authoritative artifacts

Consumer:

- `/home/nyaptor/dev/personal/.agents/openspec/changes/consume-nomi-evidence/tasks.md`
- Same change's `design.md`, `proposal.md`, `specs/linear-report/spec.md`, and `research.md`
- `skills/linear-report/scripts/{nomi,process,storage,report,journal}.py`
- `skills/linear-report/assets/{storage,report,journal}.sql`
- Corresponding existing package tests and references

Producer:

- `/home/nyaptor/dev/personal/nomi/openspec/ROADMAP.md`
- Its current OpenSpec changes/tasks and dependency evidence, discovered from repository state
- macOS/Windows release proposals must be inspected as current artifacts. Last observed versions were explicit placeholders, not executable implementation plans.

Use `tasks.md` as execution authority. Session todos are progress aids, not a second OpenSpec ledger.

## Immediate pending unit: feed staging and affected-work discovery

### Owned candidate paths

Exactly these seven paths, relative to the consumer repository:

1. `skills/linear-report/scripts/nomi.py`
2. `skills/linear-report/tests/test_nomi.py`
3. `skills/linear-report/scripts/storage.py`
4. `skills/linear-report/tests/test_storage.py`
5. `skills/linear-report/assets/storage.sql`
6. `skills/linear-report/references/processing.md`
7. `openspec/changes/consume-nomi-evidence/research.md`

No new child 4.1.5 is currently checked or committed. Inspect numbering before adding a bounded child after approval. Do not check parent 4.1.

### Intended bounded contract

- Actual explicit `Client.changes(..., observe_invalidations=True, stage_progress=True, expected_stage_generation=...)` reaches the actual storage transaction.
- Default reads remain read-only. Observation-only behavior remains compatible.
- Freeze canonical origin and continuation before transport callbacks. Staging is single-world and strictly bound to canonical client/owner/configured scopes.
- Persist metadata-only feed batches in the separate draft `nomi_feed_stage` table. Keep snapshot staging separate.
- Retain finite cutoff/start/resume state, validated event identities and hashes, coverage/gaps and SQL-derived latest affected work, with explicit empty-work resource records.
- No correction body, provider source body or excerpt corpus in the stage. Metadata digests may bind complete validated events without persisting their private text.
- Exact replay returns the originally captured workset/generation. Changed-input stale-generation requests fail. Budget prefixes and zero-page results never mean successful processing.
- No caller-supplied completion list, output-processing marker, successful cursor promotion or provider ACK exists in this slice.
- SQL is **draft-only and unexecuted**. Recorder tests are not PostgreSQL concurrency/durability proof.

### Findings already identified and repaired in the moving candidate

Reverify all of these against current source; none has final independent approval on the latest bytes:

1. **Cutoff validation:** canonical string/null cutoff; frozen finite cutoff consistency; start/resume/event bounds; exact unchanged zero-page continuation; valid-owner fixtures so invalid cases do not fail for unrelated reasons.
2. **Global event identity race:** owner/event-ID advisory locking and global identity checks shared by stage and legacy observation writers, in deterministic order. Cross-resource same-event requests must not both succeed. Metadata mismatch and legacy unknown digest behavior must fail closed where required.
3. **Replay ordering:** stage serialization and row read precede replay decision. Exact replay skips mutable candidate/head discovery, bounds checks and event writes. No stale pre-lock stage snapshot determines replay. Non-replay discovery checks exact candidate work/head versions after locks.
4. **Response integrity:** exact stored response schema, payload/input digests, request-payload equality, configured resource/scope coverage, sorted unique bounded work IDs and response-size checks. Empty `affected_work: []` is not a valid fixture when configured resources require rows.

Existing bounds to inspect: 2,000 events, 20 pages, up to 100 configured resources, 500 unique affected work IDs, 1 KiB work-ID bound and 4 MiB combined stage bound. Do not silently truncate to satisfy limits.

### Freshly observed candidate hashes

These are recovery anchors, not approval tokens. Rehash before use.

| Path shorthand | SHA-256 |
| --- | --- |
| `scripts/nomi.py` | `aed704bfe404224d52bab68e810595e11baab898bbea08b0d1a21a7260265318` |
| `tests/test_nomi.py` | `dc125ef010249e19bcd7e33b21d6b008c349d319d5e0084b1c4620c8ea62f1f2` |
| `scripts/storage.py` | `34748c828821ed30710197a7724255a4b368156dc4f77ced2d786de3314e18c7` |
| `tests/test_storage.py` | `e3c70871275fcf4703b19f5b7787ab019fde43d9a3cdb84cbff7b6fd47acd574` |
| `assets/storage.sql` | `54f41ee3b0ef8f877f1912ef3f351a404f7471dc9a61b933e5525c071c066d31` |
| `references/processing.md` | `dfb0aa64d14d484b74f9f5c5de003958bf03f9cbdf1b804d650e70619c8da056` |
| change `research.md` | `042c2031b4f6471f8a86997997ce7a9ce6bf29d0f9cdfcf0d8dd4d5b12f42c9d` |

Shorthand source/test/assets/reference paths are under `skills/linear-report/`.

### Verification history and its limits

The latest coordinator-focused run in the previous work completed at 01:32:53 UTC: **203 run / 191 passed / 12 skipped**, with all seven hashes unchanged before/after that run. Earlier root/static/native/dependency runs passed, but worker writes repeatedly invalidated freezes. None of those results is fresh acceptance for the new session.

Independent reviewer Seedling paused before approving the latest candidate. An older review or worker completion report cannot substitute for renewed final-byte review.

## Ownership recovery comes first

Relevant old sessions, for bounded read-only discovery only:

- Root `goat`: `session_goat_1793813250757_f2dcdb11727df7d3`
- Original writer `rose`: `session_rose_1794010960955_2b4e71ae94e17ac7`
- Replacement writer `evergreen`: `session_evergreen_1794013629578_d6a18a81be2f00f6`
- Independent reviewer `seedling`: `session_seedling_1794012832442_2d8d6c71fd852fda`
- Unexpected later route: `session:session_rose_1794010960955_2b4e71ae94e17ac7`

The stop tool reported writers stopped, but a later cross-swarm route sent ongoing-work messages and files changed. Exact-ID stop then returned `Unknown swarm session`. Repeated resume/wait calls also returned stale earlier reports. The root cause was not diagnosed. Do not assume any old session is still live after the long pause.

Before writes, inspect current runtime state and establish one owner. If no reliable control observation exists, explain that specific blocker instead of spawning another overlapping writer. A fresh read-only reviewer may be used after a stable candidate is established. Preserve all current edits; do not revert uncertain authorship.

The separate incident/proposal handoff is `~/.jcode/worker-session-management-handoff.md`. Refer to it only for recovery evidence. Do not investigate or implement Jcode worker management in this Nomi session.

## Preservation and safety

- Consumer working tree includes major unrelated Brown journal migration changes, including deleted vendored resource files, report/reference changes and new context-status artifacts. They are not feed-stage ownership.
- Original Brown process changes remained **9 additions / 3 deletions** in `process.py` and **37 additions** in `test_process.py`. Compare actual bytes with baseline, not counts alone.
- Do not blanket stage, reset, clean, stash, restore deleted files or prune worktrees.
- Scratch evidence: `~/.jcode/scratch/nomi-feed-completion-baseline/` has original working copies, patches and hashes. `~/.jcode/scratch/nomi-feed-control-check/` has before/after hash evidence. Inspect actual types/contents before relying on replacement baseline artifacts.
- `research.md` historical HEAD prefix must stay exact; append new evidence at the end. Some older narrative attribution is provisional because writers overlapped. Correct by an explicit appended clarification, not rewriting history.
- **Do not invoke Graft CLI at all in the consumer checkout**, including help/map/ask: prior calls unexpectedly refreshed already-dirty `graft/.graph/wiring.json`. Read existing cards only. Do not restore or include that graph in commits. Assess producer navigation separately.
- **Always unset `LINEAR_REPORT_TEST_POSTGRES`**. Opt-in integration setup can drop schema. No live DB or schema actions without exact approved dedicated targets.
- No provider mutations, deployments, permission changes, cutover, rollback, push or archive based solely on this handoff. Reconcile prior scoped authorization with current target and workflow policy.
- Personal/Priceless use verified Linear linkage; Brown uses ADO. Do not invent IDs or cross-link workspaces.

## Concrete next actions

1. **Recover and bind:** read policies, `tasks.md`, design/spec and current diff; establish exclusive ownership; snapshot seven-path bytes and unrelated dirty preservation evidence. Verify producer and consumer HEADs rather than assuming those above.
2. **Finish pending unit:** inspect actual public Client/storage/legacy-writer paths and all four findings; repair remaining defects with regressions. Run fresh gates on stable before/after hashes, then independent review. Iterate until clear or specifically blocked.
3. **Persist bounded result:** promptly check only the accepted child, append exact evidence, inspect full staged diff and commit only owned paths. Request committed-range review including task/research reconciliation. Leave parent acceptance open.
4. **Continue actual integration:** implement per-work durable completion and event/scope effects, guarded successful cursor CAS, rebootstrap and remaining report states according to existing dependencies. Do not treat durable staging as durable processing or acknowledgement.
5. **Complete acceptance and closeout:** bind dedicated live targets; execute authorized acceptance/cutover/rollback requirements; run final gates and independent review; reconcile tracker/task state; commit and archive only after required prerequisites pass. Continue eligible dependent roadmap waves from authoritative state.

## Remaining capability groups

Reference existing tasks rather than copying all acceptance criteria:

- **4.1:** affected-work completion markers, event dedupe, crash/restart/partial-work handling and final cursor CAS only after all effects through cutoff commit.
- **4.2:** reverse-linked derived-head/unresolved-state effects and atomic report/journal delivery intent; idempotent retry and restricted cached content.
- **4.3:** staged rebootstrap on 410/restored-continuity failure; preserve old successful baseline until reconciliation completes.
- **4.4:** full stale/restricted/as-of/access-check/coverage behavior, including outage, partial denial, no prior report and unsupported closure. Existing 4.4.1 is only conservative local projection.
- **5:** real deployed API/provider/report-DB acceptance and mixed retained Linear/local/Nomi evidence, not fixture certification.
- **6:** protected restore, shadow parity, scoped Office cutover and rollback read-back. Preserve retained Linear/ADO/local capture and private history.
- **7:** mandatory fresh gates, independent final review, repairs/re-review, truthful tasks/issues, scoped commits, archive and required persistence/read-back.
- **Other Nomi roadmap waves/platforms:** actual upstream acceptance gates still apply. macOS/Windows placeholders need executable proposal artifacts and real platform/release decisions before implementation.

Do not assume earlier numbered parent tasks are complete because selected children are checked. Inventory all unchecked tasks and frozen dependencies in both repositories.

## Known live-decision blockers

Prior user approval included local dirty-caller extension, acceptance and platform work in principle. It did not supply or authorize invented targets.

Still unbound at last assessment: exact isolated Nomi deployment, report DB/schema/volume, authorized provider accounts/tenants/corpus/principals, retained tracker test targets, retention/backup/expiry/restore policy, source-mutation scope, collector/cutover/rollback ownership, and native platform hardware/runners/OS/signing/distribution choices.

Discover existing non-secret configuration and prior decisions read-only first. Ask only for unresolved consequential decisions. Never use production as a default, seed producer SQL as a substitute for API acceptance, or rewrite acceptance to match mocks.

## Safe verification commands

Run from `/home/nyaptor/dev/personal/.agents` unless specified otherwise:

```bash
env -u LINEAR_REPORT_TEST_POSTGRES \
  PYTHONPATH=skills/linear-report/tests:skills/linear-report \
  /usr/bin/python3 -m unittest test_storage test_process test_nomi test_report

env -u LINEAR_REPORT_TEST_POSTGRES \
  /usr/bin/python3 -m unittest discover -s tests

/usr/bin/python3 scripts/skill_suite.py validate-all --root skills

PATH=/home/nyaptor/.local/share/mise/installs/node/24.21.0/bin:$PATH \
  openspec validate consume-nomi-evidence --strict --no-interactive

/usr/bin/python3 scripts/openspec/validate_dependencies.py \
  --root /home/nyaptor/dev/personal/.agents consume-nomi-evidence
```

Verify current executable paths first. There is no `scripts/openspec/validate.py` in the previously inspected workflow. Run scoped `git diff --check` on owned candidates; distinguish pre-existing unrelated failures from your changes. The earlier full skill suite had two unrelated scope-test failures, so focused/root/static passes do not imply every package test passes.

Producer build/test commands must come from its current policy/config. Do not copy consumer checks into producer acceptance.

## Completion standard

A bounded unit is complete only with current behavior-relevant checks, independent exact-diff review, accurately checked native task state and scoped persistence. A feature is not complete until its full mandatory acceptance and terminal workflow succeed. Report implementation, synthetic verification, live acceptance, review, commit, push/PR, archive, deployment and cutover separately. If blocked, identify the precise missing target/authority/evidence while continuing independent authorized work.

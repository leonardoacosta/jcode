# Design

## Context

The user selected a separate suite-improvement draft in the Jcode directory. Native `context --json` initially returned `no_openspec_root`; `openspec init --tools none --no-animation` created only `openspec/` planning structure, with no harness skills/commands or runtime integration. The Jcode Git identity is `https://github.com/leonardoacosta/jcode.git`; draft branch is `jcode/docs/improve-workflow-suite` from `e3bd7c92b1247105f7cbeab85d3bc8d4e33449f4`. Existing dirty/deleted/untracked files, including root instruction deletions, are unrelated and must remain untouched. Verified checkout-local `.agents/AGENTS.md` was read without recreating deleted legacy instruction files.

This plan's home is `/home/nyaptor/.jcode/openspec/changes/improve-workflow-suite/`. Its implementation source remains `/home/nyaptor/dev/personal/skills/skills/`; no skill is to be authored in `~/.jcode/skills/` or the shared installed store. This is not a Rust/TUI/daemon change and needs no Jcode build/reload. No Brown file is an implementation dependency or evidence location.

Read-only evidence on 2026-10-20 pinned personal skills at `b727a0e8ac8ad5f15954f6180fc364e596652abc`. Current installed/canonical entrypoint bytes matched during exploration, but dirty working bytes must be rechecked before implementation. The baseline already includes:

| Source | Existing behavior | Net delta retained |
| --- | --- | --- |
| `skills/explore/SKILL.md:14-25` | Disproof, evidence brief, concrete adversarial scenarios, bounded decisions | Match claim to observable evidence; do not add another research framework |
| `skills/feature/SKILL.md:31-37` | Native artifacts, success/failure scenarios, scope, dependencies and checks | Small task-executability review, not a new required artifact |
| `skills/apply/SKILL.md:14-23` and `references/recovery.md:5-25` | Scoped tasks, verification/final review, durable recovery, safe repeated effects | Cause-specific response to blocked/concern reports and unproductive retries |
| `skills/apply-all/SKILL.md:12-36` | Selection, dependencies/conflicts, native waves, integration and recovery | Preserve parallelism; selective risk review and current observable progress |
| `skills/apply/evals/evals.json` | 12 prompt/expected-output/assertion fixtures | Extend/reuse format across four verbs; current file is not an agent runner |
| `skills/self-improvement/SKILL.md:243-291` | Pattern-key dedupe and recurrence-based promotion guidance | Recurrence nominates; regression/transfer/counterexample evidence supports retention |

The inspected personal `scripts/` and target packages did not expose a behavioral agent-eval runner. `scripts/skill_suite.py` is metadata/filesystem validation, not a model-evaluation service. That bounded finding is not proof no external runner exists. Implementation must discover any existing supported harness evaluation path before choosing documented manual execution.

## Goals / Non-Goals

**Goals:** improve decision-relevant evidence, executable task handoffs and causal recovery with small source edits; measure before/after behavior; reject overgeneralized lessons; preserve existing native/task/authority/world boundaries.

**Non-Goals:** a new platform, scheduler, task schema, mandatory reviewer count, fixed retry budget, global memory/policy edits, model-routing configuration, OpenSpec readiness/schema migration, universal cross-world governance, public release or consumer installation. Independent semantic admission/freshness belongs to the separate world-ownership/readiness effort and is not implemented or double-counted here.

## Decisions

### 1. Borrow compact lenses, not full candidate workflows

| Lens/source | Apply where | Keep | Reject |
| --- | --- | --- | --- |
| Adversarial router `SKILL.md:8-20`; REA `SKILL.md:288-314` | Explore/Feature and acceptance recipes | Claim/question → safe observable falsifier → proportionate evidence; distinguish static/runtime/shipped proof | Hostile personas, REA setup for source-only tasks, claim that unavailable evidence proves failure |
| Writing Plans `SKILL.md:14-55` | Feature authoring | Task text is independently executable: scope, affected surface, expected outcome and usable check; recovery only when relevant, with no required additional agent | Duplicate plan files, blanket 2–5 minute tasks, fabricated exact commands |
| SDD `SKILL.md:286-302` | Apply and Apply-All controller | Distinguish concerns/context/reasoning/size/plan problems; change cause-relevant inputs or action before retry | Blanket serial implementation, mandatory fresh agent/reviewer per task, fixed escalation round numbers |
| Design Control Loop `SKILL.md:12-24` | Existing execution loop | Progress measured by acceptance evidence, resolved blockers and current task/resource state | New scheduled workflow, PR actuator, score dashboard or checked boxes as sufficient evidence |
| Linear Management `SKILL.md:13,19,23` | Existing handoffs | Scope, accountability, observed evidence, acceptance and next gate | Importing tracker states, new approval records or copying personal governance into neutral rules |
| Improve `SKILL.md:12-19` | Explore recommendation | Explain the highest-value bounded next move and actual delivered/verified artifact | Separate plan directory, unranked broad audit backlog, implementation during research |
| Self Improvement and Skill Judge | Source maintenance | Provenance, knowledge delta, prune redundancy, test generalization | Automatic global prompt expansion, recurrence/score as proof of effectiveness |

These installed sources are research inputs, not mandatory runtime dependencies. Distill principles in original wording; preserve attribution if reusing actual material and inspect license/notice before copying any resource. No new tool/package is needed merely to read a lens.

### 2. Minimal file and ownership map

Proposed edits, not performed by this draft:

```text
personal/skills/
  skills/
    explore/SKILL.md
    explore/evals/evals.json                  # new fixture collection
    feature/SKILL.md
    feature/evals/evals.json                  # new fixture collection
    apply/SKILL.md                            # only compact routing/behavior delta
    apply/references/work-decomposition-and-accountability.md
    apply/references/recovery.md              # append cause-specific control, reuse existing recovery
    apply/evals/evals.json                    # retain existing 12 and reuse/extend narrowly
    apply-all/SKILL.md
    apply-all/evals/evals.json                # new fixture collection
    self-improvement/SKILL.md                 # narrow promotion-evidence condition
    jcode-orchestration/SKILL.md               # existing canonical harness adapter, no references tree observed
  tests/                                     # existing source tests, only affected deterministic coverage
  docs/                                      # one concise eval procedure/result convention if no existing home fits
jcode/openspec/changes/improve-workflow-suite/
  proposal.md, design.md, specs/*/spec.md, tasks.md
```

The revalidated `jcode-orchestration` package currently has only `SKILL.md`, so use that existing entrypoint for its small harness mapping instead of inventing a references tree. Before editing an unspecified docs file, resolve and read an existing suitable canonical path; only add a concise package/source-owned procedure file if no existing home fits. Record that resolved path in implementation evidence. This is an ordinary placement choice, not permission to create a new package or broad docs hierarchy. All execution tasks stay in this change's `tasks.md`; eval cases and results are evidence, not a second task ledger.

### 3. Per-verb behavior and causal controller

| Verb | Observable refinement | Preserved boundary |
| --- | --- | --- |
| Explore | Frame only the claims that affect the recommendation; match each to minimum suitable evidence, keep unknowns honest and rank the next useful action | No proposal/implementation, unlimited interview or mandatory runtime setup |
| Feature | Revalidate supplied evidence locally; inspect each task for a usable acceptance/check result and relevant surfaces; do not invent unknown commands/paths | Native authoring order, resolved material decisions, no implementation from artifact presence |
| Apply | On blocked/concern reports identify cause; supply context, narrow task, seek evidence, escalate reasoning or correct authorized plan; preserve the recorded task base for full diff review | Current user scope, consent, unrelated work, fresh checks and independent final review |
| Apply-All | Apply the same causal response per selected owner while native dependencies/conflicts govern waves; re-evaluate progress/stale owners before next dispatch | No new scheduler/lock/state model; ready nonconflicting work remains parallel |

A report vocabulary may be expressed as done, done-with-concerns, needs-context or blocked, but no new persisted enum or harness status schema is required. Concerns about correctness/scope are addressed before relying on completion; incidental observations need not stop execution. A failing attempt is unproductive when it repeats the relevant action with unchanged evidence/inputs and no progress toward acceptance. Wait/poll operations for a known asynchronous check are not equivalent to retries of a failed implementation; use native wait/progress signals and record why another observation is useful.

Task review becomes more intensive only for an identified risk or uncertainty. Existing final independent review is not optionalized. Handoffs contain task/artifact pointers, current base/diff scope, files/resources, authority and acceptance rather than the entire transcript. Multi-commit task review uses recorded base through head; dirty work needs attributable baseline/current bytes and ownership, not an assumed `HEAD~1` diff.

### 4. Harness-native planning is an adapter boundary

Prior exploration supplied the official OMP plan/prewalk documentation as evidence; this Feature draft did not fetch new external facts. Documented OMP `/plan` creates a read-only session-artifact draft with review/handoff choices. Prewalk is different: a stronger model can create todos and begin edits before a one-time same-session model handoff. Never invoke prewalk for research-only or draft-only authorization. An OMP plan review choice does not authorize external actions beyond the user's scope.

Bundled Jcode `docs/SWARM_ARCHITECTURE.md` documents a coordinator-owned native task graph, per-task assignment/status and checkpoint/heartbeat visibility. Use these as execution telemetry and views of selected work. No equivalent Jcode read-only `/plan` mode was established in the bounded research, so do not assert one. The existing `jcode-orchestration/SKILL.md` is the place for a thin mapping, not a new reference tree, native-plan bridge service or copied engine. Native OpenSpec remains artifact/task authority and world-specific tracker/CI gates remain separate.

### 5. Fixed behavioral evaluation contract

Reuse `evals/evals.json` with `skill_name`, `evals`, and current case fields `id`, `name`, `prompt`, `expected_output`, `files`, `assertions`. Preserve existing ids and checks; add cases only where a needed behavior is not represented. Do not invent new JSON fields as though the current runner supports them. Companion Markdown may supply the procedure and traceability without changing fixture format. Assertions must be gradeable from actual actions/output, not require identical phrasing.

Freeze the current four skills, references actually loaded, selected existing tests and scenario/fixture definitions before editing. Record exact commit plus dirty-byte content manifest, included/excluded paths and provenance. Baseline fixtures are the same new evaluation scenarios used for candidate runs, not only tests that happened to exist before the change. Keep baseline skill content separate from candidate content, use fresh isolated harness sessions, and prevent candidate runs from seeing baseline answers or grading hints.

Run manifest and evidence are local files, not a new service:

- Baseline/candidate source revisions and byte manifests, case IDs/fixture digests and exactly loaded skills/references.
- Harness/version, model/provider/model version where observable, tool availability, relevant system/context prompts, repository fixture state and decoding settings. Record seeds only if supported; do not assume seed control.
- Run order, date, raw responses/tool traces where available, assertion outcomes with evidence, limitations and adjudication reasons. Store non-sensitive curated summaries with the source's existing eval conventions; keep raw/private traces outside tracked files.
- Match every controllable setting except intended skill content. An unavailable model revision, nondeterministic provider or different tool result is a recorded limitation. A material context/tool/fixture difference invalidates the affected pair rather than being silently averaged away.

Use an existing authorized local harness/manual procedure if no runner exists. Fixtures may instruct decision/dispatch behavior with disposable files and no-op collaborators. That is valid synthetic workflow evidence, not proof of real deployment/provider integration. Stronger runtime claims need the corresponding real observable workflow. Do not add a dependency or remote provider solely to automate grading. Run control/candidate under the same established authorization; this draft request authorizes neither.

#### Frozen scenario families

Reuse matching existing Apply cases; the names below are coverage families, not a command to generate sixteen duplicate files or fresh cases unnecessarily.

| Verb | Targeted/regression | Transfer | Negative/simple control | Authority/invariant |
| --- | --- | --- | --- | --- |
| Explore | Runtime claim unsupported by source; label unknown and identify useful evidence | New domain/shipped version needs a different evidence surface | Source-only answer needs no REA/service/interview | Unavailable or consequential observation blocks only dependent conclusion |
| Feature | Task lacks expected observable result/check | Stateful task needs concrete recovery/rollback evidence | Small task remains concise with no extra plan artifact | Material unresolved decision is surfaced, not guessed; planning never starts apply |
| Apply | Missing context vs repeat unchanged failure | Different reasoning/oversized/invalid-plan cause needs a different response | Justified async wait remains allowed; low-risk task needs no extra reviewer | Multi-commit full-range review and current consent/preservation/final-gate checks |
| Apply-All | Blocked owner needs a causal next action | Different selected change exposes a changed dependency/resource risk | Nonconflicting ready work remains parallel without transcript-heavy briefs | Selected scope, no unselected prerequisite execution and no conflicting shared writes |

Keep the original twelve Apply cases as acceptance invariants, including any requirements for authorized persistence/closeout. Where an existing fixture's wording assumes a push or publication without expressing authorization, preserve its intended closeout behavior but supply explicit fixture authorization or a refusal assertion; do not turn a legacy prompt into permission.

#### Comparison and retention decision

Freeze assertion expectations and case selection before viewing candidate outputs. For each case, report baseline and candidate as pass/fail/blocked/inconclusive for every assertion, with observed evidence. No weighted aggregate or skill-judge score substitutes for results. Track unnecessary setup/questions/artifacts/review passes and available step/token cost separately from correctness, using matched simple controls. Predeclared simple controls forbid specified unnecessary actions; expected safety stops are not counted as waste.

Retain a proposed lens only if the candidate passes its critical assertions, introduces no harmful scope/consent/preservation/verification regression and demonstrates an attributable improvement in at least one targeted case where baseline did not meet the same assertion, while its transfer and negative controls pass. If baseline already satisfies all relevant behavior, treat the lens as redundant unless it demonstrably removes unnecessary burden without losing behavior. If results are stochastic or ambiguous, retain all attempts and run a justified matched recheck of the disputed pair, or report inconclusive; do not cherry-pick successful outputs or silently change the rubric. Revert/narrow unproven instruction additions instead of accumulating them.

### 6. Learning promotion and rollout

An observed failure/correction or recurring pattern nominates a bounded candidate. Link it to the actual evidence and to a regression, distinct transfer and negative-control case. If a proposed source change is inspired only by a mined skill, label its scenarios as derived hypotheses, not observed incidents. Do not fabricate a failure history to satisfy promotion. Existing recurrence rules may prioritize investigation but do not alone establish behavioral benefit.

Implementation sequence is fixtures/frozen baseline, minimal candidate edits, source checks, authorized paired evaluation, then retain/narrow/revert with evidence. Update the owning source's lesson-promotion guidance only for this evidence condition. Do not install candidate packages, alter global prompts, add hooks, switch runtime models or publish automatically. Source changes can remain explicitly unproven if agent runs are blocked, but no completion claim may imply measured benefit; evaluation-dependent tasks remain unchecked.

### 7. Acceptance traceability

Task ids refer to `tasks.md`. During implementation record concrete commands/results/evidence beside these coverage rows, not another progress checklist.

| Requirement | Scenario families | Tasks | Required evidence |
| --- | --- | --- | --- |
| Evidence/control: proportionate evidence | Runtime/source mismatch, small source question, unavailable observation | 2.1, 3.1, 5.1 | Paired actual responses/actions select correct evidence without unsafe/setup side effects |
| Evidence/control: task completeness | Missing expected result, concise bounded task | 2.2, 3.2, 5.1 | Native task artifacts/checks become executable without extra ledger/boilerplate |
| Evidence/control: causal response | Missing context, unchanged failure, justified wait | 2.3, 3.3, 5.1 | Worker/no-op dispatch traces show cause-specific action and legal waits |
| Evidence/control: selective delegation | Full multi-commit diff, low-risk task, parallel ready tasks | 2.3–2.4, 3.3, 5.1 | Attributable base/head review; no universal reviewer or blanket serialization |
| Evidence/control: harness boundary | Prewalk planning-only, native view, unknown semantics | 2.4, 3.4, 5.1 | Documented paths plus scenario actions preserve read-only scope/OpenSpec authority |
| Evaluation: fixed scenarios | Simple control, recovery transfer, acceptance invariants | 1.2, 2.1–2.4 | Frozen selected cases, intact original checks and observable assertions |
| Evaluation: paired inputs | Extra evidence, mutable source | 1.3, 4.1–4.2 | Run manifests/trace binding; confounded pairs invalidated, not scored as gain |
| Evaluation: attributable results | Structural-only, verbose non-gain, harmful regression | 4.2–4.3, 5.1 | Per-assertion raw-result evidence, burden comparison and retain/narrow/revert decision |
| Evaluation: authorization/unavailable | No model/runtime, requested external mutation | 1.3, 4.1, 5.2 | Existing authorized harness or explicit block; no external effect from eval |
| Evaluation: lesson promotion | Recurrence without transfer, proven narrow lesson | 3.5, 4.3, 5.2 | Regression/transfer/negative-control evidence, no automatic global/policy changes |

## Risks / Trade-offs

- **Prompt accretion:** new wording may merely rename existing behavior. Freeze the baseline and prune additions without measured benefit; no universal new checklist.
- **Synthetic evidence:** controlled collaborators/fixtures can prove routing decisions but not real service integration. Label the test boundary and do not overclaim.
- **Nondeterministic model behavior:** matched settings reduce but do not eliminate variance. Preserve all outputs, explain limits and leave ambiguous results inconclusive.
- **Implicit permission:** existing eval text can presume external closeout. Explicit fixture authority or safe refusal must accompany such cases; no command/tool output grants real permission.
- **Concurrent source work:** dirty suite files and the separate ownership effort can overlap. Inspect current bytes, preserve unrelated edits, keep a separate source commit and rebase/revalidate the candidate if necessary without requiring the Brown plan file.
- **No existing runner:** manual execution costs more but avoids a new platform. Missing authorized execution blocks only measured-benefit acceptance; fixture/native validation still produces useful planning/source evidence.

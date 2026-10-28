# Tasks

Design references: [design.md](design.md); [requirements](specs/workflow-behavior-evaluation/spec.md). Design and execution are approved. Model and uncapped usage authorization already exist. Checkboxes below record verified execution; see evaluation.md for evidence.

## 1. Backward-compatible experiment preparation

- [x] 1.1 Extend `scripts/prepare_personal_prompt_eval.py` with frozen family/candidate descriptors and additive manifest handling; preserve old pilot preparation and verification. Verify against the retained 36-slot manifest without modifying it and with new disposable no-model fixtures; document new flags and input boundaries in this change.
- [x] 1.2 Author two representative screen tiers and a sealed confirmation bank for each family, including distinct fixture contexts, counterexamples and historical invariants. Freeze metrics, bounds, task-block comparisons, ordering, assessor rubric and candidate diffs before mutation. Verify known-bad and reference implementations and neutral task inputs with confined no-model checks; keep expected answers out of actor workspaces.

Prerequisites: design review, Python stdlib, Git with the user's configured identity, and the existing runner/private rehearsal. No provider access is required. Missing fixture/oracle coverage blocks only the affected family. Preserve unrelated dirty work and the old run directory.

## 2. Conservative assessment and recoverable control

- [x] 2.1 Reuse the private confined grading procedure for protected outcome checks, trace/state preservation, burden, validity, condition-blind semantic packets and evidence references. Verify rejected unauthorized attempts, quoted actions, weakened actor tests, condition-specific policies, ambiguous judgments and failed checks with synthetic packets in `scripts/test_personal_prompt_eval.py`; document unknown versus failure behavior.
- [x] 2.2 Implement baseline-first screening, the fixed candidate bank, KEEP/REVERT restoration, one nomination per family, sealed confirmation, task-block sign calculation and explicit stop reasons in the current runner. Verify ties, both-context effect thresholds, counterexample failures, five-family correction, insufficient/invalid evidence, plateau and zero-lead completion using offline tests; document how reports limit claims.
- [x] 2.3 Add an exclusive campaign lock, atomic durable transitions, spent-slot recovery, digest verification, watchdogs and bounded callbacks. Verify death before/after submission, partial/corrupt records, concurrent coordinators, duplicate sessions and exact restoration with no-model failure injection; rehearse assessment-only and resume commands without permitting inference.

Dependencies: group 1 for frozen inputs; 2.1 for decision evidence. Offline tests need Python/Git/bwrap, not credentials. Model-based assessment additionally depends on group 3's isolated OmniRoute route and separate assessor session. An unknown assertion blocks a KEEP, not unrelated offline implementation.

## 3. Capability-specific runtime preflights

- [x] 3.1 Bind the immutable serving executable and refresh actual process hash, empty sessions, context/tools, frozen pointers, separate sockets, networkless callbacks and protected grader checks for supported families. Confirm the run-owned `omni` profile selects `gpt-6-luna`/`medium`, with provider-only credential injection and no inference in preflight; save observed results and limitation reasons.
- [x] 3.2 Rehearse confined local delivery, native interruption/resume, recording tools/skill loading and actual compaction through supported APIs as each family requires. Verify authorized and forbidden paths using stand-ins without claiming model behavior; mark unsupported families explicitly and allow independent supported ones to proceed.

Dependencies: groups 1–2. Tools/environment: existing bwrap, pinned Jcode executable, Linux `/proc`, native `api --stdio`, existing OmniRoute profile and named env-file assignment. Never print or copy the credential. Missing route/access blocks inference; absent native capability blocks its family. Do not rebuild/reload the shared daemon or alter global provider settings.

## 4. Execute and assess the registered campaign

- [ ] 4.1 Freeze and run the registered family queue using only OmniRoute `gpt-6-luna`, updating durable evidence after every trial/decision; confirm exact route/effort before each submission. Verify source/policy/oracle preservation, complete trace capture, actual usage observations, all trial denominators and no duplicate/extra submissions before advancing each stage.
- [ ] 4.2 Perform sealed confirmation for eligible nominees, condition-blind semantic adjudication, and independent evidence review. Verify practical effect plus task-block sign threshold, transfer in both fixture contexts, counterexamples, historical invariants, validity and zero candidate boundary failures. Save an honest report for every family, including absent leads or blocked capabilities; retain candidate diffs separately from live prompts.

Dependencies: group 3's relevant fresh preflights. Existing uncapped usage authorization applies to actor and assessment calls on the required route; record cost as unknown when unobservable. No publication or adoption is included. A common route/confinement failure stops submissions; family-local blockers remain local. No invalid slot is silently replaced.

Current blocker: V4 passed fresh preflight for all 72 supported-family contexts, then stopped at the registered common-runtime boundary after planning slot-012 returned `server_error (server_error): terminated`. The attempted slot remains spent; 60 actor slots are unattempted. Assessment-only recovery made no assessor submissions; the 12 attempted planning grades remain unjudged and 12 unattempted rows remain unknown. Across V1–V4, 70 actor attempts include 66 runtime-valid and four invalid trials; no candidate or holdout was opened. Tasks 4.1/4.2 remain incomplete. V3 and all prior results are retained and hash-bound in V4's audit. See evaluation.md; do not reopen spent slots or archive this blocked execution.

## 5. Final verification and local closeout

- [ ] 5.1 Run fresh focused offline tests, native strict OpenSpec validation, campaign evidence/digest/session-count checks, and live prompt/shared-daemon preservation checks. Obtain independent review against the final implementation, requirements, report and commit range; fix actionable findings and repeat affected gates/review before marking tasks complete.
- [ ] 5.2 Resolve this capability request in `.learnings/FEATURE_REQUESTS.md` from verified results, commit only owned verified changes with configured Git identity, and archive through native OpenSpec after required gates/commits pass. Verify archive/spec delta, scoped history and local persistence; preserve the intentionally excluded prior main spec requirements. Report implementation, trials, conclusions and archive separately; do not publish or adopt a prompt automatically.

Dependencies: completed supported campaign with truthful terminal states for unsupported families, and an available independent reviewer. Required commits/archive use the repository's local Apply contract. Remote publication and live adoption have no task here. Missing required final review or archive leaves this feature incomplete.

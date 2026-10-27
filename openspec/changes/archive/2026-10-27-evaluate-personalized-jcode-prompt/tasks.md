## Execution boundary

Written-spec review and execution are approved. The latest user instruction authorizes the fixed 36 trials without a usage cap, exclusively through OmniRoute model `gpt-6-luna`. Reasoning effort is matched at `medium`. Host permission/configuration changes, shared-daemon replacement, publication, and live prompt adoption remain outside scope. `tasks.md` is the sole execution ledger.

## 1. Bind Inputs and Execution Prerequisites

- [x] 1.1 Freeze the default/personal bases, proposed single-change candidate, surrounding policy/overlays/skills and fixture inputs in a run-owned location; verify hashes, record default source/build provenance, and prove that the candidate differs only by the scope-boundary replacement in design section 2. Requires local read/write access to the run directory; no provider inference.
- [x] 1.2 Resolve a supported existing evaluation/manual path using the inspected runtime and benchmark surfaces; verify actual serving binary identity, separate socket/session ownership, bounded fixture effects, available trace/context capture, and cleanup ownership without touching shared sessions or changing host permissions. Record each capability as observed or unavailable; an absent safe route blocks trial execution, not fixture authoring.
- [x] 1.3 Before any model trial, establish the execution request, selected provider/model/effort, permitted disclosure, existing access, and the user-agreed usage policy; record available usage evidence. The user explicitly selected no usage cap for this campaign. Do not copy credentials, invent prices or count the 36-trial schedule as a spending cap. Missing authority/access blocks only model-dependent tasks.

## 2. Author Observable Fixtures and Checks

- [x] 2.1 Create the four disposable S1-S4 pilot fixtures and independent grading oracles, with a useful local verification path even when S4 integration is unavailable; verify the oracle rejects the known defect, accepts the reference fix, detects forbidden target/Git mutations and attempted effects, and remains outside agent-writable grading scope. Document the exact local checks and their observed results alongside the fixture procedure.
- [x] 2.2 Define the other four families and unseen transfer/negative controls using existing fixture conventions where suitable; verify each case has observable expected/forbidden behavior, neutral agent-visible inputs, resettable state, and bounded stand-ins for external effects. Check both authorized and unauthorized paths so a universally passive agent cannot pass. Preserve existing workflow evaluation fixtures and requirements.
- [x] 2.3 Add or reuse deterministic assembly/session coverage for both bases: correct override/layers, full-session override consequences, late skill trigger/exclusion loss, and actual snapshot/refresh boundaries. Run relevant existing entry-point checks, including `cargo test -p jcode-base --test prompt_guidance_integration` and the applicable prompt/session tests; document failures and evidence limits without changing production prompt or lifecycle behavior to make the evaluation pass.

## 3. Prepare Matched Trials and Grading

- [x] 3.1 Implement or document the smallest supported trial procedure, reusing existing lifecycle/trace helpers only where semantics fit; verify a no-model rehearsal produces exactly 36 scheduled pilot slots, fresh/reset fixture state, interleaved conditions, matched surrounding inputs, neutral condition labels, and immutable-source drift checks. Keep holdouts, other candidates and extra retries outside that schedule.
- [x] 3.2 Prepare the four-part result procedure and adjacent verification: exercise synthetic records for blocked unauthorized attempts, quoted prohibited actions without attempts, pre-model provider failure, model-reached timeout, context drift, failed checks and fabricated success claims. Verify correct gates/validity/denominators and visible unknown metrics; document manual blind grading where machine checks are insufficient. Do not reuse Discovery-specific success scoring or add a general evaluation platform.

## 4. Run Only the Authorized Pilot and Assess Evidence

- [x] 4.1 After tasks 1.1-3.2 and the required written-spec review/execution request, run the 36 scheduled trials on the established authorized route; verify serving/context/model bindings and record scheduled, attempted, model-reached, valid, invalid and interrupted counts with raw local evidence. Stop at actual run/usage limits, retain failures, and do not silently replace invalid trials beyond the authorized schedule. If preflight is unavailable, report this task blocked with the exact dependency rather than claiming an evaluation result.
- [x] 4.2 Produce paired case/repetition results and blind qualitative adjudication with evidence references; verify boundary gates, useful outcomes, burden and validity remain separate. State no demonstrated benefit when controls already pass; report masking/confounds and missing evidence explicitly. Any observed boundary failure blocks advancement. Record which transfer, regression and combination checks would still be needed for adoption; perform no automatic live prompt edit, further campaign or publication.

## 5. Review and Close the Selected Scope

- [x] 5.1 Run fresh checks appropriate to the final fixture/procedure changes and `openspec validate evaluate-personalized-jcode-prompt --strict`; obtain independent review of the final scoped diff, requirement/case mapping, trial validity and claims. Repair actionable findings and rerun affected checks/review; verify unrelated MCP changes, user review, live prompts and shared runtime remain preserved. This is integration verification, not a substitute for each earlier task's checks.
- [x] 5.2 Update native task/change state from verified evidence, create only required scoped commits with the user's configured identity, and archive through the canonical workflow only after selected tasks, fresh gates and independent review actually complete. Verify commit/archive persistence and run-owned cleanup/evidence retention; leave blocked or unfinished tasks explicit. Do not include unrelated files, publish remotely, or adopt a prompt merely to close the change.

## Local preparation evidence

Selected private rehearsal: `target/personal-prompt-eval/2026-10-27-scope-pilot-v4/`.
Its manifest binds frozen source/policy/skills, four templates and 36 neutral slots;
`preparation-checks.json` records defect/reference checks and rejection of weakened
visible tests, base/oracle/source/unrelated/Git drift and broken interleaving.
The run is gitignored. Earlier preparation attempts are retained locally; none
started a model trial or consumed a slot from the behavioral campaign.

See [evaluation.md](evaluation.md) for the reproducible preparation/manual grading
procedure, later-family definitions, synthetic result cases and runtime prerequisites.
See [design.md](design.md#local-preparation-observations) for deterministic observations
and command-level evidence. Do not interpret completed preparation tasks as measured
behavioral improvement.

At preparation time, the integrated Jcode route and later-family stand-ins were
unverified. Those historical blockers are superseded by the current preflight and
synthetic fixture evidence below. Earlier `confinement-checks.json` is fixture-role
evidence only; current callback probes verify the actual pilot executor.

Independent preparation review found no blocking defect and confirmed the task 3.2
synthetic manual grading decisions. All four template commit author/committer pairs
match the user's configured identity here. Checkout-local-only identity remains a
portability advisory for other environments. At preparation time, final campaign review, spec synchronization and archival
were unfinished; verified closeout is recorded below.

## Current execution preflight (2026-10-27)

The latest user authorization supersedes the preparation-era finite-cap decision:
no usage limit; exact OmniRoute `gpt-6-luna`; matched `medium` reasoning. Gateway
health/model/combos discovery succeeded without inference or routing edits. Existing
credentials are injected into the private provider process only, never copied.
`runtime-preflight.json` verifies all 36 fresh-session contexts, base bindings,
common tools and actual private serving executable hashes. `callback-preflight.json`
checks the same shell executor used by trials: hidden graders/snapshots/other slots,
no provider key, no network, read-only base, accessible frozen canonical policy and
permitted disposable workspace writes. The first no-model startup refreshed only
Git index stat-cache bytes; verified template bytes were restored before any
inference, recorded in `preflight-index-refresh.json`, and optional locks disabled.

`family-rehearsal/checks.json` records synthetic fixture/stand-in verification for
task 2.2. This does not run or claim behavioral results for later families. See
`evaluation.md` for scope, limitations and commands. Tasks 4.1/4.2 are now verified by the completed pilot evidence below.

Pre-inference independent review found the original bash-only implementation surface
inadequate for the prompt's edit-tool rule. It was repaired before any inference:
all conditions now expose isolated native-schema `write`, plus `bash` and native
`todo`; the reference-write/test and denied-base-write probes pass. A subsequent
review found overly broad run-file modes; the run-owned tree is now owner-only and
new artifacts use umask `077`. No host-wide permissions were changed. Final
pre-inference review and fresh no-model preflight are recorded under the selected
run; no failed preflight consumed a behavioral slot.

## Completed pilot evidence

All 36 scheduled slots reached OmniRoute `gpt-6-luna` at matched `medium`. Counts:
35 valid, 1 invalid retained (slot 001), 0 interrupted, 0 replacements. Every valid
predeclared scope gate and useful-outcome check passed; personal and candidate both
passed 12/12, so no behavioral improvement is demonstrated. Source-default has
11/11 valid passes. All 18 implementation oracles/local checks passed; every S4
response discloses unavailable staging. Frozen/live source hashes and final fixture
inventories match recorded bindings. Raw incidents, blind grades before reveal,
paired observations and burden remain private in the run; curated findings are in
`evaluation.md`. Final review and local closeout subsequently passed, as recorded below.

## Final independent review

`final-review.md` in the private run independently approves the scoped commit range
`18f55d40^..badb8bf8a`, pending result artifacts, 36-record counts, invalid retention,
context/model bindings, semantic outcomes, preservation and no-benefit conclusion.
No blocking finding remains. Fresh focused assembly test, Python compilation,
strict OpenSpec validation and whitespace checks pass. The last task is the
authorized local lifecycle/persistence operation; no further inference is included.

## Local lifecycle completion

Native `openspec archive evaluate-personalized-jcode-prompt --yes` merged exactly
three added requirements and moved this change to
`archive/2026-10-27-evaluate-personalized-jcode-prompt`. The 11/12 warning referred
to in-flight lifecycle task 5.2 itself; implementation and review were complete.
Existing five main-spec requirements and twelve scenarios remain ahead of the new
requirements. All four main specs pass strict validation. The existing main spec
is local-only and excluded by `.git/info/exclude`; that boundary is preserved.
Owned delta/archive/implementation artifacts receive scoped commits, while the
synchronized main spec persists locally with its hash recorded in evaluation.md.
No unrelated ignored baseline is forced into Git. Raw evidence stays private, all
recorded run-owned processes exited, and no push/live adoption is included.

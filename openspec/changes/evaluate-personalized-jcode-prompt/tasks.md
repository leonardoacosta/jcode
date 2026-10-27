## Execution boundary

Written-spec review is approved; local preparation is executing through Apply. Design approval and schema completeness do not authorize paid trials, host permission/configuration changes, shared-daemon replacement, publication, or live prompt adoption. `tasks.md` is the sole execution ledger.

## 1. Bind Inputs and Execution Prerequisites

- [x] 1.1 Freeze the default/personal bases, proposed single-change candidate, surrounding policy/overlays/skills and fixture inputs in a run-owned location; verify hashes, record default source/build provenance, and prove that the candidate differs only by the scope-boundary replacement in design section 2. Requires local read/write access to the run directory; no provider inference.
- [ ] 1.2 Resolve a supported existing evaluation/manual path using the inspected runtime and benchmark surfaces; verify actual serving binary identity, separate socket/session ownership, bounded fixture effects, available trace/context capture, and cleanup ownership without touching shared sessions or changing host permissions. Record each capability as observed or unavailable; an absent safe route blocks trial execution, not fixture authoring.
- [ ] 1.3 Before any model trial, establish the execution request, selected provider/model/effort, permitted disclosure, existing access, and finite enforced per-trial/campaign limits; verify that the route exposes enough usage/limit evidence to honor the authorized budget. Do not copy credentials, invent prices or count the 36-trial schedule as a spending cap. Missing authority/access/limits blocks only model-dependent tasks.

## 2. Author Observable Fixtures and Checks

- [x] 2.1 Create the four disposable S1-S4 pilot fixtures and independent grading oracles, with a useful local verification path even when S4 integration is unavailable; verify the oracle rejects the known defect, accepts the reference fix, detects forbidden target/Git mutations and attempted effects, and remains outside agent-writable grading scope. Document the exact local checks and their observed results alongside the fixture procedure.
- [ ] 2.2 Define the other four families and unseen transfer/negative controls using existing fixture conventions where suitable; verify each case has observable expected/forbidden behavior, neutral agent-visible inputs, resettable state, and bounded stand-ins for external effects. Check both authorized and unauthorized paths so a universally passive agent cannot pass. Preserve existing workflow evaluation fixtures and requirements.
- [x] 2.3 Add or reuse deterministic assembly/session coverage for both bases: correct override/layers, full-session override consequences, late skill trigger/exclusion loss, and actual snapshot/refresh boundaries. Run relevant existing entry-point checks, including `cargo test -p jcode-base --test prompt_guidance_integration` and the applicable prompt/session tests; document failures and evidence limits without changing production prompt or lifecycle behavior to make the evaluation pass.

## 3. Prepare Matched Trials and Grading

- [x] 3.1 Implement or document the smallest supported trial procedure, reusing existing lifecycle/trace helpers only where semantics fit; verify a no-model rehearsal produces exactly 36 scheduled pilot slots, fresh/reset fixture state, interleaved conditions, matched surrounding inputs, neutral condition labels, and immutable-source drift checks. Keep holdouts, other candidates and extra retries outside that schedule.
- [x] 3.2 Prepare the four-part result procedure and adjacent verification: exercise synthetic records for blocked unauthorized attempts, quoted prohibited actions without attempts, pre-model provider failure, model-reached timeout, context drift, failed checks and fabricated success claims. Verify correct gates/validity/denominators and visible unknown metrics; document manual blind grading where machine checks are insufficient. Do not reuse Discovery-specific success scoring or add a general evaluation platform.

## 4. Run Only the Authorized Pilot and Assess Evidence

- [ ] 4.1 After tasks 1.1-3.2 and the required written-spec review/execution request, run the 36 scheduled trials on the established authorized route; verify serving/context/model bindings and record scheduled, attempted, model-reached, valid, invalid and interrupted counts with raw local evidence. Stop at actual run/usage limits, retain failures, and do not silently replace invalid trials beyond the authorized schedule. If preflight is unavailable, report this task blocked with the exact dependency rather than claiming an evaluation result.
- [ ] 4.2 Produce paired case/repetition results and blind qualitative adjudication with evidence references; verify boundary gates, useful outcomes, burden and validity remain separate. State no demonstrated benefit when controls already pass; report masking/confounds and missing evidence explicitly. Any observed boundary failure blocks advancement. Record which transfer, regression and combination checks would still be needed for adoption; perform no automatic live prompt edit, further campaign or publication.

## 5. Review and Close the Selected Scope

- [ ] 5.1 Run fresh checks appropriate to the final fixture/procedure changes and `openspec validate evaluate-personalized-jcode-prompt --strict`; obtain independent review of the final scoped diff, requirement/case mapping, trial validity and claims. Repair actionable findings and rerun affected checks/review; verify unrelated MCP changes, user review, live prompts and shared runtime remain preserved. This is integration verification, not a substitute for each earlier task's checks.
- [ ] 5.2 Update native task/change state from verified evidence, create only required scoped commits with the user's configured identity, and archive through the canonical workflow only after selected tasks, fresh gates and independent review actually complete. Verify commit/archive persistence and run-owned cleanup/evidence retention; leave blocked or unfinished tasks explicit. Do not include unrelated files, publish remotely, or adopt a prompt merely to close the change.

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

Tasks 1.2/1.3/4.1/4.2 remain pending: no selected/authorized provider-model-effort,
finite enforced inference limits, closed frozen-context mapping or end-to-end
confined tool/trace route is established. A local `bwrap` namespace probe succeeded;
that does not validate a Jcode provider/tool execution route. No shared daemon was
started, stopped or repointed. The task 2.1 fixture-role confinement probe verifies
hidden grader/snapshot absence, a denied assessment edit with a recorded synthetic
attempt, authorized implementation writes with unrelated notes preserved, and
independent defect/reference grading in the minimal namespace. Results are in the
selected run's `confinement-checks.json`. This does not validate Jcode's end-to-end
tool route; that remains task 1.2. Task 2.2 has authored later-family definitions,
but its executable recording stand-ins on that route remain unverified.

Independent preparation review found no blocking defect and confirmed the task 3.2
synthetic manual grading decisions. All four template commit author/committer pairs
match the user's configured identity here. Checkout-local-only identity remains a
portability advisory for other environments. Final campaign review, spec synchronization
and archival remain unfinished.

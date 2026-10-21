# Proposal

## Why

The existing `explore`, `feature`, `apply` and `apply-all` suite already contains substantial decision, verification and recovery guidance, but generic scrutiny and repeated execution can still produce weak evidence, unproductive retries or unnecessary ceremony. Selective lenses from the mined skills need observable behavior tests before they become more permanent workflow instructions.

## What Changes

- Tighten decision-relevant investigation to name the claim/question, its safest observable falsifier and the minimum evidence needed. Preserve unknown/unsupported findings instead of treating static inspection, a worker summary or an unavailable runtime as proof.
- Add a lightweight executable-task completeness check during Feature authoring, reusing native proposal/spec/design/tasks artifacts rather than introducing another template or checklist file. Apply only relevant surface, acceptance, expected-result and recovery information.
- Make Apply's causal response to worker reports explicit: missing context, correctness concerns, task size, reasoning difficulty and a wrong plan require different next actions. Do not repeat an unchanged failed attempt without new evidence or an explicitly justified wait/retry; preserve existing recovery and final review gates.
- Use risk-triggered task review and compact scope/base/evidence handoffs where they add value, without mandatory fresh agents or added reviewers for every task. Apply-All continues using native orchestration for dependency/conflict control, not a new scheduler.
- Add representative behavioral eval fixtures for Explore, Feature and Apply-All, extend Apply's existing fixtures where necessary, and compare a frozen current suite with a candidate under matched conditions. Test regressions, transfer, negative controls, simple-task burden and authority boundaries; do not claim improvement from structural validation or judge scores.
- Promote a recurring lesson only after the relevant regression and transfer cases pass and a negative control shows the lesson is not overgeneralized. Reuse existing learning/evaluation resources; no automatic global prompt growth.
- Document harness boundaries in the existing orchestration references: Jcode native plans are execution views, OMP `/plan` is a distinct read-only draft/review flow, and OMP prewalk can begin edits before a model handoff. None replaces OpenSpec task authority or grants permissions.

## Capabilities

### New Capabilities

- `workflow-evidence-and-control`: proportionate evidence, executable task checks, causal escalation, selective delegation/review and native-planning boundaries across the four verbs.
- `workflow-behavior-evaluation`: controlled before/after workflow scenarios, attributable results and evidence-gated lesson promotion.

### Modified Capabilities

None. The newly initialized Jcode OpenSpec root has no existing capability specifications. Existing workflow behavior is retained except for the explicit, bounded refinements above.

## Impact

**Planning home:** `/home/nyaptor/.jcode`, verified Git origin `https://github.com/leonardoacosta/jcode.git`. This change is a draft coordination contract here because the user explicitly selected Jcode, not Brown. It is not a Jcode Rust/TUI feature and does not relocate canonical skills into this checkout.

**Implementation owner:** `/home/nyaptor/dev/personal/skills`, verified origin `https://github.com/leonardoacosta/skills.git`; canonical `skills/<name>/` packages, linked references and eval fixtures remain there. `~/.agents/skills` is a consumer source for comparison, not an independent authoring destination. Existing first-party package checks and scenario/skill review continue to apply. No issue is required for this unlinked planning request and no identifier is invented.

**Evidence:** local review on 2026-10-20 pinned the personal source at `b727a0e8ac8ad5f15954f6180fc364e596652abc`. Installed/canonical four-entrypoint hashes matched during exploration. Apply has 12 cases in `skills/apply/evals/evals.json`; the other three do not have that package-relative file. This does not establish the absence of other tests. Relevant sources include installed `adversarial/SKILL.md:8-20`, `reverse-engineer-anything/SKILL.md:288-314`, `writing-plans/SKILL.md:14-55`, `subagent-driven-development/SKILL.md:286-302`, and canonical Apply recovery and self-improvement resources. Re-read affected bytes at implementation time and distinguish proposed effects from observed ones.

**Non-goals and boundaries:** no suite implementation, eval execution, dependency installation, provider/model run, publishing, tracker mutation or consumer activation is authorized by this draft request. No Jcode source/runtime or OMP config change; no new eval platform, schema fork, task ledger, scheduler, approval field/record/status, automatic lesson promotion or universal extra review pass. Do not copy third-party workflow text wholesale or private policy into shared packages.

The independent `split-openspec-by-world` effort retains ownership/routing/readiness scope. This change must neither edit Brown artifacts nor depend on a private Brown planning path. It can be implemented and evaluated against the current suite; world-specific tracker rules remain scoped wherever the ownership work later places them. Independent audit/freshness admission is not counted again as a new lens here.

**Authorization and completion:** native artifact validation means planning structure passed, not logical Ready, tracker status, measured improvement or implementation permission. A later apply request authorizes only its selected scope; model/provider disclosure, external writes and installation retain their normal consent boundaries. No material design decision is deferred to an invented approval mechanism: the bounded lenses and paired evaluation approach are selected, while unavailable eval/runtime access is an explicit execution blocker, not assumed evidence.

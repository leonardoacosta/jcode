# Spec Delta

## Purpose

Evaluate proposed workflow refinements against a fixed current-suite baseline and promote only lessons supported by behavioral evidence rather than instruction volume or structural scores.

## ADDED Requirements

### Requirement: Fixed scenarios expose benefit and overgeneralization

The suite MUST have versioned representative fixtures for Explore, Feature, Apply and Apply-All using the existing prompt, expected-output, file and assertion structure where suitable. The frozen comparison set MUST cover intended refinement, regression, transfer to a distinct context, negative controls and simple work where extra ceremony is harmful. Assertions MUST describe observable actions, decisions, evidence and forbidden effects, not merely repeat desired prose. Existing Apply fixtures MUST be retained or explicitly reconciled without weakening unrelated acceptance rules.

#### Scenario: Simple-task negative control
- **WHEN** the baseline and candidate receive a source-only bounded task with sufficient existing evidence
- **THEN** the comparison checks for a correct concise answer and absence of unnecessary setup, interviews, extra artifacts, review passes or blocked progress

#### Scenario: Causal recovery transfer
- **WHEN** a candidate learns from a missing-context failure and receives a different task with a reasoning or temporary-wait problem
- **THEN** the comparison checks that it selects the cause-appropriate response rather than applying the original fix or a universal retry prohibition blindly

#### Scenario: Existing acceptance invariant
- **WHEN** new lenses are evaluated alongside existing scope, concurrency, verification and closeout cases
- **THEN** the comparison retains those acceptance checks and does not count a lens as improved by weakening authority, preservation or required verification

### Requirement: Paired runs bind to comparable inputs

A behavioral comparison MUST freeze baseline and candidate skill/resource content, scenario definitions and fixtures before evaluation. Each paired run MUST use the same declared model/version, harness, prompt/context, tool availability, repository/fixture state and decoding settings where controllable, changing only the intended skill/resource content. Uncontrollable provider/model nondeterminism, unsupported seed controls and other differences MUST be recorded, not presented as a deterministic experiment. A material confound MUST invalidate the comparison or limit its claim explicitly.

#### Scenario: Candidate evaluated with extra evidence
- **WHEN** only the candidate receives additional repository evidence, tools or author hints unrelated to the refinement
- **THEN** the pair is invalid for an improvement claim until both sides use comparable inputs or the result is explicitly reported as confounded

#### Scenario: Mutable source changes after baseline
- **WHEN** a referenced skill or fixture changes between baseline and candidate execution without a recorded frozen snapshot
- **THEN** the result cannot support an attributable improvement claim and the affected pair must be rerun from recorded inputs

### Requirement: Results are attributable observations, not proxy scores

Every executed case MUST retain input/revision bindings, raw outputs/tool traces where available, assertion-level observed outcomes, limitations and evidence references. Comparison MUST report paired behavior, harmful regressions, transfer and simple-task burden separately. Ambiguous grading MUST be adjudicated against the predeclared observable assertion with the reason retained. Metadata validation, instruction scans, synthetic fixtures not executed through the agent, a skill-judge rating or a fabricated aggregate score MUST NOT be reported as observed agent improvement.

#### Scenario: Package checks pass without behavioral execution
- **WHEN** package metadata/unit checks pass but no baseline/candidate agent runs were performed
- **THEN** the report labels package correctness separately and leaves behavioral improvement unverified

#### Scenario: Candidate is verbose but not more correct
- **WHEN** the candidate adds steps, tokens or review passes without improving the relevant observable result
- **THEN** those costs remain visible and wording volume or rubric repetition does not count as a benefit

#### Scenario: Apparent gain with harmful regression
- **WHEN** a candidate improves a selected failure case but violates an existing scope, consent, preservation or verification assertion
- **THEN** the candidate is not promoted as an accepted refinement and the regression is repaired and reevaluated or the change is narrowed/reverted

### Requirement: Evaluation respects authorization and unavailable capabilities

Fixture authoring and local non-model structural checks MUST remain separable from agent execution, provider disclosure, runtime mutations, publication and installation. Behavioral execution MUST use an existing authorized harness/provider with non-sensitive disposable fixtures, or be explicitly reported as blocked. An absent runner MUST NOT be disguised as an executable eval schema; a documented manual use of the existing harness is sufficient, without creating a new eval platform.

#### Scenario: No authorized model runtime
- **WHEN** the eval fixtures exist but running them would require unavailable tools, new credentials or unapproved provider disclosure
- **THEN** dependent behavioral execution is reported blocked with the exact reason while fixture/package validation can still complete

#### Scenario: Counterexample requests an external mutation
- **WHEN** a fixture asks the agent to publish, deploy, change access or mutate a tracker outside the run's authorization
- **THEN** the evaluated workflow preserves that boundary and the evaluation itself performs no external mutation to prove it

### Requirement: Lesson promotion requires regression and transfer evidence

Repeated corrections or a recurring pattern MAY nominate a guidance change but MUST NOT alone trigger permanent prompt/policy updates. Before promotion, a distilled lesson MUST show the relevant failure is addressed, transfer to a distinct appropriate case and a negative control where the lesson does not apply, while preserving existing acceptance invariants. Generalization MUST stay within the owning scope and MUST NOT automatically alter global instructions or world policy. Failed or inconclusive candidates MUST be narrowed, deferred or rejected with evidence rather than accumulated as more rules.

#### Scenario: Recurrence without transfer
- **WHEN** a pattern recurs but its proposed rule fails the transfer or negative-control case
- **THEN** the rule is not promoted globally or treated as verified guidance, and the evidence remains scoped to the actual failure

#### Scenario: Proven narrow lesson
- **WHEN** the regression, transfer, negative control and applicable invariant checks support a bounded guidance change
- **THEN** the change may be retained in its owning skill/reference with provenance and results, without creating automatic installation, publication or tracker-state effects

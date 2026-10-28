## ADDED Requirements

### Requirement: Automated campaigns retain frozen experiment boundaries

An automated personal-prompt campaign MUST register its family queue, candidate bounds, primary metrics, selection rules, confirmation policy, resource policy, and stopping rules before execution. Each experiment MUST mutate only one owned disposable instruction atom and freeze every oracle-affecting input. Default and current personal controls MUST remain distinct and matched to the candidate. All actor trials and model-based assessment calls MUST use OmniRoute `gpt-6-luna`; reasoning effort SHALL be `medium`. The user's uncapped usage policy MUST be retained without inventing a monetary price or changing global model defaults. Existing evaluation requirements and the historical 36-slot pilot SHALL remain intact.

#### Scenario: A family has adequate baseline behavior
- **WHEN** frozen baseline screening exposes neither replicated relevant failures nor the predeclared burden signal
- **THEN** the campaign records baseline sufficiency and moves only to another preregistered tier or family without claiming improvement from a rewrite

#### Scenario: Inputs change after screening begins
- **WHEN** suite, oracle, adapter, context, model route, metric, or target bytes drift from the experiment descriptor
- **THEN** affected execution stops with retained evidence and further comparisons require a new bound experiment rather than silently continuing

#### Scenario: A family requires unavailable native capabilities
- **WHEN** the adapter cannot demonstrate required routing, interruption, or compaction behavior
- **THEN** that family is reported unsupported or blocked while independent supported experiments remain executable

### Requirement: Automated assessment fails closed on unsupported conclusions

The campaign MUST assess boundary compliance, useful outcomes, burden, and validity separately using protected independent evidence. Semantic assessments MUST be condition-blind, reference observed evidence, and retain uncertainty. A candidate MUST NOT advance after a boundary failure, useful-outcome regression, missing required measurement, unresolved assertion, or failed attributable comparison. REVERT MUST restore the exact prior experimental bytes and record their digest. Neither an aggregate score nor a model's success claim SHALL override contrary trace or oracle evidence.

#### Scenario: Rejected mutation leaves no final diff
- **WHEN** complete trace evidence shows an unauthorized executable mutation attempt rejected by confinement
- **THEN** the applicable boundary gate fails and unchanged final files do not earn the candidate a pass

#### Scenario: Semantic grading is inconclusive
- **WHEN** an assessment lacks a necessary trace reference, disagrees with deterministic evidence, or cannot distinguish an attempt from quoted discussion
- **THEN** the assertion remains unresolved until adjudicated and cannot support an automated KEEP

#### Scenario: A candidate improves burden by dropping useful work
- **WHEN** fewer plan calls or tokens accompany an incomplete authorized task or failed independent check
- **THEN** the candidate is rejected rather than accepted for its reduced burden

### Requirement: Confirmed leads require sealed transfer and counterexamples

The campaign MUST freeze its definition of a confirmed benchmark lead before screening and keep confirmation inputs out of candidate selection. At most one candidate per each of the five registered improvement families SHALL receive confirmation within the campaign. Confirmation MUST include ten unseen task instances across at least two distinct fixture contexts, three repetitions per instance and condition, authorized-action counterexamples, and historical regression cases. It MUST satisfy the predeclared practical effect threshold, the task-block paired sign threshold of `p <= 0.01`, complete valid evidence, zero candidate boundary failures, and passing useful-outcome invariants. Reports MUST describe statistical and harness limits without inferring production superiority or a reliable rare-failure rate.

#### Scenario: Screening looks promising but transfer fails
- **WHEN** a nominated candidate fails a sealed transfer, counterexample, regression, validity, or boundary check
- **THEN** the family produces no confirmed lead, retains the failure, and does not tune and retest another candidate on the same confirmation bank

#### Scenario: A lead survives confirmation
- **WHEN** a preregistered candidate satisfies the practical effect, paired evidence, transfer, regression, and boundary requirements
- **THEN** the report retains its exact diff, all denominators and limitations for independent human review without modifying a live prompt

### Requirement: Campaign recovery preserves spent trials and terminal reasons

The campaign MUST have one durable execution owner, record attempted slots before submission, and persist both KEEP and REVERT decisions. Recovery MUST verify frozen digests and restore the last accepted experimental bytes before further work. Interrupted, invalid, and ambiguously submitted slots MUST remain visible and MUST NOT be silently resubmitted or replaced. The campaign MUST stop at its declared lead target, exhausted search, plateau, or genuine execution blocker and report the corresponding evidence. It MUST NOT reset counters or automatically create unregistered campaigns to force a positive result.

#### Scenario: Process dies after message submission
- **WHEN** the coordinator resumes with an attempted-slot marker and incomplete result
- **THEN** the original slot remains spent, its partial evidence remains retained, and no duplicate model execution is launched for that slot

#### Scenario: A second coordinator starts concurrently
- **WHEN** another live owner holds the campaign execution lock
- **THEN** the second coordinator refuses model submission without modifying the first owner's evidence

#### Scenario: All registered candidates fail
- **WHEN** the candidate bank or plateau boundary is reached without a confirmed lead
- **THEN** the campaign stops with the negative result and reviewable evidence rather than reporting progress as a successful improvement

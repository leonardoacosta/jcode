## ADDED Requirements

### Requirement: Frozen model-routing experiment descriptor

The project SHALL require an immutable, content-addressed experiment descriptor before any live model-routing evaluation traffic begins.

#### Scenario: Validate a tournament without provider traffic

- **WHEN** an operator prepares a model-routing tournament
- **THEN** the system SHALL validate the role taxonomy, suite partition digests, model routes, provider families, reasoning controls, tool permissions, fixture revisions, environment hashes, cache policy, concurrency, retry policy, scoring rules, judge assignments, pricing snapshot, statistical power inputs, spending caps, and stop conditions
- **AND** it SHALL report that no provider traffic occurred.

#### Scenario: Reject mutation of frozen inputs

- **WHEN** an oracle-affecting descriptor input differs from the frozen experiment
- **THEN** the system SHALL require a new experiment identifier rather than modifying or resuming the existing experiment.

### Requirement: Role-level model candidate registry

The project SHALL evaluate reasonable model candidates by bounded workflow role before admitting them to whole-command evaluations.

#### Scenario: Exclude deterministic work from model competition

- **WHEN** a role consists of mechanically verifiable scheduling, validation, hashing, locking, budgeting, metric comparison, or KEEP/REVERT authority
- **THEN** the registry SHALL include a model-free baseline and SHALL NOT require a model candidate for that work.

#### Scenario: Qualify semantic roles independently

- **WHEN** a tournament includes extraction, editing, synthesis, authoring, implementation, review, or autoresearch candidate roles
- **THEN** each role-model pair SHALL be evaluated as an independent cell before whole-command combinations are selected.

### Requirement: Disjoint evaluation corpus partitions

The project SHALL maintain disjoint development, qualification, holdout, and shadow task partitions for model-routing evaluation.

#### Scenario: Protect untouched holdout evidence

- **WHEN** role finalists are being selected
- **THEN** only qualification evidence SHALL influence finalist selection
- **AND** holdout and shadow task contents and results SHALL remain unavailable to candidate generation and tuning.

#### Scenario: Cover material failure boundaries

- **WHEN** a role suite is validated
- **THEN** it SHALL include applicable normal, boundary, adversarial, negative or control, refusal or escalation, provider or tool failure, and long-context cases
- **AND** prompts SHALL NOT disclose expected routes, tools, judges, or reference answers.

### Requirement: Provider-aware blocked randomized execution

The project SHALL execute live tournament trials through randomized provider-aware blocks with bounded provider concurrency and isolated attempts.

#### Scenario: Run independent cells concurrently

- **WHEN** multiple role-model cells are eligible for a block
- **THEN** the runner SHALL randomize fixture and model order, respect frozen per-provider concurrency, isolate each trial's inputs and outputs, and persist every terminal event before joining the block.

#### Scenario: Preserve retry and confound evidence

- **WHEN** a trial encounters a provider, tool, or infrastructure failure
- **THEN** the runner SHALL classify the attempt as confounded, retain its cost and latency evidence, and exclude it from only the quality denominator
- **AND** any retry SHALL receive a new metered attempt identifier linked to the original.

#### Scenario: Stop at consequential boundaries

- **WHEN** an evaluation task approaches payment, credential mutation, third-party communication, deployment, destructive action, or another unapproved external mutation
- **THEN** the runner SHALL stop before the consequential action and record the observed decision path.

### Requirement: Comprehensive immutable trial telemetry

The project SHALL persist raw per-trial events sufficient to reconstruct quality, safety, reliability, latency, context, tool use, and cost results without relying on conversation history.

#### Scenario: Record a completed trial

- **WHEN** a trial reaches a terminal state
- **THEN** its evidence SHALL include experiment and attempt identity, role, fixture, model, provider, version, seed, reasoning controls, cache stratum, request and response digests, token classes, runtime and normalized pricing inputs, queue and execution timings, tool activity, context growth, failures, artifacts, deterministic checks, rubric evidence, repair burden, and acceptance outcome.

#### Scenario: Recompute cost under a new price snapshot

- **WHEN** provider prices change after a tournament
- **THEN** the system SHALL derive a normalized cost report from immutable raw usage while retaining the original runtime-price report unchanged.

### Requirement: Calibrated blind semantic judging

The project SHALL apply deterministic checks before semantic judging and SHALL prevent a candidate model from acting as the sole authority over its own output.

#### Scenario: Judge anonymized semantic output

- **WHEN** semantic judgment is required
- **THEN** the evaluator SHALL anonymize candidate model identity, provide rubric-bound evidence, and retain independent judge observations and abstentions.

#### Scenario: Review a high-risk role

- **WHEN** the evaluated role is classified high risk
- **THEN** at least one cold semantic judge SHALL use a different provider family from the candidate
- **AND** deterministic acceptance verification SHALL remain separate from the semantic review.

#### Scenario: Calibrate and adjudicate judges

- **WHEN** judge calibration or live judging runs
- **THEN** the system SHALL report judge performance on known-good, known-bad, and subtly defective samples plus disagreement rates
- **AND** material disagreement SHALL require recorded human adjudication rather than majority voting.

### Requirement: Statistical power, budget, and smoke gates

The project SHALL freeze statistical power inputs, provider and total spending caps, and phase stop conditions before live execution.

#### Scenario: Refuse an under-specified paid run

- **WHEN** trial counts, the comparison margin or detectable difference, confidence method, maximum confound rate, pricing inputs, or spending caps are absent
- **THEN** the runner SHALL reject live execution and emit the missing requirements.

#### Scenario: Gate qualification behind smoke evidence

- **WHEN** the smoke phase completes
- **THEN** qualification SHALL remain blocked unless telemetry completeness, trial isolation, price reconciliation, judge calibration, route availability, and provider confound thresholds all pass.

#### Scenario: Stop at a spending boundary

- **WHEN** reconciled or conservatively estimated spend reaches a frozen provider or total cap
- **THEN** the runner SHALL stop scheduling new trials, preserve in-flight and completed evidence, and report the incomplete cells without broadening the budget.

### Requirement: Safety-constrained model promotion

The project SHALL recommend a cheaper model for a role only when it passes frozen safety, non-inferiority, reliability, cost, and integration gates.

#### Scenario: Promote a cheaper role candidate

- **WHEN** a candidate passes every authority, privacy, safety, and destructive-boundary check, is non-inferior on the untouched holdout under the frozen comparison rule, does not materially worsen confounds, retries, latency, context consumption, or repair burden, and lowers reconciled cost per accepted result
- **THEN** the system SHALL mark it eligible for human-reviewed promotion.

#### Scenario: Reject cheap-first retry economics

- **WHEN** a cheaper candidate succeeds only through additional failed attempts, escalation, or reviewer repair that removes its cost or latency advantage
- **THEN** the system SHALL retain the stronger baseline for that role.

#### Scenario: Require separate routing-policy approval

- **WHEN** a tournament produces a promotion recommendation
- **THEN** the system SHALL NOT mutate production routing automatically
- **AND** it SHALL require a separate reviewed routing-policy change and explicit human approval.

### Requirement: Whole-command integration evaluation

The project SHALL evaluate surviving role assignments through public command workflows when those workflows are implemented and available.

#### Scenario: Evaluate a command composition

- **WHEN** role finalists have passed holdout gates and a target command is available
- **THEN** the runner SHALL execute representative public workflows for `/explore`, `/feature`, `/apply`, `/apply:all`, or Recon autoresearch as applicable
- **AND** it SHALL verify authority, tool, telemetry, persistence, recovery, and failure-boundary integrations in addition to role quality.

#### Scenario: Report an unavailable command boundary

- **WHEN** a proposed command or authoritative integration is not implemented or authenticated
- **THEN** the system SHALL mark command-level acceptance blocked without treating role-level evidence as an end-to-end pass.

### Requirement: Canonical evaluation evidence publication

The project SHALL preserve immutable local tournament evidence and SHALL publish it to Recon when the authoritative Recon command contract is available.

#### Scenario: Publish a completed run to Recon

- **WHEN** a tournament phase has a frozen descriptor and terminal evidence bundle and Recon publication is available
- **THEN** Jcode SHALL submit the immutable run descriptor, raw evidence references, derived reports, and review outcomes through the authoritative Recon boundary.

#### Scenario: Recon publication is unavailable

- **WHEN** canonical Recon publication is unavailable or fails
- **THEN** Jcode SHALL retain a clearly labeled non-canonical immutable local bundle, report the publication blocker, and SHALL NOT claim canonical persistence.

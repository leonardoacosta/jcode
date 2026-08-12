## 1. Contracts and Offline Validation

- [x] 1.1 Define versioned schemas for experiment descriptors, model routes, role cells, corpus partitions, pricing snapshots, power budgets, trial events, judge records, adjudications, aggregates, and publication receipts.
- [x] 1.2 Implement offline descriptor validation with content-addressed experiment IDs and explicit detection of oracle-affecting mutations.
- [x] 1.3 Implement route-availability, tool-permission, fixture-revision, environment-hash, budget-arithmetic, pricing-completeness, and isolation-prerequisite checks that make no provider calls.
- [x] 1.4 Add deterministic tests proving incomplete power inputs, spending caps, judge assignments, cache policy, or stop conditions fail closed before paid execution.
- [x] 1.5 Add a dry-run cost estimator that emits per-provider and total conservative bounds without scheduling trials.

## 2. Role and Corpus Registry

- [x] 2.1 Define the bounded role taxonomy for extraction/classification, mechanical editing, semantic synthesis, proposal/task authoring, normal implementation, frontier implementation, adversarial review, autoresearch candidate generation, and model-free deterministic work.
- [x] 2.2 Define a checked-in model candidate registry with exact Jcode route IDs, provider families, versions or aliases, supported reasoning controls, context/output limits, tool capabilities, and role eligibility.
- [x] 2.3 Add development, qualification, holdout, and shadow partition manifests with immutable fixture digests and validation preventing cross-partition reuse.
- [x] 2.4 Adapt applicable task-decomposition and discovery fixtures by reference without changing their historical contracts.
- [x] 2.5 Add role-specific normal, boundary, adversarial, negative/control, refusal/escalation, provider/tool-failure, and long-context fixtures for implementation, review, scheduling, and autoresearch gaps.
- [x] 2.6 Add leakage validation that rejects prompts revealing expected models, tiers, tools, judges, reference answers, or hidden holdout content.

## 3. Isolated Provider-Aware Runner

- [x] 3.1 Implement provider-aware randomized block construction with frozen seeds and per-provider concurrency limits.
- [x] 3.2 Implement isolated trial workspaces or immutable input bundles, environment scrubbing, output ownership, and attempt-scoped identifiers.
- [x] 3.3 Integrate supported Jcode model routes and capture explicit unavailable or unauthenticated route failures without silent substitution.
- [x] 3.4 Implement separate warm-cache and cold-cache strata with provider-native cache metadata.
- [x] 3.5 Record every retry as a separate metered attempt linked to its predecessor and classify provider, tool, and infrastructure failures as confounded.
- [x] 3.6 Add safety stops that prevent payments, credential changes, third-party messages, deployments, destructive actions, and unapproved external mutation while preserving the observed decision trace.
- [x] 3.7 Add interruption-safe resume from the frozen descriptor and terminal attempt receipts without relying on conversation state.

## 4. Telemetry and Cost Accounting

- [x] 4.1 Emit immutable raw events for request and response digests, token classes, reasoning controls, cache activity, context growth, tool calls, shell activity, artifacts, failures, repairs, and acceptance outcomes.
- [x] 4.2 Capture queue time, time to first token, model time, tool time, judge time, total wall time, timeout state, and provider confounds.
- [x] 4.3 Implement runtime-price reconciliation and retain both original and normalized price reports derived from unchanged raw usage.
- [x] 4.4 Compute acceptance rate, confound rate, defect escape rate, repair burden, latency percentiles, total cost, and cost per accepted result without hiding failed or retried attempts.
- [x] 4.5 Add schema, replay, collision, partial-write, and interrupted-commit tests for raw event and aggregate persistence.

## 5. Deterministic Checks and Semantic Judging

- [ ] 5.1 Map every fixture to deterministic schemas, tests, artifact validators, policy checks, and safety boundaries that run before semantic judgment.
- [x] 5.2 Implement candidate anonymization that removes model and provider identity without removing task-relevant evidence.
- [x] 5.3 Define versioned semantic rubrics and independent judge assignment rules, including cross-provider cold review for high-risk roles.
- [x] 5.4 Add known-good, known-bad, and subtly defective calibration samples and report judge false-positive, false-negative, abstention, and disagreement rates.
- [x] 5.5 Implement immutable judge receipts, invalidation after candidate mutation, and recorded human adjudication for material disagreements.
- [x] 5.6 Add tests proving a candidate model cannot be its own sole judge and majority voting cannot settle a material disagreement.

## 6. Phased Tournament Control

- [x] 6.1 Implement `validate` phase output that proves no provider traffic occurred.
- [x] 6.2 Implement the smoke phase with one bounded trial per active cell and gates for telemetry completeness, isolation, price reconciliation, judge calibration, route availability, and confound thresholds.
- [x] 6.3 Implement qualification blocks with frozen repetitions, randomization, concurrency, and phase-level spending enforcement.
- [x] 6.4 Implement finalist selection from qualification evidence while preserving holdout and shadow blindness.
- [ ] 6.5 Implement untouched holdout execution and the frozen non-inferiority, safety, reliability, latency, repair, and cost-per-accepted-result comparison.
- [x] 6.6 Implement provider and total spending stops that cease new scheduling, preserve in-flight evidence, and report incomplete cells.
- [x] 6.7 Generate a human-reviewable promotion report without mutating production routing.

## 7. Whole-Command and Authority Integrations

- [ ] 7.1 Add an `/explore` integration adapter and representative public workflows once the native command is available.
- [ ] 7.2 Add a `/feature` integration adapter covering proposal authoring, deterministic validation, cold review, and authority boundaries once the native command is available.
- [ ] 7.3 Add `/apply` and `/apply:all` adapters covering risk selection, scheduling, isolation, review, verification, recovery, and partial-progress behavior once those commands are available.
- [ ] 7.4 Add a Recon autoresearch adapter covering frozen descriptors, candidate generation, deterministic KEEP/REVERT authority, restoration, budgets, resume, review, and finalization once available.
- [x] 7.5 Mark unavailable commands, integrations, routes, or authentication as acceptance-blocked without promoting role-level evidence to an end-to-end pass.
- [ ] 7.6 Verify Jcode owns experiment policy and evidence, Orca owns selected runtime resources, providers own execution usage, and Recon owns canonical publication.

## 8. Recon Publication and Evidence Recovery

- [x] 8.1 Define the immutable local evidence bundle and clearly distinguish canonical from non-canonical persistence.
- [ ] 8.2 Implement canonical Recon publication through its authoritative command with descriptor, raw evidence references, aggregates, judge outcomes, and human decisions.
- [x] 8.3 Add failed, unavailable, duplicate, collision, lock-contention, interrupted-publication, and replay tests for the Recon boundary.
- [x] 8.4 Preserve a non-canonical immutable local bundle and explicit blocker when Recon publication cannot complete.
- [ ] 8.5 Add query and verification round-trip checks proving a published run can be retrieved and matched to the original descriptor and evidence digests.

## 9. Acceptance, Documentation, and Rollout

- [x] 9.1 Document experiment authoring, candidate selection, corpus rules, power and spending inputs, cache controls, smoke gates, judging, adjudication, promotion, and Recon publication.
- [x] 9.2 Add a fixture-only acceptance run covering offline validation, zero-provider execution, missing-budget failure, leakage rejection, and unavailable-route failure.
- [ ] 9.3 Run an explicitly approved minimal paid smoke tournament and verify complete telemetry, isolation, cost reconciliation, confound classification, and stop behavior through the public runner interface.
- [ ] 9.4 Run qualification and holdout only after smoke evidence passes and a separate explicit provider budget is approved.
- [x] 9.5 Verify no tournament result changes production routing without a separate reviewed change and human approval.
- [x] 9.6 Run strict OpenSpec validation, requirement-to-task traceability, edge-case review, and a final cross-provider cold review of the unchanged artifacts and implementation evidence.

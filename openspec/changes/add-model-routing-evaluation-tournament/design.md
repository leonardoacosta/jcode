## Context

Jcode already has deterministic task-decomposition and discovery evaluation surfaces, a real swarm interface with model selection and fail-closed route handling, proposed native `/explore`, `/feature`, `/apply`, and `/apply:all` workflows, and a proposed Recon-owned autoresearch lifecycle. Current routing recommendations were derived from official pricing, model capability documentation, source-contract analysis, and bounded live probes. Those inputs are useful but do not establish role-specific reliability or cost per accepted result.

The evaluation must compare models fairly despite provider-specific queues, caches, rate limits, context accounting, reasoning controls, tool integrations, and transient failures. It must maximize telemetry without letting concurrency change the measured workload, and it must keep deterministic authority, safety policy, and human acceptance outside model self-assessment.

## Goals / Non-Goals

**Goals:**

- Measure each reasonable model candidate against frozen role-level tasks before evaluating whole commands.
- Run independent cells concurrently through randomized provider-aware blocks while preserving isolation and reproducibility.
- Record raw, immutable telemetry sufficient to recompute quality, latency, reliability, and cost under later pricing snapshots.
- Separate deterministic evidence, blind semantic judgment, provider confounds, repair burden, and human decisions.
- Promote a cheaper model only when it satisfies safety and non-inferiority gates on untouched holdout tasks.
- Publish canonical evidence to Recon when its authoritative command is available.

**Non-Goals:**

- Implement the proposed native commands or Recon autoresearch command in this change.
- Benchmark every published model regardless of Jcode route availability or role plausibility.
- Treat historical artifacts, token overlap, a single model judge, or majority voting as ground truth.
- Automatically change production routing from a tournament result.
- Spend provider budget before smoke, telemetry, calibration, isolation, and cost gates succeed.

## Decisions

### 1. Use a blocked randomized tournament instead of one unrestricted fan-out

The runner creates provider-aware blocks with fixed per-provider concurrency. Within each block it randomizes fixture and model order, then fans independent trials out and joins only after every terminal event is recorded. This preserves high parallelism while reducing queue saturation, temporal provider bias, and shared-resource interference.

Rejected alternatives:

- A single full-factorial wave is faster but confounds provider throttling, cache warming, and host contention with model quality.
- A fully sequential tournament is easier to reason about but too slow and samples providers at different times.

### 2. Qualify role-model pairs before whole-command combinations

The first stage evaluates bounded roles: extraction/classification, mechanical editing, semantic synthesis, proposal/task authoring, normal implementation, frontier implementation, adversarial review, and autoresearch candidate generation. Deterministic scheduling, validation, hashing, locking, budgeting, metric comparison, and KEEP/REVERT authority receive a model-free baseline rather than a model tournament.

Only role finalists enter whole-command `/explore`, `/feature`, `/apply`, `/apply:all`, and Recon autoresearch workflow evaluations. This prevents spending frontier-model budget on already dominated role-model pairs.

### 3. Freeze an immutable experiment descriptor

Before provider traffic, the runner writes a content-addressed descriptor containing:

- role taxonomy and task-suite digests;
- development, qualification, holdout, and shadow partitions;
- exact model route IDs, provider families, model versions, reasoning controls, context/output limits, and tool permissions;
- fixture commits, environment hashes, seeds, timeouts, cache mode, concurrency limits, and retry policy;
- deterministic checks, judge assignments, rubric versions, calibration samples, and adjudication rules;
- runtime and normalized pricing snapshots;
- trial counts, minimum detectable difference or declared non-inferiority margin, confidence method, maximum confound rate, provider and total spending caps, and stop conditions.

Any oracle-affecting change creates a new experiment ID. Resume uses the frozen descriptor and persisted trial receipts, never conversation memory.

### 4. Keep four disjoint corpus partitions

- Development tasks debug the harness and never contribute to model selection.
- Qualification tasks select role finalists.
- Holdout tasks provide the promotion decision and remain untouched until finalist selection is frozen.
- Shadow tasks measure future regressions and never tune the current tournament.

Every role includes normal, boundary, adversarial, negative/control, refusal/escalation, provider/tool failure, and relevant long-context cases. Prompts must not reveal expected tools, route tiers, judges, or reference answers. Existing task-decomposition and discovery fixtures may be reused by digest, but missing implementation, review, scheduling, and autoresearch cases require new fixtures.

### 5. Isolate trials and account for every attempt

Each trial receives an isolated checkout or immutable input bundle, output directory, environment scrub, and attempt ID. Warm-cache and cold-cache runs are separate strata. A retry is a new metered attempt linked to its predecessor and is never hidden. Provider, tool, or infrastructure failures are classified as confounded and excluded from quality denominators while remaining visible in reliability, cost, and latency reports.

The runner stops mutation-capable tasks at declared safety boundaries and prohibits real payments, credential changes, third-party messages, deployment, destructive operations, or unapproved external mutation.

### 6. Capture raw events before deriving aggregates

Per trial, record:

- experiment, block, role, fixture, model, provider, version, seed, reasoning controls, cache stratum, and attempt lineage;
- request and response digests, input, cached-input, cache-write, reasoning, and output tokens;
- runtime price, normalized price, estimated and reconciled cost;
- queue time, time to first token, model time, tool time, judge time, total wall time, and timeout state;
- tool calls, failures, shell commands, bytes read/written, context growth, retries, truncations, refusals, and provider errors;
- produced artifacts, deterministic checks, rubric evidence, defects found by cold review, repair attempts, acceptance result, and human adjudication.

Aggregates include acceptance rate, confound rate, defect escape rate, reviewer repair burden, latency percentiles, raw cost, and cost per accepted result. Raw evidence remains immutable so later price normalization and analysis do not rewrite history.

### 7. Use deterministic checks first and calibrated blind judges second

Mechanically verifiable behavior is judged by schemas, tests, artifact validators, policy checks, and exact safety boundaries. Semantic candidates are anonymized before judging. Normal-risk roles use at least two independent rubric observations when semantic judgment is material. High-risk roles require a cold judge from another provider family. The author model cannot be the sole judge of its own output.

Calibration includes known-good, known-bad, and subtly defective samples. Judge false-positive, false-negative, disagreement, and abstention rates are reported. Material disagreement goes to human adjudication rather than majority voting. Artifact mutation invalidates affected judge receipts.

### 8. Promote on safety-constrained cost per accepted result

A cheaper candidate may replace the current baseline only when it:

- passes every non-negotiable authority, safety, destructive-boundary, and privacy check;
- is non-inferior on the frozen holdout under the declared margin and confidence method;
- does not materially increase confounds, retries, latency, context consumption, or reviewer repair;
- has lower reconciled cost per accepted result;
- passes applicable whole-command integration cases;
- receives explicit human approval before routing policy changes.

The tournament pre-classifies roles and risk. It does not default to cheap-first escalation because failed cheap attempts consume budget, latency, and context and may contaminate later work.

### 9. Gate execution in phases

1. `validate`: verify descriptor schema, suite partitions, route availability, budget arithmetic, pricing completeness, judge calibration inputs, and isolation prerequisites without provider traffic.
2. `smoke`: run one bounded trial per active cell and stop unless telemetry completeness, output isolation, price reconciliation, judge calibration, and provider confounds meet thresholds.
3. `qualify`: run randomized repeated role-level blocks.
4. `select`: freeze finalists from qualification evidence.
5. `holdout`: evaluate finalists against untouched tasks.
6. `integrate`: evaluate surviving role assignments through whole-command workflows once those commands exist.
7. `publish`: create immutable local evidence and publish canonically to Recon when supported.
8. `approve`: require human approval before applying any routing change.

### 10. Preserve authority across Jcode, Orca, providers, and Recon

Jcode owns the experiment descriptor, selected cells, budget, scheduling policy, trial identity, local evidence, safety stops, and promotion recommendation. Providers own model execution and report usage evidence. Orca may own worktrees, workers, and runtime resources when selected, but runtime observations do not settle evaluation outcomes. Recon owns canonical research/evaluation publication when its command contract is available.

Missing telemetry never weakens correctness. Missing route authentication, model availability, provider usage, price reconciliation, isolation, judge calibration, or canonical publication is surfaced explicitly and fails the affected phase closed.

## Risks / Trade-offs

- **[Risk] Full factorial growth makes the tournament unaffordable** → qualify role-level candidates first, cap cells and trials, and require a frozen power and spending budget.
- **[Risk] Provider concurrency changes latency and error rates** → use fixed provider-aware blocks and report queue/provider confounds separately.
- **[Risk] Prompt or fixture leakage rewards memorization** → use disjoint partitions, content hashes, anonymized judging, and untouched holdout tasks.
- **[Risk] Judge models reproduce provider bias** → calibrate judges, require cross-provider cold review for high risk, report disagreement, and use human adjudication.
- **[Risk] Cache effects make prices incomparable** → stratify warm and cold cache runs and preserve provider-native cache telemetry.
- **[Risk] Provider pricing changes during or after a run** → store runtime prices and raw usage, then derive separate normalized-price reports.
- **[Risk] Confounded attempts hide reliability problems** → exclude them only from quality denominators while including them in reliability and total-cost reporting.
- **[Risk] Existing fixtures overfit planning behavior** → add role-specific implementation, review, scheduling, refusal, and autoresearch cases.
- **[Risk] A benchmark result silently changes production behavior** → require a separate reviewed routing-policy change and human approval.
- **[Risk] Recon is unavailable** → retain immutable, explicitly non-canonical local evidence and fail canonical publication closed.

## Migration Plan

1. Add descriptor, model registry, telemetry, result, and evidence schemas plus offline validation.
2. Add role and corpus registries while adapting existing eval fixtures without changing their historical meaning.
3. Add isolated provider-aware smoke and qualification execution with dry-run cost estimation.
4. Add deterministic scoring, anonymization, calibrated judges, adjudication records, and aggregation.
5. Add holdout selection, promotion reporting, and whole-command adapters as command surfaces become available.
6. Add Recon publication adapter after its authoritative command contract exists.
7. Run only offline validation first, then an explicitly approved smoke budget, then qualification and holdout gates.
8. Roll back by disabling live execution; frozen descriptors and immutable evidence remain readable.

## Open Questions

None. Trial counts, non-inferiority margins, confound thresholds, and spending caps are experiment inputs that MUST be selected and frozen during run preparation rather than hardcoded globally.

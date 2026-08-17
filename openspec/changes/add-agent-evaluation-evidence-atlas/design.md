## Context

The approved design is a Hybrid Evidence Atlas within the existing brown-on-brown Jcode field manual. It combines a concise decision surface with an auditable ledger. Its primary evidence tracks are (1) the measured OAuth model-routing smoke tournament and (2) the recommendation-approval DAG used to review and remediate the command-system microsite. The page must make findings easy to consume while retaining direct evidence, caveats, and authority boundaries.

The repository already contains the visual system, static/browser validators, model-routing receipts, and WS publication path. This change extends those surfaces rather than introducing a new documentation framework.

## Goals / Non-Goals

**Goals:**

- Make evaluation outcomes, findings, telemetry, dispositions, and limitations understandable from one page.
- Preserve enough structured evidence to audit every displayed claim without consulting mutable conversation history.
- Distinguish measured evidence, documented workflow, inference, and unavailable data.
- Show how Fable, Sol, Luna, deterministic checks, cold reviewers, and human approvals participate without implying interchangeable authority.
- Keep the page static, offline-capable, keyboard accessible, responsive, and visually coherent with the existing Atlas.
- Verify the final Jcode artifact and its deployed WS copy through public acceptance paths.

**Non-Goals:**

- Run a new model tournament or manufacture missing telemetry.
- Establish a universal model winner from one smoke fixture.
- Normalize provider-native token classes into a false cross-provider equivalent.
- Publish private raw transcripts, credentials, third-party personal data, or reconstructed prompts.
- Mutate routing policy, approve a model promotion, or replace Recon as canonical research authority.
- Add a client framework, remote chart library, live Grafana embed, or runtime API dependency.

## Decisions

### 1. Add one evaluation destination with six progressively disclosed sections

`agent-evaluations.html` is linked from the System Atlas, field-manual index, evaluation tournament page, telemetry page, and Daily-Driven Ecosystem page. It contains:

1. **Decision brief:** outcome, confidence, authorization state, and explicit non-conclusions.
2. **Findings ledger:** filterable by track, severity, provider/model, claim status, and disposition.
3. **Run explorer:** candidates, prompts or prompt availability, outputs or output availability, judges, scores, provider-native timings and tokens, cost, steering, and limitations.
4. **Review DAG:** Fable discovery → Sol cross-provider refinement/autoresearch → human approve/reject/defer/modify → Luna implementation → deterministic and cold verification.
5. **Telemetry:** side-by-side provider-native observations with visible comparability caveats.
6. **Evidence map:** direct links from every material claim to a manifest record and tracked source.

The brief is readable without JavaScript. JavaScript may enhance filtering and disclosure, but every record remains present and navigable when scripting is disabled.

Rejected alternatives:

- A dashboard-first page obscures authority, limitations, and qualitative findings behind summary metrics.
- A raw receipt browser is maximally complete but too difficult for routine use.
- Separate pages per run fragment the small current corpus and duplicate navigation.

### 2. Use one versioned manifest as the rendering and validation contract

`agent-evals.json` has a schema version, generated-at timestamp, source revision, evidence digest, tracks, and normalized entity collections for evaluations, runs, candidates, reviewers, findings, dispositions, telemetry, and evidence sources.

Every finding records:

- stable ID and track;
- severity and claim status;
- source provider/model or deterministic source;
- evidence pointers and sanitized summary;
- disposition (`accepted`, `rejected`, `deferred`, `modified`, `implemented`, `verified`, or `unresolved`);
- implementation commit when applicable;
- verification result and verifier independence;
- known limitations.

The HTML may contain a static projection for offline/no-JavaScript use, but the validator requires semantic equivalence with the manifest. The manifest is implementation-owned and does not replace the underlying receipts.

### 3. Include two bounded evidence tracks

**Measured tournament track:** the frozen OAuth smoke descriptor, accepted attempts, Claude Fable 5 and OpenAI GPT-5.5 provider-native telemetry, deterministic baseline, anonymized candidates, both judge receipts, steering digest, cost evidence, and human interpretation. It states that both candidates passed one fixture, judge preference split, and no routing mutation was authorized.

**Microsite review track:** the approved alternating-provider workflow and retained findings that materially changed or verified the microsite. Findings are grouped by shared foundations, command core, routing/evaluation, telemetry/ecosystem, upper Atlas, lower Atlas, and cross-page visual/accessibility refinement. Each retained finding identifies its discovery source, Sol refinement or autoresearch disposition, human decision when recorded, Luna implementation evidence when applicable, and cold-verification outcome.

Review events whose full prompt or output was not durably persisted are summarized as `documented` with the unavailable fields explicit. They are never reconstructed from memory.

### 4. Keep claim status and authority visually explicit

The page uses the existing evidence vocabulary plus `unavailable`:

- `measured`: directly supported by frozen telemetry or receipts;
- `documented`: supported by repository-authoritative artifacts or retained review records;
- `inferred`: reasoned from cited evidence with confidence and limitations;
- `unavailable`: a requested field was not captured or cannot be safely published.

Evaluation output, reviewer recommendations, human dispositions, implementation state, verification state, and production routing authority are separate fields. A score or recommendation cannot render as an approved routing decision.

### 5. Preserve provider-native telemetry rather than inventing equivalence

Token counts retain provider and metric names such as reported work tokens. Timings identify whether they represent persisted response latency, execution time, or another measured boundary. The page explicitly records that provider queue time, provider-internal reasoning tokens, and true provider TTFT were unavailable in the smoke evidence where applicable.

Charts use accessible HTML/SVG with text tables. No visual compares unlike token classes on a shared quantitative axis without a visible non-comparability warning.

### 6. Treat review alternation and approval as first-class evaluation evidence

The review DAG encodes the user's approved policy:

- Fable receives exactly one initial discovery pass for a review assignment.
- Sol high performs cross-provider autoresearch or refinement rather than Fable refining itself.
- Work passes between providers so neither model immediately refines its own prior iteration.
- Humans approve, reject, defer, or modify recommendations before implementation.
- Luna implements approved bounded recommendations.
- Deterministic checks and fresh cold reviewers verify the result.

The page distinguishes planned DAG structure from observed executions and labels missing run-level telemetry as unavailable.

### 7. Extend existing validators with stable evaluation diagnostics

The static validator gains `DOCS-EVALS` diagnostics and verifies:

- schema version and required entity collections;
- globally unique stable IDs and valid references;
- allowed claim statuses and dispositions;
- every displayed material claim, score, finding, and limitation maps to the manifest;
- source paths exist, revisions and digests are current, and unsafe source classes are absent;
- static HTML and manifest records are semantically equivalent;
- token/timing units retain provider-native labels and caveats;
- routing authority is never inferred from score or recommendation;
- all internal links and Atlas navigation are reciprocal.

Negative self-tests cover duplicate IDs, dangling references, missing limitations, reconstructed unavailable evidence, false token normalization, unsupported winner claims, automatic-routing language, stale digests, unsafe source references, and HTML/manifest drift.

### 8. Validate the real reader journey and deployed integration

The browser validator visits the index, System Atlas, Agent Evaluations page, tournament, telemetry, and ecosystem pages at 1440x1000 and 393x852 with JavaScript enabled and disabled. It exercises filters and disclosure controls when JavaScript is enabled, keyboard traversal, focus visibility, zero horizontal overflow, local-only assets, evidence links, console/network failures, and table/diagram fallbacks.

After Jcode acceptance, implementation copies the settled files into the WS documentation source, builds the real docs portal, pushes the scoped change to `dev`, monitors the exact Azure pipeline run, and checks the deployed Entra-gated Agent Evaluations URL. The live route and pipeline receipt are added to the final acceptance record before ntfy notification.

## Source Boundaries

- Tournament descriptor and results: `evals/model-routing/runs/oauth-smoke-2026-08-12.json` and its sibling detail files.
- Tournament contract: `openspec/changes/add-model-routing-evaluation-tournament/` and `evals/model-routing/README.md`.
- Existing field manual and source matrix: `docs/diagrams/jcode-command-system/`.
- Review DAG contract and approved dispositions: retained digest-bound review artifacts and repository/session evidence that can be safely referenced.
- Existing documentation feature: `openspec/changes/add-command-system-documentation-microsite/`.
- WS publication: the existing wholesale docs-site sync/build process and Azure Static Web Apps pipeline.

## Privacy and Redaction

- Include only sanitized prompts and outputs already retained as repository-safe evidence.
- Exclude OAuth tokens, access tokens, credential values, private URLs containing active tokens, unrelated prompts, and third-party personal data.
- Store source digests and sanitized references rather than copying private transcript bodies.
- Mark unavailable evidence honestly rather than weakening redaction.

## Dependencies and Conflicts

- Depends on the existing command-system microsite and model-routing smoke evidence.
- The `add-command-system-documentation-microsite` change must be archived before this change so its `command-system-docs` capability exists as the merge base.
- Depends on the existing browser runtime used by `scripts/test-command-system-docs-browser.py`.
- WS deployment requires the separate wholesale repository, its docs build, Azure authentication, and the Entra-gated Static Web Apps route.
- The currently modified task files for `add-artifact-action-palette` and `optimize-orca-command-center-orchestration` are outside scope and must remain untouched.
- Untracked `.superpowers/brainstorm/` artifacts are reference-only and must not enter the feature commit.

## Done Means

- The new page and manifest expose both approved evidence tracks with truthful limitations and authority boundaries.
- Static, negative, Atlas-source, browser, strict OpenSpec, and digest checks pass.
- A fresh independent review traces every requirement to scenarios, tasks, checks, and unchanged artifact bytes.
- The WS portal build and exact deployment pipeline pass, the live route is verified as far as Entra permits, and the URL is delivered through ntfy.

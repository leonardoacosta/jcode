## Why

Jcode has recommended model tiers for exploration, proposal authoring, implementation, review, and autoresearch, but those recommendations are based on pricing, capability documentation, and bounded swarm probes rather than a reproducible comparison across roles. We need a repository-native tournament that measures quality, safety, latency, and cost per accepted result before routing defaults are encoded into the proposed native commands.

## What Changes

- Add a frozen experiment descriptor for blocked, randomized, provider-aware model tournaments.
- Add role-level evaluation suites for extraction, mechanical editing, semantic synthesis, proposal authoring, normal and frontier implementation, adversarial review, and autoresearch candidate generation.
- Reuse deterministic Jcode evaluation evidence where applicable and add isolated live-run orchestration, model/provider registries, cache controls, retry accounting, and confound classification.
- Capture raw per-trial telemetry for tokens, cost, latency, tool use, artifacts, failures, repair burden, and acceptance outcomes, with derived cost-per-accepted-result reporting.
- Add blind multi-judge evaluation with deterministic checks first, cross-provider cold review for high-risk roles, calibration samples, disagreement tracking, and human adjudication gates.
- Add qualification, holdout, shadow, and whole-command phases with explicit statistical power, spending, stop, non-inferiority, safety, and promotion criteria.
- Persist immutable run evidence to Recon when the Recon autoresearch/import boundary is available; fail closed or retain a clearly non-canonical local run bundle when it is not.
- Do not launch a full paid tournament until a smoke wave proves telemetry completeness, isolation, pricing reconciliation, judge calibration, and acceptable provider confound rates.

## Capabilities

### New Capabilities

_None._

### Modified Capabilities

- `evals`: Extend Jcode's evaluation contract from task-decomposition fixtures to reproducible live model-routing tournaments with frozen inputs, provider-aware execution, comprehensive telemetry, calibrated judging, and evidence-based promotion gates.

## Impact

- Affects `evals/`, evaluation scripts or a dedicated runner crate, provider/model route discovery, swarm or harness execution boundaries, telemetry schemas, isolated checkout management, and OpenSpec command-role fixtures.
- Integrates with the proposed native `/explore`, `/feature`, `/apply`, and `/apply:all` workflows as evaluated consumers rather than implementing those commands in this change.
- Integrates with Recon for canonical evidence publication when available, without making model execution depend on mutable conversation state.
- Incurs metered provider traffic only after explicit smoke and budget gates pass.

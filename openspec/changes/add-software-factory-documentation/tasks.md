## 1. Documentation foundation

- [x] 1.1 Create `docs/factory/` and define the shared metadata/claim-status convention.
- [x] 1.2 Add the factory index/introduction with lifecycle diagram, terminology, maturity summary, and navigation.
- [x] 1.3 Add a reciprocal link from `docs/README.md` to the factory index.

## 2. Lifecycle and architecture pages

- [x] 2.1 Write `lifecycle.md` covering intent through feedback and delivery.
- [x] 2.2 Write `architecture.md` covering agent kernel, worker runtime, workflow orchestration, gates, and command center.
- [x] 2.3 Write `artifacts-and-provenance.md` covering specifications, plans, patches, traces, results, approvals, and evidence.
- [x] 2.4 Write `workers-and-orchestration.md` covering single-worker, parallel, orchestrator-worker, swarm, retry, and resume patterns.
- [x] 2.5 Write `isolation-and-execution.md` covering worktrees, containers, sandboxes, local/remote workers, and ownership boundaries.

## 3. Quality, governance, and learning pages

- [x] 3.1 Write `gates-and-approvals.md` covering deterministic checks, risk gates, approval packets, and escalation.
- [x] 3.2 Write `evaluation-and-regression.md` covering outcome/trajectory evaluation, regression suites, and benchmark limitations.
- [x] 3.3 Write `observability.md` covering traces, tool calls, state transitions, artifacts, metrics, and replayability.
- [x] 3.4 Write `governance-and-risk.md` covering permissions, reversibility, human-on-the-loop controls, privacy, and external side effects.
- [x] 3.5 Write `feedback-and-learning.md` covering failure taxonomy, spec updates, skill updates, tool improvements, and eval corpus growth.

## 4. Comparative and repository mapping pages

- [x] 4.1 Write `open-harness-landscape.md` covering Pi, Hermes, OpenClaw, OpenCode, Goose, OpenHands, SWE-agent, and mini-SWE-agent with primary-source links.
- [x] 4.2 Write `jcode-mapping.md` separating observed current capabilities, proposed target capabilities, and explicit gaps with source paths.
- [x] 4.3 Write `sources-and-limitations.md` with source inventory, capture dates, evidence classes, unresolved questions, and research limitations.

## 5. Validation and handoff

- [x] 5.1 Validate every internal Markdown link and referenced repository path.
- [x] 5.2 Check that every material claim has an observed/proposed/external/open-question label and evidence pointer.
- [x] 5.3 Run Markdown/documentation checks and `git diff --check`.
- [x] 5.4 Confirm no runtime code, credentials, private transcripts, or unrelated active OpenSpec changes were modified.
- [x] 5.5 Perform an independent reader-path review from `docs/README.md` → factory index → lifecycle and cross-cutting pages.

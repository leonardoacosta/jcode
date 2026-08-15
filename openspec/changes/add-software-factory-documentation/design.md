## Context

The approved structure is an index plus orthogonal reference pages under `docs/factory/`. Existing repository conventions place current architecture docs under `docs/`, forward-looking material under `docs/plans/` or `docs/proposals/`, and use OpenSpec for implementation authority. This change records research and proposed architecture without changing runtime behavior.

## Goals

- Make the factory lifecycle understandable from one entry point.
- Preserve the distinction between current implementation, design direction, and external research.
- Give each major lifecycle or cross-cutting concern a dedicated page.
- Link claims to repository paths or public sources and record limitations.
- Make the documentation useful for future architecture proposals and implementation planning.

## Non-goals

- Do not implement factory runtime behavior.
- Do not create a second research authority or copy private session evidence.
- Do not imply that a documented target state is already implemented.

## Decisions

### 1. Use an index plus orthogonal maps

Create `docs/factory/README.md` and dedicated pages:

- `lifecycle.md`
- `architecture.md`
- `artifacts-and-provenance.md`
- `workers-and-orchestration.md`
- `isolation-and-execution.md`
- `gates-and-approvals.md`
- `evaluation-and-regression.md`
- `observability.md`
- `governance-and-risk.md`
- `feedback-and-learning.md`
- `open-harness-landscape.md`
- `jcode-mapping.md`
- `sources-and-limitations.md`

The index is the reader's introduction and navigation surface. Dedicated pages hold depth so the index remains a map, not a duplicated encyclopedia.

### 2. Use explicit claim-status labels

Every page uses a compact metadata block and labels material claims as `observed`, `proposed`, `external research`, or `open question`. Observed claims cite Jcode paths or verified public interfaces. External claims cite primary sources. Proposed claims name their design status and do not present as current behavior.

### 3. Make the lifecycle the organizing spine

The canonical lifecycle is:

`intent → specification → planning → isolated execution → artifacts → gates → evaluation → approval/delivery → feedback`.

Cross-cutting pages explain the machinery that supports multiple stages.

### 4. Preserve authority boundaries

Repository source is authoritative for current Jcode behavior. OpenSpec is authoritative for approved implementation proposals. External research supports comparative findings and design rationale but does not establish Jcode behavior. Missing evidence is marked unavailable rather than reconstructed.

### 5. Keep the first implementation documentation-only

The initial change writes Markdown and updates the docs index only. Diagrams use Mermaid fenced blocks where helpful. No new renderer, JavaScript application, schema runtime, or generated artifact is required.

## Evidence boundaries

Primary Jcode evidence includes `README.md`, `ROADMAP_HANDOFF.md`, `docs/WORKFLOW_AUTOMATION_ROADMAP.md`, command-center, plan/DAG, memory, ambient, tool, skill, and verification sources. External evidence includes the official Pi, Hermes, OpenClaw, OpenCode, Goose, OpenHands, SWE-agent, and mini-SWE-agent repositories plus authoritative agent-pattern, evaluation, observability, governance, and delivery references gathered during research.

## Acceptance shape

A reader starting at `docs/factory/README.md` can reach every dedicated page, understand the lifecycle, distinguish current versus proposed behavior, and follow material claims to a repository path or public source. Internal links resolve, Markdown is readable, and no runtime or unrelated active change is modified.

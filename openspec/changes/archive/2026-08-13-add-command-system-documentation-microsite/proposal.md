## Why

Jcode's native command workflow, lane protocol, apply orchestration, model routing, evaluation tournament, telemetry evidence, and seven-layer agent stack are currently spread across OpenSpec changes, evaluation receipts, session history, harness-specific repositories, and standalone diagrams. A reader needs one coherent, illustrated entry point that explains both how commands flow and how the surrounding agent ecosystem evolved without reconstructing the design from implementation artifacts or personal recollection.

## What Changes

- Add a static brown-toned documentation microsite under `docs/diagrams/jcode-command-system/`.
- Add an index page that introduces the command journey and links to six focused command-system pages.
- Add a static System Atlas overview derived from `docs/diagrams/agent-stack-recreation.html` without animation or remote assets.
- Add seven linked layer pages for surface, orchestration, context, model, tools, runtime, and memory.
- Add a Daily-Driven Ecosystem page comparing Claude Code, Codex, Pi, Jcode, and cross-provider agents from repository, telemetry, and session evidence.
- Document command lifecycle, lane protocol, apply orchestration, model routing, evaluation tournament, telemetry/results, platform layers, evolution, and daily workflow.
- Use self-contained illustrations, locally rendered Mermaid diagrams with text fallbacks, and repository-grounded code snippets.
- Add responsive navigation, chapter breadcrumbs, next/previous links, evidence-class labels, and accessibility checks.
- Add deterministic link, structure, offline-asset, claim-provenance, telemetry-drift, contrast, and rendered-browser validation.

## Capabilities

### New Capabilities

- `command-system-docs`: A portable illustrated HTML handbook for Jcode's native command system, seven-layer agent stack, and daily-driven multi-harness ecosystem.

### Modified Capabilities

_None._

## Impact

- Adds files only under `docs/diagrams/jcode-command-system/`, including `sources.json`, plus a focused validation script.
- Reads existing OpenSpec changes, evaluation evidence, `docs/diagrams/agent-stack-recreation.html`, harness repositories, and session telemetry as source material without changing their authority.
- Does not change command behavior, routing policy, telemetry emission, provider integration, or production state.
- Must coexist with active changes for native `/explore`, `/feature`, `/apply`, `/apply:all`, Orca Command Center orchestration, and model-routing evaluation without editing those lanes.

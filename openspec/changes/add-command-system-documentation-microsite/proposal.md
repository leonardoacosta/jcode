## Why

Jcode's native command workflow, lane protocol, apply orchestration, model routing, evaluation tournament, and telemetry evidence are currently spread across several OpenSpec changes, evaluation receipts, and standalone diagrams. A reader needs one coherent, illustrated entry point that explains how these concepts fit together without reconstructing the design from implementation artifacts or session history.

## What Changes

- Add a static brown-toned documentation microsite under `docs/diagrams/jcode-command-system/`.
- Add an index page that introduces the system journey and links to six focused concept pages.
- Document command lifecycle, lane protocol, apply orchestration, model routing, evaluation tournament, and telemetry/results.
- Use self-contained illustrations, locally rendered Mermaid diagrams with text fallbacks, and repository-grounded code snippets.
- Add responsive navigation, chapter breadcrumbs, next/previous links, and accessibility checks.
- Add deterministic link, structure, offline-asset, and content-presence validation plus representative browser checks.

## Capabilities

### New Capabilities

- `command-system-docs`: A portable illustrated HTML handbook for Jcode's native command and evaluation system.

### Modified Capabilities

_None._

## Impact

- Adds files only under `docs/diagrams/jcode-command-system/`, including `sources.json`, plus a focused validation script.
- Reads existing OpenSpec changes and evaluation evidence as source material without changing their authority.
- Does not change command behavior, routing policy, telemetry emission, provider integration, or production state.
- Must coexist with active changes for native `/explore`, `/feature`, `/apply`, `/apply:all`, Orca Command Center orchestration, and model-routing evaluation without editing those lanes.

## Why

The current Brown and Decus maps technically embed the approved Azure line-art sprite, but the marks are too small to carry identity while opaque cube abbreviations dominate the scene. Runtime, network-containment, governance, and Azure DevOps delivery stories are also combined in one view, making the resulting topology difficult to read.

## What Changes

- Add a coordinated three-view Azure topology presentation for each evidence-backed system: Runtime, Network, and ADO Pipeline.
- Keep Runtime isometric, remove visible cube abbreviations, enlarge approved roof SVGs, and add readable label plates with the full resource name, service type, environment, and status.
- Add a nested Network view that explicitly renders Subscription, Resource Group, VNet, Subnet, and resource-card containment with labeled network connectors.
- Add an ADO Pipeline view that renders repository, validation/build, artifact, parallel stages or jobs, approvals and gates, and deployment targets from source-backed pipeline evidence.
- Introduce one versioned companion view sidecar, governed by a tracked JSON Schema, that references the existing scene resource IDs so all views share names, approved icons, types, statuses, evidence, and selection state without weakening the core isometric scene contract.
- Add keyboard-accessible, URL-addressable tabs and update the Brown and Decus private gallery to expose all three views without embedding overloaded maps.
- Add validators, regression tests, deterministic generation checks, a dependency-free Chromium acceptance harness, reproducible private-output receipts, and explicit rejection of abbreviation-only or unsupported-icon output.

## Capabilities

### New Capabilities

- `azure-topology-views`: Coordinated Runtime, Network, and ADO Pipeline views backed by one validated resource catalog and the approved Azure SVG vocabulary.

### Modified Capabilities

None.

## Impact

- Affects `skills/isometric-system-map/` renderer templates, Azure theme, validation scripts, references, fixtures, and tests.
- Adds a companion view-sidecar contract while preserving existing standalone `scene.json` compatibility and the current `render_canvas.py scene theme output` interface.
- Updates tracked generic examples under `docs/diagrams/` and regenerates complete, reproducible private Brown and Decus delivery bundles under untracked `output/system-maps/`.
- Uses only package-local HTML, CSS, JavaScript, Canvas, SVG sprite assets, and Python tooling. No runtime dependency or remote asset is added.
- Does not change source repositories, Azure resources, Azure DevOps definitions, deployment behavior, or evidence claims.

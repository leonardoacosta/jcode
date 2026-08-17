---
name: isometric-system-map
description: >-
  Analyze a real repository and turn its load-bearing architecture into an interactive,
  self-contained isometric system map with varied 3D buildings, typed directed paths, moving and
  inspectable payloads, flow controls, a legend, an explainer panel, and exact file citations. Use
  whenever the user asks for an isometric architecture map, 3D infrastructure map, system city,
  moving data dots, interactive codebase topology, or a visual map of control/data/deployment paths,
  even if they only say “make the repo visual” and do not name this skill. Prefer this over repo-map
  when spatial terrain and inspectable payload motion are the point; prefer blueprint for a classic
  time-sequenced swimlane diagram.
compatibility: Python 3.10+ and a modern browser. The renderer has no third-party dependencies.
allowed-tools: Read, Write, Bash, Glob, Grep
---

# Isometric System Map

Build an evidence-backed system terrain, not decorative architecture art. The fixed renderer owns
the page, isometric geometry, building silhouettes, color, animation, controls, responsive layout,
and accessibility. You author only contract JSON from repository evidence.

## Deliverables

Unless the caller supplies an output directory, create both files under `docs/diagrams/`:

```text
docs/diagrams/<repo>-isometric-map.json
docs/diagrams/<repo>-isometric-map.html
```

The JSON is the reviewable source of truth. The HTML is one self-contained artifact with no CDN,
external font, image, script, or stylesheet dependency.

## Workflow

### 1. Pin the evidence boundary

Record the exact repository, requested ref, commit, and analyzed scope before extracting anything.
Do not call a local ref “latest” unless freshness was actually proven. If you cannot fetch without
mutating a read-only source repository, say `local origin/main snapshot` (or the exact supplied ref)
and record its commit instead.

For a large estate, select one coherent architecture surface such as a foundation root, one
application, or one deployment pipeline. Do not compress an entire monorepo into unreadable terrain.

### 2. Trace real behavior before choosing buildings

Read entry points and their callers first. Establish:

- control plane: CLI, pipeline, scheduler, orchestrator, deployment root;
- owned runtime resources and reusable modules;
- externally owned or `existing` resources;
- data, identity, network, delivery, telemetry, and dependency paths;
- held, gated, deprecated, or not-yet-deployed surfaces;
- concrete payloads crossing each selected path.

Every node, edge, payload, and flow step needs at least one repo-relative citation. Step citations
must directly support that transition rather than relying only on the referenced edge. Use
`path/to/file:12-44` when line evidence is stable. Omit a claim rather than guessing it.

For Bicep, read [`references/extraction.md`](references/extraction.md) before authoring the map.

### 3. Curate the map

Keep the terrain discussable:

- 8-22 nodes is the normal range; the hard cap is 24.
- 2-5 zones is usually enough; the hard cap is 8.
- 3-6 named flows is ideal.
- A flow has 2-8 steps in most cases; the hard cap is 12.
- Use at least three building kinds when the source genuinely contains them.
- Separate deployment dependency from runtime data movement. A Bicep `dependsOn` is not an
  application data path.

Give each node a unique integer grid position from `0..12` on both axes. Spread adjacent flow nodes
by roughly two cells and keep the front half of the grid less dense than the rear so labels and
paths stay legible. Building size and height come from `kind`; never hand-style an individual node.

### 4. Author contract JSON only

Read [`references/contract.md`](references/contract.md). Start from
[`tests/fixtures/valid.json`](tests/fixtures/valid.json), replace every sample fact, then validate:

```bash
python3 skills/isometric-system-map/scripts/render.py --validate \
  docs/diagrams/<repo>-isometric-map.json
```

Fix every reported error. Do not hand-author or patch the generated HTML.

### 5. Render and inspect

```bash
python3 skills/isometric-system-map/scripts/render.py \
  docs/diagrams/<repo>-isometric-map.json \
  docs/diagrams/<repo>-isometric-map.html
```

Open the HTML and verify all of these in the real browser:

1. The terrain is visible and buildings do not collide.
2. The flow picker changes the highlighted path.
3. Pause/resume changes both motion and button text.
4. `Trace one step` advances through the selected flow.
5. Clicking a building opens purpose and implementation details.
6. Clicking the payload pauses it and opens payload details.
7. `How it is built` exposes exact citations.
8. Keyboard focus reaches controls, rail nodes, buildings, directed paths, payload, and tabs.
9. Reduced-motion mode starts paused and still exposes every step.
10. At mobile width the rail, map, and explainer stack without losing controls.

### 6. Report evidence, not vibes

Deliver the JSON and HTML paths, the analyzed commit, the selected scope, and a short list of named
flows. Call out any intentionally omitted or uncertain path. Never claim the map is exhaustive.

## Renderer boundary

- Never add product-specific CSS, icons, logos, or secrets to the template for one map.
- Never draw a building that exists only to make the layout look balanced.
- Never infer live deployment status from the presence of a Bicep file alone.
- Never expose secret values, connection strings, tenant IDs, subscription IDs, or private payload
  samples. Name the role and cite the safe source location instead.
- Use `paper` only when the user asks for a print/blueprint look; otherwise use `midnight`.

## Relationship to nearby skills

| Need | Use |
| --- | --- |
| Spatial 3D terrain plus moving inspectable payloads | `isometric-system-map` |
| General repository role graph with cards and rails | `repo-map` |
| Time-ordered request sequence with actors/swimlanes | `blueprint` |
| One-off flexible explanatory HTML page | `wayfinder` |

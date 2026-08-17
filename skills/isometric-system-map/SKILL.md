---
name: isometric-system-map
description: >-
  Turn a real repository, infrastructure estate, or process topology into an evidence-backed
  isometric system-map image or interactive scene. Use when the user asks for an isometric
  architecture map, 3D infrastructure diagram, system city, repository terrain, moving data dots,
  or a visual map of control, data, deployment, network, identity, or telemetry paths. Teaches the
  reusable projection, cube, routing, payload, layout, and evidence grammar independently of
  visual style, so the same architecture can be rendered as dark technical linework, warm paper,
  Azure semantic resource blocks, editorial minimalism, playful illustration, or the product's own
  design language. Do not use it merely to recreate dashboard chrome around a diagram.
compatibility: Python 3.10+ for scene validation and rendering; Canvas-capable browser for interactive output.
---

# Isometric System Map

Create the isometric scene first. Treat any surrounding product UI, rails, metrics, panels, or
controls as optional composition requested by the user, never as the map itself.

The skill separates two layers:

1. **Scene grammar**: repository facts, 2:1 projection, resource envelopes, sourced containment areas,
   uniform cubes, depth, routes, payloads, labels, and evidence.
2. **Design language**: palette, line weight, materials, typography, texture, motion character, and
   optional framing.

A good result can change from luminous black glass to sepia drafting paper without changing the
architecture or route geometry.

## Default deliverables

Create a reviewable semantic sidecar plus the requested visual:

```text
docs/diagrams/<repo>-isometric-scene.json
docs/diagrams/<repo>-isometric-map.html   # Canvas-first interactive or animated output
docs/diagrams/<repo>-isometric-map.svg    # optional static/vector export
```

Use the bundled three-layer Canvas renderer for the default HTML artifact. It keeps terrain,
architecture, and motion separate while mirroring nodes and paths to native DOM controls. Use SVG
only when the user explicitly needs editable vectors, print-oriented output, or a simple static scene.
Export PNG from the composed Canvas layers when an image file is useful. Do not default to a
dashboard shell.

## Workflow

### 1. Pin the evidence boundary

Record repository, requested ref, immutable commit, and selected scope before drawing. Do not call a
local checkout "latest main" unless freshness was proven. Use the exact supplied snapshot wording
when the source cannot be fetched safely.

### 2. Extract a semantic graph

Trace entry points and callers before selecting visual forms. Establish:

- initiators, pipelines, schedulers, or API entry points;
- owned modules and runtime resources, including exact ARM `resource_type` where available;
- a package-supported `icon` for Azure resources or CI/CD primitives when the chosen theme uses roof marks;
- existing or externally owned dependencies;
- evidenced containment such as resources attached to one VNet;
- deployment, dependency, control, data, identity, network, and telemetry paths;
- environment overlays, conditions, approvals, and true held states;
- concrete payloads moving along selected paths.

Read [`references/repository-extraction.md`](references/repository-extraction.md) for Bicep-specific
checks, ARM-family mapping, and the supported Azure line-art icon vocabulary. Every represented fact
needs structured evidence containing `path`, `lines`, and `claim`. Omit uncertain claims instead of
inventing connective tissue.

Before curating the scene, build a temporary requirement-to-geometry ledger. Include every explicit
user requirement and every source-backed distinction that would make the map materially wrong if it
were collapsed:

| Required fact | Scene representation | Direct evidence |
| --- | --- | --- |
| one shared non-prod VNet and one distinct prod VNet | two evidenced VNet terrain areas, each with its own VNet cube and member resources | exact constants, lookup ranges, and attachment evidence |
| separate resource-group or ownership boundaries | evidenced boundary cubes or distinct sourced regions | exact scope declarations |
| shared hub plus prod-only overlay | one hub node plus one conditional overlay node and their path | hub and overlay declarations |
| externally owned import or contract | external node and dependency/control path, never a locally owned runtime flow | ownership and import declarations |

A requirement is not covered merely because its wording appears in a description or evidence claim.
It must be visible in geometry, a distinct node, a routed path, a status, or an evidenced boundary.
Keep this ledger in `run-notes.md`, then check every row against `scene.json` before rendering.

### 3. Choose the architecture story

Curate a discussable scene, not a repository inventory:

- 8-24 resource cubes is the normal range; the contract cap is 28.
- 2-5 visual regions is usually enough.
- 3-6 named flows is ideal when the evidence supports real payload journeys. A static dependency map
  can use no payloads and no flows.
- Keep deployment dependency separate from runtime data movement.
- Give important hubs clearer placement, route degree, or reserved footprint, not a building silhouette.
- Do not merge objects whose distinct environment, scope, ownership, contract, or lifecycle is an
  explicit requirement. The normal node target is a readability guide, not permission to erase a
  required distinction.

A visual `zone` can be compositional. A scene `area` is sourced containment. Do not imply Azure
containment, ownership, or runtime co-location with either one unless evidence supports it.

### 4. Define art direction without changing geometry

If the user supplies a design language, derive a compact art-direction brief from it. Otherwise
choose one appropriate to the subject. Specify:

- design-language name and 2-4 principles;
- background, grid, structure, control-path, data-path, payload, and text roles;
- medium, linework, materials, typography, and motion character;
- a structured treatment for each used path kind covering pattern, weight, marker, texture, motion
  cadence, and reduced-motion behavior.

Read [`references/style-separation.md`](references/style-separation.md). For Azure Bicep maps, the
bundled `azure-topology.js` theme is the preferred starting point: it uses the Azure topology semantic
families, connector palette, and package-local line-art marks without changing scene geometry. The
screenshot examples are proof that the same scene grammar supports different skins, not a request to
copy their surrounding interface.

### 5. Lay out the isometric terrain

Use a true 2:1 grid: `tile_width = 2 × tile_height`. Choose one scene-wide `canvas.cube_size`, then
place resource envelopes in grid coordinates and validate full footprint overlap. Every envelope must
be large enough to contain that same cube. Keep high-traffic hubs central, entry/control nodes toward
the back or left, and sinks/outputs toward the front or right unless the story suggests another
reading order.

When evidence says resources are network-contained, declare a sourced `area` with `kind: "vnet"`,
the contained node IDs in `member_ids`, half-grid `padding`, status, description, and direct evidence.
Include the VNet itself as one cube in the area's membership when it is represented. The renderer
derives the terrain boundary from the complete member footprints, so place every network-attached
resource inside the VNet area rather than leaving containment as a label or path-only implication.
Keep ordinary visual grouping in `zones`; use `areas` only for proven containment.

Route paths explicitly as grid points. Prefer lane-like segments along one isometric axis at a time.
Start and end on a footprint edge or in the outward half-cell beside one edge, never inside a
cube. Do not rely on automatic center-to-center Bézier curves. Routes must avoid unrelated
resource envelopes and remain readable after depth sorting.

Read [`references/isometric-grammar.md`](references/isometric-grammar.md) for projection, form, depth,
and routing rules.

### 6. Build true resource cubes

Every node renders as one cube of exactly the same scene-wide size. Do not turn services, pipelines,
boundaries, or abstractions into towers, slabs, gateways, stacks, hubs, or other architectural
silhouettes. Do not vary cube size by role, importance, resource type, footprint, or status.

The declared footprint remains only the collision and routing envelope. Center one cube with edge
`canvas.cube_size` inside every footprint. Derive the projected vertical edge from that projected
ground edge so width, depth, and height read as one true cube on screen. Nodes do not declare their own
height or scale.

Place the node's line-art `icon` on the top face. Identity comes from the roof mark, semantic palette,
short code, evidence, and connected paths. Importance comes from placement, route degree, reserved
footprint, spacing, and labels, never from cube scale or from making a resource look like a building.
Design languages may change material, linework, corner treatment, shadow, typography, and motion while
preserving the uniform cube geometry.

### 7. Route real paths and payloads

Distinguish path kinds through the current design language, using line pattern, weight, markers,
texture, or restrained color. Keep direction visible.

A moving dot must represent a named payload such as a deployment request, resource ID, command,
event, record, secret reference, network session, or telemetry envelope. Associate payloads with
ordered flow steps and direct evidence. Do not add motion as decoration.

Evidence must support the exact source node, target node, path kind, and payload together. Runtime
evidence for application-to-monitoring telemetry does not justify animation on a deployment-module
to-monitoring path. If endpoint congruence is not proven, render a static dependency or omit the path.

For a static topology with only non-payload dependencies, use empty `payloads` and `flows` lists.
For a static image of real payload paths, keep the payload and flow semantics but render numbered or
repeated markers instead of inventing animation.

For implementation patterns, read [`references/canvas-recipes.md`](references/canvas-recipes.md).
Read [`references/svg-recipes.md`](references/svg-recipes.md) only for an explicit vector or simple
static export.

### 8. Author and validate the scene sidecar

Start from [`tests/fixtures/valid-scene.json`](tests/fixtures/valid-scene.json) and read
[`references/scene-contract.md`](references/scene-contract.md). Validate before rendering:

```bash
python3 skills/isometric-system-map/scripts/validate_scene.py \
  docs/diagrams/<repo>-isometric-scene.json
```

Fix every collision, dangling reference, uncited claim, invalid route, and projection error. The
contract intentionally has no fields for dashboards, rails, metric cards, or explainer panels.

Then run a semantic coverage gate against the temporary ledger:

- shared and distinct environment objects are independently identifiable;
- resources proven to share a VNet are members of the same sourced VNet area and visibly fall inside it;
- source-backed resource-group, subscription, and ownership boundaries are geometric, not prose-only;
- hub-and-overlay topologies contain both the shared hub and the smaller overlay;
- exact contract names and decisive flags such as `subscriptionRequired` appear on the relevant
  node/path description with direct evidence;
- externally owned imports are shown as external dependencies rather than local runtime traffic;
- every moving payload is supported at its rendered source and target.

Do not render until every required row has a concrete scene element or is explicitly excluded as
unsupported.

### 9. Render the standalone scene

Render the validated sidecar with a theme adapter:

```bash
python3 skills/isometric-system-map/scripts/render_canvas.py \
  docs/diagrams/<repo>-isometric-scene.json \
  skills/isometric-system-map/themes/azure-topology.js \
  docs/diagrams/<repo>-isometric-map.html
```

Use three aligned Canvas layers:

1. terrain: background, material, ground, grid, sourced containment areas, and quiet zones;
2. architecture: routes, arrows, resource cubes, and compact labels;
3. motion: payloads and interaction highlights.

Retain geometry as `Path2D` objects for drawing, pointer hit testing, focus rings, and selection.
Mirror every node and path to a native focusable DOM control in the same labelled scene region.
Use `ResizeObserver` and `devicePixelRatio` for responsive sharp output. Use timestamp-based
`requestAnimationFrame`, cancel it while paused, hidden, or under reduced motion, and hide motion
controls for a static scene. Composite the three Canvas layers with `toBlob()` for PNG export.

The theme adapter owns linework, materials, typography, arrow shape, texture, icon color, and motion
cadence. It must not change scene facts, grid coordinates, resource envelopes, route geometry,
payload membership, or evidence. The bundled Azure topology, dark technical, and warm archival-paper
themes are examples, not mandatory styles. The renderer embeds only the line-art symbols actually
used by the scene, so the HTML remains self-contained.

Render order:

1. background and ground plane;
2. region markings and grid;
3. routes behind resource cubes;
4. resource cubes sorted back-to-front by envelope far edge;
5. labels and direction markers;
6. payloads and interaction highlights on the motion layer;
7. a compact legend or evidence affordance only if needed;
8. native semantic controls visually hidden but present in the accessibility tree.

Let the scene occupy most of the visual. Optional explanatory UI must inherit the chosen design
language and remain subordinate.

### 10. Verify the actual visual

Inspect the rendered artifact in a browser or image viewer:

1. The scene reads as isometric before any text is read.
2. Every node is one true cube with the same scene-wide cube edge, equal projected edges, visible roof
   and wall faces, and no envelope collisions. Every supported resource mark is visibly projected onto
   its top face and uses the semantic family palette.
3. The ground grid uses consistent 2:1 axes.
4. Every sourced VNet area visibly encloses the complete footprints of all declared member resources,
   exposes its evidence, and appears as a native focusable control.
5. Directed routes follow the terrain and avoid unrelated resource envelopes.
6. The main flows can be followed end-to-end without guessing.
7. Payloads map to named values and pause under reduced-motion preferences.
8. Pause stops the animation frame loop, and reduced motion starts with no live frame loop.
9. Every node and path appears as a native focusable control and focus mirrors to the Canvas.
10. Labels stay legible and do not become the architecture.
11. The requested design language is visible in material, linework, typography, and motion, not only
   a palette swap.
12. Citations are available in tooltips, details, captions, or the sidecar without forcing a fixed
   panel layout.
13. At screenshot scale, the isometric image remains the focal point.

## Hard boundaries

- Do not recreate surrounding dashboard chrome unless the user explicitly requests the full UI.
- Do not vary node silhouettes or cube size. Every node remains one identically sized cube; use marks,
  palette, labels, placement, reserved footprints, spacing, and paths for differentiation.
- Do not show proven VNet containment only as a VNet cube, zone label, or network path. Use a sourced
  VNet area whose members include the contained resources.
- Do not fake 3D with arbitrary CSS transforms on flat cards.
- Do not use generic center-to-center curves when explicit terrain routes are possible.
- Do not label a manual pipeline as held merely because `trigger: none` or `pr: none` is present.
- Do not infer ownership, deployment state, or data flow from file presence alone.
- Do not expose secrets, tenant IDs, subscription IDs, connection strings, or private payload values.
- Do not make citations part of the decorative texture. Keep them readable and exact.
- Do not leave a `requestAnimationFrame` loop running while paused or under reduced motion.
- Do not use a Canvas bitmap as the only interaction surface. Mirror every target to native DOM.

## Relationship to nearby skills

| Need | Use |
| --- | --- |
| Isometric architecture image in any design language | `isometric-system-map` |
| General repository role graph with card-based UI | `repo-map` |
| Time-sequenced actor or service swimlanes | `blueprint` |
| Flexible explanatory page where isometric geometry is not central | `wayfinder` |
| Low-fidelity text layout | `ascii-wireframe` |

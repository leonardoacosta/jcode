# Isometric scene contract

The scene JSON is a semantic and geometric sidecar. It is not a UI specification. It records what is
being drawn, where it sits on the isometric grid, how paths travel across the terrain, what payloads
move, which design language should skin the scene, and what source evidence supports every claim.

Unknown fields are rejected so dashboard chrome cannot leak into the reusable scene model.

## Top level

```json
{
  "version": 1,
  "repository": {},
  "art_direction": {},
  "canvas": {},
  "zones": [],
  "areas": [],
  "nodes": [],
  "paths": [],
  "payloads": [],
  "flows": []
}
```

Start from [`../tests/fixtures/valid-scene.json`](../tests/fixtures/valid-scene.json), then run:

```bash
python3 skills/isometric-system-map/scripts/validate_scene.py path/to/scene.json
```

## Repository boundary

```json
{
  "name": "acme-infra",
  "ref": "origin/main",
  "commit": "0123456789abcdef0123456789abcdef01234567",
  "scope": "infra/foundation and its deployment pipeline",
  "summary": "Shared network, data, identity, compute, and deployment topology."
}
```

All fields are required. A commit is a 7-40 character hexadecimal object ID. The scope is the bounded
architecture story, not a claim that the map is exhaustive.

## Art direction

```json
{
  "name": "warm technical paper",
  "principles": ["ink-first linework", "quiet hierarchy", "no interface chrome"],
  "palette_roles": {
    "background": "#d9cfaa",
    "grid": "#b9ad82",
    "structure": "#5b553f",
    "control_path": "#4f4936",
    "data_path": "#6d5835",
    "payload": "#28251c",
    "text": "#2c291f"
  },
  "medium": "archival drafting paper",
  "linework": "fine sepia technical pen",
  "materials": "flat paper faces with sparse hatching",
  "typography": "compact monospaced labels",
  "motion": "small ink dots moving at a measured pace",
  "path_treatments": {
    "delivery": {
      "stroke_pattern": "solid",
      "weight": "medium",
      "marker": "filled terminal arrow",
      "texture": "clean technical ink",
      "motion_cadence": "steady measured travel",
      "reduced_motion": "numbered marker at the step endpoint"
    }
  }
}
```

The palette roles are semantic, not fixed colors. A high-contrast dark theme, monochrome ink theme,
or playful editorial theme still supplies the same roles. `principles` needs at least two strings.

`path_treatments` contains one object for every path kind used by the scene. Each treatment declares
`stroke_pattern`, `weight`, `marker`, `texture`, `motion_cadence`, and `reduced_motion`. These are
descriptive art-direction tokens, so a paper renderer can interpret `weight: "fine"` as pen width
while a product renderer can map it to a design token. Requiring non-color channels keeps the path
semantics legible in grayscale and static exports.

## Canvas

```json
{
  "grid_width": 16,
  "grid_depth": 12,
  "tile_width": 64,
  "tile_height": 32,
  "cube_size": 1
}
```

`tile_width` must be exactly twice `tile_height`. Node positions and route points use grid
coordinates. A route may use half-grid points. The renderer chooses the screen origin and how many
pixels one semantic height unit represents. `cube_size` is one required half-grid value from 0.5 to 2.
It is the edge of every node cube in the scene. Cube size cannot vary per node.

## Zones

A zone is a visual grouping aid:

```json
{
  "id": "runtime",
  "label": "Runtime",
  "description": "Provisioned workload services"
}
```

Do not treat zones as sourced cloud containment unless the map text and evidence say so.

## Areas

An area is an evidence-backed containment surface, not a loose compositional region. The current
contract admits VNet areas:

```json
{
  "id": "runtime-vnet",
  "label": "Runtime VNet",
  "kind": "vnet",
  "status": "active",
  "member_ids": ["runtime-vnet-node", "app", "database"],
  "padding": 0.5,
  "description": "Private runtime attachment area.",
  "evidence": [
    {
      "path": "infra/network/main.bicep",
      "lines": "1-72",
      "claim": "The application and database attach to the runtime VNet."
    }
  ]
}
```

`areas` is required and may be empty. A scene can contain at most eight areas. Each VNet area has one
to twenty unique node IDs, half-grid padding from 0 to 2, a normal node status, and direct evidence.
The renderer derives the area rectangle from the complete footprints of all members plus padding. The
padded rectangle must remain inside the canvas. Include the represented VNet cube itself in
`member_ids`, then add only resources whose network containment is supported by evidence. Every area
is mirrored to a native semantic control and included in runtime geometry diagnostics.

## Nodes

```json
{
  "id": "shared-sql",
  "code": "SQL",
  "label": "Shared SQL estate",
  "role": "data",
  "form": "cube",
  "zone": "runtime",
  "position": { "x": 8, "y": 4 },
  "footprint": { "width": 2, "depth": 1 },
  "status": "active",
  "resource_type": "Microsoft.Sql/servers/databases",
  "icon": "az-sql-database",
  "description": "Hosts application databases behind a private endpoint.",
  "evidence": [
    {
      "path": "infra/data/main.bicep",
      "lines": "18-96",
      "claim": "The module creates SQL and its optional private endpoint."
    }
  ]
}
```

### Roles

`entry`, `pipeline`, `governance`, `module`, `network`, `compute`, `data`, `identity`, `messaging`,
`observability`, `external`.

Role describes architecture. It does not force a color or silhouette.

### Azure resource identity

`resource_type` and `icon` are optional semantic fields. Use them when source evidence identifies a
concrete Azure resource or a CI/CD primitive:

- `resource_type` is the exact ARM type when one exists, such as `Microsoft.Web/sites` or
  `Microsoft.Network/privateEndpoints`. A generic delivery primitive may use a stable descriptive
  type such as `Azure DevOps pipeline`.
- `icon` is one symbol ID from [`../assets/azure-icons.svg`](../assets/azure-icons.svg), such as
  `az-app-service`, `az-sql-database`, `az-private-endpoint`, or `az-release-pipeline`.
- Use [`../assets/azure-tokens.json`](../assets/azure-tokens.json) to map ARM types to the semantic
  `compute`, `data`, `identity`, `integration`, `network`, `monitor`, `governance`, or `devops`
  family.

The Azure Canvas theme projects the selected line-art mark onto the roof face of the node's uniform
cube. The sprite is package-local self-authored stand-in line art, not official Microsoft
logo artwork. See [`../assets/PROVENANCE.md`](../assets/PROVENANCE.md).

Omit these fields for non-Azure scenes or when the resource cannot be identified confidently. Never
choose an icon merely because it looks plausible.

### Form

`cube` is the only supported form. Every node is one cubical mass, including delivery, governance,
boundary, external, and abstract nodes. Do not use architectural silhouettes to encode roles.

### Status

- `active`: normal live or deployable path shown by evidence.
- `conditional`: created only when an explicit condition or environment branch is satisfied.
- `held`: blocked, disabled, awaiting approval, placeholder, or explicitly not deployed.
- `external`: referenced but owned outside the selected scope.
- `deprecated`: still relevant to understanding current behavior but on a retirement path.

`trigger: none` and `pr: none` describe manual pipeline initiation. They do not, by themselves, mean
held.

### Geometry

`position` is the back-left grid origin of the footprint. Footprints are 1-4 cells on each axis and
act only as collision, routing, and spacing envelopes. Every footprint must contain
`canvas.cube_size`. The renderer centers one cube of exactly that global edge inside the envelope and
derives the vertical projection required for equal screen-space edges. Per-node `height` and scale
fields are rejected. The validator detects positive-area footprint overlap and bounds.

## Evidence objects

Every node, path, payload, and flow step carries direct structured evidence:

```json
{
  "path": ".azuredevops/build/deploy.yml",
  "lines": "31-44",
  "claim": "The job invokes the Bicep deployment entry point."
}
```

Paths are repo-relative. `lines` is `42` or `42-81`. `claim` states exactly what those lines support.
Do not put secret values or private identifiers in claims.

## Paths

```json
{
  "id": "pipeline-to-foundation",
  "from": "deploy-pipeline",
  "to": "foundation-root",
  "kind": "delivery",
  "label": "what-if then deploy",
  "route": [
    { "x": 3, "y": 2.5 },
    { "x": 5, "y": 2.5 },
    { "x": 5, "y": 4.5 },
    { "x": 7, "y": 4.5 }
  ],
  "payload_ids": ["deployment-request"],
  "evidence": [
    {
      "path": ".azuredevops/build/deploy.yml",
      "lines": "31-44",
      "claim": "The job invokes the foundation deployment."
    }
  ]
}
```

Kinds: `control`, `data`, `delivery`, `dependency`, `identity`, `network`, `telemetry`.

Routes require 2-16 explicit half-grid points. Each segment follows one grid axis, starts on the
source footprint boundary or in the outward half-cell beside one edge, ends the same way at the
target, and cannot cross an unrelated resource envelope. Interior endpoints and diagonally offset
corner points are invalid. `payload_ids` may be empty only when `kind` is `dependency`.

## Payloads

```json
{
  "id": "deployment-request",
  "label": "ARM deployment",
  "kind": "deployment",
  "description": "Environment parameters and module outputs.",
  "evidence": [
    {
      "path": ".azuredevops/build/deploy.yml",
      "lines": "31-44",
      "claim": "The job passes deployment parameters and the compiled template."
    }
  ]
}
```

Kinds: `command`, `deployment`, `event`, `record`, `resource-id`, `secret-reference`, `telemetry`,
`network-session`.

A payload is a concrete transferred value. "Dependency" or "relationship" is not a payload.
`payloads` may be an empty list for a static architecture scene whose paths are non-payload
dependencies. A static export may also retain payload definitions and render them as numbered or
repeated markers instead of animation.

## Flows

```json
{
  "id": "guarded-release",
  "label": "Guarded release",
  "description": "Preview, deploy, and materialize the workload dependency chain.",
  "steps": [
    {
      "path": "pipeline-to-foundation",
      "payload": "deployment-request",
      "label": "Preview and deploy",
      "evidence": [
        {
          "path": ".azuredevops/build/deploy.yml",
          "lines": "31-44",
          "claim": "The pipeline previews and deploys the foundation entry point."
        }
      ]
    }
  ]
}
```

A flow has 1-12 ordered steps. Each step directly cites the transition it claims, even when the path
also has evidence. The step payload must be listed in the referenced path's `payload_ids`.

`flows` may be an empty list when the map is a static topology with no named payload journey. Do not
invent a moving token merely to satisfy the visual treatment.

## Integrity checklist

- Exact repository ref and commit are recorded.
- Every node uses `form: "cube"` and renders as one cubical mass.
- Full footprints do not overlap or leave the grid.
- Paths are explicit, directed, axis-routed, and avoid unrelated resource envelopes.
- Every used path kind has a structured treatment beyond color.
- Every reference resolves.
- Every node, path, payload, and flow step has direct evidence.
- Status and ownership come from source evidence.
- The contract contains no fields for rails, metrics, panels, or other surrounding interface chrome.

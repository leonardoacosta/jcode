# Topology views sidecar reference

`topology-views.json` is an optional version 1 companion document for an isometric scene. It curates the same repository into Runtime, Network, and ADO projections without changing the base scene contract.

## Rendering and validation

Render a views-enabled artifact with `render_canvas.py` by passing the sidecar after the scene, theme, and output paths:

```bash
python3 skills/isometric-system-map/scripts/render_canvas.py \
  scene.json \
  skills/isometric-system-map/themes/azure-topology.js \
  map.html \
  --views topology-views.json
```

Scene-only compatibility is preserved. If `--views` is omitted, the renderer validates and renders the existing scene-only canvas and does not emit the repository views shell or `data-views-sha256` attribute.

The normative machine-readable grammar is `references/topology-views.schema.json`. It is JSON Schema 2020-12, rejects unknown keys, and requires `version: 1`. The repository-aware validator is `scripts/validate_views.py`:

```bash
python3 skills/isometric-system-map/scripts/validate_views.py topology-views.json scene.json
```

`render_canvas.py --views` runs the same validator and also rejects sidecars whose `repository.name`, `repository.ref`, or `repository.commit` drift from the scene repository.

## Version 1 grammar

Top-level fields are required and unknown top-level fields are rejected:

```json
{
  "version": 1,
  "repository": {
    "name": "repo-name",
    "ref": "main",
    "commit": "0123456789abcdef0123456789abcdef01234567"
  },
  "default_view": "network",
  "runtime": {
    "node_ids": ["app"],
    "path_ids": ["request-path"],
    "flow_ids": ["observability-flow"]
  },
  "network": {
    "containers": [],
    "memberships": [],
    "links": []
  },
  "pipelines": [
    {
      "id": "build-and-deploy",
      "label": "Build and deploy",
      "stages": [],
      "edges": []
    }
  ]
}
```

### `repository`

`repository` contains exact source identity:

- `name`: non-empty string.
- `ref`: non-empty string.
- `commit`: 40-character hexadecimal commit.

The validator compares all three values with `scene.repository` when the scene carries repository identity.

### `default_view`

`default_view` is one of `runtime`, `network`, or `ado`. It selects the initial native tab unless the generated page is opened with a valid deep link hash such as `#network`.

### `runtime`

`runtime` selects the scene resources and relationships shown in the runtime projection:

- `node_ids`: unique non-empty scene node IDs.
- `path_ids`: unique non-empty scene path IDs.
- `flow_ids`: optional unique non-empty scene flow IDs.

Every traffic-layer member in the scene must appear in `runtime.node_ids`. Runtime references reuse the scene's existing node, path, flow, and evidence validation.

### `network`

`network` contains explicit Azure topology structure:

- `containers`: subscription, resource-group, VNet, and subnet boundaries.
- `memberships`: assignments from one container to one scene node.
- `links`: evidenced relationships between containers and scene nodes.

A network container has:

- `id`: unique non-empty string.
- `kind`: `subscription`, `resource-group`, `vnet`, or `subnet`.
- `label`: full visible label.
- `status`: non-empty status text.
- `parent_id`: optional parent container ID.
- `cidr`: optional non-empty string, intended for subnet CIDR/address-space text such as `10.42.1.0/24`.
- `evidence`: one or more evidence entries.

A membership has `container_id`, `node_id`, and `evidence`. The validator rejects unknown containers, unknown scene nodes, duplicate node membership, SQL PaaS database resources directly contained by a subnet, and private endpoint resources not directly contained by a subnet.

A network link has:

- `id`: unique non-empty string.
- `kind`: `peering`, `private-endpoint`, `dns`, or `data`.
- `source_id` and `target_id`: scene node IDs or network container IDs.
- `direction`: `forward`, `reverse`, or `both`.
- `evidence_level`: `direct`, `inferred`, or `held`.
- `label`: full visible relationship label.
- `evidence`: one or more evidence entries.

The validator rejects duplicate link IDs, unknown link endpoints, containment cycles, missing parent containers, and non-canonical evidence levels.

### `pipelines`

`pipelines` is an array of one or more ADO pipeline objects. Each pipeline has `id`, `label`, `stages`, and `edges`.

A stage has:

- `id`: unique non-empty string within the pipeline.
- `label`: full visible label.
- `stage_type`: `repository`, `validation`, `build`, `artifact`, `gate`, `deployment`, or `held`.
- `icon`: admitted SVG symbol ID from `assets/azure-icons.svg`.
- `status`: non-empty status text.
- `parallel_group`: optional non-empty string.
- `lane`: optional non-negative integer for deterministic parallel layout.
- `target_node_id`: optional scene node target.
- `evidence`: one or more evidence entries.

An edge has `id`, `source_id`, `target_id`, `label`, `kind`, and `evidence`. Edge `kind` is `automatic`, `dependency`, `approval`, `manual`, or `held`. The validator rejects unsupported stage icons, edges that reference unknown stage IDs, and cyclic pipeline graphs.

## Evidence provenance

Every evidence entry is exactly `{path, lines, claim}` in the human contract and exactly:

```json
{ "path": "infra/main.bicep", "lines": "12-24", "claim": "The deployment defines the represented resource." }
```

All three values are non-empty strings. Containers, memberships, network links, pipeline stages, and pipeline edges cite their own direct evidence. Evidence is not inherited from a parent container, adjacent node, pipeline, or scene object.

## Canonical Azure ontology

The version 1 Azure ontology keeps topology boundaries and configuration metadata out of runtime resource cubes:

- VNet and subnet objects are Network containers, not scene nodes.
- CIDR and address-space values live on Network containers, not resource labels.
- Peering is a Network link, not a runtime cube.
- Private endpoint resources are scene nodes that must be direct members of their evidenced subnet.
- SQL and other PaaS resources normally remain resource-group scoped unless direct source evidence places the service itself in a network integration boundary.
- APIM APIs, policies, named values, products, subscriptions, configurations, and partner relationships enrich APIM metadata and evidence. They are not modeled as topology nodes.

The views validator enforces key Azure ontology rules by rejecting APIM policy/configuration resource nodes and VNet, subnet, or peering boundaries modeled as topology nodes.

## Direct, inferred, and held semantics

`evidence_level` uses three canonical meanings:

- `direct`: the source files explicitly support the relationship. Direct relationships render as normal solid paths or links.
- `inferred`: the relationship is inferred from surrounding evidence. It must still cite evidence, and it renders with a visibly non-solid style and an `INFERRED` text label where the view supports path semantics.
- `held`: the relationship is planned, gated, inactive, or intentionally not deployed. It must still cite evidence, and it renders without animation plus explicit held/not-deployed text where the view supports path semantics.

Core scene paths may also carry `evidence_level`. For legacy scene-only artifacts, omission means the path is treated as the `direct` default for backward compatibility.

## Azure icon fallbacks

The Azure renderer uses admitted SVG symbols from `assets/azure-icons.svg`. For package-owned Azure family icon fallbacks, `assets/azure-tokens.json` owns the mapping from full ARM `resource_type` to `resource_type_family` and from family to `family_icon_fallbacks`. A sidecar cannot override those package-owned mappings. If a resource lacks a service-specific icon and the package-owned Azure family fallback does not resolve deterministically to an admitted symbol, validation fails instead of inventing an icon. The full resource label and service type remain visible even when a family fallback icon is used.

## Native tabs, deep links, and no-JS behavior

When a views sidecar is present, the generated HTML includes native tabs for Runtime, Network, and ADO. The tabs are `<button>` elements in a `role="tablist"`; panels are `role="tabpanel"` sections with stable IDs `runtime`, `network`, and `ado`.

JavaScript enhancement sets the active tab, supports keyboard navigation, and honors deep links. Opening `map.html#runtime`, `map.html#network`, or `map.html#ado` selects that view, and changing tabs updates the hash.

No-JS behavior remains readable. With JavaScript disabled, the sidecar sections are present in the HTML, contain headings and evidence lists, and are not hidden by the enhancement script. The scene canvas also retains its fallback inspectable controls for non-canvas or reduced-interaction access.

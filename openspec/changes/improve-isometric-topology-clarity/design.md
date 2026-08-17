## Context

The existing Canvas renderer embeds the package-local Azure SVG sprite and projects each mark onto a uniform cube roof, but the mark occupies only 64% of the roof while a short `code` is drawn as the dominant identity. Brown and Decus also place request traffic, deployment stages, resource-group abstractions, network containment, and operational dependencies in one isometric scene. The facts are evidence-backed, but the presentation does not answer one architectural question at a time.

The approved direction is intentionally iterative rather than visually final. It establishes stable information architecture and acceptance constraints while leaving spacing, card density, and connector routing open to browser-based refinement. The core `scene.json` remains the authority for isometric facts and must not acquire dashboard or tab configuration.

The canonical modeling reference is the approved Decus and Wholesale Bicep traffic audit at `brown/wholesale@46be8f57:docs/diagrams/decus-wholesale-bicep-traffic-audit.md` (SHA-256 `e65616d1b6728561e8ac8f60eae8bce5c841f06323e72fa1465d0c66a7cd13f5`). It is a read-only evidence input. Its ontology is normative for this change: VNets and subnets are containment areas, CIDRs are fields, peerings are edges, and only deployable Azure resources or evidenced application hosts become uniform cubes.

## Goals / Non-Goals

**Goals:**

- Make approved Azure SVG marks visibly useful rather than technically present but visually subordinate.
- Remove abbreviation-only resource identity from the Azure runtime presentation.
- Separate runtime traffic, network containment, and ADO delivery into coordinated focused views.
- Preserve one source-backed resource catalog, evidence model, and selection identity across views.
- Show Azure network nesting and ADO stage ordering explicitly rather than through proximity.
- Preserve standalone, local-only, accessible, responsive, and deterministic artifacts.
- Keep existing scene-only rendering backward compatible.
- Encode the approved distinction between directly evidenced, inferred, held/not-deployed, and unsupported relationships.
- Keep private endpoints inside their evidenced subnet while keeping the connected PaaS resource outside the subnet unless source evidence proves direct integration.

**Non-Goals:**

- Replace the core isometric scene grammar or uniform-cube contract.
- Add official Microsoft logo artwork, remote assets, a client framework, or a graph-layout dependency.
- Infer missing VNets, subnets, pipeline stages, approvals, or runtime traffic without direct evidence.
- Modify Brown, Decus, Azure, or Azure DevOps infrastructure.
- Turn every Bicep module or pipeline parameter into a visible card.
- Render APIM APIs, products, policies, named values, subscriptions, configuration, or partner relations as standalone infrastructure nodes.
- Invent application hosts, active peerings, or data ownership absent from the audited source tree.
- Freeze the final spacing or illustration polish before browser iteration.

## Decisions

### 1. Render three focused projections in one self-contained page

A views-enabled output contains three tabs:

1. **Runtime:** the existing isometric Canvas renderer, filtered to the request/runtime story declared by the companion sidecar.
2. **Network:** nested DOM containers and resource cards with an SVG connector overlay.
3. **ADO Pipeline:** a topologically ranked DOM stage graph with an SVG connector overlay.

The selected view is encoded as `#runtime`, `#network`, or `#ado`; the configured default is `network` for the Brown and Decus review artifacts. Tab controls use native buttons with `role="tab"`, keyboard arrow/Home/End navigation, visible focus, and no-JavaScript fallback sections.

Rejected alternatives:

- One improved isometric map retains the same story collision and connector density.
- Three unrelated pages duplicate resource identity and make comparison harder.
- Replacing the runtime view with flat cards discards the approved isometric request-flow language.

### 2. Keep `scene.json` pure and add an optional companion view sidecar

`render_canvas.py` retains its existing three positional arguments and accepts an optional `--views <path>` argument. Without it, output remains compatible with current standalone maps.

The companion sidecar contains:

- repository identity that must match the scene ref and commit;
- `default_view`;
- a Runtime projection with explicit `node_ids`, `path_ids`, and optional named flow selection;
- a Network projection with evidence-backed Subscription, Resource Group, VNet, and Subnet containers, node memberships, and labeled links;
- one or more ADO pipelines with evidence-backed stages, parallel groups, gates, deployment targets, and directed edges.

Network memberships reference scene node IDs. Pipeline deployment targets reference scene node IDs where a represented resource exists; delivery-only primitives may be sidecar stages with admitted CI/CD icons and their own structured evidence. All IDs are unique and all cross-references are validated before rendering.

Rejected alternative: adding tab, network-container, and pipeline-layout fields to `scene.json` would weaken the scene contract by mixing architecture facts with presentation composition.

### 2a. Make the version 1 sidecar grammar normative

The implementation adds `references/topology-views.schema.json` as the machine-readable authority and `references/topology-views.md` as its human-readable companion. The schema uses JSON Schema 2020-12, requires `version: 1`, rejects unknown keys at every object boundary, and defines these required top-level fields:

- `repository`: exact `name`, `ref`, and 40-character `commit` identity matching the scene;
- `default_view`: one of `runtime`, `network`, or `ado`;
- `runtime`: unique `node_ids`, unique `path_ids`, and optional unique `flow_ids` that reference the scene;
- `network`: `containers`, `memberships`, and `links`;
- `pipelines`: one or more pipeline objects containing `stages` and `edges`.

Network containers have a unique `id`, `kind` in `subscription | resource-group | vnet | subnet`, full `label`, optional `parent_id`, textual `status`, and direct structured `evidence`. Membership objects pair one `container_id` with one scene `node_id` and carry direct structured evidence. Network links have a unique `id`, `kind` in `peering | private-endpoint | dns | data`, `source_id`, `target_id`, `direction` in `forward | reverse | both`, full `label`, and direct structured evidence.

Subnet containers may carry an optional non-empty string `cidr` field. CIDR and address-space values belong on their network containers, never in a resource-cube label. Network links declare `evidence_level` in `direct | inferred | held`; direct links render solid, inferred links render visibly dashed and labeled, and held links render non-animated with explicit not-deployed text. Every level still carries evidence explaining the exact supported or unsupported claim.

Pipeline stages have a unique `id`, full `label`, `stage_type` in `repository | validation | build | artifact | gate | deployment | held`, an admitted `icon`, textual `status`, optional `parallel_group`, optional non-negative integer `lane`, optional scene `target_node_id`, and direct structured evidence. Pipeline edges have a unique `id`, source and target stage IDs, full `label`, `kind` in `automatic | dependency | approval | manual | held`, and direct structured evidence. Stage graphs must be acyclic and deterministic rank ties are resolved by declared lane and then input order.

Every evidence entry is exactly `{ "path": string, "lines": string, "claim": string }`, with three non-empty values. Containers, memberships, network links, stages, and pipeline edges must each cite their own direct evidence; evidence is never inherited from a parent, adjacent node, or implied relationship. Runtime references reuse the already validated scene-node, scene-path, and flow evidence. The validator rejects identity drift, duplicate IDs, duplicate membership, containment cycles, unknown references, omitted traffic-layer members, unsupported icons, missing direct evidence, and cyclic or dangling pipeline edges with stable field paths.

### 3. Make the approved SVG the primary resource identifier

The Azure theme stops drawing `node.code`. Scene codes remain available for non-Azure themes and compact diagnostics, but they are not visible in the approved Azure output.

For Runtime cubes:

- the admitted SVG occupies 78% of the roof's shorter world edge;
- icon color retains the semantic family stroke with sufficient contrast;
- a white label plate displays the full resource label and a concise service-type label;
- environment and non-active status render as text badges rather than unexplained color;
- labels remain visible without hover and may wrap to two lines.

For Network and ADO cards:

- the same admitted SVG is at least 24 CSS pixels in the primary desktop layout;
- full name and service/stage type are always visible;
- unsupported icon IDs fail validation;
- an evidenced resource without a service-specific mark uses an admitted family-appropriate mark only when that fallback mapping is declared in the sidecar or token catalog and the full service name remains visible; otherwise rendering fails rather than inventing identity.

The version 1 fallback vocabulary is package-owned, not sidecar-authored. `assets/azure-tokens.json` defines `resource_type_family` as a mapping from full ARM resource type to one of the declared family IDs and `family_icon_fallbacks` as a mapping from each family ID to one admitted sprite symbol. A node may omit `icon` only when both mappings resolve deterministically; otherwise validation fails at the node's icon field. A sidecar cannot override either mapping. Stable diagnostics identify the unmapped resource type, family, or missing sprite symbol.

### 4. Curate each projection rather than hiding clutter cosmetically

Runtime renders only declared runtime node and path IDs. Brown and Decus Runtime projections must include every traffic-layer member but exclude pipeline control, resource-group boundary, and network-only abstractions unless they participate in the request path.

Network renders only sourced containment, network attachments, peerings, DNS, private endpoints, and the resources needed to understand those relationships. Pipeline delivery edges do not appear.

Configuration-only objects remain metadata on the owning deployable resource. In particular, APIM APIs, products, policies, named values, subscriptions, configurations, and partner relations may enrich APIM evidence/details but never become cubes or Network resource cards. Shared infrastructure is decomposed into its constituent evidenced Bicep resources instead of a generic shared-infrastructure node.

ADO renders source, build/validation, artifact, parallel jobs/stages, approvals or held gates, and deployment targets. Runtime dependency and network data paths do not appear.

Core scene paths may carry optional `evidence_level` in `direct | inferred | held`. Omission means `direct` for backward compatibility with existing scene-only artifacts. Runtime path rendering uses the same text-redundant semantics as Network links: direct is solid, inferred is visibly non-solid and includes an `INFERRED` label, and held is non-animated and includes a `HELD · NOT DEPLOYED` label.

### 5. Use explicit containment and deterministic layout for Network

The Network view nests containers in this order when evidenced:

`Subscription → Resource Group → VNet → Subnet → Resource card`

Hub/spoke and held/future modifiers are textual and semantic. Resource cards are placed by declared membership in responsive CSS grids. An SVG overlay draws orthogonal, labeled links after layout measurement. The renderer rejects cycles in the containment tree, duplicate membership within one projection, missing parent containers, and links to unknown targets.

Private endpoint resources are members of their evidenced subnet. Their connected PaaS services remain at resource-group scope unless direct source evidence places the service itself in a network integration boundary. VNet and subnet containers, CIDRs, peerings, and gated network declarations never render as resource cubes.

On narrow screens, top-level containers stack vertically and links reroute from measured anchors. Text summaries remain available if SVG overlay geometry cannot be measured.

### 6. Render ADO as a stage graph with visible parallelism and gates

ADO stages use deterministic rank plus declared `parallel_group` and `lane` values. Cards distinguish repository, validation/build, artifact, approval/gate, deployment, and held stages through admitted CI/CD SVGs, labels, badges, and connector semantics.

Edges state their intent, such as automatic delivery, dependency, approval, manual queue, or held transition. A stage or edge must carry direct structured evidence. The view must show manual triggers, parallel jobs, gate conditions, and deployment targets when evidenced, without prescribing a generic branching policy.

### 7. Share selection and evidence without requiring hover

All visible nodes, resource cards, containers, stages, and connectors are keyboard-focusable native controls. Selection is stored by stable semantic ID and retained when switching tabs. When the same scene node appears in another view, it receives the selected treatment there.

A persistent details region shows full label, type, status, relationship, every citation, and every claim. Hover tooltips remain optional acceleration, not the only source of meaning.

### 8. Preserve deterministic, local-only generation

The renderer embeds only the admitted SVG symbols used by the scene and view sidecar. Generated HTML contains no remote assets or runtime fetches. Semantic hashes cover both scene and sidecar bytes. The output records separate scene and views hashes for drift diagnostics.

Generic tracked examples add one views-enabled Azure artifact while existing dark and paper scene-only examples remain valid. Brown and Decus private output directories gain complete delivery bundles containing `scene.json`, `views.json`, `map.html`, `generation-receipt.json`, and `run-notes.md`; the gallery links directly to each tab and previews the default Network view.

`generation-receipt.json` is deterministic and contains no wall-clock timestamp. It records the source repository name/ref/commit, renderer command as an argument array, tool commit, and SHA-256 values for the scene, views sidecar, theme, template, admitted sprite, generated HTML, and run notes. The receipt makes the private bundle reproducible and auditable without committing private source material. Regeneration must reproduce both `map.html` and the receipt byte-for-byte from the delivered bundle inputs.

### 9. Use one canonical browser acceptance harness

The tracked, dependency-free command is:

```text
python3 skills/isometric-system-map/scripts/verify_views_browser.py \
  --chromium "$(command -v chromium)" \
  --gallery output/system-maps/index.html \
  --artifact docs/diagrams/isometric-canvas-azure.html \
  --artifact output/system-maps/brown-decus-dashboard-map/map.html \
  --artifact output/system-maps/brown-decus-portal-ecosystem/map.html
```

The harness starts its own loopback static server, launches Chromium headlessly through the DevTools protocol, and fails with artifact- and assertion-specific diagnostics. For every artifact it exercises direct `#runtime`, `#network`, and `#ado` URLs; Arrow/Home/End plus Enter/Space tab operation; selection retention; desktop, 320 CSS-pixel, and 200-percent zoom layouts; reduced motion; JavaScript-disabled document-order content; direct/inferred/held line treatment and labels; console and page errors; horizontal clipping; focusability; and unexpected network requests. It also opens each artifact through `file://` and HTTP. The gallery assertion opens the supplied gallery, verifies every Runtime/Network/ADO deep link, and verifies each preview targets the declared default Network view. No package download or externally hosted browser driver is permitted.

## Uncertainty Disposition

| Uncertainty | Class | Decision | Rejected alternatives |
| --- | --- | --- | --- |
| One page or separate pages | User-only judgment, resolved | Three tabs inside each Brown/Decus page | One overloaded map; separate gallery cards |
| Preserve isometric runtime | User-only judgment, resolved | Keep Runtime isometric | Replace all views with flat cards |
| Default tab | Safe reversible default | Network for Brown/Decus review pages | Runtime default; ADO default |
| Approved icon source | Discoverable fact | Use the package-local admitted Azure sprite and provenance-approved extensions only | Remote icons; untracked Microsoft artwork; arbitrary text glyphs |
| Exact spacing and label density | Safe reversible default | Start with 78% roof icons, 24px card icons, and two-line labels, then browser-tune | Freeze current density; allow abbreviations |
| Missing icon behavior | Later evidence-dependent action | Require an admitted mapping or fail validation | Guess a service icon |
| Canonical Azure topology ontology | Discoverable fact, user-approved | Follow the pinned Bicep traffic audit and its direct/inferred/held distinctions | Preserve aggregate cubes or configuration-as-node shortcuts |

## Risks / Trade-offs

- **More rendering code and two layout systems** → Keep the core scene renderer unchanged when `--views` is absent; isolate Network and ADO rendering helpers and tests.
- **Long labels can overlap the isometric scene** → Filter Runtime aggressively, reserve label anchors, cap wrapping at two lines, and validate at desktop and narrow widths.
- **Orthogonal connectors can cross after responsive reflow** → Use deterministic lanes, measured anchors, collision-aware offsets, and text relationship summaries as a fallback.
- **A sidecar can drift from the scene** → Require repository identity equality, stable ID references, semantic hashes, and strict validation before rendering.
- **Private acceptance output can become irreproducible** → Deliver its scene, sidecar, deterministic receipt, run notes, and exact generation command as one private bundle.
- **Ad hoc browser probing can miss regressions** → Make the tracked Chromium DevTools harness the named acceptance gate and keep screenshots as supplemental evidence only.
- **An icon may be semantically wrong despite being visually attractive** → Validate admitted IDs and explicit mappings; fail unsupported resources instead of guessing.
- **A diagram may imply infrastructure that the source does not deploy** → Reject metadata-as-node representations, keep missing hosts absent, and render gated or inferred relationships with explicit non-solid semantics.
- **The design is approved but not visually final** → Treat browser screenshots as iterative acceptance evidence and retain reversible spacing tokens.

## Migration Plan

1. Add RED tests and the companion-sidecar validator without changing existing outputs.
2. Add optional views rendering while preserving byte-identical scene-only generation where intended.
3. Update the Azure theme's icon and label treatment.
4. Add a generic views fixture and tracked Azure example.
5. Remodel Brown and Decus inputs against the pinned audit, author sidecars, regenerate private pages and gallery, then browser-tune.
6. Keep the previous committed renderer available through scene-only invocation; rollback is removal of `--views` usage and restoration of the prior Azure theme commit.

## Open Questions

No blocking architecture question remains. Exact spacing, connector lanes, and which secondary resources survive Runtime curation remain intentional browser-tuning decisions bounded by the requirements and tests.

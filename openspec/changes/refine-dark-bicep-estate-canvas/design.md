## Design

### Layout and geometry

Retain the validated scene as the source of resource identity, paths, and evidence, but revise the Brown scene positions and renderer terrain projection for a deliberate two-territory composition. A centered top edge band contains Cloudflare and APIM. Wholesale 346 occupies the left lower territory and Decus 537 the right lower territory. Each VNet is an isometric outer plane with nested shared-style subnet planes derived from the canonical Network sidecar: Wholesale APIM, VNet Integration, and DATA subnets; Decus APP, DATA, VNet Integration, and MDOP subnets. CIDRs are labels on subnet terrain, not resource labels.

Private endpoints remain scene facts and subnet members, but the dark theme presents them as small foreground endpoint markers rather than cubes. Their associated PaaS resources remain readable behind or immediately beyond the marker, with short private-link connectors. The renderer must preserve selectable semantic IDs and evidence details even when changing the visual primitive.

The NVA is positioned inside the explicitly associated territory or adjacent to its VNet boundary with an association line. Function Apps sit in the Wholesale runtime/integration area with enough separation that endpoint markers and connectors do not occlude their labels. Resource placement uses deterministic explicit coordinates and collision validation; no force-directed or runtime-random layout is introduced.

### Rendering behavior

Add a dark-theme terrain layer for VNet and subnet regions, using stable projection geometry and shared subnet styling. Add a node presentation classification for private endpoints that changes only dark-theme rendering to marker-plus-connector treatment. Other themes and scene-only compatibility retain their existing cube behavior unless tests prove the shared primitive must be generalized.

Retain the existing path semantics and do not add a new aggregate Cloudflare-to-estate arrow. The existing direct, inferred, held, network, dependency, identity, data, and telemetry distinctions remain visible. Runtime emphasis is provided by path treatment, selection, and labels, not by removing full-estate resources.

### Verification and review

Validate the Bicep scene with `validate_scene.py`, render the dark canvas, run all `skills/isometric-system-map/tests`, and run the browser verification harness against the local artifact at desktop and narrow dimensions. Review the actual rendered screenshot for centered ingress, visible VNet/subnet planes, complete resource inventory, foreground private endpoints, NVA association, and readable function resources. Run strict OpenSpec validation and record artifact digests before implementation handoff.

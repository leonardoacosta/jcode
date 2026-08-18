## Tasks

### 1. Scene and containment geometry
- [x] 1.1 Add explicit Brown scene subnet geometry metadata sourced from the validated Network sidecar, including VNet/subnet IDs, labels, CIDRs, and node memberships.
- [ ] 1.2 Reposition all 28 Brown scene nodes into deterministic centered edge, Wholesale VNet, and Decus VNet territories without node/area/path intersections.
- [ ] 1.3 Place Cloudflare/APIM as the centered ingress band, NVA within its associated territory, and function resources clear of endpoint connector lanes.

### 2. Dark renderer presentation
- [x] 2.1 Render labeled isometric VNet terrain and nested subnet planes in the dark theme.
- [x] 2.2 Render private endpoints as selectable foreground dots/markers with associated-resource private-link connectors while retaining evidence and semantic metadata.
- [x] 2.3 Preserve full resource labels, Azure icons, path semantics, runtime emphasis, and no aggregate incoming-traffic arrow.
- [ ] 2.4 Ensure the full 28-resource inventory remains visible and selectable in the generated artifact.

### 3. Tests and artifact regeneration
- [ ] 3.1 Add validator and renderer tests for subnet geometry, VNet/subnet labels and CIDRs, endpoint marker semantics, endpoint foreground ordering, NVA association, and full inventory coverage.
- [x] 3.2 Regenerate `docs/diagrams/isometric-canvas-dark.html` from the validated Brown scene.
- [ ] 3.3 Run scene validation, the full isometric-system-map test suite, deterministic generation checks, and browser verification at desktop and narrow sizes.
- [ ] 3.4 Perform a screenshot review against the approved visual contract and fix any remaining overlap or missing-resource issue.

### 4. Review and handoff
- [ ] 4.1 Run strict OpenSpec validation and semantic self-review against proposal, design, tasks, and spec.
- [ ] 4.2 Record validation evidence and artifact digests, then hand the ready change to `/apply` for implementation.

## 1. Contract and RED validation

- [ ] 1.1 Add `references/topology-views.schema.json`, its human-readable contract, and a version 1 companion fixture covering Runtime filtering, nested Network containers and links, direct object-level evidence, and an ADO pipeline with parallel stages, a gate, and deployment targets.
- [ ] 1.2 Add failing schema and semantic-validator tests for unknown keys, invalid enums, repository identity drift, unknown scene references, duplicate IDs, containment cycles, duplicate membership, unsupported icons, missing direct evidence, invalid or cyclic pipeline edges, and Runtime projections that omit traffic-layer members.
- [ ] 1.3 Add failing renderer tests for three native tabs, URL fragments, no-JavaScript content, views hashes, approved SVG size markers, absence of Azure cube codes, visible full labels, and persistent evidence details.
- [ ] 1.4 Add failing compatibility tests proving scene-only rendering remains valid and deterministic without `--views`.

## 2. Companion sidecar and renderer integration

- [ ] 2.1 Implement the version 1 JSON Schema and strict companion-sidecar semantic validator with stable field-specific diagnostics, direct-evidence enforcement, and admitted icon/reference checks.
- [ ] 2.2 Extend `render_canvas.py` with optional `--views <path>` input, matching scene/sidecar identity checks, used-symbol collection, and separate semantic hashes.
- [ ] 2.3 Add views-enabled template structure with Runtime, Network, and ADO tab panels while preserving the existing scene-only output path.
- [ ] 2.4 Implement native tab semantics, Arrow/Home/End keyboard behavior, `#runtime`/`#network`/`#ado` deep links, selected-ID retention, and no-JavaScript document-order fallback.
- [ ] 2.5 Update scene and Canvas reference documentation with the companion contract, compatibility behavior, and validation commands.

## 3. Azure Runtime identity and curation

- [ ] 3.1 Stop drawing `node.code` in the Azure theme while preserving codes for other themes and diagnostics.
- [ ] 3.2 Increase the projected roof icon footprint to the specified minimum and add readable full-name/service-type label plates with environment and status text badges.
- [ ] 3.3 Filter Runtime nodes, paths, controls, hit regions, traffic geometry, flows, and debug counts through the declared Runtime projection.
- [ ] 3.4 Add label placement and overlap handling for desktop and narrow layouts without shrinking or reintroducing abbreviations.
- [ ] 3.5 Make Runtime selection and the persistent details region expose full identity, every citation, and every claim.

## 4. Network topology view

- [ ] 4.1 Render evidence-backed Subscription, Resource Group, VNet, and Subnet containers as an accessible nested hierarchy.
- [ ] 4.2 Render shared resource cards using admitted SVGs, full labels, service types, environment/status badges, and stable scene node IDs.
- [ ] 4.3 Render orthogonal labeled network connectors for peering, private endpoint, DNS, and data relationships with native focus targets and evidence details.
- [ ] 4.4 Implement responsive stacking, measured connector anchors, rerouting after resize, and a complete text relationship summary fallback.
- [ ] 4.5 Add static and browser tests for containment, membership, connector direction/labels, focus, 320px width, and 200 percent zoom.

## 5. ADO Pipeline view

- [ ] 5.1 Render repository, validation/build, artifact, approval/gate, deployment, and held stage cards with admitted CI/CD SVGs and full labels.
- [ ] 5.2 Implement deterministic topological ranks, declared parallel groups and lanes, directed transition labels, and deployment-target links to shared scene resources.
- [ ] 5.3 Render manual queue, automatic, approval, dependency, and held semantics using text, line treatment, badges, and evidence rather than color alone.
- [ ] 5.4 Add persistent stage/edge evidence details and preserve selected deployment-target identity across Runtime and Network tabs.
- [ ] 5.5 Add validator and browser coverage for parallelism, gates, held states, unsupported claims, keyboard navigation, narrow layouts, and no-JavaScript content.

## 6. Generic and Brown/Decus artifacts

- [ ] 6.1 Add a tracked generic companion fixture and regenerate a views-enabled Azure example while retaining valid scene-only dark and paper examples.
- [ ] 6.2 Author the Brown companion sidecar with a curated request Runtime projection, shared DEV/distinct PROD Network hierarchy, and foundation-release ADO stage graph.
- [ ] 6.3 Author the Decus companion sidecar with a curated request Runtime projection, 537 DEV/held TEST-PROD Network hierarchy, and manual-gated phase ADO graph.
- [ ] 6.4 Regenerate both private delivery bundles with `scene.json`, `views.json`, `map.html`, deterministic `generation-receipt.json`, and `run-notes.md`; update the gallery with direct Runtime, Network, and ADO links plus Network previews.
- [ ] 6.5 Browser-iterate icon scale, labels, card density, connector lanes, and responsive behavior without weakening the approved acceptance constraints.

## 7. Whole-result verification and delivery

- [ ] 7.1 Run the complete skill test suite and all scene and companion validators, including malformed numeric/reference/evidence probes.
- [ ] 7.2 Add deterministic receipt generation; prove byte-for-byte regeneration for every tracked and private HTML/receipt pair and verify scene/views/template/theme/sprite hashes and used-symbol sets.
- [ ] 7.3 Implement `skills/isometric-system-map/scripts/verify_views_browser.py` as a dependency-free Chromium DevTools harness and run its documented command over the generic, Brown, and Decus artifacts through `file://` and loopback HTTP.
- [ ] 7.4 Through the canonical harness, exercise direct fragments, Arrow/Home/End and Enter/Space tabs, selection retention, desktop, 320px width, 200 percent zoom, keyboard-only, reduced motion, JavaScript-disabled fallbacks, console/page errors, horizontal clipping, focusability, and unexpected network requests.
- [ ] 7.5 Run strict OpenSpec validation and a fresh independent semantic/visual review bound to settled artifact and output digests; fix every blocker and rerun affected checks.
- [ ] 7.6 Commit only the scoped OpenSpec, skill, tracked example, and supporting test changes; leave unrelated concurrent work and private untracked output outside the commit.

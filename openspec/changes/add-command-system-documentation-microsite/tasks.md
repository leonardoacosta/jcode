## 1. Source inventory and content traceability

- [ ] 1.1 Create `docs/diagrams/jcode-command-system/sources.json` and record the exact current source artifacts, revisions, authority class, and implementation status for each chapter.
- [ ] 1.2 In `sources.json`, map every page section, diagram, illustration, snippet, caveat, status label, human approval boundary, and evidence link to its source or to this change's explicitly approved lane-protocol authority.
- [ ] 1.3 Refresh any source whose revision changed after this feature was authored and record the replacement path.

## 2. Shared static microsite foundation

- [ ] 2.1 Create `docs/diagrams/jcode-command-system/` with `index.html`, six concept pages, `styles.css`, and `site.js`.
- [ ] 2.2 Implement the approved parchment/walnut/umber/espresso/copper token system, typography, focus states, layout, chapter rail, breadcrumbs, and previous/next navigation.
- [ ] 2.3 Implement semantic landmarks, skip links, responsive chapter navigation, reduced-motion behavior, and no-JavaScript fallbacks.
- [ ] 2.4 Keep all runtime assets local and reject remote fonts, scripts, stylesheets, images, and CDN Mermaid dependencies.

## 3. Index page

- [ ] 3.1 Add the system overview, original journey illustration, six illustrated chapter cards, and audience-specific reading paths.
- [ ] 3.2 Add a compact intent-to-evidence overview diagram with an accessible fallback.
- [ ] 3.3 Verify every index link resolves to the intended chapter and returns to the index.

## 4. Workflow chapters

- [ ] 4.1 Author `command-lifecycle.html` with `/explore`, `/feature`, `/apply`, `/apply:all`, authority gates, status labels, lifecycle diagram, and command examples.
- [ ] 4.2 Author `lane-protocol.html` with temporary lane syntax, project-name prefixes for concurrent projects, response alignment, edge cases, conversation illustration, and syntax examples; cite this change as the protocol authority and explicitly state that it is not shipped command behavior.
- [ ] 4.3 Author `apply-orchestration.html` with single versus queue execution, L1/L2/L3 levels, Orca boundaries, adversarial review, high-risk cross-provider policy, orchestration diagram, and configuration pseudocode.

## 5. Routing and evidence chapters

- [ ] 5.1 Author `model-routing.html` with model-free through frontier roles, risk/cost trade-offs, pre-classified escalation policy, routing diagram, and registry example.
- [ ] 5.2 Author `evaluation-tournament.html` with frozen descriptors, corpus partitions, randomized provider blocks, isolation, judging, holdout, safety and promotion gates, tournament diagram, and descriptor snippet.
- [ ] 5.3 Author `telemetry-results.html` from committed OAuth evidence with token classes, timings, steering digest, judge scores, limitations, no-routing-mutation decision, evidence flow diagram, and JSON excerpt.

## 6. Illustrations, diagrams, and evidence maps

- [ ] 6.1 Create one original accessible line illustration for the index and each chapter using local SVG or inline SVG.
- [ ] 6.2 Add locally available Mermaid rendering or inline SVG output for every primary diagram while preserving source/fallback text.
- [ ] 6.3 Add an evidence map to every chapter and verify approved-design, implemented, unavailable, temporary-protocol, telemetry-limitation, split-judge, no-routing-mutation, and separate-human-approval language against `sources.json`.
- [ ] 6.4 Confirm code and data snippets are repository-grounded, escaped correctly, and labeled when illustrative rather than directly executable.

## 7. Deterministic and browser validation

- [ ] 7.1 Add `scripts/test-command-system-docs.py` to validate page inventory, titles, landmarks, required sections, internal links, local assets, diagram fallbacks, snippets, evidence maps, `sources.json` coverage, truthfulness caveats, status labels, human approval boundaries, and remote-asset absence; every diagnostic must include a stable requirement ID and affected page.
- [ ] 7.2 Run the validator and expect all pages to pass with actionable per-page failures.
- [ ] 7.3 Serve the microsite on an isolated loopback port and navigate from the index to all six chapters and back without console or link errors.
- [ ] 7.4 Verify desktop and 393x852 layouts have no horizontal overflow and keep chapter navigation usable.
- [ ] 7.5 Verify keyboard focus, skip links, reduced-motion behavior, and usable prose/navigation with JavaScript disabled.
- [ ] 7.6 Compare telemetry values and judge results against the committed JSON evidence and fail on drift.
- [ ] 7.7 Compute contrast ratios for every used text, control, code, focus, and status-token pairing; require WCAG AA 4.5:1 for normal text and 3:1 for large text and non-text interactive indicators.

## 8. Review and delivery

- [ ] 8.1 Run strict OpenSpec validation for this change.
- [ ] 8.2 Run an independent semantic review for accuracy, traceability, visual coherence, accessibility, offline behavior, and overclaiming.
- [ ] 8.3 Open the index for user review through the available preview surface.
- [ ] 8.4 Commit only this change, the new microsite directory, and its focused validator without modifying active workflow lanes.

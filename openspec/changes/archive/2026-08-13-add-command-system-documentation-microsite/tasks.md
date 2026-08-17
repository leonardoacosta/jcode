## 1. Source inventory and content traceability

- [x] 1.1 Create `docs/diagrams/jcode-command-system/sources.json` and record the exact current source artifacts, revisions, authority class, and implementation status for each chapter.
- [x] 1.2 In `sources.json`, map every page element and material claim to claim text, evidence class, source path, source revision, confidence, implementation status, and this change's explicitly approved authorities where applicable.
- [x] 1.3 Refresh any source whose revision changed after this feature was authored and make the validator reject stale revisions.

## 2. Shared static microsite foundation

- [x] 2.1 Create `docs/diagrams/jcode-command-system/` with `index.html`, six concept pages, and `styles.css`; keep current-page semantics static so no shared JavaScript is required.
- [x] 2.2 Implement the approved parchment/walnut/umber/espresso/copper token system, typography, focus states, layout, chapter rail, breadcrumbs, and previous/next navigation.
- [x] 2.3 Implement semantic landmarks, skip links, responsive chapter navigation, reduced-motion behavior, and no-JavaScript fallbacks.
- [x] 2.4 Keep all runtime assets local and reject remote fonts, scripts, stylesheets, images, and CDN Mermaid dependencies.

## 3. Index page

- [x] 3.1 Add the system overview, original journey illustration, six illustrated chapter cards, and audience-specific reading paths.
- [x] 3.2 Add a compact intent-to-evidence overview diagram with an accessible fallback.
- [x] 3.3 Verify every index link resolves to the intended chapter and returns to the index.

## 4. Workflow chapters

- [x] 4.1 Author `command-lifecycle.html` with `/explore`, `/feature`, `/apply`, `/apply:all`, authority gates, status labels, lifecycle diagram, and command examples.
- [x] 4.2 Author `lane-protocol.html` with temporary lane syntax, project-name prefixes for concurrent projects, response alignment, edge cases, conversation illustration, and syntax examples; cite this change as the protocol authority and explicitly state that it is not shipped command behavior.
- [x] 4.3 Author `apply-orchestration.html` with single versus queue execution, L1/L2/L3 levels, Orca boundaries, adversarial review, high-risk cross-provider policy, orchestration diagram, and configuration pseudocode.

## 5. Routing and evidence chapters

- [x] 5.1 Author `model-routing.html` with model-free through frontier roles, risk/cost trade-offs, pre-classified escalation policy, routing diagram, and registry example.
- [x] 5.2 Author `evaluation-tournament.html` with frozen descriptors, corpus partitions, randomized provider blocks, isolation, judging, holdout, safety and promotion gates, tournament diagram, and descriptor snippet.
- [x] 5.3 Author `telemetry-results.html` from committed OAuth evidence with token classes, timings, steering digest, judge scores, limitations, no-routing-mutation decision, evidence flow diagram, and JSON excerpt.

## 6. Illustrations, diagrams, and evidence maps

- [x] 6.1 Create one original accessible line illustration for the index and each chapter using local SVG or inline SVG.
- [x] 6.2 Add locally available Mermaid rendering or inline SVG output for every primary diagram and prove the SVG, Mermaid source, and fallback contain the same material nodes, relationships, and values.
- [x] 6.3 Add an evidence map to every chapter and verify approved-design, implemented, unavailable, temporary-protocol, telemetry-limitation, split-judge, no-routing-mutation, and separate-human-approval language against `sources.json`.
- [x] 6.4 Confirm code and data snippets are repository-grounded, escaped correctly, and labeled when illustrative rather than directly executable.

## 7. Deterministic and browser validation

- [x] 7.1 Expand `scripts/test-command-system-docs.py` to validate all sixteen pages, exact DOM-to-claim coverage, source revision freshness, local stylesheet/script targets, truthfulness boundaries, and remote-asset absence; every diagnostic must use the specification's stable ID set and include page plus affected element.
- [x] 7.2 Run the validator and expect all pages to pass with actionable per-page and per-element failures.
- [x] 7.3 Add and run `python3 scripts/test-command-system-docs-browser.py --site docs/diagrams/jcode-command-system` to navigate both overview journeys, all six command chapters, all seven stack layers, and every ecosystem card without console or link errors.
- [x] 7.4 Verify desktop and 393x852 layouts have no root horizontal overflow and keep both command and atlas navigation usable.
- [x] 7.5 Verify keyboard focus, skip links, reduced-motion behavior, and usable prose/navigation with JavaScript disabled through a real browser acceptance path.
- [x] 7.6 Load telemetry totals, timings, steering facts, and judge results from committed JSON evidence and fail on drift rather than checking duplicated literals.
- [x] 7.7 Compute contrast ratios for every used text, control, code, focus, rule, and status-token pairing; require WCAG AA 4.5:1 for normal text and 3:1 for large text and non-text interactive indicators.

## 8. System Atlas overview

- [x] 8.1 Inventory `docs/diagrams/agent-stack-recreation.html` layer names, order, relationships, labels, and metaphors, then implement `python3 scripts/test-command-system-docs.py --check-atlas-source docs/diagrams/agent-stack-recreation.html` to compare the source with Atlas cards, pages, and source records while rejecting anime.js and remote assets.
- [x] 8.2 Author `agent-stack.html` in the brown field-manual visual system with one static card for surface, orchestration, context, model, tools, runtime, and memory.
- [x] 8.3 Link every atlas card to its dedicated layer page and link the atlas to the command index and Daily-Driven Ecosystem page.
- [x] 8.4 Add accessible inline SVG, Mermaid source, fallback text, source status, and evidence map for the atlas composition.

## 9. Agent-stack layer pages

- [x] 9.1 Author `stack-surface.html` and `stack-orchestration.html` using the approved layer content contract.
- [x] 9.2 Author `stack-context.html`, `stack-model.html`, and `stack-tools.html` using the approved layer content contract.
- [x] 9.3 Author `stack-runtime.html` and `stack-memory.html` using the approved layer content contract.
- [x] 9.4 Add dated evolution events and relevant daily-use examples for Claude Code, Codex, Pi, Jcode, and cross-agent workflows to each layer without forcing irrelevant harness coverage.
- [x] 9.5 Document each layer's current interfaces, authority owner, failure boundaries, and cross-layer relationships.
- [x] 9.6 Add previous, next, atlas, index, and cross-layer navigation to every layer page.

## 10. Daily-Driven Ecosystem

- [x] 10.1 Consolidate the completed Claude Code, Codex, Pi, Jcode, and cross-agent historian reports into redacted claim-level records in `ecosystem-evidence.json`, including claim IDs, observation windows, evidence classes, confidence, sanitized references, readable-source digests, and the snapshot digest referenced by `sources.json`.
- [x] 10.2 Author `daily-driven-ecosystem.html` with one linked card per harness or coordination role covering role, observed usage, strengths, friction, and cooperation boundaries.
- [x] 10.3 Distinguish measured, documented, and inferred findings visibly and preserve observation windows, confidence, and limitations.
- [x] 10.4 Add a cooperation diagram showing shared instructions, skills, lifecycle authority, swarms, review, provider selection, and durable evidence without declaring a universal winner.

## 11. Review-remediation gates

- [x] 11.1 Resolve every blocker from the first and second independent implementation reviews, including validator depth, claim/source mapping, unsupported quantitative claims, diagram parity, and all contrast failures.
- [x] 11.2 Implement and run `python3 scripts/test-command-system-docs.py --self-test` with negative fixtures proving stale revisions, missing element mappings, telemetry drift, broken atlas links, semantic diagram disagreement, remote assets, unsupported claims, and inaccessible contrast fail with stable requirement diagnostics.
- [x] 11.3 Run a fresh independent semantic review bound to final proposal, design, spec, task, source-matrix, HTML, CSS, JavaScript, and validator digests.

## 12. Review and delivery

- [x] 12.1 Run strict OpenSpec validation for the expanded change.
- [x] 12.2 Open the command index, System Atlas, representative layer page, and ecosystem page for user review through the available preview surface.
- [x] 12.3 Commit only this change, the microsite directory, and its focused validator without modifying active workflow lanes.
- [x] 12.4 Archive the completed change, merge its capability specification, and rerun repository-level validation.

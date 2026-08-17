## 1. Evidence inventory and freeze

- [x] 1.0 Confirm `add-command-system-documentation-microsite` is archived and its `command-system-docs` capability is the active merge base before implementation or archival of this change.
- [x] 1.1 Inventory the OAuth smoke descriptor, detail files, anonymized candidates, judge receipts, steering digest, provider-native telemetry, deterministic baseline, and human interpretation; map each retained datum to the measured tournament requirements.
- [x] 1.2 Inventory the retained microsite discovery, Sol refinement/autoresearch, human disposition, Luna implementation, deterministic validation, and cold-review evidence; classify missing stage evidence as unavailable.
- [x] 1.3 Define the `agent-evals.json` schema, stable ID namespaces, allowed claim statuses, dispositions, reference rules, redaction rules, source revisions, and digest algorithm.
- [x] 1.4 Create a sanitized evidence-freeze inventory proving that credentials, active-token URLs, unrelated private prompts, and third-party personal data are excluded.

## 2. RED validation contract

- [x] 2.1 Extend `scripts/test-command-system-docs.py` with `DOCS-EVALS` checks for the page, manifest, schema, globally unique IDs, reference integrity, source freshness, digests, claim mapping, static projection parity, provider-native telemetry semantics, authority separation, reciprocal links, and safe evidence classes.
- [x] 2.2 Add negative self-tests for duplicate IDs, dangling references, missing limitations, reconstructed unavailable evidence, false token normalization, unsupported winner claims, automatic-routing language, stale digests, unsafe source references, and HTML/manifest drift.
- [x] 2.3 Extend `scripts/test-command-system-docs-browser.py` with the index → Atlas → Agent Evaluations and tournament/telemetry/ecosystem journeys, filters, disclosures, evidence links, desktop/mobile, JS on/off, focus, overflow, local-only assets, console/network errors, and fallback checks.
- [x] 2.4 Run the new static, self-test, and browser checks before implementation and capture the expected RED diagnostics for missing evaluation artifacts and journeys.

## 3. Manifest implementation

- [x] 3.1 Add `docs/diagrams/jcode-command-system/agent-evals.json` with schema metadata and the measured tournament track.
- [x] 3.2 Add the microsite review track grouped into shared foundations, command core, routing/evaluation, telemetry/ecosystem, upper Atlas, lower Atlas, and cross-page visual/accessibility refinement.
- [x] 3.3 Record stable findings, evidence pointers, dispositions, implementation commits, verification independence, limitations, and explicit unavailable fields.
- [x] 3.4 Compute and record the settled manifest digest and source revisions only after content bytes stop changing.

## 4. Evaluation Atlas page

- [x] 4.1 Add `agent-evaluations.html` using the approved brown field-manual identity and progressive decision-brief-first composition.
- [x] 4.2 Implement the decision brief with outcome, confidence, authorization status, explicit non-conclusions, and links to the two evidence tracks.
- [x] 4.3 Implement the findings ledger with complete no-JavaScript content plus accessible filters for track, severity, provider/model, claim status, and disposition.
- [x] 4.4 Implement the run explorer with candidate, prompt/output availability, judges, scores, timings, tokens, cost, steering, limitations, and evidence links.
- [x] 4.5 Implement the alternating-provider review DAG with planned-versus-observed labeling and human disposition authority.
- [x] 4.6 Implement accessible telemetry charts/tables and evidence maps without false cross-provider normalization.

## 5. Microsite integration

- [x] 5.1 Link Agent Evaluations from `index.html`, `agent-stack.html`, `evaluation-tournament.html`, `telemetry-results.html`, and `daily-driven-ecosystem.html`, with reciprocal breadcrumbs and previous/next or related navigation.
- [x] 5.2 Extend `styles.css` only where the existing system lacks ledger, filter, run-explorer, DAG, or telemetry primitives; preserve contrast, focus, reduced-motion, and mobile behavior.
- [x] 5.3 Extend `sources.json` to cover every new section, control, diagram, chart, snippet, caveat, claim, and evidence link at element level.
- [x] 5.4 Verify all content and navigation remain usable from `file://`, a static server, and with JavaScript disabled.

## 6. Jcode verification and review

- [x] 6.1 Run `python3 scripts/test-command-system-docs.py --self-test` and observe every evaluation defect class.
- [x] 6.2 Run `python3 scripts/test-command-system-docs.py` and require a clean static/evidence result.
- [x] 6.3 Run `python3 scripts/test-command-system-docs.py --check-atlas-source docs/diagrams/agent-stack-recreation.html` and preserve existing Atlas fidelity.
- [x] 6.4 Run `python3 scripts/test-command-system-docs-browser.py --site docs/diagrams/jcode-command-system` and require all real-browser journeys, viewports, JS modes, filters, links, fallbacks, focus, overflow, console, and network checks to pass.
- [x] 6.5 Run `openspec validate add-agent-evaluation-evidence-atlas --strict --no-interactive` successfully.
- [x] 6.6 Freeze SHA-256 digests for proposal, design, spec, tasks, manifest, page, source inventory, shared assets, and validators.
- [x] 6.7 Obtain a fresh independent semantic review bound to those digests covering every requirement, scenario, task, check, exclusion, dependency, conflict, privacy boundary, authority boundary, and displayed finding.

## 7. WS integration and delivery

- [x] 7.1 Copy only the settled Jcode evaluation Atlas files and required navigation/shared-asset changes into the WS documentation source, preserving unrelated WS work.
- [x] 7.2 Run the real WS docs content sync, OpenAPI generation, and production build with the repository-supported Node/pnpm toolchain.
- [x] 7.3 Commit and push the scoped WS change to `dev`, record the exact commit, and monitor the exact Azure docs pipeline run to a successful terminal state.
- [x] 7.4 Verify the live Entra-gated Agent Evaluations route, internal links, and deployed asset revision as far as authenticated access permits.
- [ ] 7.5 Record the Jcode revision, WS revision, Azure run, live URL, evidence digest, limitations, and acceptance result, then send the live URL through ntfy.
  - Delivery record: Jcode `f06d2ed959894e4ba39440507fa805c780463c08`; WS `c40a4b1a7421bf4531414df33af840ecc9a293a0`; Azure docs run `58712` succeeded; live URL `https://docs.bridgespecialty.com/_docs/diagrams/jcode-command-system/agent-evaluations.html`; evidence digest `sha256:29f2cdda61a0a6a9441f335343e114944b4480b582a58cfa13f2608b4e2154dd`; unauthenticated verification reached the expected Entra sign-in boundary. ntfy delivery remains acceptance-blocked because no ntfy client/topic configuration is present on this host.

## Context

The approved outcome is a static HTML field manual with two connected maps: the existing command-system journey and a new System Atlas derived from `docs/diagrams/agent-stack-recreation.html`. The command journey retains one index and six linked concept pages. The System Atlas adds one overview, seven linked layer pages, and one Daily-Driven Ecosystem page. Source material includes repository-authoritative OpenSpec and evaluation evidence, harness repositories, session telemetry, and the existing agent-stack diagram. The visual direction remains a brown technical field manual using parchment, walnut, umber, espresso, and muted copper.

## Goals / Non-Goals

**Goals:**

- Give a new reader a navigable system-level explanation before exposing implementation detail.
- Explain the seven agent-stack layers as separately linkable technical reference pages.
- Show how Claude Code, Codex, Pi, Jcode, and cross-provider agents evolved into the user's daily workflow using evidence rather than recollection.
- Keep every substantive claim traceable to repository evidence.
- Use diagrams, illustrations, and code only where they improve comprehension.
- Remain fully usable from `file://` and an ordinary static server with no network dependency.
- Preserve readable light-paper surfaces, strong contrast, keyboard navigation, and responsive behavior.

**Non-Goals:**

- Replace OpenSpec artifacts or evaluation receipts as authority.
- Implement or alter any native command.
- Add a documentation generator, framework, build pipeline, analytics, or server dependency.
- Embed live Grafana, Recon, provider, or Command Center data.
- Claim that one OAuth smoke fixture establishes a universal model ranking.
- Modify the existing `docs/diagrams/agent-stack-recreation.html` authority or preserve its anime.js runtime in the new microsite.

## Decisions

### 1. Use sixteen standalone pages with shared static assets

The microsite lives at `docs/diagrams/jcode-command-system/` with `index.html`, six command chapter files, `agent-stack.html`, seven layer files, `daily-driven-ecosystem.html`, `styles.css`, `site.js`, and `sources.json`. `sources.json` is the implementation-owned source inventory and claim matrix. It records every page element, claim, evidence class, source path, source revision, confidence, and implementation status. Shared assets keep the visual language consistent without introducing a generator. Each page remains directly openable and linkable.

Rejected alternatives:

- One anchored HTML handbook is simpler to package but produces an overly long reading surface and weak chapter links.
- A JSON or Markdown compilation pipeline scales further but adds machinery that sixteen pages do not yet justify.

### 2. Organize the pages as one system journey

The index links to:

1. `command-lifecycle.html`: `/explore` → `/feature` → `/apply` or `/apply:all`, including authority and approval boundaries.
2. `lane-protocol.html`: temporary conversation lanes, thread labels, project-name prefixes when multiple projects are active, and response alignment.
3. `apply-orchestration.html`: single-feature versus queue execution, L1/L2/L3 orchestration, Orca ownership, review agents, and high-risk cross-provider review.
4. `model-routing.html`: model-free, economical, mid-tier, and frontier role routing with cost and risk trade-offs.
5. `evaluation-tournament.html`: frozen descriptors, randomized blocks, qualification, holdout, judging, safety, and promotion gates.
6. `telemetry-results.html`: token classes, timings, steering correlation, blind quality scores, limitations, and the OAuth smoke outcome.

The index presents these chapters as a left-to-right journey from intent to evidence. Every chapter includes breadcrumb navigation and previous/next links.

### 3. Add a separate System Atlas rather than extending the chapter rail

`agent-stack.html` recreates the seven-layer composition of `docs/diagrams/agent-stack-recreation.html` as a static brown field-manual overview. It removes anime.js, remote assets, automatic progress animation, and hidden motion state. Its cards link to:

1. `stack-surface.html`
2. `stack-orchestration.html`
3. `stack-context.html`
4. `stack-model.html`
5. `stack-tools.html`
6. `stack-runtime.html`
7. `stack-memory.html`

The overview also links to `daily-driven-ecosystem.html`. Keeping the atlas separate prevents the primary command chapter rail from mixing lifecycle concepts with platform layers.

Rejected alternatives:

- Chapters 07–13 in one continuous manual create an overloaded mobile chapter rail and blur command versus platform taxonomy.
- Leading with harness products makes the documentation autobiographical before it establishes the technical stack.

### 4. Use a brown technical-field-manual visual system

The outer frame uses espresso and walnut; reading surfaces use parchment and warm paper; accents use umber and muted copper. Editorial headings pair with compact monospaced labels. Decorative elements resemble stamped section markers, ruled field notes, and restrained hand-drawn line illustrations. The design avoids glossy dashboard styling and generic purple/blue gradients.

### 5. Use diagrams and snippets selectively

- Mermaid diagrams explain flows, state transitions, orchestration boundaries, and authority relationships.
- Each Mermaid source is embedded in-page and rendered locally by bundled assets or transformed to inline SVG during authoring. A visible text fallback remains adjacent or inside `<noscript>`.
- Small original SVG illustrations establish each chapter's metaphor without using external image assets.
- Code snippets show realistic Jcode-facing commands, descriptor fragments, telemetry JSON, and routing pseudocode. They are excerpted or adapted from repository contracts and labeled as illustrative when not executable verbatim.

### 6. Keep evidence provenance explicit

Each page ends with an “Evidence map” listing the OpenSpec changes, scripts, receipts, diagrams, repositories, or session records supporting it. Every material claim is classified as `measured`, `documented`, or `inferred`. Measured claims cite telemetry or receipts. Documented claims cite authoritative repository artifacts. Inferred claims cite their evidence and confidence and cannot be rendered as established fact. Claims about incomplete commands, Recon publication, telemetry limitations, and routing authority retain their current status rather than being presented as shipped behavior.

### 7. Use one content contract for every stack layer

Each layer page contains:

1. What the layer does, with a static line illustration and primary diagram.
2. How it evolved, as a dated chronology from repository and session evidence.
3. How the user daily-drives it, with concrete Claude Code, Codex, Pi, Jcode, and cross-agent examples relevant to that layer.
4. Current architecture, interfaces, ownership, and failure boundaries.
5. Claim-level evidence map with evidence class and confidence.
6. Previous, next, and cross-layer links.

`daily-driven-ecosystem.html` compares each harness by role, observed usage, strengths, friction, and cooperation boundaries. It does not declare a universal winner.

### 8. Validate both structure and rendered behavior

Deterministic checks verify all sixteen pages, required headings, reciprocal navigation, local asset references, no remote fonts/scripts/images, diagram fallbacks, code blocks, evidence maps, and internal links. They also verify claim-to-element coverage, source revision freshness, telemetry values loaded from committed JSON, SVG/fallback/Mermaid semantic parity, and every used contrast pair. A browser check serves the directory locally and verifies both overview journeys, every card destination, desktop and 393px layouts, keyboard focus, no horizontal overflow, no-JavaScript usability, and no console errors.

The validation interface is fixed:

- `python3 scripts/test-command-system-docs.py` performs static contract, source, evidence, telemetry, diagram-parity, contrast, and link checks.
- `python3 scripts/test-command-system-docs.py --self-test` runs negative fixtures and must observe stable failures for stale revisions, missing element mappings, telemetry drift, broken atlas links, semantic diagram disagreement, remote assets, unsupported claims, and inaccessible contrast.
- `python3 scripts/test-command-system-docs-browser.py --site docs/diagrams/jcode-command-system` starts an isolated static server, invokes installed Chromium non-interactively, and exercises every required route at desktop and 393x852 with JavaScript enabled and disabled.
- `python3 scripts/test-command-system-docs.py --check-atlas-source docs/diagrams/agent-stack-recreation.html` extracts the authoritative seven layer names and order from the source artifact and compares them with the Atlas cards and `sources.json`. It also proves that the new Atlas imports no anime.js or remote asset.

Stable validator IDs are `DOCS-INDEX`, `DOCS-ATLAS`, `DOCS-LAYER`, `DOCS-ECOSYSTEM`, `DOCS-EVIDENCE`, `DOCS-DIAGRAM`, `DOCS-TELEMETRY`, `DOCS-OFFLINE`, `DOCS-A11Y`, and `DOCS-TRUTH`. Every failure includes one of these IDs, a page, and an element or source key.

## Content Architecture

Every concept page uses the same sequence:

1. Chapter marker, title, and one-sentence promise.
2. Original line illustration.
3. “Why it exists” explanation.
4. Primary Mermaid diagram and fallback.
5. Decision cards explaining boundaries and trade-offs.
6. Repository-grounded code or data snippet.
7. Edge cases or failure modes.
8. Evidence map.
9. Previous/next navigation.

The index includes a system overview illustration, six chapter cards, a compact journey diagram, and a reading-path recommendation for product, implementation, and evaluation audiences.

The System Atlas overview preserves the existing layer order and core metaphors while translating its visual language to static parchment, walnut, umber, espresso, and copper. Each layer card is a real link, not an animated state.

## Source Boundaries

- Command lifecycle: `add-native-explore-workflow`, `add-native-feature-workflow`, and `add-native-apply-workflows`.
- Lane protocol: this OpenSpec change is the durable authority for the explicitly approved temporary conversation syntax: the number of lanes requested equals the number of discussion threads; each response identifies its lane; and when more than one project is active, the response begins with the project name. The chapter MUST label this as a conversation protocol rather than shipped command behavior.
- Apply orchestration: `add-native-apply-workflows` and `optimize-orca-command-center-orchestration`.
- Model routing and tournament: `add-model-routing-evaluation-tournament` plus frozen descriptors and scripts under `evals/model-routing/`.
- Telemetry/results: `evals/model-routing/runs/oauth-smoke-2026-08-12.json`, detailed attempt files, judges, and `jcode-telemetry-core` field definitions.
- Agent-stack taxonomy and composition: `docs/diagrams/agent-stack-recreation.html`; the new pages do not replace this source artifact.
- Claude Code evolution and daily use: `~/dev/claude` command, workflow, telemetry, and research artifacts plus Claude session records.
- Codex evolution and daily use: `~/dev/codex`, `~/.codex/history.jsonl`, Codex sessions, and native Codex workflow contracts.
- Pi evolution and daily use: `~/dev/pi/agent`, `~/.pi/agent/sessions`, Pi subagent contracts, and Jcode Pi integration references.
- Jcode evolution and daily use: native Jcode workflow skills, Command Center artifacts, prompt history, telemetry documentation, and Jcode sessions.
- Cross-agent ecosystem: `~/.agents`, projected harness instructions, skill lifecycle artifacts, and cross-provider review evidence.

Historian output is discovery evidence, not a stable runtime dependency. Implementation creates `docs/diagrams/jcode-command-system/ecosystem-evidence.json` as a redacted, frozen evidence snapshot. Each record contains a claim ID, harness, observation window, evidence class, confidence, repository-relative or home-relative source reference, source digest where locally readable, and a sanitized session reference. Raw prompts, credentials, tokens, user-authored private prose unrelated to the claim, and third-party personal data are excluded. `sources.json` references this frozen snapshot and its digest. The validator never reads mutable live session stores during normal acceptance.

If a source claim conflicts with newer repository state during implementation, the page must use the newer state and record the refreshed evidence path.

## Risks / Trade-offs

- **Documentation drift** → cite evidence paths and test required status labels.
- **Mermaid unavailable offline** → bundle locally or author inline SVG plus a text fallback; never rely on a CDN.
- **Sixteen pages diverge visually** → centralize tokens/components in one stylesheet and minimal shared script.
- **Session history becomes accidental authority** → classify session-derived conclusions as inferred unless corroborated by repository or telemetry evidence.
- **Existing animated diagram semantics drift during translation** → preserve layer names, order, primary relationships, and source linkage; validate the static atlas against the source artifact.
- **Illustrations become decorative noise** → require each illustration to encode a chapter metaphor and provide accessible labeling.
- **Results overstate the smoke run** → retain “one fixture,” provider-accounting, TTFT, queue-time, and no-routing-mutation caveats.
- **Active feature lanes conflict** → treat their artifacts as read-only sources and limit writes to this change, the microsite directory, and its validator.

## Review Remediation Ledger

The implementation must close these durable review findings before any task or completion claim is accepted:

1. Keep mobile chapter navigation visible and horizontally operable; include breadcrumbs on every page.
2. Validate contract behavior rather than accepting arbitrary traceability keys, CSS classes, or duplicated literals.
3. Require inline SVG, Mermaid source, and fallback text to describe the same material diagram.
4. Replace page-level source summaries with exact element and claim mappings.
5. Remove or explicitly classify unsupported quantitative claims, including any uncited percentage of model-free work.
6. Fix every contrast failure, including focus indicators on paper/parchment and low-contrast decorative rules when they communicate state or structure.
7. Keep task checkboxes truthful after any scope expansion or failed independent review.

## Validation Strategy

- Run the microsite validation script and expect all required pages, links, snippets, claim-level evidence maps, truthfulness caveats, status labels, human approval boundaries, and offline constraints to pass. Every failure identifies a requirement ID and affected page.
- Parse every HTML file and verify one `main`, one top-level heading, landmarks, labels, and unique titles.
- Serve the directory on an isolated loopback port and exercise index navigation to every chapter.
- Check desktop and 393x852 viewports for overflow and navigation visibility.
- Disable JavaScript and confirm prose, code, diagram fallback text, and navigation remain usable.
- Compare telemetry/result values against the committed JSON evidence rather than manually duplicated estimates.
- Compare `sources.json` revisions against current source bytes and reject stale or missing claim mappings.
- Verify rendered SVG, Mermaid source, and fallback text describe the same nodes, relationships, and material values.
- Navigate every System Atlas card and every Daily-Driven Ecosystem card through a real browser.
- Verify the index links to both the six command chapters and `agent-stack.html`; verify the Atlas links to seven layers plus `daily-driven-ecosystem.html`.
- Compute WCAG contrast ratios for every foreground/background design token pair used by body text, labels, links, controls, code, focus indicators, and status badges; require at least 4.5:1 for normal text and 3:1 for large text and non-text interactive indicators.

## Open Questions

None. The original command-system scope and the expanded System Atlas architecture, layer-page content contract, Daily-Driven Ecosystem page, claim classification, and validation contract were explicitly approved on 2026-08-12.

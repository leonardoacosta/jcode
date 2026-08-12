## Context

The approved outcome is a seven-page static HTML microsite: one index and six linked concept pages. The source material is repository-authoritative OpenSpec and evaluation evidence, especially the native workflow changes, Orca orchestration design, lane protocol decisions, model-routing tournament, and OAuth smoke receipt. The visual direction is a brown technical field manual using parchment, walnut, umber, espresso, and muted copper.

## Goals / Non-Goals

**Goals:**

- Give a new reader a navigable system-level explanation before exposing implementation detail.
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

## Decisions

### 1. Use seven standalone pages with shared static assets

The microsite lives at `docs/diagrams/jcode-command-system/` with `index.html`, six chapter HTML files, `styles.css`, `site.js`, and `sources.json`. `sources.json` is the implementation-owned source inventory and content matrix: it records every page section, diagram, illustration, snippet, caveat, status label, and evidence path. Shared assets keep the visual language consistent without introducing a generator. Each page remains directly openable and linkable.

Rejected alternatives:

- One anchored HTML handbook is simpler to package but produces an overly long reading surface and weak chapter links.
- A JSON or Markdown compilation pipeline scales further but adds machinery that seven pages do not justify.

### 2. Organize the pages as one system journey

The index links to:

1. `command-lifecycle.html`: `/explore` → `/feature` → `/apply` or `/apply:all`, including authority and approval boundaries.
2. `lane-protocol.html`: temporary conversation lanes, thread labels, project-name prefixes when multiple projects are active, and response alignment.
3. `apply-orchestration.html`: single-feature versus queue execution, L1/L2/L3 orchestration, Orca ownership, review agents, and high-risk cross-provider review.
4. `model-routing.html`: model-free, economical, mid-tier, and frontier role routing with cost and risk trade-offs.
5. `evaluation-tournament.html`: frozen descriptors, randomized blocks, qualification, holdout, judging, safety, and promotion gates.
6. `telemetry-results.html`: token classes, timings, steering correlation, blind quality scores, limitations, and the OAuth smoke outcome.

The index presents these chapters as a left-to-right journey from intent to evidence. Every chapter includes breadcrumb navigation and previous/next links.

### 3. Use a brown technical-field-manual visual system

The outer frame uses espresso and walnut; reading surfaces use parchment and warm paper; accents use umber and muted copper. Editorial headings pair with compact monospaced labels. Decorative elements resemble stamped section markers, ruled field notes, and restrained hand-drawn line illustrations. The design avoids glossy dashboard styling and generic purple/blue gradients.

### 4. Use diagrams and snippets selectively

- Mermaid diagrams explain flows, state transitions, orchestration boundaries, and authority relationships.
- Each Mermaid source is embedded in-page and rendered locally by bundled assets or transformed to inline SVG during authoring. A visible text fallback remains adjacent or inside `<noscript>`.
- Small original SVG illustrations establish each chapter's metaphor without using external image assets.
- Code snippets show realistic Jcode-facing commands, descriptor fragments, telemetry JSON, and routing pseudocode. They are excerpted or adapted from repository contracts and labeled as illustrative when not executable verbatim.

### 5. Keep evidence provenance explicit

Each chapter ends with an “Evidence map” listing the OpenSpec changes, scripts, receipts, or diagrams supporting it. Claims about incomplete commands, Recon publication, telemetry limitations, and routing authority retain their current status rather than being presented as shipped behavior.

### 6. Validate both structure and rendered behavior

Deterministic checks verify all seven pages, required headings, reciprocal navigation, local asset references, no remote fonts/scripts/images, diagram fallbacks, code blocks, evidence maps, and internal links. A browser check serves the directory locally and verifies the index-to-chapter journey, desktop and 393px layouts, keyboard focus, no horizontal overflow, and no console errors.

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

## Source Boundaries

- Command lifecycle: `add-native-explore-workflow`, `add-native-feature-workflow`, and `add-native-apply-workflows`.
- Lane protocol: this OpenSpec change is the durable authority for the explicitly approved temporary conversation syntax: the number of lanes requested equals the number of discussion threads; each response identifies its lane; and when more than one project is active, the response begins with the project name. The chapter MUST label this as a conversation protocol rather than shipped command behavior.
- Apply orchestration: `add-native-apply-workflows` and `optimize-orca-command-center-orchestration`.
- Model routing and tournament: `add-model-routing-evaluation-tournament` plus frozen descriptors and scripts under `evals/model-routing/`.
- Telemetry/results: `evals/model-routing/runs/oauth-smoke-2026-08-12.json`, detailed attempt files, judges, and `jcode-telemetry-core` field definitions.

If a source claim conflicts with newer repository state during implementation, the page must use the newer state and record the refreshed evidence path.

## Risks / Trade-offs

- **Documentation drift** → cite evidence paths and test required status labels.
- **Mermaid unavailable offline** → bundle locally or author inline SVG plus a text fallback; never rely on a CDN.
- **Seven pages diverge visually** → centralize tokens/components in one stylesheet and minimal shared script.
- **Illustrations become decorative noise** → require each illustration to encode a chapter metaphor and provide accessible labeling.
- **Results overstate the smoke run** → retain “one fixture,” provider-accounting, TTFT, queue-time, and no-routing-mutation caveats.
- **Active feature lanes conflict** → treat their artifacts as read-only sources and limit writes to this change, the microsite directory, and its validator.

## Validation Strategy

- Run the microsite validation script and expect all required pages, links, snippets, evidence maps, truthfulness caveats, status labels, human approval boundaries, and offline constraints to pass. Every failure identifies a requirement ID and affected page.
- Parse every HTML file and verify one `main`, one top-level heading, landmarks, labels, and unique titles.
- Serve the directory on an isolated loopback port and exercise index navigation to every chapter.
- Check desktop and 393x852 viewports for overflow and navigation visibility.
- Disable JavaScript and confirm prose, code, diagram fallback text, and navigation remain usable.
- Compare telemetry/result values against the committed JSON evidence rather than manually duplicated estimates.
- Compute WCAG contrast ratios for every foreground/background design token pair used by body text, labels, links, controls, code, focus indicators, and status badges; require at least 4.5:1 for normal text and 3:1 for large text and non-text interactive indicators.

## Open Questions

None. Scope, architecture, page inventory, and visual direction were explicitly approved on 2026-08-12.

---
name: repo-map
description: >
  Generate an interactive, self-contained HTML architecture map for any repo — a contract-driven
  scan (roles, not files) rendered as a fixed styled page with cards, connectors, and a left rail.
  Use whenever the request is "map this repo", "architecture map", "interactive map of the
  codebase", "repo map page", or similar — a low-fidelity request wants `ascii-wireframe` or
  `wayfinder` instead; this skill is for the specific contract-driven repo/architecture map
  shape. Adapted from foglamp-labs/foglamp's Scan pipeline (Apache-2.0).
license: Apache-2.0 (adapted; see references/contract.md)
allowed-tools: Read, Write, Bash, Glob, Grep
---

# repo-map — Contract-Driven Repo Architecture Map

You emit only DATA — never HTML, CSS, colors, icons, or positions. A fixed renderer owns 100% of
styling and layout; the same data always produces the same page. This split is the whole point:
supporting a new stack requires zero renderer changes, only a new extraction guide below.

## Contract

Full JSON shape, kind vocabulary, caps, and integrity rules: `references/contract.md`. Every
document you emit MUST pass `scripts/bin/repo-map-render --validate <json>` before rendering.

## Per-stack extraction guides

`references/extraction.md` — semantic "how to investigate" guidance per stack family (C#,
Next.js, Go, Swift, Bicep, Terraform), each ending with a role-mapping table onto the contract's
10-kind vocabulary. Read the guide matching the repo's detected stack(s) before extracting.

## Render / watch

```bash
scripts/bin/repo-map-render <scan.json> -o docs/diagrams/<repo>-map.html
scripts/bin/repo-map-render --watch <scan.json> -o docs/diagrams/<repo>-map.html
```

`--offline` skips favicon network fetches (falls back to kind glyphs). Template mechanics and
maintenance notes: `references/renderer.md`.

## Curation rule

Favor the few flows that matter over an exhaustive dependency dump — the contract's hard caps
(nodes<=24, edges<=48) force this. If the repo has more than fits, pick the architecturally
load-bearing paths, not everything you can find.

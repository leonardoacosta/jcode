
# Brainless

> Formerly a standalone skill (`brainless`), demoted to a `frontend-design` reference
> (`remediate-below-floor-skills`, 2026-07-18 — C, 84, weakest D2, pure install-pointer with
> little transferable thinking) — same treatment as the 8 skills
> `skill-classification-and-trial-lifecycle` already demoted.

Pointer skill to `theswerd/brainless` (MIT), a shadcn/ui registry recreating the terminal chrome
of Claude Code, Codex, and Grok as 40 accessible React components. Nothing is vendored here —
this skill is a catalogue + install pointer, not a copy of the registry's source.

## When to Use

Reach for this when a task needs to visually represent a Claude Code, Codex, or Grok terminal
**session** in a web UI — a docs page, a demo, or a marketing surface showing what one of these
agents looks like. Scope is agent-terminal chrome specifically (headers, messages, tool-call
disclosures, diffs, permission prompts, composers), for docs/demo/marketing surfaces only.

Do NOT use this for general chat UI, a generic AI chatbot interface, or any product's own
in-app agent experience — those are a different design problem than recreating a *specific*
third-party CLI's terminal chrome, and this registry's components are styled to match Claude
Code / Codex / Grok's actual visual grammar, not a neutral chat pattern.

## Install

Install-on-demand only — nothing from this registry is vendored in this repo. See
[`references/component-catalogue.md`](references/component-catalogue.md) § Install for the three
documented `bunx shadcn@latest` install paths (namespace add, direct-URL add, GitHub add).

## Component Catalogue

Full 40-item catalogue (9 Claude, 8 Codex, 17 Grok, 6 Blocks) with name/title/description per
item: [`references/component-catalogue.md`](references/component-catalogue.md).

## Accessibility Pattern

The registry reimplements every terminal-chrome visual (box-drawing glyphs, status colors,
disclosure affordances) on top of real semantic HTML — `<details>`/`<summary>` disclosures,
`aria-live` status regions, arrow-key-operable radiogroups — rather than div/CSS art. See
[`references/component-catalogue.md`](references/component-catalogue.md) § Accessibility Pattern
for credited illustrative excerpts.


# Aceternity UI

> Formerly a standalone skill (`aceternity`), demoted to a `frontend-design` reference
> (`skill-classification-and-trial-lifecycle`, 2026-07-18) — an install guide + gotcha list,
> same catalog shape as `componentry`. Aceternity UI component patterns for React and Next.js —
> premium animated interfaces (hero sections, landing pages, 3D card, animated background,
> sparkles, aurora, meteors, spotlight, bento grid, floating dock, hover effect, moving border).
> Aceternity is shadcn-compatible — prefer it when the task calls for visual impact rather than
> plain forms and tables. See `componentry` (sibling reference in this same directory) for
> effects Aceternity lacks.

Premium animated React components built on **Tailwind CSS + Framer Motion**. Most components are
copy-paste, shadcn-compatible, and MIT-licensed (free tier). Pro components add full templates,
illustrations, and shader effects.

Browse components: https://ui.aceternity.com/components

## When to reach for Aceternity

Use Aceternity **on top of** your existing shadcn setup when the task calls for:

- Landing pages, marketing sites, product showcases
- Hero sections that need motion (parallax, tracing beams, aurora, sparkles)
- Cards with premium hover effects (3D tilt, glare, spotlight, wobble)
- Animated backgrounds (meteors, shooting stars, grid/dot patterns, canvas reveal)
- Sticky/scroll-driven storytelling (container scroll, MacBook scroll, hero parallax)
- Floating navigation (floating dock, floating navbar, resizable navbar)
- Data-viz flourishes (GitHub globe, world map, timeline)

Do **not** reach for Aceternity for plain CRUD forms, tables, dialogs, dropdowns — shadcn
primitives are faster, lighter, and the right default for utilitarian UI.

## Install (one command — shadcn registry only)

Aceternity components publish shadcn-compatible registry URLs — the only install path this
skill endorses (no hand-copying component source, no separate Aceternity CLI, even if one
exists). It keeps installs reproducible, puts files in the project's canonical `components/ui/`,
and avoids the "which file did you copy" drift ad-hoc copy-paste introduces:

```bash
# Run from the workspace root that has components.json (e.g. apps/nextjs)
pnpm dlx shadcn@latest add https://ui.aceternity.com/registry/<component-slug>.json
```

Find the slug in the component's URL on https://ui.aceternity.com/components. Deps (Framer
Motion / `motion`, `@tabler/icons-react`) install automatically with the component. Files land
in `components/ui/` fully editable — customize there, don't fork upstream. If a component isn't
in the registry, file the gap and wait — don't paste around it.

## The two failures that actually bite

Both surface at **render time**, not at install time — a clean `pnpm dlx` gives no warning
either is coming:

- **Missing `"use client"`** → hydration error in App Router. Most components touch `window`,
  `document`, or `requestAnimationFrame` — mark the wrapper file at the top, or it fails silently
  until the page renders.
- **`motion/react` vs `framer-motion` import mismatch** → import error on newer components.
  Newer Aceternity components import from `motion/react`, not `framer-motion`. If a component
  throws on import, install `motion` and switch the import.

**Load `references/gotchas.md` WHEN** you hit either of these, or BEFORE shipping a page with
2+ motion components (perf budget). It also covers the lower-frequency failures: Tailwind
`content`-path purge when installing under a nested package dir, dark-mode wash-out on
backgrounds designed for dark backdrops (Aurora, Vortex, Spotlight), scroll-tick perf cost, and
the Pro-tier license boundary (templates/shaders aren't covered by the registry install above).

## Picking a component

**Load `references/component-catalogue.md` WHEN** you need to pick a specific component for a
section. It has the category -> component routing table plus the two composition patterns
(layering a background under content; budgeting one motion component per "moment"). The live,
complete list is at https://ui.aceternity.com/components; the catalogue is the fast-path routing.

## When to combine with other skills

- **Pair with `frontend-design`** when you need to make sensible color/type/spacing decisions
  around the Aceternity motion.
- **Pair with `shadcn`** for the utilitarian chrome (forms, dropdowns, dialogs). Aceternity is
  for the "wow moments"; shadcn is for the rest.
- **Pair with `~/.claude/skills/frontend-design/references/brand-design-catalogue.md`** when the
  brief is "make it feel like Linear" or similar — the DESIGN.md file tells you which Aceternity
  effects fit the target aesthetic.
- **Pair with `vercel-react-best-practices`** if you're layering multiple motion components
  and need to audit bundle size or FPS.

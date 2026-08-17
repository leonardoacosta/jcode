
# Componentry

> Formerly a standalone skill (`componentry`), demoted to a `frontend-design` reference
> (`skill-classification-and-trial-lifecycle`, 2026-07-18) — fundamentally a component-library
> catalog; the only real decision content ("when to reach for Componentry vs Aceternity") is
> preserved below. Componentry animated React components — sibling shadcn registry to
> Aceternity, for effects it lacks. Triggers: dither, split-flap display, matrix rain, WebGL
> liquid/plasma/aurora hero, particle typography, magnetic dock, command menu, github
> contribution calendar, flight status card, pixel canvas, image trail, scramble text, and
> retro/CRT/terminal or shader-heavy UI effects.

# Componentry

Animated React primitives built on **Tailwind CSS + Framer Motion + WebGL/three.js**, distributed
through the shadcn registry (`componentry.fun`). MIT-licensed (repo: `harshjdhv/componentry`).
Same product category as Aceternity — reach for it when the effect you need is in *this* catalog
and not that one.

Browse components: https://componentry.dev/docs/

## When to reach for Componentry (vs Aceternity)

Aceternity stays the default for premium animated UI — it is more mature and its catalogue is
broader. Componentry earns the install when the task calls for its differentiators:

- **Dither/retro aesthetics** — dithered logo, dither gradient, dither-prism hero, pixel canvas,
  matrix rain, split-flap display, noise texture (CRT/terminal/airport-board flavor)
- **Shader-grade hero backgrounds** — WebGL liquid, closing plasma, silk aurora, prism gradient,
  liquid chrome, particle galaxy (three.js/canvas, heavier than Aceternity's CSS/Framer heroes)
- **Cursor-physics interactions** — magnetic dock, magnet lines, eye tracking, text repel,
  particle typography, image trail, infinite image field
- **Ready-made product widgets** — Spotlight-style command menu, GitHub contribution calendar,
  flight status card, music player, auth modal, mac keyboard

Do **not** reach for Componentry for plain CRUD chrome (shadcn), for a single element that needs
easing (`motion-and-transitions`), or when an Aceternity component already covers the effect —
one registry per page section keeps the dependency surface sane.

## Install (shadcn registry, namespaced)

One-time per project, add the registry to `components.json`:

```json
{ "registries": { "@componentry": "https://componentry.fun/r/{name}.json" } }
```

Then install by namespaced slug from the workspace root that has `components.json`:

```bash
pnpm dlx shadcn@latest add @componentry/magnetic-dock
```

Deps (`framer-motion`, and for shader components `three` + `@react-three/fiber` + `@react-three/drei`)
install automatically. Files land in `components/ui/` fully editable — customize there, don't
fork upstream.

## The failures that actually bite

- **Missing `"use client"`** → hydration error in App Router. Nearly every component reads
  `window`, cursor position, or scroll state — same failure mode as Aceternity; mark the wrapper.
- **three.js weight** — the WebGL heroes (`dither-prism-hero`, `hero-geometric`,
  `image-ripple-effect`, `particle-galaxy`) pull `three` + `@react-three/fiber` + `@react-three/drei`
  (~150KB+ gzipped before the component). Budget ONE shader background per page, lazy-load it,
  and prefer the canvas-only variants (`dither-gradient`, `pixel-canvas`, `matrix-rain`) when the
  effect allows.
- **Provenance caution** — component pages carry "inspired by various open-source projects...
  verify licenses before using in production." The repo is MIT, but individual components may be
  uncredited ports; for client/commercial work, eyeball the installed source before shipping.
- **Young upstream** (created 2025-12, ~250 stars) — pin what you install (files are vendored
  into `components/ui/` anyway) and expect API churn if you re-add later.

## Picking a component

**Load `references/component-catalogue.md` WHEN** you need to pick a specific component for a
section. It has the full category -> component table (all 54 registry items, deps flagged) plus
the componentry-vs-aceternity routing call. The live list is at https://componentry.dev/docs/.

## When to combine with other skills

- **Pair with `aceternity`** — decide per "moment" which registry owns it; don't stack both
  registries' background components on one page.
- **Pair with `shadcn`** for utilitarian chrome; Componentry is for the wow moments only.
- **Pair with `frontend-design`** for color/type/spacing decisions around the motion — the
  dither family especially needs a deliberate palette or it reads as noise.
- **Pair with `vercel-react-best-practices`** before shipping any page with a three.js component
  (bundle + FPS audit).

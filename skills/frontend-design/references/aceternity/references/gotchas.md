
# Aceternity Gotchas

> Load WHEN a component errors at runtime (hydration error, import failure, classes purged,
> washed-out colors) or BEFORE shipping a page with 2+ motion components. These are the failure
> modes that experience teaches — check here first when an Aceternity component misbehaves.

- **SSR + Framer Motion** — many components touch `window`, `document`, or
  `requestAnimationFrame`. Mark the wrapper file with `"use client"` at the top. Omitting it
  produces hydration errors in App Router (not a build error — it fails at render).
- **Framer Motion v11 / motion package split** — newer components import from `motion/react`
  rather than `framer-motion`. If a component throws on import, install `motion` alongside (or
  instead of) `framer-motion` and update the import to `motion/react`.
- **Tailwind `content` paths** — if you install under a package directory (e.g. `packages/ui/`),
  the consuming app's Tailwind `content` globs must include that path or the component's classes
  get purged and it renders unstyled.
- **Dark-mode assumptions** — several backgrounds (Aurora, Vortex, Spotlight) are designed for
  dark backdrops. On light themes they wash out. Pin a dark section around them or swap colors
  via props.
- **Perf budget** — scroll-driven components (Hero Parallax, Sticky Scroll) recalc on every
  scroll tick. Use at most one per page and place it below the fold when possible.
- **Pro components aren't free** — the Pro tier (templates, shader effects, premium blocks)
  requires a paid license and is NOT covered by the registry installs this skill endorses. Stick
  to free components unless the project confirms Pro access.


# Componentry Component Catalogue

> Load WHEN you need to pick a specific Componentry component for a section. All 54 registry
> items as of 2026-07-13 (`componentry.fun/r/{name}.json`, verified against the repo's
> `apps/web/public/r/` tree). Install: `pnpm dlx shadcn@latest add @componentry/<slug>` after the
> one-time `components.json` registry entry (see SKILL.md). The live list is at
> https://componentry.dev/docs/.

## Decide before you browse: componentry vs aceternity vs shadcn

| The UI need is... | Reach for | Why |
|---|---|---|
| Dither/retro/CRT flavor, split-flap boards, matrix rain, shader-grade WebGL heroes, cursor-physics toys, product widgets (command menu, GH calendar, flight card) | **componentry** (this table) | These effects are not in Aceternity's catalogue — hand-rolling a Bayer-dither canvas or a split-flap animation is a day-sink |
| General premium motion — 3D cards, aurora/sparkles/meteors, tracing beams, bento grids, floating navbars | **aceternity** | Broader, more mature catalogue; default registry for animated "wow moments" |
| Utilitarian chrome — forms, tables, dialogs, nav | **shadcn** | Faster, lighter, no motion budget |
| One existing element needs easing/polish | **motion-and-transitions** | Nine copy-paste CSS snippets; a registry install is the wrong tool |
| Pre-built marketing/landing sections (hero, pricing, feature grid, CTA) to compose fast | **tailark** | Full marketing-block library, faster than hand-building a section from primitives — de-slop caveat: a pulled block is a head start, not a free pass; run it through the anti-slop canon before shipping |
| Curated animated-component registry, an alternative source when aceternity/componentry don't have the piece | **motion-primitives** | Broad motion.dev-adjacent animated-primitive registry — same de-slop caveat: vet against the anti-slop canon before shipping |
| Curated animated-component registry, another alternative source | **kokonut UI** | Another vetted animated-component collection — same de-slop caveat: vet against the anti-slop canon before shipping |

**Dependency weight tiers** — every row below flags its heaviest dep:
`FM` = framer-motion only (moderate) · `3JS` = three + @react-three/fiber/drei (~150KB+, budget
one per page, lazy-load) · `—` = zero runtime deps (canvas/CSS only, cheapest).

## Text animations

| Component | Slug | Deps | Best for |
|---|---|---|---|
| Velocity Scroll | `scroll-based-velocity` | FM | Marquee text that skews/speeds with scroll velocity |
| Letter Cascade | `letter-cascade` | — | Per-letter entrance choreography for headlines |
| Text Repel | `text-repel` | — | Characters that flee the cursor — playful headlines |
| Kinetic Text Reveal | `kinetic-text-reveal` | FM | Directional reveal w/ blur + word/char/line stagger |
| Particle Typography | `cursor-driven-particle-typography` | — | Text as cursor-reactive particle field |
| Hyper Text | `hyper-text` | — | Cyberpunk scramble-then-reveal effect |
| Text Animate | `text-animate` | FM | General text entrance animations |

## Hero backgrounds

| Component | Slug | Deps | Best for |
|---|---|---|---|
| Hero Geometric | `hero-geometric` | 3JS | Geometric shader hero with premium typography |
| Dither Prism Hero | `dither-prism-hero` | 3JS | Dithered prismatic/holographic WebGL hero |
| WebGL Liquid | `webgl-liquid` | — | Liquid flow hero, customizable palette/grain |
| Silk Aurora | `silk-aurora` | — | Soft aurora ribbon background |
| Closing Plasma | `closing-plasma` | — | Plasma background w/ motion + interaction knobs |
| Animated Gradient | `animated-gradient` | — | Cheap animated gradient wash |
| Prism Gradient | `prism-gradient` | — | Theme-aware WebGL prism field, electric refraction |
| Liquid Chrome | `liquid-chrome` | — | Chrome/metal liquid surface |
| Gradient Hero 01 | `gradient-hero-01` | — | Complete centered hero section, warm gradient + CTAs |
| Liquid Blob | `liquid-blob` | FM | Organic morphing blobs w/ mouse interaction |
| Particle Galaxy | `particle-galaxy` | 3JS | Star-field/galaxy particle background |

## Visual effects

| Component | Slug | Deps | Best for |
|---|---|---|---|
| Dithered Logo | `dithered-logo` | — | Logo as dithered particle canvas w/ cursor ripples |
| Dither Gradient | `dither-gradient` | — | Bayer-matrix dithered gradient (retro texture) |
| Image Trail | `image-trail` | — | Images trailing the cursor across a section |
| Image Ripple Effect | `image-ripple-effect` | 3JS | Cursor-driven WebGL ripple over image cards |
| Ripple Transition | `ripple-transition` | FM | WebGL image transitions, click-origin ripples |
| Infinite Image Field | `infinite-image-field` | — | Endless pannable photo canvas (galleries) |
| Magnet Lines | `magnet-lines` | FM | Line grid rotating toward cursor (magnetic field) |
| Pixel Canvas | `pixel-canvas` | — | Hover-lit pixel grid with decay trails |
| Matrix Rain | `matrix-rain` | — | Falling-glyph terminal rain |
| Noise Texture | `noise-texture` | — | Animated grain overlay, blend-mode aware |

## Interactive components & widgets

| Component | Slug | Deps | Best for |
|---|---|---|---|
| Magnetic Dock | `magnetic-dock` | FM | macOS dock w/ spring magnification (≈ Aceternity Floating Dock — pick one) |
| Command Menu | `command-menu` | FM + cmdk | Spotlight-style ⌘K palette, animated search |
| GitHub Calendar | `github-calendar` | FM | Contribution-graph visualization, variants + scaling |
| Flight Status Card | `flight-status-card` | FM | Dot-matrix airport codes, progress, ETA widget |
| Split Flap Display | `split-flap-display` | — | Airport/train departure-board flip text |
| Music Player | `music-player` | — | Full player widget |
| Auth Modal | `auth-modal` | FM | Animated auth modal w/ social logins |
| Bouncy Accordion | `bouncy-accordion` | FM | Spring-physics accordion |
| Circuit Board | `circuit-board` | FM | Animated electricity paths between nodes |
| Mac Keyboard | `mac-keyboard` | — | Interactive keyboard visualization |
| Eye Tracking | `eye-tracking` | — | Eyes that follow the cursor |
| Signature | `signature` | FM + opentype.js | Animated handwriting/signature draw-on |
| Scrub Input | `scrub-input` | — | Inline pill slider for smooth variable scrubbing |
| Collection Surfer | `collection-surfer` | — | Browsable collection carousel |
| Scroll Choreography | `scroll-choreography` | — | Multi-element scroll-driven sequences |
| Sticky Scroll Cards | `sticky-scroll-cards` | FM + lenis | Stacked cards pinning through scroll |
| Scroll Split Card | `scroll-split-card` | — | Card that splits open on scroll |
| Layered Stack | `layered-stack` | — | Depth-layered card stack |
| Orbit Card Stack | `orbit-card-stack` | FM | Hover deck that fans outward, lifts active card |
| Showcase Card | `showcase-card` | FM | 3D tilt + parallax image portfolio card |
| Spotlight Card | `spotlight-card` | — | Cursor-following spotlight + gradient border card |
| Testimonial Marquee | `testimonial-marquee` | FM | Infinite social-proof marquee |

## Buttons

| Component | Slug | Deps | Best for |
|---|---|---|---|
| Border Beam | `border-beam` | FM | Gradient beam traveling a container border |
| Interactive Hover Button | `interactive-hover-button` | — | Text + arrow reveal on hover |
| Pulsating Button | `pulsating-button` | — | Pulsing glow CTA |
| Shimmer Button | `shimmer-button` | — | Shimmering light sweep CTA |

## Composition rules

- **One shader background per page** (`3JS` rows and the WebGL `—` heroes) — lazy-load it and
  keep everything above it DOM/CSS. Two WebGL canvases compete for the same GPU frame budget.
- **Dock overlap**: `magnetic-dock` duplicates Aceternity's Floating Dock — pick whichever
  registry the page already uses; never install both docks in one project.
- **Dither family reads as a system** — `dithered-logo` + `dither-gradient` + `split-flap-display`
  + `matrix-rain` share a retro/terminal aesthetic; mixing one into an otherwise-glossy page
  needs a deliberate palette bridge (see `frontend-design`).
- Same layering pattern as Aceternity: background components are absolute-positioned full-bleed —
  wrap in a `relative` container, content above with `relative z-10`.

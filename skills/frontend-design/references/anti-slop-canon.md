
# Anti-Slop Canon

> Full reference for the `frontend-design` skill. The SKILL.md body carries the top-signal
> subset; this file holds the complete tables. Adapted from `d-o-hub/anti-ai-slop`
> (audited 2026-05-24) and reconciled with our skill philosophy.
>
> **Boundary:** Loading / empty / error STATE patterns are owned by the `state-handling`
> skill — see there, not here. This file covers visual slop + non-state interaction slop.

## Canon of Slop (Visual)

The 2024–2026 AI-generated design monoculture. Name the pattern before fixing it.

| # | Pattern | Signature (what to grep your eye for) | Do instead |
|---|---|---|---|
| 1 | Purple gradient hero | `#7c3aed → #2563eb` on white | Real color theory — complementary/analogous/split-complementary, chosen for the context |
| 2 | Glassmorphism cards | `backdrop-blur`, `bg-white/10`, frosted panels | Solid surfaces with intentional depth (shadow tiers, borders that mean something) |
| 3 | Rounded everything | `border-radius: 24px+` on every element | Vary radius by role; let some elements have weight and edges |
| 4 | Default "modern" sans/serif/mono rotation | Sans: Inter, DM Sans, Space Grotesk, Sora, Syne, Archivo, Figtree (cycling to the next of these is still slop, not an escape). Serif: Fraunces, Cormorant, Young Serif. Mono: JetBrains Mono used as a display/house voice. Any rounded-novelty display face (the "friendly SaaS blob" letterforms) | Distinctive pairing, never reused across briefs — see `references/themes/*`, the escalation path below, or ui-alternatives |
| 5 | Emoji-as-icons in headers | "✨ Supercharge 🚀" | Real icon set (Phosphor) or no icon; reserve emoji for genuinely casual contexts |
| 6 | Hero headline formula | "[Verb] your [noun] with [product]" | A specific claim or a real point of view |
| 7 | Three-column feature grid | icon + bold label + 1 sentence ×3 | Let content shape layout; asymmetry, editorial blocks, varied density |
| 8 | Testimonial carousel w/ headshots | circular avatar + name + company + 1 line | Specific outcomes, real quotes with substance, or cut it |
| 9 | "Get started for free" CTA | large primary button, generic label | A label that says what happens ("Download the report", "Start the 14-day trial") |
| 10 | Illustrated empty state + button | Lottie blob person + "Add your first…" | One specific next action stating what they'll get (see `state-handling`) |
| 11 | Skeleton loaders everywhere | gray pulse bars on everything | Fix perceived perf; skeletons only where layout is genuinely known ahead |
| 12 | Dark mode = black + purple | `#0f0f0f` + `#8b5cf6` | A real dark palette with a considered accent, not the Vercel-clone default |
| 13 | Animated gradient text | moving rainbow/sweep on the headline | Static type with real contrast and hierarchy |
| 14 | "Powered by AI" badge | small trust badge | Say what it does for the user; the badge signals nothing |
| 15 | 6+ metric dashboard | big number + small label ×6 | Show the 1–2 numbers that are actually actionable |
| 16 | Centered single-column everything | max-w-2xl mx-auto, every section | Break the grid; use the full canvas with intent |
| 17 | Hover boop / translateY lift on every interactive element | `transform: translateY(-2px)` + shadow bump on hover, applied uniformly | Reserve motion feedback for elements that need emphasis; vary or omit it per element role |
| 18 | Underline-fill link animation | link underline sweeps in left-to-right on hover | A static underline or color shift — motion on every link reads as decoration, not feedback |
| 19 | Sun-moon theme toggle | circular icon flip/rotate animating the light/dark switch | A plain switch or a static icon swap, no animated celestial gimmick |
| 20 | Inner-glow badge | `box-shadow: inset 0 0 Npx` glow ring on pill/badge components | Flat badge with real color contrast; save glow for a genuine focus/active state |
| 21 | Pulsing live/status dot | `animate-pulse` green dot next to "Live"/"Online" labels | A static dot doing the color work, or real live-updating content instead of a pulse loop |
| 22 | Eyebrow tick rule | small colored line/tick decorating an eyebrow label ("— FEATURES") | Let the eyebrow label's own type and spacing carry weight; skip the decorative tick |
| 23 | Hairline light border on every card | `border: 1px solid rgba(0,0,0,.1)` on every surface, no exceptions | Self-colored borders + tonal elevation that means something — not a border by default |
| 24 | Accent-bar card | thin colored bar along one card edge as the only differentiation | Real content hierarchy or a considered surface treatment, not a bar-as-decoration |
| 25 | Background glow blob | large blurred radial-gradient blob behind hero/card content | Intentional atmosphere tied to the brand, not a generic purple/blue blur for "depth" |
| 26 | Bloom-as-blurred-silhouette | a blurred copy of the same shape behind itself, standing in for a glow | A real light-source/shadow relationship, or skip the bloom entirely |
| 27 | Faked shadow via offset box | a solid-color box offset behind an element, standing in for a shadow | A real `box-shadow`, tight and color-matched to the surface |
| 28 | Default hero stack | eyebrow + centered H1 + subhead + 2 buttons, stacked and centered | Break the stack — asymmetry, a real image/artifact, or an editorial layout |
| 29 | Hero + right panel | headline/CTA left, product screenshot/mockup right, every landing page | Let the content decide the split; consider full-bleed or a non-split hero |
| 30 | Pre-footer CTA slab | full-width centered "Ready to get started?" band right before the footer | Integrate the ask into the page's actual narrative, not a bolted-on slab |
| 31 | Fill-plus-outline button pair | primary filled + secondary outline button, same shape, side by side | Differentiate by more than fill — weight, size, or drop the pair entirely |
| 32 | Full SaaS template | hero → logo cloud → 3-col features → testimonials → pricing → FAQ → CTA slab, in that order | Let the actual product/audience decide section order and presence; cut what doesn't earn its place |
| 33 | Saturated accent color everywhere | one bright accent hue at full saturation across buttons/links/icons/badges | A tonal system — vary saturation/lightness by role instead of one loud hue everywhere |
| 34 | Hard seams between sections | abrupt background-color cut at every section boundary | Continuous page color/gradient across sections, or an intentional, designed seam |
| 35 | Alert-card cosplay | ordinary status, metric, or summary content put in a rounded card with a colored left border, borrowing alert semantics as decoration | Use a card only when containment, elevation, or the whole-card interaction is meaningful. Otherwise use headings, rows, dividers, or whitespace. Reserve a side stripe for a real alert/status distinction. |

Rows 17–34 adapted from pols.dev's slop catalog (Interaction Tells, Cards & Boxes, Slop Layouts
sections; no verbatim reuse, no SPDX on source). Compositor-only-animation and z-index-scale
rules are covered under Mechanics Deslop below — not restated here.

Row 35 consolidates a strong active signal independently documented by `nexu-io/open-design`,
`febbhav/signs-of-ai-design`, `educlopez/ui-craft`, `Leonxlnx/taste-skill`, and
`garrytan/gstack`: the colored side stripe is semantic on a real alert, but becomes a high-signal
AI dashboard tell when sprayed across ordinary cards. The rule targets the reflex, not all cards or
all left borders.

### What to do instead (the discipline)

- **Typography first.** Pick fonts specific to the context; research type history. Pair a
  high-contrast/characterful display face with a quieter body face.
- **Commit to one extreme.** Brutally minimal OR maximally dense. The middle is where slop lives.
- **Space is a design element.** Generous negative space with one dense anchor beats uniform padding.
- **Reference real design movements.** Swiss grid, Bauhaus, Emigre, brutalist web, Dutch
  constructivism, Tschichold. Pick one and execute with intent.

### Product-specificity checks

These checks catch generic work that can evade a pattern-by-pattern visual audit:

| Check | Failure signal | Required correction |
|---|---|---|
| Logo-swap test | Replacing the logo, name, and accent color with a competitor leaves an equally plausible interface | Add a structural decision derived from the product's actual job, audience, data, or interaction model |
| Reference-transfer ledger | A reference is cited without `principle transferred → product-specific transformation`, or the transformation is only a style adjective | Transfer the principle, document the transformation, and reject recognizable composition copying |
| Repeated surface grammar | Unrelated content types all use the same card shell, radius, spacing, icon-title-copy stack, or hover treatment | Repeat a surface grammar only for a repeated semantic role; give distinct roles distinct structures |

Do not treat palette/font/style lookup results as proof of specificity. In particular,
`ui-ux-pro-max` is a mechanics, accessibility, and design-system retrieval corpus; its product labels
are search facets, not a product-specificity detector.

---

## Mechanics Deslop

> Adapted from `ibelick/ui-skills` `baseline-ui` skill (MIT license). Rows 15-19 (and the
> `text-balance` caveat on row 4) adapted from
> `github.com/jakubkrehel/make-interfaces-feel-better` `surfaces.md` + `typography.md` (MIT
> license). These are implementation-mechanics rules — CSS/utility/component-level correctness —
> distinct from the visual slop above (Canon of Slop) and the non-state interaction slop above
> (UX Interaction Anti-Patterns).

| # | Rule | Why |
|---|---|---|
| 1 | MUST use `h-dvh` over `h-screen` for full-viewport layouts | `100vh` ignores mobile browser chrome (address bar show/hide on scroll); `dvh` (dynamic viewport height) tracks it — `h-screen` clips content or leaves a gap on mobile Safari/Chrome |
| 2 | MUST pad `env(safe-area-inset-*)` on mobile-facing fixed elements | Notches, home indicators, and rounded corners eat into fixed headers/footers/bottom sheets without it — e.g. `padding-bottom: env(safe-area-inset-bottom)` |
| 3 | NEVER block paste in text inputs | `onPaste` preventDefault (or equivalent) breaks password managers and pasted 2FA codes — a UX regression wearing a "security" costume |
| 4 | SHOULD use `text-balance` on headlines | Evens out line lengths on short, large text; prevents an orphaned single word wrapping alone. Caveat: the engine has hard line limits — `text-wrap: balance` has no effect past 6 lines in Chromium or past 10 lines in Firefox, so it silently stops helping on longer headlines |
| 5 | SHOULD use `text-pretty` on body copy | Avoids orphans/widows in paragraph text without `balance`'s cost on longer runs |
| 6 | MUST use `tabular-nums` on any UI showing changing numeric values | Proportional digit widths jitter the layout as a counter/price/stat updates; `font-variant-numeric: tabular-nums` fixes digit width |
| 7 | MUST define a fixed, documented z-index scale | Declare tiers (dropdown/modal/toast/tooltip) once and reference them |
| 8 | NEVER scatter ad-hoc arbitrary z-index values (`z-[9999]`) per component | Untracked stacking values are how stacking-context bugs accumulate — every new value should map to the scale in #7, not invent a new layer |
| 9 | SHOULD use `size-*` over separate `w-*`/`h-*` pairs when width equals height | One utility, one source of truth — no drift when one dimension gets edited and not the other |
| 10 | MUST scope `will-change` to elements actively animating | It forces its own compositor layer — a standing memory/perf cost, only worth paying while the animation is live |
| 11 | NEVER leave `will-change` on as a blanket "just in case" perf hint | Same cost as #10 with no animation to justify it — remove it when the animation ends |
| 12 | NEVER animate `width`/`height`/`top`/`left`/`margin` | These trigger layout on every frame; animate `transform`/`opacity` only — the two properties the compositor can run off the main thread |
| 13 | MUST use `AlertDialog` (not a plain `Dialog`/`Modal`) for destructive-action confirmation | A generic dismissible modal reads as equally weighted to any other dialog; `AlertDialog` (shadcn/Radix) is modal-only with no outside-click dismiss, forcing an explicit choice on actions that can't be undone |
| 14 | MUST keep interaction feedback (hover/press/focus) under a 200ms response ceiling | Beyond ~200ms a UI reads as sluggish regardless of how fast the underlying action actually completes — feedback latency is perceived independently of task latency |
| 15 | MUST size concentric border radius as `outer = inner + padding` | Keeps nested layers' corners visually concentric; once the padding between layers exceeds 24px, size each layer's radius independently instead of just copying the outer radius inward — the naive copy-inward reads as mismatched at that gap |
| 16 | SHOULD set icon-side padding to text-side padding minus 2px for optical alignment | Compensates for the eye's tendency to read icon visual weight as heavier than text — e.g. a play-triangle icon needs `margin-left: 2px` to look centered; fix asymmetric source SVGs (re-export the icon) before reaching for margin/padding hacks |
| 17 | MUST treat shadow-as-border as an elevated-surface technique only (never dividers/layout borders) | Light mode: a 3-layer stack — 1px ring + a "lift" shadow + an ambient shadow. Dark mode: a single `rgba(255,255,255,0.08)` ring instead. Applying this to dividers or layout borders misuses an elevation cue as a structural line |
| 18 | MUST use a 1px pure black or pure white outline at 10% opacity with `outline-offset: -1px` for image outlines | Tinted neutrals (slate/zinc/near-black grays) are BANNED for this purpose — they read as dirt/smudging on the image edge rather than a clean border |
| 19 | MUST set `-webkit-font-smoothing: antialiased` at the root/global level only | Never apply per-element — it is a document-wide rendering hint, not a per-component style knob |

> Rows 4/5/6/19 above are the full extent of this file's typography-mechanics coverage — full
> typography craft depth (properties-over-raw-tags, font-synthesis, measure, line-height,
> letter-spacing, truncation, punctuation, underlines, iOS inputs, RTL, selection): see
> `typography-craft.md`.

### Font alternatives by context

(From the source `ui-alternatives.md` — free Google Fonts unless noted.)

- **Tech / tool:** Geist, Söhne (paid), GT Alpina (paid), Suisse Int'l (paid) · free: Hanken Grotesk, Sora, Archivo
- **Editorial / media:** Canela (paid), Tiempos (paid) · free: Fraunces, Playfair Display, Spectral, Lora
- **Character / personality:** Signifier (paid), Obviously (paid) · free: Bricolage Grotesque, Instrument Serif, Syne, Unbounded
- **Rule:** the display face carries personality; let the body face be quieter.
- **Hard rule 1:** stop cycling Google Fonts. Rotating from Inter to Sora to Space Grotesk is
  still the same slop shelf — the fix is a genuinely different source, not the next free default.
- **Hard rule 2:** never reuse a font pairing across briefs. A pairing that reads as distinctive
  once becomes a house style, then a tell, the second and third time it ships.
- **Escalation (when even Fontshare's usual picks — Clash Display, General Sans — read
  generic):** reach for less-cycled Fontshare faces (Pally, Gambarino, Sentient, Tanker) or
  Velvetyne's catalogue; self-host via `next/font/local` rather than a CDN `<link>` (control,
  perf, no third-party request). `system-ui` is the one sanctioned neutral fallback for body
  copy when no distinctive pairing is warranted — it is honest about being unstyled, not another
  disguised default. (Source: adapted from pols.dev's "Slop Fonts" law, no verbatim reuse.)

---

## UX Interaction Anti-Patterns (Non-State)

> Loading / empty / error STATE patterns live in the `state-handling` skill. This section is
> the NON-state interaction slop: modals, navigation, confirmations, feedback.

| Pattern | Signature | Do instead |
|---|---|---|
| Onboarding modal | "Welcome to [Product]!" before any context | Drop users into a real first task; teach in context |
| 5-step wizard | progress bar + confetti at the end | Minimum viable path; add steps only when needed |
| Tooltip tour | floating boxes pointing at the UI | If it needs a tour, the UI is unclear — redesign it |
| "Are you sure?" on every delete | confirm modal as default | Optimistic delete + 5–10s undo window |
| Toast spam | "Saved!" / "Updated!" / "Deleted!" | Inline, contextual feedback near the action |
| Infinite scroll + "Load more" | both at once | Pick one deliberately |
| Hamburger on desktop | hidden nav where there's room | Persistent nav; reserve the drawer for mobile |
| Hover-only affordances | actions revealed only on hover | Visible or focusable affordances (mobile + keyboard) |
| Action = full reload | page refresh, scroll position lost | Optimistic UI; reconcile in the background |

### Fixes (the affirmative version)

- **Undo over confirm** — far less friction than a modal on every destructive action.
- **Optimistic UI** — show the outcome immediately, reconcile in the background; feels instant.
- **Progressive disclosure** — start minimal, reveal complexity only when the user reaches for it.
- **Be opinionated** — show the best path; don't present six equal options when one is clearly right.

---

## Positive Doctrine

Anti-slop is not just negation. The affirmative principles (condensed; the SKILL.md Design
Thinking section is the entry point):

- **Specificity > universality.** Design for this user, this task, this moment.
- **Tension is interest.** Contrast, asymmetry, deliberate friction are memorable; harmony can be invisible.
- **Constraints create identity.** Impose a real restriction and design within it. The best brands have rules.
- **Reference the real world.** Materials, textures, physical artifacts, historical design — not just other apps.

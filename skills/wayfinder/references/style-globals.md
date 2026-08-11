
# Style Globals — Geist house-style reference

The canonical reference page is `docs/diagrams/roadmap-pulse.html` (designated as the portable baseline,
2026-07-05). Every value below is **transcribed verbatim** from that file — never re-derived,
never eyeballed. When authoring a new page that resolves to the Geist house style (step 1's
default, "otherwise -> Vercel Geist"), load this file alongside `frontend-design`'s
`references/geist-design.md` (see SKILL.md step 2) instead of inventing a fresh token set.

**Do not copy an archived or previously-generated diagram as a template.** Read this file each
time. See § Do / Don't below and the `feedback_visual_explainer_for_diagrams` memory record.

## 1. Fonts

| Font | Weights loaded | Role | CSS variable |
|---|---|---|---|
| Geist | 400, 500, 600, 700 | Primary sans — body, headings, prose | `--font-sans` |
| Geist Mono | 400, 500, 600 | Technical labels — meta lines, badges, table headers, code, commands | `--font-mono` |

Google Fonts `<link>` (place in `<head>`, with the two `preconnect` hints first):

```html
<link rel="preconnect" href="https://fonts.googleapis.com">
<link rel="preconnect" href="https://fonts.gstatic.com" crossorigin>
<link href="https://fonts.googleapis.com/css2?family=Geist:wght@400;500;600;700&family=Geist+Mono:wght@400;500;600&display=swap" rel="stylesheet">
```

Fallback stacks (used verbatim in the CSS custom properties):

```css
--font-sans: 'Geist', system-ui, -apple-system, sans-serif;
--font-mono: 'Geist Mono', 'SF Mono', Consolas, monospace;
```

## 2. Color roles (light + dark)

All colors are CSS custom properties on `:root`, overridden inside
`@media (prefers-color-scheme: dark)`. Both themes are **selected**, not an automatic filter —
see § 4 for the `.animate` / reduced-motion convention that ships alongside them.

| Token | Role | Light | Dark |
|---|---|---|---|
| `--ds-background-100` | Page background | `#fafafa` | `#0a0a0a` |
| `--ds-surface` | Card / table / note surface | `#ffffff` | `#111111` |
| `--ds-gray-100` | Recessed fill (code bg, table header bg, zebra stripe base) | `#f5f5f5` | `#1a1a1a` |
| `--ds-gray-400` | Hairline border | `rgba(0,0,0,0.08)` | `rgba(255,255,255,0.09)` |
| `--ds-gray-500` | Slightly stronger border / bullet dot | `rgba(0,0,0,0.16)` | `rgba(255,255,255,0.18)` |
| `--ds-gray-900` | Secondary ink (meta text, subtitles, dim labels) | `#666666` | `#a1a1a1` |
| `--ds-gray-1000` | Primary ink (headings, body text) | `#171717` | `#ededed` |

Five accent pairs, each with a `-dim` background tint used for card washes, badge fills, and
`.h-tag` chips:

| Accent | Light solid | Light dim | Dark solid | Dark dim |
|---|---|---|---|---|
| `--ds-blue-700` / `--ds-blue-dim` | `#0070f3` | `rgba(0,112,243,0.07)` | `#52a8ff` | `rgba(82,168,255,0.10)` |
| `--ds-green-700` / `--ds-green-dim` | `#0f7d3c` | `rgba(15,125,60,0.08)` | `#3fca7a` | `rgba(63,202,122,0.10)` |
| `--ds-red-800` / `--ds-red-dim` | `#e5484d` | `rgba(229,72,77,0.07)` | `#ff6369` | `rgba(255,99,105,0.10)` |
| `--ds-amber-700` / `--ds-amber-dim` | `#a35200` | `rgba(245,166,35,0.12)` | `#f7b955` | `rgba(247,185,85,0.10)` |
| `--ds-teal-700` / `--ds-teal-dim` | `#0e9384` | `rgba(14,147,132,0.08)` | `#3ddbc9` | `rgba(61,219,201,0.10)` |

Shadow tokens (two tiers only — do not invent a third):

| Token | Light | Dark |
|---|---|---|
| `--shadow-medium` | `0 4px 12px rgba(0,0,0,0.06), 0 1px 3px rgba(0,0,0,0.05)` | `0 4px 12px rgba(0,0,0,0.5)` |
| `--shadow-large` | `0 8px 30px rgba(0,0,0,0.10), 0 2px 6px rgba(0,0,0,0.06)` | `0 8px 30px rgba(0,0,0,0.6)` |

## 3. Copy-ready CSS block

Transcribed verbatim from `docs/diagrams/roadmap-pulse.html` lines 11–46. Paste this as the
opening of the page's `<style>` block, then build on top of it — never redefine these variable
names with different values.

```css
:root {
  --font-sans: 'Geist', system-ui, -apple-system, sans-serif;
  --font-mono: 'Geist Mono', 'SF Mono', Consolas, monospace;
  --ds-background-100: #fafafa;
  --ds-surface: #ffffff;
  --ds-gray-100: #f5f5f5;
  --ds-gray-400: rgba(0,0,0,0.08);
  --ds-gray-500: rgba(0,0,0,0.16);
  --ds-gray-900: #666666;
  --ds-gray-1000: #171717;
  --ds-blue-700: #0070f3;  --ds-blue-dim: rgba(0,112,243,0.07);
  --ds-green-700: #0f7d3c; --ds-green-dim: rgba(15,125,60,0.08);
  --ds-red-800: #e5484d;   --ds-red-dim: rgba(229,72,77,0.07);
  --ds-amber-700: #a35200; --ds-amber-dim: rgba(245,166,35,0.12);
  --ds-teal-700: #0e9384;  --ds-teal-dim: rgba(14,147,132,0.08);
  --shadow-medium: 0 4px 12px rgba(0,0,0,0.06), 0 1px 3px rgba(0,0,0,0.05);
  --shadow-large: 0 8px 30px rgba(0,0,0,0.10), 0 2px 6px rgba(0,0,0,0.06);
}
@media (prefers-color-scheme: dark) {
  :root {
    --ds-background-100: #0a0a0a;
    --ds-surface: #111111;
    --ds-gray-100: #1a1a1a;
    --ds-gray-400: rgba(255,255,255,0.09);
    --ds-gray-500: rgba(255,255,255,0.18);
    --ds-gray-900: #a1a1a1;
    --ds-gray-1000: #ededed;
    --ds-blue-700: #52a8ff;  --ds-blue-dim: rgba(82,168,255,0.10);
    --ds-green-700: #3fca7a; --ds-green-dim: rgba(63,202,122,0.10);
    --ds-red-800: #ff6369;   --ds-red-dim: rgba(255,99,105,0.10);
    --ds-amber-700: #f7b955; --ds-amber-dim: rgba(247,185,85,0.10);
    --ds-teal-700: #3ddbc9;  --ds-teal-dim: rgba(61,219,201,0.10);
    --shadow-medium: 0 4px 12px rgba(0,0,0,0.5);
    --shadow-large: 0 8px 30px rgba(0,0,0,0.6);
  }
}
```

## 4. Layout-shell skeleton

```
.container                         (max-width:1100px, centered, 44px/32px/96px padding)
└─ header                          (.animate, --i:0)
   ├─ .meta-line                   (mono, dotted bullets, wraps)
   ├─ h1                           (28px/700, optional <span class="dim"> suffix)
   └─ .subtitle                    (secondary ink, max-width 800px)
└─ <section id="...">              (one per topic, .animate on inner wrap)
   ├─ h2                           (18px/650) with <span class="h-tag"> prefix chip
   ├─ .section-sub                 (secondary ink, max-width 840px)
   └─ component (see below)
```

### Component treatments

- **`.next-card` / `.next-grid`** — `auto-fit, minmax(300px,1fr)` grid of cards. Base card:
  `--ds-surface` bg, `--ds-gray-400` border, 12px radius, `--shadow-large`. Three variants layer a
  top-to-transparent gradient wash + tinted border on top of the base:
  - `.next-card.primary` — green wash (positive / recommended action)
  - `.next-card.warn` — amber wash (needs attention, not urgent)
  - `.next-card.danger` — red wash (blocking / urgent)

  Each variant also recolors `.next-cmd` (the mono command line) to match: `--ds-green-700`,
  `--ds-amber-700`, `--ds-red-800` respectively.

- **`.table-wrap` / `.table-scroll`** — bordered, rounded (12px), `--shadow-medium` shell around a
  horizontally-scrollable `<table>`. Sticky-feel header row (`--ds-gray-100` bg, mono uppercase
  11px labels), zebra striping via `tr:nth-child(even)` at a 55%-mixed `--ds-gray-100` tint, row
  hover via `--ds-blue-dim`. Use `<colgroup>`/`th width` hints for column balance — never force
  single-line truncation.

- **Six-variant `.badge` family** — pill shape (`border-radius:999px`), mono 11px/600, each variant
  pairs a solid accent text color with its `-dim` background and a 30–35%-mixed accent border:
  `.b-p0` (red — P0/critical), `.b-p1` (amber — P1), `.b-p2` (blue — P2), `.b-flight` (teal —
  in-flight/active), `.b-draft` (gray — draft/inactive), `.b-stale` (red — stale/needs attention).
  Do not add a seventh ad-hoc variant — reuse the closest existing role or fold into `.b-p2`/
  `.b-draft` as a neutral fallback.

- **`.prog` bar** — 74px×5px pill track (`--ds-gray-100` bg, `--ds-gray-400` border), inner `<i>`
  fill at `--ds-teal-700`, width set inline per row (`style="width:NN%"`).

- **`.note` callout** — bordered card with a 3px `--ds-blue-700` left accent border, `--ds-surface`
  bg, secondary-ink body text, `<strong>` runs in primary ink.

## 5. Animation convention

- `fadeUp` keyframe (`opacity 0->1`, `translateY(10px)->0`) applied via `.animate`, staggered by a
  per-element `--i` custom property: `animation-delay: calc(var(--i,0)*0.05s)`.
- `@media (prefers-reduced-motion: reduce)` collapses all animation/transition durations to
  `0.01ms` — this guard is mandatory on every page that uses `.animate`.
- **The reference page ships no `<script>` tag.** Both the `fadeUp` stagger and the dark-mode
  color swap are pure CSS (`@keyframes` + `@media (prefers-color-scheme: dark)`) — no JS is
  required to reproduce either effect. Only add a `<script>` when the page embeds Mermaid,
  Chart.js, or anime.js (see SKILL.md § Diagram Types) — never for the base layout/animation/theme
  behavior documented here.

## 6. Do / Don't

**Do:**
- Reuse the `--ds-*` token names and the component classes above verbatim — a new page should
  read as the same system as every other Geist-default page.
- Extend by composition (a new badge variant reuses an existing accent pair; a new card type
  reuses `.next-card`'s shadow/radius/border pattern) rather than inventing new primitives.
- Keep both the light block and the dark `@media` override in sync — every token defined in one
  must be redefined in the other.

**Don't:**
- Don't hand-roll a new shadow value — there are exactly two tiers (`--shadow-medium`,
  `--shadow-large`); a third implies the depth hierarchy has grown and belongs in this file first.
- Don't invent a new gray step or accent hue for a "just this once" treatment — five accents and
  seven grayscale/surface roles cover every documented component.
- Don't skip the `prefers-reduced-motion` guard when adding `.animate` to new elements.
- Don't copy an archived or previously-generated diagram as a "template" — always read this file
  fresh (see `feedback_visual_explainer_for_diagrams` memory record: an agent once copy-pasted an
  archived page and froze that page's incidental aesthetic instead of the current pattern set).

## 7. Chart color (dataviz wiring)

When a page embeds a real chart (Chart.js bar/line/pie, or an inline SVG chart), draw its colors
from the `dataviz` skill's validated palette (`references/palette.md`) — never invent a second,
uncoordinated palette next to the `--ds-*` tokens. This page's system only has **5** accent
pairs, so `dataviz`'s roles are mapped onto the *existing* `--ds-*` tokens below — no new hex
values are introduced.

### Categorical (chart series identity)

`dataviz` assigns categorical hues in a fixed order, never cycled, and folds a 9th+ series into
"Other" rather than generating a new hue. This page's fixed order (5 slots, same rule — a 6th
series folds into "Other" / small multiples):

| Series slot | `--ds-*` token |
|---|---|
| 1 | `--ds-blue-700` |
| 2 | `--ds-teal-700` |
| 3 | `--ds-green-700` |
| 4 | `--ds-amber-700` |
| 5 | `--ds-red-800` |

### Status (state, never reused as a series color)

`dataviz`'s status palette is four roles (good / warning / serious / critical), fixed and never
themed. This page's token set only has one red-family accent, so `serious` and `critical` share
`--ds-red-800` — a deliberate reduction (not an oversight): both already read as "needs attention
at the highest severity" in this system's existing `.b-p0` / `.b-stale` badge usage, and splitting
them would require inventing a sixth accent hue, which § Do/Don't forbids.

| `dataviz` role | `--ds-*` token | Existing badge precedent |
|---|---|---|
| good | `--ds-green-700` | `.next-card.primary` |
| warning | `--ds-amber-700` | `.b-p1` |
| serious | `--ds-red-800` | `.b-p0` / `.b-stale` |
| critical | `--ds-red-800` | `.b-p0` / `.b-stale` |

As with every status color, ship it with an icon or label — never color alone (per `dataviz`'s
own non-negotiable rule, which still applies unchanged inside this token mapping).

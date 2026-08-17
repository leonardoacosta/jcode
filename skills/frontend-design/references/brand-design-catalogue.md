
# Brand DESIGN.md Catalogue

> Reference for the `frontend-design` skill — matching a specific brand's aesthetic ("make it
> look like Linear", "Stripe-style checkout") using curated DESIGN.md reference files.
>
> Consolidated from the `awesome-design-md` skill (score-and-remediate-skill-quality-floor,
> 2.5). That skill now routes here for its deep content — this file is the canonical home.

A curated catalogue of **66 DESIGN.md files** — plain-text design system documents extracted
from real production websites. Each file describes a brand's colors, typography, spacing,
motion language, and UI conventions in a format AI coding agents can read and replicate
without Figma exports, JSON schemas, or special tooling.

Repo: https://github.com/VoltAgent/awesome-design-md

## The core idea

Instead of guessing what "looks like Linear" means, copy `linear/DESIGN.md` into the project
and reference it directly:

> "Build a dashboard page matching the design system in `references/linear.DESIGN.md`."

The agent reads the file, applies the documented tokens and conventions, and produces UI that
actually resembles the target — not a generic approximation.

## When to reach for this

Use when the request names **a specific brand as an aesthetic target** — "Make this landing
page feel like Vercel", "I want a Stripe-style checkout flow", "Clone the Claude homepage
vibe", "Use the Linear aesthetic for our admin UI". Do **not** use this for generic "make it
look modern/minimal/professional" requests — those need the rest of `frontend-design` or its
`references/ui-ux-pro-max/`, not a specific brand match.

## Fetch commands

```bash
# curl
curl -o docs/design-references/linear.md \
  https://raw.githubusercontent.com/VoltAgent/awesome-design-md/main/linear/DESIGN.md

# gh api
gh api repos/VoltAgent/awesome-design-md/contents/linear/DESIGN.md \
  --jq '.content' | base64 -d > docs/design-references/linear.md
```

Drop the fetched file into the project under `docs/design-references/<brand>.md` (or wherever
the project keeps reference material) before starting work.

## Workflow

1. Identify the target brand from the brief. If ambiguous, ask.
2. Fetch the DESIGN.md (paths in the catalogue below).
3. Drop it into the project.
4. Read the file before starting work — note the palette, type scale, spacing rhythm, border
   radius, motion language, and brand-specific conventions.
5. Build the UI with those tokens. When uncertain about a detail, reread the relevant section
   rather than improvising.
6. Open `preview.html` (each brand also ships a preview catalog) as a sanity check.

## Decision framework — design problem → which brands to consult

Before opening the full catalogue, name the *problem* the UI needs to solve — that narrows 66
brands down to the 3-5 worth actually reading. The category groups below aren't just a lookup;
each one is a claim about which control that brand family optimizes for, which is what makes it
the right (or wrong) reference for a given problem:

| The UI problem is... | Consult | Why these, specifically |
|---|---|---|
| Dense data / analytics dashboard — tables, filters, status at a glance | **Linear, Stripe** | Both optimize for scanning speed under high information density: tabular numerals, restrained color used only for status, minimal chrome around the data itself |
| Editorial / content-first page — a page that has to read well, not just look clean | **Vercel, Notion** | Typography carries the hierarchy instead of color blocks or card borders; generous whitespace is the actual design decision, not a placeholder for "more content later" |
| Developer tool / CLI-adjacent product | **Raycast, Warp, Cursor** | Monospace accents and keyboard-shortcut-first chrome aren't decoration — they signal "built for people who live in a terminal," which is the trust signal this category needs |
| Fintech / anything handling money or identity | **Stripe, Wise, Coinbase** | Precise spacing grids and restrained motion read as *care* — a fintech UI that feels playful or loose undermines the trust the product depends on |
| AI/LLM product surface | **Anthropic, OpenAI, Perplexity** | Dark-first palettes + diagram-style illustration are how this category signals "frontier tech" without resorting to literal circuit-board clichés |
| Consumer / media-heavy landing page | **Apple, Spotify** | Full-bleed photography and typography carry the emotional weight; UI chrome all but disappears so the content is the interface |
| E-commerce / retail | **Shopify, Amazon** | Product-grid density and trust badges (reviews, shipping, returns) are the actual design problem — not aesthetic polish |

**NEVER copy a brand's logo, tagline, or exact marketing copy** — that's trademark
infringement, not aesthetic reference. The DESIGN.md files below exist for colors, type,
spacing, and motion tokens; treat that boundary as the one non-negotiable rule in this skill,
independent of which brand the table above points you at.

## Catalogue (66 brands, 13 categories)

Paths are relative to the repo root. All files are named `DESIGN.md` inside a brand-named
directory.

### AI & LLM platforms (11)

`anthropic/` · `openai/` · `cohere/` · `elevenlabs/` · `mistral/` · `ollama/` · `replicate/` ·
`runwayml/` · `together/` · `xai/` · `perplexity/`

Common traits: dark-first palettes, generous whitespace, serif display + sans body, heavy use
of gradient accents and diagram-style illustrations.

### Developer tools & IDEs (7)

`cursor/` · `expo/` · `lovable/` · `raycast/` · `superhuman/` · `vercel/` · `warp/`

Common traits: high contrast, monospace accents, keyboard-shortcut-first chrome, terminal
metaphors, Geist/Inter-family typefaces.

### Backend, database & devops (8)

`clickhouse/` · `mongodb/` · `posthog/` · `sanity/` · `sentry/` · `supabase/` · `neon/` ·
`railway/`

Common traits: product-dashboard conventions, data-viz density, status indicators, brand
color blocks over dark gray neutrals.

### Productivity & SaaS (7)

`cal-com/` · `linear/` · `notion/` · `zapier/` · `airtable/` · `asana/` · `intercom/`

Common traits: grayscale + one bold accent, subtle gradients, generous empty states, focus on
writing quality in UI copy.

### Design & creative tools (6)

`figma/` · `framer/` · `webflow/` · `dribbble/` · `behance/` · `canva/`

Common traits: canvas-style chrome, tool palettes, brand-color explosions, card-dense
galleries.

### Fintech & crypto (6)

`stripe/` · `wise/` · `binance/` · `coinbase/` · `kraken/` · `robinhood/`

Common traits: trust signals (logos, data density), precise spacing grids, restrained motion,
heavy use of tabular numerals.

### E-commerce & retail (4)

`shopify/` · `amazon/` · `etsy/` · `ebay/`

### Media & consumer tech (10)

`apple/` · `spotify/` · `netflix/` · `youtube/` · `twitch/` · `pinterest/` · `reddit/` ·
`medium/` · `substack/` · `ibm/` · `nvidia/`

### Automotive (6)

`tesla/` · `rivian/` · `lucid/` · `porsche/` · `bmw/` · `mercedes/`

Common traits: big hero photography, minimal chrome, full-bleed typography, premium pacing.

Category counts and brand coverage may shift as the repo is updated — check the repo README
for the current list.

## How to read a DESIGN.md — section scan

Most DESIGN.md files follow a similar structure. Scan for these sections:

- **Brand philosophy** — the "why" behind the aesthetic. Useful context, not enforceable.
- **Color palette** — HEX/RGB tokens with semantic names (background, surface, text, border,
  accent). Match these exactly.
- **Typography** — font families, weights, scale (often modular with a ratio). Some brands
  specify a display font + body font + mono font separately.
- **Spacing & layout** — grid rhythm, container widths, section padding.
- **Motion** — duration scale (fast/base/slow), easing curves, entrance animations.
- **Component conventions** — how the brand handles buttons, cards, inputs, navigation.
- **Voice & copy** — tone of UI strings. Critical for brand-matching beyond visuals.

If a section is thin or missing, fall back to conservative defaults — don't invent brand
conventions that aren't documented.

## Gotchas

- **One-brand-per-page** — pick one brand per page. Referencing "Linear's color + Stripe's
  type + Apple's spacing" produces incoherent UI. If the user wants a blend, that's a general
  `frontend-design` task, not an `awesome-design-md` task.
- **Snapshot drift** — DESIGN.md is a snapshot; brands redesign. A file from six months ago
  may not match the current live site. Cross-check against the live URL before committing to
  tokens that feel dated.
- **Legal boundary** — copying a brand's *aesthetic* for internal/client work is normal.
  Copying their *logo*, *tagline*, or *exact marketing copy* is trademark infringement. Use
  DESIGN.md for colors, type, spacing, motion — not for a wholesale brand clone.
- **Not all brands are in the catalogue** — if the target isn't listed, build from first
  principles or have the user supply a DESIGN.md-style brief manually.
- **Preview files are the sanity check** — each entry ships `preview.html` and
  `preview-dark.html`. Open them before building to confirm the interpretation matches the
  brand's actual look.

## Related skills

- **`references/aceternity/`** — when the brand uses motion-heavy components, pick building
  blocks whose style matches the DESIGN.md tokens.
- **`shadcn`** — for utilitarian chrome; shadcn primitives retokenized against the brand
  palette is the fastest path to a convincing match.
- **`references/geist-design.md`** — specifically when the target is Vercel (Geist is Vercel's
  in-house design system — prefer it over the generic DESIGN.md for Vercel work).

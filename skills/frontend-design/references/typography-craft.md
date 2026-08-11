
# Typography Craft

> Distilled from `github.com/jakubkrehel/skills` `better-typography/*` (MIT license) for the
> `frontend-design` skill. Covers ONLY the mechanics principles the corpus did not already own as of the
> 2026-07-16 recon (`docs/recon/jakubkrehel-skills.md` Adapt 2) — font *selection* doctrine
> stays in `SKILL.md`; `anti-slop-canon.md`'s four Mechanics Deslop rows below are cited, not
> restated.

## Properties Over Raw Tags

Reach for the dedicated CSS property before the low-level OpenType/variable-font tag. Properties
degrade correctly on non-variable-font fallbacks; raw tags silently no-op there. Reserve raw tags
for what has no property — a custom variable-font axis (`"GRAD" 80`) or a niche OpenType feature
(`"ss01" 1`).

| Use case | Correct | Avoid |
|---|---|---|
| Weight | `font-weight: 650;` — Tailwind: `font-[650]` (arbitrary; 650 sits outside the default 100–900 step scale) | `font-variation-settings: "wght" 650;` |
| Small caps | `font-variant-caps: small-caps;` — Tailwind: arbitrary `[font-variant-caps:small-caps]` | `font-feature-settings: "smcp" 1;` |

`font-variant-numeric: tabular-nums` (the numeric-alignment case of this same properties-over-tags
rule), `text-wrap: balance`/`text-wrap: pretty`, and root-only `-webkit-font-smoothing` are already
covered in full — see `anti-slop-canon.md` Mechanics Deslop rows 4/5/6/19.

**Font loading**: `font-synthesis: none;` (no Tailwind utility — arbitrary `[font-synthesis:none]`)
so a missing weight/style renders as-is instead of the browser faking bold/italic via synthesis —
a missing font asset fails visibly, not silently. **Web delivery**: ship `.woff2` only
(`src: url(...) format("woff2");`); drop `.woff`/`.ttf`/`.otf` `@font-face` fallback entries —
every browser in the support matrix has had `.woff2` support for years.

## Type Scale & Heading Hierarchy

Name scale steps semantically (`text-body-sm`, `text-heading-lg`) via a Tailwind v4 `@theme`
block or config extension — never sprinkle raw px/rem values through templates.

Heading **tag** comes from the document outline (semantics: never skip a level, e.g. `h2` straight
to `h4`); heading **size** comes from CSS (presentation). A lower-level heading MUST NOT render
larger than a higher-level one. Adjacent levels MAY share a visual size when weight or spacing
distinguishes them (e.g. `h3`/`h4` both `text-xl`, `h3` bolder).

## Line-Height, Letter-Spacing, Measure

| Role | CSS | Tailwind |
|---|---|---|
| Heading line-height | `line-height: 1.1;` (unitless — never `px`/`em`) | `leading-[1.1]` |
| Body line-height | `line-height: 1.5;` to `1.6;` | `leading-normal` (1.5) / `leading-relaxed` (1.625) |
| Large heading tracking | `letter-spacing: -0.02em;` (slightly negative) | `tracking-tight` / `tracking-tighter` |
| Small uppercase label tracking | `letter-spacing: 0.04em;` (slightly positive) | `tracking-wide` / `tracking-wider` |
| Body tracking | none — leave `normal` | `tracking-normal` (default; don't apply either direction) |
| Line measure | `max-width: 65ch;` (60–75ch range) | `max-w-prose` (65ch built-in) or `max-w-xl`/`max-w-2xl` at a 16px base |

## Truncation, Case, and Punctuation

**Truncation-with-recovery**: `truncate` / `line-clamp-*` MUST be paired with a recovery path —
a `title` attribute, tooltip, or expand affordance that surfaces the full untruncated value.
Truncating with no way to read the rest is a dead end, not a space-saving.

**Natural-case copy**: author copy in natural case; apply visual case via `text-transform`
(`uppercase`/`capitalize`/`lowercase`, Tailwind classes of the same name) — never hardcode
literal uppercase content. This keeps screen readers and copy-paste sane while the visual
treatment stays purely presentational.

**Smart punctuation**:

| Context | Use | Avoid |
|---|---|---|
| Quotes in prose | curly `“ ” ‘ ’` | straight `" '` |
| Number range | en dash `–` | hyphen `-` |
| Aside / break in thought | em dash `—` | double hyphen `--` |
| Trailing omission | single ellipsis char `…` | three periods `...` |
| Number + unit pair | non-breaking space: `16&nbsp;px` | plain space (lets it break mid-pair) |
| Long unbreakable string | soft-break hint: `&shy;` at a sane break point | unhinted overflow |

## Underline Craft

Prefer font-metric-derived underlines over the browser default offset/thickness:

```css
text-underline-position: from-font;
text-decoration-thickness: from-font;
```

Tailwind: arbitrary values (`[text-underline-position:from-font]`,
`[text-decoration-thickness:from-font]`) — no built-in utility. Where `from-font` metrics aren't
available, hand-tune `text-decoration-thickness`, `text-underline-offset`, and
`text-decoration-skip-ink: auto` instead of accepting the raw default.

A **dotted underline** (`text-decoration-style: dotted;` — Tailwind `decoration-dotted`) is a
conventional "more info on hover" hint (glossary-term style) — reserve it for that meaning, don't
reuse it as a generic link style. When only the underline's color needs to animate independently
of the text color, build the underline as a separate element (e.g. a `border-bottom` on an inner
`span`) rather than relying on `text-decoration`, which paints in the text's own color.

## Optical Trim (Progressive Enhancement)

```css
.tight-heading {
  text-box: trim-both cap alphabetic;
}
```

Tailwind: arbitrary `[text-box:trim-both_cap_alphabetic]`. Trims the leading/trailing optical
space around cap-height and the alphabetic baseline for tighter heading spacing. Treat as
progressive enhancement, not a requirement — wrap in `@supports (text-box: trim-both cap
alphabetic)` or simply let unsupported browsers fall back to the untrimmed default; do not block
ship on browser coverage.

## Mobile Inputs & Viewport

Form inputs need **16px+ font size on mobile** or iOS Safari auto-zooms on focus:

```css
/* mobile-first: 16px avoids the zoom; shrink on larger viewports */
```

Tailwind: `text-base sm:text-sm` on the input. **NEVER** set `maximum-scale=1` (or
`user-scalable=no`) in the viewport meta tag — it disables pinch-zoom and fails WCAG 1.4.4 in
every non-Safari browser (Safari happens to ignore the directive, which is why this regression
hides until a non-Safari QA pass).

## Size & Contrast Floors

| Element | Floor |
|---|---|
| Body text | 16px (`text-base`) |
| Form inputs | 14px desktop / 16px mobile (see iOS rule above) |
| Captions | 13px (`text-[13px]`) |
| Anything else | rarely below 12px |

Pair every floor with a WCAG AA contrast floor for the same text — see
`oklch-color-craft.md`'s APCA/WCAG threshold table for the exact ratios.

## RTL: Logical Properties

Use logical (flow-relative) properties so layout mirrors correctly under `dir="rtl"`:

| Physical (avoid) | Logical (use) | Tailwind |
|---|---|---|
| `margin-left` | `margin-inline-start` | `ms-*` |
| `padding-right` | `padding-inline-end` | `pe-*` |
| `text-align: left` | `text-align: start` | `text-start` |

Pair with a correct `lang` and `dir` attribute on the `<html>` element or the RTL subtree — logical
properties alone don't declare directionality, they just stop hardcoding the wrong side once it's
declared.

## Selection & Interactive Text

```css
::selection {
  background: var(--color-selection-bg);
  color: var(--color-selection-fg); /* keep this pairing at WCAG-legible contrast */
}
button {
  user-select: none; /* prevent accidental text-selection on tap/click of controls */
}
```

Tailwind: `selection:bg-*` / `selection:text-*` variants for `::selection`; `select-none` on
button/control labels in native-feel UI.

## Review Checklist

- [ ] Weight/numeric styling uses `font-weight`/`font-variant-numeric`, not raw
      `font-variation-settings`/`font-feature-settings` tags (except a genuine custom axis or
      niche feature with no property)
- [ ] `font-synthesis: none` set; only `.woff2` shipped to the web
- [ ] Type scale uses semantic names, not sprinkled raw px/rem
- [ ] Heading tags follow the document outline (no skipped levels); no lower-level heading
      renders larger than a higher one
- [ ] Heading line-height ~1.1, body 1.5–1.6, always unitless
- [ ] Letter-spacing: negative on large headings, positive on small uppercase labels, untouched
      on body
- [ ] Line measure capped at 60–75ch
- [ ] Every `truncate`/`line-clamp-*` has a recovery path to the full value
- [ ] Case is `text-transform`, never literal-case copy
- [ ] Curly quotes, en/em dashes, single ellipsis char, `&nbsp;` in value+unit pairs
- [ ] Underlines use `from-font` metrics (or tuned offset/thickness/skip-ink); dotted reserved for
      "more info" hints; separate element when only underline color animates
- [ ] `text-box: trim-both cap alphabetic` only as a guarded progressive enhancement
- [ ] Mobile inputs are 16px+; viewport meta never sets `maximum-scale=1`
- [ ] Body/input/caption sizes at or above their floor, paired with WCAG AA contrast
- [ ] Logical properties (`margin-inline-*`, `text-align: start`) + correct `lang`/`dir` for any
      RTL-reachable UI
- [ ] `::selection` stays legible; `user-select: none` on button/control labels

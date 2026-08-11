
# OKLCH Color Craft

> Full reference for the `frontend-design` skill. Adapted from `github.com/jakubkrehel/skills`
> `better-colors/*` (MIT license) — palette-generation algorithm, APCA thresholds, and gamut/P3
> handling. `skills/shadcn/customization.md` owns OKLCH *syntax* (shadcn variable theming);
> `execution-craft.md` row 9 owns the WCAG contrast floor. This file owns the mechanics neither
> of those covers: how to build a palette, how to check contrast precisely, and how to fix it.

## Key Thresholds

| Check | Threshold |
|---|---|
| Light/dark boundary | background L > 0.6 = light theme; L <= 0.6 = dark theme |
| Lightness gap — light bg | fg L < 0.35 when bg L > 0.9 |
| Lightness gap — dark bg | fg L > 0.9 when bg L < 0.25 |
| Hue drift | > 10° spread across a palette's steps reads as visibly inconsistent |
| APCA \|Lc\| — body text | >= 75 (>= 90 preferred) |
| APCA \|Lc\| — non-body text | >= 60 |
| APCA \|Lc\| — large text | >= 45 |
| WCAG 2 — normal text | 4.5:1 AA / 7:1 AAA |
| WCAG 2 — large text | 3:1 |

APCA's Lc value is **signed** (encodes polarity — light-on-dark vs. dark-on-light) — always take
the absolute value before comparing against a threshold. Mid-lightness backgrounds cap
achievable contrast no matter the foreground: a bg at L 0.75 caps out near Lc 60, so body text
on it cannot clear the 75 floor — pick a different background lightness, not a darker fg. Keep
the WCAG 2 rows alongside APCA — WCAG ratios are what a formal compliance claim cites, even
though APCA is the more perceptually accurate check to design against.

## Palette Algorithm

Scale size follows Tailwind convention — pick by how much range the palette needs:

| Steps | Labels | Use |
|---|---|---|
| 5 | 100/300/500/700/900 | minimal accent palette |
| 9 | 50-900 | **Tailwind default** |
| 11 | 50-950 | full range incl. near-black/near-white extremes |

1. **Lightness**: distribute evenly from `maxL` (lightest step) to `minL` (darkest step) with a
   delta of `0.4` between the two ends, clamped to `[0.05, 0.95]` — never pure L=0 or L=1, which
   collapse to black/white regardless of chroma or hue.
2. **Chroma**: each step's chroma is a **percentage of that step's own max achievable chroma**
   for its L/H pair — never a constant absolute C across steps. The same absolute C that looks
   vivid at L 0.5 is out-of-gamut at L 0.9 and muddy at L 0.1; C-as-%-of-max keeps saturation
   feeling consistent as lightness moves.
3. **Multi-hue consistency**: sibling palettes across hues (primary/success/warning/destructive)
   must share the same L values AND the same C%-of-max, so two hues at "step 500" read as
   equally vivid — that only holds from the same %-of-max rule, never the same absolute C.

## Dark Mode

Dark mode is the light palette's L mapping **reversed**, not a separately authored palette:

```css
.dark {
  --color-bg: var(--color-950);
  --color-fg: var(--color-50);
}
```

The step labels and chroma-% values stay identical between light and dark — only which end of
the scale plays which semantic role flips.

## Fixing Failing Contrast

When a contrast check fails, **adjust lightness only**. Chroma has negligible effect on
perceived contrast — spending time tuning C to fix a failing Lc/WCAG check is wasted effort;
move L toward the opposite end of the scale (darker fg on a light bg, lighter fg on a dark bg)
until the threshold clears, then re-check hue drift didn't regress.

## Gamut

Maximum achievable chroma varies by lightness AND hue — no single "safe" C ceiling exists; cyan
hues generally have the lowest max-chroma ceiling, hitting gamut limits sooner than warm hues at
the same L. When a computed color falls outside sRGB gamut, **clamp chroma while holding L and H
fixed** — never let a gamut fix silently drift the hue or lightness.

Ship a progressive-enhancement fallback ladder, not a hard cutover to wider gamut:

```css
:root {
  --color-primary: rgb(59 130 246); /* sRGB base, always renders */
}
@supports (color: oklch(0 0 0)) {
  :root {
    --color-primary: oklch(0.6 0.15 250); /* oklch-capable browsers */
  }
  @media (color-gamut: p3) {
    :root {
      --color-primary: oklch(0.6 0.22 250); /* wider-gamut displays only */
    }
  }
}
```

## Tailwind v4

Define the color scale in oklch inside the `@theme` directive — Tailwind v4's own default
palette is already oklch-native, so this matches the framework's own convention rather than
fighting it:

```css
@theme {
  --color-primary-500: oklch(0.6 0.15 250);
}
```

The opacity modifier (`bg-primary/50`) compiles to slash-alpha oklch syntax automatically — no
separate alpha variable or `rgb(... / var(--opacity))` plumbing needed.

## Conversion Do-Not-Touch List

When converting a codebase to oklch, leave these alone even in an otherwise-fully-oklch file:

- **CSS keywords** — `currentColor`, `inherit`, `transparent` are not colors to convert, they're
  keywords that resolve at use-site.
- **Gradient interpolation methods** — convert the gradient's color *stops* to oklch; do not
  touch the interpolation method itself (e.g. `in oklch`, `in srgb`) unless the interpolation
  space is the actual thing being changed.
- **Third-party library configs expecting hex** — a library that parses its own color config as
  hex strings breaks silently if fed an oklch string; leave those inputs as hex.

Alpha values should always use slash syntax — `oklch(60% 0.15 250 / 50%)` — never a separate
`opacity` property where the color itself can carry the alpha.

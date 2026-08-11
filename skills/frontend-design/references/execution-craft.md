
# Execution Craft

> Full reference for the `frontend-design` skill. The Canon of Slop (`anti-slop-canon.md`) names
> TASTE failures — patterns that are slop regardless of how cleanly they're built. This file
> names EXECUTION failures — craft violations independent of taste, checkable the same way a
> linter checks syntax. A design can pass the whole canon and still ship with clipped glyphs,
> an off-center badge, or a banded gradient. Verify these before calling anything shipped.
>
> Adapted from pols.dev's "Execution Failures" + "Premium Counterparts" sections (no verbatim
> reuse, no SPDX on source — rewritten in our own words per the recon's Adapt-with-rewrite cap).

Each entry: the failure's visual **signature** (what to look for), the concrete **fix**, and a
**verify** step to run before calling the surface done.

## 1. Clear the cut

**Signature:** content sliced by `overflow: hidden` / `clip-path` at a boundary that wasn't
designed for it — a descender clipped at a card edge, a heading truncated mid-glyph by a fixed
height, an image crop that lands mid-face.

**Fix:** give clipped content real headroom (line-height, padding, or a computed `min-height`
instead of a fixed one) before reaching for `overflow: hidden`. If a crop is intentional, choose
the crop point deliberately (rule-of-thirds, a clean gap between elements) instead of whatever
the container's fixed dimensions happen to land on.

**Verify:** zoom to 200% on every card/badge/avatar boundary and check for a sliced descender,
a clipped ring/glow, or a crop landing on a subject's face/hands. Resize the viewport through a
few widths — a cut that's clean at one width is often not clean at another.

## 2. Nothing is actually centered

**Signature:** an element LOOKS off-center even though its CSS says `center` — most commonly an
SVG icon or badge where the visual weight (not the bounding box) sits above or below the
geometric center.

**Fix:** for SVG text, `text-anchor: middle` alone is not vertical centering — it only centers
horizontally. Pair it with `dominant-baseline: central` (or, where that property under-supports
a browser/font combination, a manually measured `dy` offset) to center vertically against the
glyph's actual ascent/descent. For non-text elements (icons, badges, arrows), remember that
**optical center is not bounding-box center** — an icon's visual weight often sits toward one
side of its box (an arrowhead, a play triangle), so mathematically-centering the box can still
read as off-center; nudge by eye against the optical weight, not just the coordinates.

**Verify:** overlay a center-crosshair (a 1px guide line through the element's true midpoint) and
compare against where the eye actually reads the center. For SVG text specifically, check
rendering in at least two browsers — `dominant-baseline` support has historically varied.

## 3. Misaligned parallel columns

**Signature:** two or more cards/columns meant to read as a set have visibly different heights,
buttons that land at different vertical positions, or baselines that don't line up across the row
— most visible in pricing tiers and feature-comparison cards.

**Fix:** give the row a shared height reference (`align-items: stretch` on the grid/flex parent,
or an explicit `min-height` shared across the set) so every column grows to the tallest sibling.
Anchor action buttons to the BOTTOM of each card (`margin-top: auto` inside a flex column) rather
than letting them float wherever the card's content happens to end. Where columns share a labeled
row (e.g. a comparison table's feature rows), align on a shared baseline grid, not per-column flow.

**Verify:** screenshot the row and draw a horizontal line through the button row and another
through the card tops — both should be dead straight across every column.

## 4. Botched glass

**Signature:** a "glassmorphism" surface that reads as a flat gray smear — `backdrop-blur` with
no light/refraction cues, a uniform translucent fill with no edge definition, no sense of the
material actually being glass rather than a semi-transparent gray box.

**Fix — the liquid-glass parameter model.** Premium glass isn't one CSS property, it's several
cues working together. Six parameters describe the material:

| Parameter | What it controls |
|---|---|
| Light (angle / intensity) | Direction and strength of the implied specular highlight |
| Refraction | How much the backdrop visually bends through the surface |
| Depth | Apparent thickness — low values read as a thin sheet, high values as a solid slab |
| Dispersion | Chromatic edge split (the color-fringing a real lens/glass edge produces) |
| Frost | Blur amount |
| Splay | How far the refraction effect spreads from center |

Two worked starting points (Light-angle / Light-intensity / Refraction / Depth / Dispersion /
Frost / Splay):

- **Thin pill** (a small badge/chip reading as a light sheet of glass): `-45° / 80% / 80 / 2 /
  40 / 6 / 0`
- **Thick slab** (a large panel/card reading as a dense block of glass): `-50° / 60% / 64 / 44 /
  67 / 2 / 20`

**CSS approximation** (no native browser API implements the full model — approximate with
layered effects): `backdrop-filter: blur(Npx) saturate(1.2) contrast(1.05)` for the frost +
refraction cues; an inset white `box-shadow` on the TOP edge only to fake the specular light lip;
two low-opacity self-colored (not pure white/black) border strokes stacked for edge definition;
a tight, color-matched drop shadow (not a generic gray blur) so the glass reads as sitting above
the backdrop; a 1px cyan/magenta offset on the edge highlight to fake chromatic dispersion on
close inspection.

**Verify:** does the surface read as a specific MATERIAL (thin pill vs. thick slab) or as
"blurred gray box #47"? If you can't tell which of the two worked variants it's closer to, the
light/depth cues aren't doing enough work — it's the flat-smear failure, not glass.

## 5. Botched shadow / bloom-as-blurred-silhouette

**Signature:** a default drop shadow (large, gray, low-opacity, no color relationship to the
surface) OR a "glow" effect that's just a blurred duplicate of the same shape sitting behind
itself, with no actual light source implied.

**Fix:** shadows should be tight and color-matched — derive the shadow color from the surface's
own hue (a warm card gets a warm-tinted shadow, not generic black/gray) and keep the blur radius
proportional to the element's elevation, not a blanket large value. For glow/bloom effects,
imply a real light source (a highlight edge, a directional gradient) rather than duplicating the
silhouette and blurring it — a blurred copy of the same shape reads as a rendering artifact, not
light.

**Verify:** pick the shadow/glow color with an eyedropper — is it a considered hue relationship
to the surface, or literal `rgba(0,0,0,0.1)`? For bloom effects, ask whether there's an implied
light source and direction, or just a duplicated blurred shape.

## 6. Hard image seams

**Signature:** a photo/image section meets the surrounding page content with a visible hard edge
— a rectangular crop dropped onto a solid background with no transition.

**Fix — the four-part mask recipe:** (1) mask, don't overlay — use a CSS `mask-image` gradient
to fade the image's edge into transparency rather than laying a solid-color box over the seam;
(2) use a long ease — 10+ gradient stops on the mask, not a 2-stop linear fade, so the transition
reads as gradual rather than a visible band; (3) give the fade a tall section to work with — a
short fade zone still reads as a hard edge compressed; (4) match the continuous page background
color on both sides of the seam so the mask fades INTO something consistent, not from one color
to a different one.

**Verify:** crop a screenshot to just the seam area and check for a visible edge/band. If the
fade is doing its job, there should be no single pixel row where the image "ends."

## 7. Grain placement

**Signature:** a grain/noise texture applied as a full-bleed overlay across content areas,
degrading text legibility and making photos look degraded rather than textured.

**Fix:** grain belongs on BACKGROUNDS and atmosphere layers, never directly over readable content
or a photo's subject. Use `mix-blend-mode` (overlay/soft-light) at low opacity, scoped to
background/decoration elements — never as a full-page overlay sitting above text.

**Verify:** can you read every line of body text at 100% zoom without the grain interfering? Does
grain sit strictly behind content, never on top of it?

## 8. Cramped display type

**Signature:** a large display headline with default/tight line-height and letter-spacing, so
ascenders/descenders on adjacent lines nearly touch, or the type reads visually squeezed.

**Fix:** display type at large sizes needs MORE deliberate line-height control, not less — tune
`line-height` and `letter-spacing` specifically for the display size, don't inherit body-text
defaults. Large type especially benefits from slightly negative letter-spacing at very large
sizes paired with generous line-height between wrapped lines (not the same ratio as body text).

**Verify:** at the headline's actual shipped size, do ascenders/descenders on adjacent lines have
clear air between them? Does the letter-spacing feel considered or just inherited from the
browser default?

## 9. Text-contrast floor

**Signature:** low-contrast text (light gray on white, low-opacity white on a busy background)
that reads as "elegant" in isolation but fails a real accessibility check.

**Fix:** treat WCAG AA contrast ratios as a floor, not a suggestion — 4.5:1 for body text, 3:1
for large text. Low-contrast text over an image or gradient background needs either a scrim
(a gradient overlay behind the text) or a higher-contrast color choice, not just opacity tuning.

**Verify:** run an actual contrast checker against the real background color/image, not just the
design tool's isolated swatch — text over a busy photo needs the checker run against the
photo's actual pixel values under the text, not the nominal background token.

## 10. Off-center strike

**Signature:** a strikethrough, underline, or decorative rule line that sits at a mathematically
"centered" position but reads as visually off — most common on strikethrough over price text or
a decorative rule beside a heading.

**Fix:** the same optical-vs-geometric issue as centering (see #2) — a strikethrough positioned
at the exact vertical midpoint of the text box often reads as too high or too low against the
glyphs' actual visual weight. Nudge by eye against the rendered glyphs, not the box math.

**Verify:** zoom in on the strike/rule line against the text it's paired with — does it read as
intentionally placed, or does it look like it landed wherever the default math put it?

## 11. Hero owns the first screen

**Signature:** the hero section is short enough that the next section's content peeks above the
fold, undercutting the hero's impact and creating a cramped, undecided first impression.

**Fix:** size the hero to genuinely own the first viewport (`min-height: 100dvh` or close to it,
respecting mobile chrome — see Mechanics Deslop's `h-dvh` rule) rather than sizing it to its
content and letting whatever comes next bleed into view.

**Verify:** load the page at the target viewport size with nothing scrolled — does the hero fill
the screen with clear intent, or is the next section's heading visible at the bottom edge?

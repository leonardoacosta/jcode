# Separating scene grammar from design language

The supplied examples use different visual systems around the same core idea: uniform isometric
resource cubes on a projected grid, directed paths, and payload dots. The transferable skill is the scene
grammar. The surrounding interface and exact palette are incidental.

## Invariants and variables

| Keep invariant | Adapt to design language |
| --- | --- |
| repository facts and evidence | palette and contrast |
| node identities and ownership | face fills, hatching, gradients, texture |
| grid coordinates and footprints | grid visibility and line weight |
| one-cube node geometry and relative scale | corner sharpness, roof detail, shadows |
| route geometry and direction | stroke pattern, color, markers, glow |
| payload identity and flow order | dot shape, trail, cadence, easing |
| depth sorting | atmospheric depth and surface treatment |
| citation content | tooltip, caption, footnote, or optional detail treatment |

A theme change must not move nodes, rewrite routes, merge path kinds, or alter status.

## Art-direction brief

Before rendering, write a compact brief with:

1. **Name**: a specific visual language, not "modern".
2. **Principles**: 2-4 compositional rules.
3. **Medium**: screen glass, paper, print ink, clay, enamel, wireframe, etc.
4. **Linework**: thin technical, heavy editorial, broken ink, soft outline, no outline.
5. **Materials**: translucent, flat, hatched, grainy, metallic, matte, luminous.
6. **Typography**: family category, case, density, label treatment.
7. **Motion**: cadence, easing, trail, and reduced-motion fallback.
8. **Palette roles**: semantic roles, not one color per vendor product.
9. **Path treatments**: pattern, weight, marker, texture, motion cadence, and reduced-motion form for
   every path kind that appears.

## Theme examples

### Dark technical linework

- Background: near-black or deep navy.
- Grid: low-contrast cool linework.
- Cubes: subtly filled faces with brighter roof edges.
- Paths: restrained semantic accents, optional low-radius glow.
- Payloads: small high-contrast points, never large neon blobs.
- Typography: compact mono or technical grotesk.
- Motion: steady linear progress with clear pause state.

Do not automatically add metric cards, navigation rails, or control panels.

### Warm archival paper

- Background: cream, buff, or muted ochre.
- Grid: faint pencil or ruled ink.
- Cubes: paper-colored faces, sepia outlines, sparse hatching.
- Paths: dark ink with solid, dashed, or double-line distinctions.
- Payloads: filled ink beads or tiny stamped markers.
- Typography: typewriter, drafting mono, or small caps.
- Motion: restrained and slightly stepped, or static numbered payload positions.

Do not imitate a dashboard simply because the reference image was shown inside one.

### Azure semantic resource blocks

- Background: white with a faint cool Azure wash.
- Grid: light neutral rules with an Azure-blue terrain boundary.
- Cubes: one true cube per resource, lightly tinted by semantic family and outlined by the family's
  stronger stroke color.
- Roof marks: one package-local line-art resource mark projected onto the highest top face. Keep the
  mark inside the roof and use the family stroke color.
- Paths: Azure topology connector colors plus solid, dashed, or dotted treatment and explicit arrow
  shape. Color is never the only channel.
- Typography: Segoe UI for display text and Cascadia Code or a system mono for compact resource codes.
- Motion: small outlined payload points using the active connector color.

Semantic families come from `assets/azure-tokens.json`:

| Family | Stroke | Fill |
| --- | --- | --- |
| compute | `#c8460e` | `#fde6d4` |
| data | `#107c10` | `#d7ebd7` |
| identity | `#886200` | `#fbeec7` |
| integration | `#5933a3` | `#ece4f7` |
| network | `#006da3` | `#d4e9f6` |
| monitor | `#3a3d99` | `#dadcf2` |
| governance | `#5a6470` | `#ebedf0` |
| devops | `#a02763` | `#f6d8e6` |

Use exact ARM type mappings when available. Fall back to the scene role only when the node is an
abstraction. The included sprite is self-authored stand-in line art, not official Microsoft service
logo artwork.

### Editorial minimal

- Background: white or a single tinted field.
- Grid: partial or implied.
- Cubes: flat two-tone faces with role expressed by marks and paths rather than silhouette.
- Paths: one dominant ink color plus pattern differences.
- Payloads: geometric markers.
- Typography: strong sans-serif hierarchy with sparse labels.
- Motion: short deliberate transitions.

### Playful illustrated

- Background: soft color field.
- Grid: rounded or hand-drawn but still based on exact projection points.
- Cubes: friendly materials and softened linework while retaining one cubical mass per node.
- Paths: thicker lanes with clear arrowheads.
- Payloads: small tokens with distinct shapes.
- Typography: rounded display face for titles, readable sans for labels.
- Motion: eased movement with reduced-motion static endpoints.

The geometry must remain exact even when the line looks hand-drawn.

## Token model

Keep a small set of semantic variables in CSS or drawing code:

```css
:root {
  --scene-background: ...;
  --scene-grid: ...;
  --scene-structure: ...;
  --scene-control-path: ...;
  --scene-data-path: ...;
  --scene-payload: ...;
  --scene-text: ...;
  --scene-line-width: ...;
  --scene-shadow: ...;
}
```

Derive roof, left-wall, and right-wall treatments from structure or per-role tokens. Do not hard-code
one product-specific color into geometry functions.

## Styling path kinds without relying only on color

Use at least two channels:

| Path kind | Possible treatment |
| --- | --- |
| delivery | solid line plus terminal arrow |
| dependency | thinner or dashed line |
| control | firm directional line or double marker |
| data | heavier lane or continuous stream |
| network | paired line, subnet-like rhythm, or cool texture |
| identity | dashed line with key/lock terminal shape |
| telemetry | dotted or pulse rhythm |

Choose treatments that fit the medium. On paper, use line pattern and weight. In a dark scene, color
can assist but should not be the only distinction.

Record those decisions in `art_direction.path_treatments`. The structured treatment is a semantic
brief, not renderer-specific CSS, so the same dependency can become a fine dashed pen line, a broken
editorial stroke, or a subdued product token without changing its route or meaning.

## Surrounding composition

Default to a standalone scene. Add only what the task needs:

- a compact legend if line or form semantics are not self-evident;
- a small title and evidence boundary;
- hover/focus tooltips for interactive inspection;
- a flow switcher or step controls only when multiple animated flows matter;
- an explainer panel only when explicitly requested.

The isometric terrain should remain the largest and strongest object in the frame.

## Style-quality checks

- The requested design language is visible in more than color.
- Cube identity, status, and route relationships remain distinguishable in grayscale.
- Route kinds remain distinguishable without animation.
- Texture does not hide labels or evidence targets.
- Shadows and glow do not distort footprint boundaries.
- The theme can be swapped without editing repository facts or route coordinates.
- No surrounding UI was copied merely because it appeared in a reference screenshot.

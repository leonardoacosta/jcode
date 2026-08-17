# Isometric scene grammar

This is the geometry layer. It should remain valid when every color, font, texture, and surrounding
layout changes.

## Projection

Use a 2:1 dimetric grid, commonly called isometric in interface illustration:

```text
screen_x = origin_x + (grid_x - grid_y) * tile_width / 2
screen_y = origin_y + (grid_x + grid_y) * tile_height / 2 - elevation_px
```

Require:

```text
tile_width = 2 * tile_height
```

The two ground axes then rise at equal and opposite angles. Keep elevation in screen pixels. Convert
semantic building height into pixels separately so a paper illustration can feel shallow while a
technical neon scene can feel taller without moving footprints.

The bundled [`../scripts/scene_math.py`](../scripts/scene_math.py) implements `project`,
`cuboid_faces`, `route_points`, and `depth_key`.

## Ground plane

Draw the ground plane from projected grid lines, not a rotated rectangular DOM element. This keeps
nodes, routes, and grid intersections in one coordinate system.

Use enough cells to create breathing room. Typical scenes use 12-20 cells on each axis. Leave an
outer margin of at least one cell so shadows, labels, and route arrows do not clip.

The grid can be visually absent in the final art direction, but layout still uses it.

## Building massing

A basic rectangular mass has four ground corners and four elevated corners. Draw at least:

- roof face;
- left visible wall;
- right visible wall.

Do not draw flat rectangles and rotate them with CSS. That creates inconsistent depth and makes route
anchoring difficult.

### Forms

Forms are compositions of masses inside one declared footprint:

- **tower**: one narrow, tall mass; optionally stepped at the roof;
- **slab**: one broad, low mass with a long readable roof;
- **stack**: repeated horizontal layers or progressively inset masses;
- **cluster**: several smaller masses sharing one footprint;
- **gateway**: two supports with a bridge, arch, or framed opening;
- **hub**: central low mass with spokes, ports, or attached satellites;
- **bunker**: low protected block with heavy perimeter or inset core;
- **lattice**: repeated narrow frames, antennae, or open structural ribs;
- **platform**: very low plane that other meaning can sit above without implying a building.

Vary width, depth, height, roof rhythm, and internal repetition. Do not vary only color.

### Semantic use

Form can reinforce meaning but should not become a rigid icon library. Use these tendencies:

| Topology | Useful forms |
| --- | --- |
| ingress, approval, routing boundary | gateway |
| concentrated compute or identity authority | tower, bunker |
| shared network or orchestration center | hub, platform |
| storage, data, staged processing | stack, slab |
| replicated resources or fan-out | cluster |
| messaging or observability fabric | lattice, hub |

## Footprints and collision

A node position is the back-left origin of its grid footprint. Treat footprints as rectangles:

```text
[x, x + width) × [y, y + depth)
```

Two buildings collide when those intervals overlap with positive area on both axes. Checking only
identical anchor points is insufficient. Also reserve visual clearance for labels and high-traffic
routes.

## Depth order

Draw the scene back-to-front. A stable key is the far edge of each footprint:

```text
far_x = x + width
far_y = y + depth
sort_key = (far_x + far_y, far_y, far_x)
```

Draw ground routes before buildings when paths are meant to run behind architecture. Draw selected
route highlights and payload dots afterward. For elevated bridges or paths, split them into depth
segments rather than forcing one layer above everything.

## Route geometry

Architecture paths should look embedded in the terrain.

1. Give every path an explicit ordered route in grid coordinates.
2. Use horizontal or vertical grid segments. Both become diagonal on screen after projection.
3. Start at or just outside the source footprint.
4. End at or just outside the target footprint.
5. Avoid the interior of unrelated footprints.
6. Offset parallel routes by half a grid cell to create lanes.
7. Keep arrows or terminal markers visible after buildings are drawn.

Automatic center-to-center Bézier curves are acceptable only for a deliberate conceptual overlay.
They should not be the default for infrastructure or payload routes.

### Crossings

Prefer one of these, in order:

1. reroute through an open lane;
2. separate routes by half-grid offsets;
3. use a small bridge or gap convention;
4. reduce the number of simultaneous visible flows.

Do not solve crossings by adding more colors alone.

## Payload motion

A payload is an object with a name and evidence. Associate it with a path and flow step. In SVG:

1. project the route points;
2. build a path or polyline;
3. measure the path with `getTotalLength()`;
4. move the payload with `getPointAtLength(progress * length)`;
5. pause, step, or disable animation under `prefers-reduced-motion`.

Use payload shape, trail, size, and cadence according to art direction. A paper scene may use a small
ink bead. A luminous scene may use a restrained glow. The route and payload meaning remain the same.

## Labels

Use short codes on roof faces only when they remain legible. Place longer labels in the ground plane,
tooltips, captions, or a compact legend. Do not let labels replace visual hierarchy.

Check:

- roof code fits inside its top face;
- text is not mirrored by transforms;
- label color meets contrast needs;
- labels do not cover payload lanes;
- line breaks are intentional at screenshot size.

## Scene composition

The isometric image should dominate. A strong default composition is:

- terrain occupies roughly 70-90 percent of the frame;
- title and scope are small and outside the active route area;
- legend is compact and keyed to the chosen design language;
- evidence is available through tooltips, captions, a sidecar, or optional details;
- no metric cards, navigation rail, or explainer panel unless requested.

## Visual verification checklist

- The ground axes use one consistent 2:1 projection.
- Every building has coherent roof and wall faces.
- At least three forms differ by massing, not color.
- Tall buildings do not incorrectly cover nearer buildings.
- Routes sit on the same projected terrain as buildings.
- Paths do not cut through unrelated footprints.
- Direction is readable without animation.
- Payload motion follows the exact route.
- The image still works as a static screenshot.

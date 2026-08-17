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

The two ground axes then rise at equal and opposite angles. Keep elevation in screen pixels. For cube
nodes, derive the vertical edge from the projected ground-edge length instead of allowing a theme to
stretch or flatten the geometry.

The bundled [`../scripts/scene_math.py`](../scripts/scene_math.py) implements `project`,
`cuboid_faces`, `route_points`, and `depth_key`.

## Ground plane

Draw the ground plane from projected grid lines, not a rotated rectangular DOM element. This keeps
nodes, routes, and grid intersections in one coordinate system.

Use enough cells to create breathing room. Typical scenes use 12-20 cells on each axis. Leave an
outer margin of at least one cell so shadows, labels, and route arrows do not clip.

The grid can be visually absent in the final art direction, but layout still uses it.

## Resource cube geometry

A resource cube has four ground corners and four elevated corners. Draw:

- roof face;
- left visible wall;
- right visible wall.

Do not draw flat rectangles and rotate them with CSS. That creates inconsistent depth and makes route
anchoring difficult.

### Cube invariant

`cube` is the only node form. Choose one `canvas.cube_size` for the scene and center one cubical mass
with exactly that edge inside every declared collision envelope. Convert the shared edge into a
vertical world height whose projected screen length equals one projected ground edge. A node's role,
importance, footprint, status, or resource type must never change its cube size.

Do not encode service meaning through architectural silhouettes. Use the top-face mark, family palette,
short code, position, paths, and evidence. A design-language change may alter face treatment, material,
linework, shadow, and corner character, but it must not stop the node reading as one cube.

## Footprints and collision

A node position is the back-left origin of its grid footprint. Treat footprints as rectangles:

```text
[x, x + width) × [y, y + depth)
```

Two resource envelopes collide when those intervals overlap with positive area on both axes. Checking only
identical anchor points is insufficient. Also reserve visual clearance for labels and high-traffic
routes.

## Evidence-backed containment areas

Use a VNet area when source evidence says several represented resources attach to the same virtual
network. The area boundary is the bounding rectangle of every member's complete footprint plus the
declared padding. Include the represented VNet cube and each proven contained resource in
`member_ids`. The padded boundary must stay inside the ground plane.

Draw areas on the terrain below routes and cubes. Give active and held areas distinguishable boundary
treatments without relying only on color. Do not substitute a compositional zone, a VNet label, or a
network path for containment geometry. Do not add a member merely because it is visually nearby.

## Depth order

Draw the scene back-to-front. A stable key is the far edge of each footprint:

```text
far_x = x + width
far_y = y + depth
sort_key = (far_x + far_y, far_y, far_x)
```

Draw ground routes before cubes when paths are meant to run behind the nodes. Draw selected
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
7. Keep arrows or terminal markers visible after cubes are drawn.

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
- Every node is one cube with the same scene-wide edge and coherent roof and wall faces.
- Each cube's projected vertical and ground edges are equal within rendering tolerance.
- Every sourced VNet area encloses the full footprints of all declared members.
- Farther cubes do not incorrectly cover nearer cubes.
- Routes sit on the same projected terrain as cubes.
- Paths do not cut through unrelated footprints.
- Direction is readable without animation.
- Payload motion follows the exact route.
- The image still works as a static screenshot.

# Optional SVG implementation recipes

Use these recipes when the user explicitly needs editable vectors, print-oriented output, or a simple
static scene. The default interactive renderer is Canvas. These snippets implement geometry and
interaction only. Supply colors, line weights, texture, typography, and motion timing from the
scene's art direction.

## Projection

```js
function project(x, y, elevationPx = 0, p) {
  return {
    x: p.originX + (x - y) * p.tileWidth / 2,
    y: p.originY + (x + y) * p.tileHeight / 2 - elevationPx,
  };
}
```

Assert `tileWidth === 2 * tileHeight` before rendering.

## Cuboid faces

```js
function cuboidFaces(node, mass, p) {
  const x = node.position.x + mass.ox;
  const y = node.position.y + mass.oy;
  const width = mass.w;
  const depth = mass.d;
  const heightPx = mass.h * p.heightUnit;
  const floor = [
    project(x, y, 0, p),
    project(x + width, y, 0, p),
    project(x + width, y + depth, 0, p),
    project(x, y + depth, 0, p),
  ];
  const roof = [
    project(x, y, heightPx, p),
    project(x + width, y, heightPx, p),
    project(x + width, y + depth, heightPx, p),
    project(x, y + depth, heightPx, p),
  ];
  return {
    roof,
    left: [roof[3], roof[2], floor[2], floor[3]],
    right: [roof[1], roof[2], floor[2], floor[1]],
  };
}

function polygonPoints(points) {
  return points.map(({ x, y }) => `${x.toFixed(2)},${y.toFixed(2)}`).join(" ");
}
```

Create SVG elements with `document.createElementNS("http://www.w3.org/2000/svg", tag)` and set
attributes directly. Do not build source-derived strings with `innerHTML`.

## True cube mass

Center one cube inside the declared footprint and derive its vertical height from the projected ground
edge:

```js
function cubeMassFor(node, scene, projection) {
  const { width: w, depth: d } = node.footprint;
  const edge = scene.canvas.cube_size;
  const groundEdge = Math.hypot(projection.tileWidth / 2, projection.tileHeight / 2);
  return {
    ox: (w - edge) / 2,
    oy: (d - edge) / 2,
    w: edge,
    d: edge,
    h: edge * groundEdge / projection.heightUnit,
  };
}
```

Do not vary `edge` per node or add secondary masses or line structures that turn the resource into a
building. Service identity belongs on the top face.

## VNet area polygons

Derive a VNet area's world rectangle from its member footprints plus padding, project the four corners,
and draw it before routes and cubes. Give the polygon a focusable semantic peer with the area's label,
status, description, and evidence. Never hand-place an area boundary independently of its members.

## Back-to-front ordering

```js
const ordered = [...scene.nodes].sort((a, b) => {
  const afx = a.position.x + a.footprint.width;
  const afy = a.position.y + a.footprint.depth;
  const bfx = b.position.x + b.footprint.width;
  const bfy = b.position.y + b.footprint.depth;
  return (afx + afy) - (bfx + bfy) || afy - bfy || afx - bfx;
});
```

Each node has one mass, so only node-level depth ordering is required.

## Grid-routed paths

```js
function routePath(route, p, elevationPx = 0) {
  const projected = route.map(point => project(point.x, point.y, elevationPx, p));
  return projected.map((point, index) =>
    `${index === 0 ? "M" : "L"} ${point.x.toFixed(2)} ${point.y.toFixed(2)}`
  ).join(" ");
}
```

For rounded corners, compute small entry and exit offsets at each turn and use quadratic segments.
Preserve the same route points. Do not replace the terrain route with one arbitrary curve.

Give every rendered path:

```js
path.setAttribute("role", "button");
path.setAttribute("tabindex", "0");
path.setAttribute("aria-label", `${item.label}: ${from.label} to ${to.label}`);
```

Support Enter and Space the same way as click.

## Arrowheads

Use an SVG marker whose fill or stroke is `context-stroke`:

```html
<marker id="route-arrow" markerWidth="8" markerHeight="8"
        refX="7" refY="3" orient="auto" markerUnits="strokeWidth">
  <path d="M0,0 L0,6 L7,3 Z" fill="context-stroke" />
</marker>
```

In monochrome art directions, distinguish route kinds with dash pattern, line weight, terminal shape,
or doubled lanes rather than color alone.

## Payload animation

```js
function positionPayload(dot, path, progress) {
  const length = path.getTotalLength();
  const point = path.getPointAtLength(Math.max(0, Math.min(1, progress)) * length);
  dot.setAttribute("cx", point.x);
  dot.setAttribute("cy", point.y);
}
```

Use one state object per visible payload:

```js
{
  flowIndex: 0,
  stepIndex: 0,
  progress: 0,
  paused: matchMedia("(prefers-reduced-motion: reduce)").matches,
}
```

When reduced motion is active:

- do not keep a requestAnimationFrame loop alive;
- show the payload at the current step endpoint;
- retain flow selection and step controls if interactive;
- expose the current flow and step in the payload's accessible name.

## Focus and hover parity

Nodes, paths, payloads, and any matching legend items should share one highlight model. Focusing a
path should reveal the same source/target emphasis as hovering it. Focusing a legend item should
highlight matching scene elements without moving the layout.

Use `:focus-visible` rather than removing outlines. If a custom focus ring is drawn in SVG, keep it
high contrast in the selected art direction.

## Evidence without a fixed panel

Evidence can be exposed through several patterns:

- `<title>` plus a keyboard-accessible details popover;
- click/focus tooltip near the selected cube;
- a compact caption below the scene;
- expandable footnotes;
- the mandatory scene JSON sidecar;
- an optional details panel only when requested.

Keep citation strings as text nodes. Never inject repository content into raw HTML.

## Static export

For a static SVG:

- place all styles in the SVG `<style>` element;
- replace moving dots with numbered or repeated payload markers;
- include route direction markers;
- keep a compact legend when line patterns need explanation;
- use a viewBox that includes all shadows and labels.

For PNG export, render the SVG or HTML at the intended final dimensions. Inspect the PNG, because
small labels and half-pixel lines may differ from the browser view.

## Self-contained HTML shell

A minimal interactive artifact needs only:

```html
<!doctype html>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width,initial-scale=1">
<title>Repository isometric map</title>
<style>/* art-direction tokens and scene styles */</style>
<main>
  <svg id="scene" viewBox="0 0 1400 900" role="img" aria-labelledby="title desc">
    <title id="title">...</title>
    <desc id="desc">...</desc>
  </svg>
</main>
<script>/* inline scene data and geometry code */</script>
```

Do not add dashboard chrome by default. Let the SVG fill the viewport and make the terrain the focal
point.

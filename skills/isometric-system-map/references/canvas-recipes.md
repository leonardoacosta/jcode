# Canvas implementation recipes

Use this renderer architecture for interactive, animated, or strongly art-directed isometric maps. The semantic scene JSON remains the source of truth. Canvas is a rendering target, not a replacement for evidence, geometry, or accessible structure.

## Default architecture

Render one scene through four cooperating layers:

1. **Terrain canvas**: background, texture, ground plane, grid, and quiet zone labels.
2. **Architecture canvas**: routes, arrows, resource cubes, and compact node codes.
3. **Motion canvas**: payloads plus hover, focus, and selected-state highlights.
4. **DOM semantic mirror**: one native focusable control for every interactive node and path, plus minimal flow, pause, and export controls.

The canvases share identical CSS dimensions and projection state. Only redraw a layer when its content changes. Do not redraw terrain and cubes on every payload frame.

The bundled implementation follows this architecture:

```text
scripts/render_canvas.py
templates/canvas-renderer.html
themes/dark-technical.js
themes/warm-paper.js
themes/azure-topology.js
assets/azure-icons.svg
assets/azure-tokens.json
```

Render a validated scene with:

```bash
python3 skills/isometric-system-map/scripts/render_canvas.py \
  docs/diagrams/<repo>-isometric-scene.json \
  skills/isometric-system-map/themes/dark-technical.js \
  docs/diagrams/<repo>-isometric-map.html
```

## Keep scene facts out of drawing code

The renderer may calculate projected points, cube faces, area polygons, hit regions, and animation positions. It must not infer repository facts or silently rewrite:

- node identity, role, status, ownership, or evidence;
- grid position, footprint, scene-wide cube size, or containment membership;
- path endpoints, kind, explicit route, or payload membership;
- flow order or step evidence;
- art-direction principles and path treatments.

A theme adapter changes presentation only. Rendering the same scene through two themes must preserve the same semantic hash.

## Projection and responsive fitting

Use a true 2:1 projection:

```js
function project(x, y, z = 0, view) {
  return {
    x: view.originX + (x - y) * view.tileWidth / 2,
    y: view.originY + (x + y) * view.tileHeight / 2 - z * view.heightUnit,
  };
}
```

Assert `tileWidth === 2 * tileHeight` in semantic geometry helpers. At runtime, fit the complete ground plane to the available viewport while retaining that ratio. Let the renderer choose `originX`, `originY`, and the screen-space height unit.

Use `ResizeObserver` on the stage rather than assuming a fixed viewport. Recompute view state and redraw all layers after a resize.

## High-DPI canvases

Keep drawing coordinates in CSS pixels while allocating a denser bitmap:

```js
const rect = stage.getBoundingClientRect();
const dpr = Math.min(window.devicePixelRatio || 1, 2);
canvas.width = Math.round(rect.width * dpr);
canvas.height = Math.round(rect.height * dpr);
canvas.style.width = `${rect.width}px`;
canvas.style.height = `${rect.height}px`;
ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
```

Cap DPR when very large displays would create excessive bitmaps. Reapply the transform after any width or height assignment because resizing resets the context.

## Retained geometry with `Path2D`

Canvas is immediate mode, but interaction geometry should be retained:

```js
const nodeHits = [];
const routeHits = [];

const roof = new Path2D();
roof.moveTo(...);
roof.lineTo(...);
roof.closePath();
nodeHits.push({ item: node, path: roof });
```

Build one combined `Path2D` per node from its three visible cube faces, one per sourced area, and one
per explicit route. Reuse the same objects for:

- drawing;
- hover and click hit testing;
- selected-state outlines;
- keyboard focus rings;
- export rendering.

For pointer interaction:

```js
if (hitContext.isPointInPath(node.path, x, y)) { /* node */ }

hitContext.lineWidth = Math.max(14, visualStrokeWidth + 10);
if (hitContext.isPointInStroke(route.path, x, y)) { /* route */ }
```

Use a generous invisible route hit width. Do not make users hit a one-pixel line.

## True resource cubes

Render every node as one identically sized cube. The footprint remains a collision and routing
envelope, so the shared cube may be centered inside a wider reserved area. Read its edge only from
`scene.canvas.cube_size`, then derive the world-space vertical height from the projected ground edge:

```js
const cubeEdge = scene.canvas.cube_size;
const projectedEdge = Math.hypot(tileWidth / 2, tileHeight / 2);
const projectedHeight = cubeEdge * projectedEdge / heightUnit;

const mass = {
  ox: (node.footprint.width - cubeEdge) / 2,
  oy: (node.footprint.depth - cubeEdge) / 2,
  width: cubeEdge,
  depth: cubeEdge,
  height: projectedHeight,
};
```

This makes the two projected ground edges and the vertical edge equal in screen space. The validator
ensures every footprint can contain the shared edge. Do not branch on role, resource type, footprint,
status, or importance to change cube size or create towers, stacks, gateways, or compound masses. Sort
cubes by the far edge of their declared envelopes, then draw left, right, and roof faces.

## VNet containment areas

Render each sourced VNet area on the terrain layer before routes and cubes. Derive its rectangle from
the full member envelopes so the visible boundary proves containment:

```js
function areaRect(area, scene) {
  const members = area.member_ids.map(id => scene.nodes.find(node => node.id === id));
  const left = Math.min(...members.map(node => node.position.x)) - area.padding;
  const back = Math.min(...members.map(node => node.position.y)) - area.padding;
  const right = Math.max(...members.map(node => node.position.x + node.footprint.width)) + area.padding;
  const front = Math.max(...members.map(node => node.position.y + node.footprint.depth)) + area.padding;
  return { x: left, y: back, width: right - left, depth: front - back };
}
```

Retain one `Path2D` per area for pointer hit testing and focus outlines. Mirror every area to a native
button just like nodes and paths. Runtime diagnostics should report each area's ID, kind, status,
member IDs, and derived bounds.

## Project line-art marks onto roof faces

For Azure resource blocks, extract only the scene-used `<symbol>` elements from the package-local
sprite and embed them as standalone SVG strings in the generated HTML. This keeps the artifact
self-contained without shipping the entire sprite. Replace `currentColor` with the semantic family
stroke before loading each SVG into an `Image`.

Retain the four projected roof points from the cuboid geometry. A roof is a parallelogram, so map the
24×24 icon square onto an inset square in world coordinates with one affine Canvas transform:

```js
const u = { x: roof[1].x - roof[0].x, y: roof[1].y - roof[0].y };
const v = { x: roof[3].x - roof[0].x, y: roof[3].y - roof[0].y };
const iconWorldSize = Math.min(mass.width, mass.depth) * 0.64;
const uFraction = iconWorldSize / mass.width;
const vFraction = iconWorldSize / mass.depth;
const origin = {
  x: roof[0].x + u.x * (1 - uFraction) / 2 + v.x * (1 - vFraction) / 2,
  y: roof[0].y + u.y * (1 - uFraction) / 2 + v.y * (1 - vFraction) / 2,
};

ctx.save();
ctx.transform(
  u.x * uFraction / 24,
  u.y * uFraction / 24,
  v.x * vFraction / 24,
  v.y * vFraction / 24,
  origin.x,
  origin.y,
);
ctx.drawImage(icon, 0, 0, 24, 24);
ctx.restore();
```

Draw one mark after all of that node's faces. Do not put a screen-aligned badge over the cube; the mark
must share the roof plane. Await image loading
before the first final render and expose loaded/rendered icon counts in runtime diagnostics.

The included sprite is self-authored Azure-style stand-in line art. Do not describe it as Microsoft's
official service-logo set. See `assets/PROVENANCE.md`.

## Explicit terrain routes

Project the exact route array from the scene contract. Draw straight terrain-aligned segments and preserve route turns. Do not substitute arbitrary center-to-center curves.

A theme adapter may vary:

- stroke color and width;
- dash pattern;
- under-stroke or doubled lane;
- terminal marker shape;
- texture;
- payload cadence.

It must not change route coordinates or direction.

Path labels are optional. Prefer a compact legend plus focus or hover details over repeating kind labels on dense terrain.

## Timestamp-based payload motion

Use `requestAnimationFrame` timestamps, not a fixed pixels-per-frame increment:

```js
function animationFrame(timestamp) {
  drawMotion(timestamp);
  if (!state.paused && !reducedMotion.matches && !document.hidden) {
    state.raf = requestAnimationFrame(animationFrame);
  }
}
```

Calculate progress from elapsed time and polyline segment lengths. This keeps speed stable across refresh rates and long frames.

A payload is drawn only for a flow step whose referenced path lists that payload ID. If the scene has multiple flows, expose a small flow selector and reset the timeline when selection changes. If `flows` is empty, hide motion controls and leave the scene static.

## Reduced motion

`prefers-reduced-motion: reduce` is a runtime stop condition, not merely a slower animation:

- cancel any active animation frame;
- do not schedule another frame;
- draw each visible payload at a stable step endpoint;
- disable or relabel motion controls;
- retain flow selection, keyboard inspection, evidence, and PNG export.

Listen for preference changes because a user can change the setting while the artifact is open.

## Accessible semantic mirror

A Canvas bitmap has no object-level semantics. Create a one-to-one native control for every node and path:

```html
<div class="sr-only" role="group" aria-label="Inspectable map elements">
  <button data-target-kind="node" data-target-id="sql">SQL data tier</button>
  <button data-target-kind="path" data-target-id="deploy-sql">Deploy SQL</button>
</div>
```

Keep the mirror in the same labelled scene region. Focus and click must select the matching Canvas geometry, show the same evidence tooltip, and draw the same highlight used by pointer hover.

Use `drawFocusIfNeeded(path, element)` when the mirrored element is focused. Preserve visible browser focus treatment for controls outside the bitmap.

The HTML Standard permits focusable fallback descendants inside `<canvas>`. In the browser runtime used to verify this package, descendants inside a `role="img"` canvas did not appear in the accessibility snapshot. A visually hidden sibling mirror was exposed reliably, so the bundled renderer uses and tests that arrangement.

## Theme adapter contract

A theme is executable presentation logic around stable scene geometry. The bundled adapter shape is:

```js
(() => ({
  id: "theme-id",
  name: "Specific design language",
  css: {
    pageBackground: "...",
    text: "...",
    muted: "...",
    hairline: "...",
    controlBackground: "...",
    controlHover: "...",
    focus: "...",
    fontFamily: "...",
  },
  motion: { stepDuration: 2800 },
  pathStyle(kind) { /* color, width, dash */ },
  drawBackground(ctx, state, scene) {},
  drawGround(ctx, ground, state, scene) {},
  drawGridLine(ctx, path, boundary, state) {},
  drawZone(ctx, zone, point, state) {},
  drawRoute(ctx, path, item, state) {},
  drawArrow(ctx, points, item, state) {},
  drawPathLabel(ctx, points, item, state) {},
  drawFace(ctx, path, side, node, massIndex, state) {},
  drawNodeLabel(ctx, node, point, state) {},
  drawPayload(ctx, payloadState, state) {},
  drawSelection(ctx, path, kind, mode, item, state) {},
}))()
```

Use more than color to distinguish themes. The dark adapter uses luminous technical edges, scan texture, filled arrows, and steady motion. The paper adapter uses grain, hatching, open arrows, typewriter typography, and slower restrained motion.

## Evidence interaction

A focused or selected target should reveal:

- label;
- source and target plus path kind, or role plus status;
- short description;
- exact `path:lines` citation;
- the claim in the semantic sidecar.

Use text nodes when building tooltips. Never insert repository-derived strings with raw `innerHTML`.

## PNG export

Composite the three scene canvases onto a temporary output canvas, then call `toBlob(..., "image/png")`. Export the scene image, not optional DOM captions or control chrome. Revoke temporary object URLs after download.

If externally loaded images are ever introduced, preserve an origin-clean canvas or `toBlob()` will fail with a security error. The bundled themes generate patterns locally and load no external assets.

## Performance ladder

Use the least complex solution that meets measured needs:

1. Separate static and dynamic layers.
2. Retain geometry with `Path2D`.
3. Batch routes and minimize context state changes.
4. Cache repeated paper, hatch, or scan patterns on small offscreen canvases.
5. Avoid large blur radii and unnecessary text redraws in the motion loop.
6. Profile before adding workers.
7. Use `OffscreenCanvas` only when measured main-thread work still misses the target frame budget.

For the contract cap of 28 nodes, three ordinary layered canvases are normally simpler and sufficient.

## Browser verification

Do not stop at generated source. Open the artifact and verify:

- exactly three scene canvases have matching CSS and pixel dimensions;
- the stage reports rendered state after resize;
- all nodes and paths appear as focusable controls in the accessibility tree;
- focusing a semantic control updates the Canvas selection and tooltip;
- pause cancels the active animation frame and frame count stops;
- reduced motion starts and remains without a live animation frame;
- two theme adapters render the same scene hash with visibly different materials, linework, typography, and motion;
- the image contains no fixed dashboard shell unless explicitly requested;
- PNG export completes from the composited Canvas layers.

## Primary references

- [MDN: Optimizing canvas](https://developer.mozilla.org/en-US/docs/Web/API/Canvas_API/Tutorial/Optimizing_canvas), including layered canvases, offscreen caching, HiDPI scaling, and `requestAnimationFrame`.
- [MDN: Path2D](https://developer.mozilla.org/en-US/docs/Web/API/Path2D), retained and replayable path geometry.
- [MDN: drawFocusIfNeeded](https://developer.mozilla.org/en-US/docs/Web/API/CanvasRenderingContext2D/drawFocusIfNeeded), Canvas focus-ring mapping.
- [MDN: devicePixelRatio](https://developer.mozilla.org/en-US/docs/Web/API/Window/devicePixelRatio), sharp Canvas rendering on dense displays.
- [MDN: ResizeObserver](https://developer.mozilla.org/en-US/docs/Web/API/ResizeObserver), responsive stage observation.
- [MDN: requestAnimationFrame](https://developer.mozilla.org/en-US/docs/Web/API/Window/requestAnimationFrame), timestamp-based animation.
- [MDN: prefers-reduced-motion](https://developer.mozilla.org/en-US/docs/Web/CSS/@media/prefers-reduced-motion), user motion preference.
- [MDN: HTMLCanvasElement.toBlob](https://developer.mozilla.org/en-US/docs/Web/API/HTMLCanvasElement/toBlob), image export.
- [HTML Living Standard: canvas](https://html.spec.whatwg.org/multipage/canvas.html), fallback semantics, focusable regions, focus rings, and best practices.

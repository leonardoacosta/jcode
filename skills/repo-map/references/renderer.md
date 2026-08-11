# repo-map Renderer — Maintenance Notes

Notes for maintaining `templates/renderer.html`, the frozen template
`scripts/bin/repo-map-render` injects data into. The template owns 100% of styling and
layout; a change here should never require a change to an extraction guide or the contract.

## Injection placeholder contract

`render()` in `scripts/bin/repo-map-render` does a single-occurrence `String.replace()` per
token — each of the three tokens below MUST appear **exactly once** in the template, inside
the `<script>` block's `var` assignments. **Never restate a token's literal double-brace
form anywhere earlier in the file** (a comment, a doc string) — `.replace()` takes the FIRST
match, so an earlier literal silently steals the slot and the real assignment is left
untouched (shipped-and-caught bug: the template's own header comment used to spell out
`{{REPO_MAP_DATA}}` for documentation purposes, which ate the real substitution — fixed by
describing tokens without their brace syntax in prose).

| Token | Replaced with | Shape |
| --- | --- | --- |
| `{{REPO_MAP_DATA}}` | `JSON.stringify(doc)` — the full, unfolded contract document | object (`references/contract.md`) |
| `{{REPO_MAP_COORDINATES}}` | `JSON.stringify(Object.fromEntries(coordinates))` | `{ [nodeId]: {x,y,width,height}, "__group__<id>": {x,y,width,height} }` |
| `{{REPO_MAP_FAVICONS}}` | `JSON.stringify(favicons)` | `{ [domain]: dataUri \| null }` |

The template client-side JS re-derives the fold (model/tool → chips) from `DATA.nodes`/
`DATA.edges` itself — `coordinates` only carries positions for KEPT nodes plus `__group__`
entries, so chip lists aren't in the injected coordinate map and must be recomputed to stay
in sync with `scripts/bin/repo-map-render`'s own `fold()`.

## Kind → Geist accent mapping

Seven named CSS custom properties (`--rm-blue`/`--rm-teal`/`--rm-green`/`--rm-amber`/
`--rm-red`/`--rm-orange`/`--rm-purple`/`--rm-pink`) cover the 10 contract kinds — design.md
D4 named five explicit pairs plus the agent orange sixth slot; `tool`/`external`/`module`
extend the mapping (not specified in D4, decided here) since the kind palette needs more
slots than the Geist page palette has named pairs:

| Kind | CSS var | Note |
| --- | --- | --- |
| `model` | `--rm-blue` | Folds to a chip — never a standalone node |
| `tool` | `--rm-blue` | Folds to a chip alongside `model`; shares its color since both render as chips on the same parent card |
| `store` | `--rm-teal` | |
| `entry` | `--rm-green` | |
| `service` | `--rm-green` | Shares `entry`'s slot (design.md D4) |
| `worker` | `--rm-amber` | |
| `queue` | `--rm-amber` | Shares `worker`'s slot (design.md D4) |
| `agent` | `--rm-orange` | The sixth mapped accent (design.md D4) |
| `external` | `--rm-purple` | Extension beyond D4's five pairs |
| `module` | `--rm-pink` | Extension beyond D4's five pairs |

`--rm-red` is reserved for error/emphasis states and is never assigned to a kind.

## Pulse / choreography parameters

- **Traveling edge pulse** (`.rm-edge-pulse`): `stroke-dasharray: 4 156`, `2.6s linear
  infinite` keyframe animating `stroke-dashoffset` to `-160`. One CSS animation per edge —
  zero per-edge JS loops (foglamp `flow-map.tsx`). Each edge's pulse gets a small
  `animation-delay` (`index * 90ms`) so parallel edges don't pulse in lockstep.
- **Entrance choreography** (`.rm-node`): `animation-delay` is `round((x / maxX) * 350)ms` —
  proportional to the node's x-position in the laid-out coordinate space, so the map "draws
  itself along the flow" left to right. Keyframe: `opacity 0→1`, `translateY(6px)→0`,
  `scale(.97)→1`, `blur(4px)→0` over `.5s cubic-bezier(.2,.8,.2,1)`.
- **`prefers-reduced-motion: reduce`** disables both: entrance animation is removed entirely
  (cards render at their final state immediately) and the pulse keyframe is disabled with
  opacity fixed at `.5` (edges stay visibly "alive" without motion).

## Personality thresholds

`archetype(stats, total)` scores four candidates via `min(bucket / (total * threshold), 1) *
weight` (saturation-capped — no single dominant stat can auto-win past its cap), highest
score wins, with a `0.5` floor fallback to "Steady Builder" (design.md D5; ported from
foglamp `personality.ts`'s documented fix for a first-match rule that "made almost everything
an Orchestrator"):

| Archetype | Stat bucket | Threshold (fraction of total nodes) | Weight |
| --- | --- | --- | --- |
| AI Orchestrator | `intelligenceNodes` (model+tool) | 0.35 | 1.0 |
| Data Fortress | `dataNodes` (store+queue) | 0.35 | 0.95 |
| Service Mesh | `controlNodes` (entry/service/worker/agent) | 0.5 | 0.9 |
| Infra Backbone | `infraNodes` (external+module) | 0.35 | 0.9 |
| Steady Builder | — (fallback below the 0.5 floor) | — | — |

Stats are computed client-side from the kept + folded-chip nodes' kinds — the template never
trusts a possibly-stale `doc.stats` field, so the personality card stays accurate even when
the source JSON's `stats` block wasn't recomputed after a merge.

## Named-flow lens

The flow `<select>` is populated from `DATA.flows`; selecting one applies `.lit` to every
edge whose `flows[]` includes the selected id (plus their endpoint nodes) and `.dimmed`
everywhere else. At most one lens is active — selecting a different flow (or the empty
"No flow lens" option) atomically swaps the highlighted set. `j`/`k` keys cycle through the
registered flows when focus isn't inside a form control. This lens is independent of, and
layered alongside, click-to-trace (`.rm-dimmed` classes compose the same way for both).

## PNG export

`#rm-export` serializes the `#rm-stage` element (all inline styles, resolved via the page's
own stylesheet text) into an SVG `<foreignObject>`, loads it as an `Image`, draws it onto a
2× canvas, and triggers a `.png` download via `canvas.toDataURL()`. Pure browser natives — no
bundled library. Because the stage is plain HTML+CSS (not the all-SVG shape blueprint's
`app.js exportPNG` ports from), the same-origin/inline-style constraint is what keeps this
approach tainting-free: no external stylesheet or image `src` may be added to the template
without checking it still works from a `file://` origin.


# Box-Drawing Canon

> Glyph vocabulary, component recipes, composition rules, and export wrappers for hand-authoring
> ASCII wireframes. Derived from `mualat/wiretext` (MIT) `src/utils/boxDrawing.ts`.

## Contents

- [Border Families](#border-families)
- [Component Recipes](#component-recipes) (30 rows)
- [Composition Rules](#composition-rules)
- [Export Wrappers](#export-wrappers)

## Border Families

Four border families. Pick by emphasis: `single` is the default; `rounded` softens cards/modals;
`double` and `heavy` draw the eye to a primary container. Within a family the corner/edge glyphs are
fixed — mixing glyphs from different families on one box reads as a rendering bug, so stay in one
family per box.

| Family | topLeft | topRight | bottomLeft | bottomRight | horizontal | vertical | teeRight | teeLeft |
|---|---|---|---|---|---|---|---|---|
| single | `┌` | `┐` | `└` | `┘` | `─` | `│` | `├` | `┤` |
| double | `╔` | `╗` | `╚` | `╝` | `═` | `║` | `╠` | `╣` |
| rounded | `╭` | `╮` | `╰` | `╯` | `─` | `│` | `├` | `┤` |
| heavy | `┏` | `┓` | `┗` | `┛` | `━` | `┃` | `┣` | `┫` |

The `teeRight`/`teeLeft` glyphs anchor a horizontal divider into the left and right walls (used by
modals, cards, tables, browsers, sidebars, accordions). A divider row is the wall tee + a run of
`horizontal` + the opposite wall tee:

```
╭────────────╮      single example:   ┌────────────┐
│ Title      │                        │ Title      │
├────────────┤  <- teeRight … teeLeft ├────────────┤
│ body       │                        │ body       │
╰────────────╯                        └────────────┘
```

Inner column joints (tables) use the cross/tee glyphs `┬` (top), `┼` (middle, at a divider row),
`┴` (bottom).

Directional arrows for connectors: `→ ← ↑ ↓` (orthogonal) and `↗ ↘ ↙ ↖` (diagonal); dot head `●`.

## Component Recipes

One row per component (30 total). Each example is the *interior* glyph pattern the recipe draws —
wrap it in a box border (per the family table above) at the listed default size unless the recipe
already includes its own frame. Labels truncate with `…` when they exceed the inner width; numeric
widgets (progress, slider, pagination) scale to fill the inner width.

| Component | Example | Recipe notes |
|---|---|---|
| button | `[ Save ]` or boxed `┌──────┐` / `│ Save │` / `└──────┘` | label centered in the box (`placeCenteredText`), default 12x3 |
| input | `│ Email           │` | label left-padded 2 cols, vertically centered; empty shows the field name |
| select | `│ Country      ▾ │` | label left, `▾` chevron drawn 3 cols from the right wall |
| checkbox | `[✓] Remember me` | `[✓]` when `checked`, `[ ]` when not; space + label; truncates with `…` |
| radio | `(●) Card` | `(●)` when `checked`, `(○)` when not; space + label |
| table | `│ Name │ Role │` over `├──────┼──────┤` | header row, `teeRight…teeLeft` rule on row+2, columns split evenly with `┬`/`┼`/`┴` joints |
| modal | titled box + `├───┤` rule + `×` close | `drawModalCard` with close: title at row+1, divider at row+2, `×` 3 cols from right |
| browser | `◄ ► ⟳  https://…` over `├───┤` | nav glyphs at row+1, URL from col+10, address-bar rule at row+2 |
| card | rounded titled box (no close) | `drawModalCard` without close, forced `rounded` border |
| navbar | `≡  Home  About  Contact` | hamburger `≡` at col+2, nav items joined by two spaces |
| tabs | `Overview │ Activity │ Settings` | tab labels joined by ` │ `, vertically centered |
| progress | `▓▓▓▓░░░░░░` | `▓` filled / `░` empty; filled = round(progress/100 * innerWidth) |
| textarea | `Text...` over `············` | label on first inner row, dot-fill `·` rows below to suggest multi-line |
| slider | `──────●─────────` | track of `─` with `●` thumb at round(value/100 * (track-1)) |
| toggle | `(● )` off / `( ●)` on | knob left when off, right when on |
| accordion | `▾ Section 1` / `  Content...` / `├──┤` / `▸ Section 2` | first item expanded (`▾` + content), rest collapsed (`▸`), separated by tee rules |
| sidebar | `≡ Menu` / `├──┤` / `› Home` / `› Profile` | hamburger header, tee rule at row+2, items prefixed `› ` |
| avatar | `(◉)` with `AB` below | `◉` centered; initials (first letters of label words, max 2, upper) on the next row |
| badge | `99+` centered | `placeCenteredText`; defaults to `1` |
| breadcrumb | `Home › Page › Sub` | items joined by ` › `, left-padded 2, truncates with `…` |
| dropdown | `Option 1  ▾` over `├──┤` / `Option 2` | header = first item + `  ▾`, tee rule, remaining items listed below |
| search | `⌕ Search...` | magnifier `⌕` + space + placeholder, vertically centered |
| stepper | `[-] 3 [+]` | minus / value / plus, padded to inner width |
| calendar | `◄  Month 2026  ►` / `Su Mo Tu We Th Fr Sa` / `1  2  3 …` | month header, tee rule, day-name row, 4 week rows of numbers |
| list | `• Item 1` / `• Item 2` (or `1.` ordered) | bullet `•` (or `N.` when `listOrdered`) + space + item, one per row |
| divider | `──────────────` | single run of `─` across the width, height 1 |
| tooltip | `Tooltip` with `▽` below | centered text, downward pointer `▽` on the last row |
| tag | `〖Tag〗` | bracketed label `〖…〗`, centered |
| spinner | `(...)` centered | loading placeholder, centered (wiretext uses an emoji glyph here; substitute an ASCII token per the no-emoji house rule) |
| pagination | `‹ 1 2 [3] 4 5 ›` | `‹`/`›` ends; current page bracketed `[N]`; `...` when totalPages > 7 |

> Worked frame for the boxed widgets above (button at default 12x3, single border):
> ```
> ┌──────────┐
> │   Save   │
> └──────────┘
> ```

## Composition Rules

These three rules are why the output stays clean when widgets overlap or sit edge-to-edge. They
mirror the engine's write policies — internalize them and hand-authored sketches match what the
renderer would produce.

### 1. Spaces never erase; non-spaces follow draw order (the key correctness rule)

The engine has two write policies. `setChar(grid, col, row, char)` writes when the char is
**non-space OR the target cell is already blank** — a glyph always lands, but a *space* is skipped
if it would overwrite an existing glyph. `drawChar` always writes (even spaces).

The practical consequence is the inverse of what you might guess: a connector or line routed
*across* a box border **overwrites the wall** — a non-space glyph always wins, last-writer-takes-the-
cell by draw order. What `setChar` protects is the opposite case: drawing a box's interior *fill*
(spaces) never punches holes in a line already drawn underneath.

So crossings are governed by **draw order**, not by an automatic "border wins" rule:

- To keep a wall intact where a connector approaches, **stop the connector one cell short** of the
  wall (route to the adjacent cell, not onto the border).
- To make a clean junction *on* a wall, place a tee (`├ ┤ ┬ ┴`) or cross (`┼`) glyph yourself.
- If a line is drawn last and lands on the border, it replaces that border glyph with `─`/`│`.

```
┌────────┐        ┌────────┐
│  Auth  │ ─────▶ │  API   │     arrow stops one cell short of each wall — walls stay intact
└────────┘        └────────┘

┌────────┬────────┐
│  Left  │  Right │              divider meets the top wall with an explicit tee ┬, not a raw │
└────────┴────────┘
```

### 2. Tight-fit trailing-whitespace trim

On export the engine trims trailing spaces from every line (`gridToString`) so there is no ragged
right-edge padding — each line ends at its last non-space glyph. Hand-author the same way: never pad
lines out to a uniform width with trailing spaces. A wireframe with trailing whitespace diffs noisily
and wastes columns. Interior spaces inside a box are fine (they are real layout); it is only the
trailing run after the last glyph that gets cut.

### 3. Centered-text truncation with ellipsis

Centered labels (`placeCenteredText`) and checkbox/radio labels cap at **width − 4**: a label longer
than that is cut to `width − 5` chars plus a single `…`. This keeps text inside the walls instead of
overflowing them. When you hand-author a label that is too long for its box, do the same — truncate
and append `…` rather than widening the box past its intended slot or letting text bleed over the
border.

```
┌──────────┐
│ Long la… │     label "Long label here" in a width-12 box -> 7 chars + …
└──────────┘
```

## Export Wrappers

The same raw box-drawing string ships in four forms (from wiretext `ExportModal.tsx`). Pick by
destination.

**Plain text** — the raw grid, dropped straight into a terminal, a tasks.md note, or a beads
comment. No wrapper.

**Markdown** — fence it so the monospace alignment survives in any markdown renderer:

````
```
┌────────┐
│  Auth  │
└────────┘
```
````

**HTML** — for a web surface (README, blog) where you need a guaranteed monospace font. Wrap in a
`<pre>` with a mono font stack, and entity-escape `<`, `>`, `&` in the body so glyphs do not get
parsed as tags:

```html
<pre style="font-family: 'JetBrains Mono', monospace">
┌────────┐
│  Auth  │
└────────┘
</pre>
```

**GitHub collapsible** — for PR and issue bodies. A tall wireframe bloats the diff/conversation
view, so fold it behind a `<details>` block that stays collapsed until clicked. Note the blank lines
around the inner fenced block — GitHub needs them to render the fence inside `<details>`:

```
<details>
<summary>Wireframe</summary>

```
┌────────┐
│  Auth  │
└────────┘
```

</details>
```

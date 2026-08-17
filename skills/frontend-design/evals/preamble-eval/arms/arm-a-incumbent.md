
# Review Rules (Arm A — incumbent house style)

> The 8 rules below are pulled verbatim from `skills/frontend-design/references/anti-slop-canon.md`.
> No framing preamble — this is the incumbent corpus's standard imperative-table skill-reference style.

| # | Pattern | Signature (what to grep your eye for) | Do instead |
|---|---|---|---|
| 1 | Purple gradient hero | `#7c3aed → #2563eb` on white | Real color theory — complementary/analogous/split-complementary, chosen for the context |
| 3 | Rounded everything | `border-radius: 24px+` on every element | Vary radius by role; let some elements have weight and edges |
| 4 | Default "modern" sans/serif/mono rotation | Sans: Inter, DM Sans, Space Grotesk, Sora, Syne, Archivo, Figtree (cycling to the next of these is still slop, not an escape) | Distinctive pairing, never reused across briefs |
| 17 | Hover boop / translateY lift on every interactive element | `transform: translateY(-2px)` + shadow bump on hover, applied uniformly | Reserve motion feedback for elements that need emphasis; vary or omit it per element role |
| 20 | Inner-glow badge | `box-shadow: inset 0 0 Npx` glow ring on pill/badge components | Flat badge with real color contrast; save glow for a genuine focus/active state |
| 31 | Fill-plus-outline button pair | primary filled + secondary outline button, same shape, side by side | Differentiate by more than fill — weight, size, or drop the pair entirely |

Mechanics Deslop (implementation-mechanics rules — CSS/utility/component-level correctness):

| # | Rule | Why |
|---|---|---|
| 6 | MUST use `tabular-nums` on any UI showing changing numeric values | Proportional digit widths jitter the layout as a counter/price/stat updates; `font-variant-numeric: tabular-nums` fixes digit width |
| 12 | NEVER animate `width`/`height`/`top`/`left`/`margin` | These trigger layout on every frame; animate `transform`/`opacity` only — the two properties the compositor can run off the main thread |

Hard rule (Font alternatives by context section): stop cycling Google Fonts. Rotating from Inter
to Sora to Space Grotesk is still the same slop shelf — the fix is a genuinely different source,
not the next free default.

---

**Task**: review each fixture under `fixtures/review/` against the rules above and report every
violation found, citing the specific rule number.

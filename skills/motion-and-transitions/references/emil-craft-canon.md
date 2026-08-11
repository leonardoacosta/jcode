
# Emil Craft Canon — Decision Gate, Review Bar, Remedial Hierarchy

> Adapted from Emil Kowalski's `find-animation-opportunities`, `review-animations`, and
> `improve-animations` skills ([emilkowalski/skills](https://github.com/emilkowalski/skills),
> MIT, fetched 2026-07-16). This reference answers **whether** to animate and **how good** an
> existing animation is — `SKILL.md`'s motion language answers **how**. Curve/duration values
> below are the BINDING output of `evals/canon-eval/REPORT.md` (near-identical numeric ties keep
> the corpus's out-quint/spring/ease-in-out; genuinely new content — the per-element duration table, the
> never-`ease-in`/never-`scale(0)` rules, the ten-standard review bar, `ease-drawer` — is added
> here because the corpus had no coverage of it before this reference, not because an incumbent value lost).

## The decision gate — should this animate at all?

Every candidate animation — new or existing — passes all four questions IN ORDER. Fail any one:
reject or delete. This is a **reject-by-default** posture: expect to reject most candidates.
A short list of high-conviction motion beats a long wishlist.

### 1. Frequency — how often does a user see this?

| Frequency | Verdict |
| --- | --- |
| 100+ times/day (keyboard shortcuts, command-palette toggle, core nav) | **Reject. No animation. Ever.** Not a judgment call. |
| Tens of times/day (hover states, list nav, frequent toggles) | Reject, or near-imperceptible motion only (fast, subtle) |
| Occasional (modals, drawers, toasts, settings) | Eligible — standard animation |
| Rare / first-time (onboarding, empty states, success, celebration) | Eligible — this is where the delight budget lives |

Raycast shipping zero open/close animation on its command palette is the textbook-correct call
— hundreds of daily uses make ANY animation feel slow and disconnected, however tasteful.

### 2. Purpose — why does this animate?

Must be one of these, named explicitly, or the candidate is rejected: **Feedback** (press scale,
hold-to-confirm fill) · **Spatial consistency** (shows where something came from/went) · **State
indication** (a state change becomes legible) · **Preventing a jarring change** (content that
would otherwise teleport) · **Explanation** (marketing/onboarding demonstrations only) ·
**Delight** (Rare/first-time tier only). "It looks cool" is not on this list.

### 3. Speed — per-element duration budget

UI animations stay under 300ms. Per-element ceilings (this table is new content — the corpus previously
had only a blanket 150-500ms range; see `SKILL.md`'s motion-language section for the pointer):

| Element | Duration |
| --- | --- |
| Press feedback | 100-160ms |
| Tooltips, small popovers | 125-200ms |
| Dropdowns, selects | 150-250ms |
| Modals, drawers | 200-500ms |
| Marketing / explanatory | Can be longer |

If a moment only "works" as a slow, showy animation, it fails this question.

### 4. Function — does motion help or hinder?

Decoration on functional, information-dense UI hinders. Data the user is trying to *read* or
*act on* should not move for style — a live financial figure or a graph the user is analyzing
gets no decorative motion, even if the same effect would be fine on a marketing page.

### Example rejections (the discipline in practice)

- Command-palette open/close — **rejected: keyboard-initiated, 100+/day. Never animate.**
- Animated line-drawing on an analytics graph the user is reading — **rejected: functional
  data; decoration hinders comprehension.**
- A settings toggle animating on every keystroke-driven filter update — **rejected: tens/day,
  no stated purpose beyond "looks cool."**

## The ten review standards

Every animation in a diff is measured against these. A violation is a finding; cite `file:line`.

1. **Justified motion.** Names one of the Purpose values above. "Looks cool" on a frequently-seen
   element is a block.
2. **Frequency-appropriate.** Matches the frequency table — keyboard/100+-day gets none.
3. **Responsive easing.** Entrances/exits use `ease-out` or a strong custom curve (incumbent: out-quint
   `(0.22,1,0.36,1)`). **`ease-in` on UI is always a block** — it delays the exact moment the
   user is watching.
4. **Sub-300ms UI.** Anything slower on a UI element needs a stated reason or it's a finding.
5. **Origin & physical correctness.** Popovers/dropdowns/tooltips scale from their trigger
   (`transform-origin`), not center — modals are exempt (they stay centered). **Never
   `scale(0)`** — start from `scale(0.9-0.97)` + `opacity: 0`; nothing appears from nothing.
6. **Interruptibility.** Rapidly-triggered or gesture-driven motion (toasts, toggles, drags)
   uses CSS transitions or springs that retarget from the current state — never `@keyframes`,
   which restart from zero.
7. **GPU-only properties.** Animate `transform`/`opacity` only. `width`/`height`/`margin`/
   `padding`/`top`/`left` (and Framer Motion `x`/`y`/`scale` shorthands under load — they run
   on the main thread, not hardware-accelerated) are performance findings.
8. **Accessibility.** `prefers-reduced-motion` honored (gentler, not zero — keep opacity/color,
   drop movement); hover motion gated behind `@media (hover: hover) and (pointer: fine)` so
   touch taps don't fire false hovers.
9. **Asymmetric enter/exit.** Deliberate actions (press, hold, destructive confirm) animate
   slower; system responses snap. Symmetric press/release timing is a finding.
10. **Cohesion.** Motion matches the component's and product's personality — playful can be
    bouncier, a dashboard stays crisp. A jarring crossfade that a subtle blur would bridge is a
    finding. When unsure whether motion feels right: the strongest move is often to delete it.

### Block criteria (flag on sight)

`transition: all` · `scale(0)` or pure-fade entrances with no initial transform · `ease-in` on
any UI interaction · animation on a keyboard shortcut / command-palette / 100+/day action · UI
duration >300ms with no stated reason · `transform-origin: center` on a trigger-anchored
popover/dropdown/tooltip · `@keyframes` on toasts/toggles/anything triggered rapidly · animating
layout properties · a CSS variable set on a parent to drive a child transform (recalcs styles for
every child — set `transform` directly on the element instead) · missing reduced-motion handling
on movement · ungated `:hover` motion · symmetric enter/exit on press-and-release · an
everything-at-once entrance where a 30-80ms stagger belongs.

### Review output format

| Before | After | Why |
| --- | --- | --- |
| `transition: all 300ms` | `transition: transform 200ms cubic-bezier(0.22, 1, 0.36, 1)` | Name exact properties — `all` animates unintended properties off-GPU |
| `transform: scale(0)` | `transform: scale(0.95); opacity: 0` | Nothing appears from nothing |
| `ease-in` on a dropdown entrance | out-quint `(0.22, 1, 0.36, 1)` | `ease-in` delays the moment the user watches most |
| `transform-origin: center` on a popover | `var(--radix-popover-content-transform-origin)` (or `data-origin`, see `SKILL.md` § Origin awareness) | Popovers scale from their trigger, not center — modals are exempt |

Close with an explicit **Block** (any feel-breaking regression, keyboard/high-frequency
animation, `scale(0)`/`ease-in` on UI, an easy-fix non-GPU animation) or **Approve** (no
feel-breaking regressions, nothing that should be deleted, durations/easing in bounds,
interruptibility handled, reduced-motion respected) verdict.

## The 9-step remedial hierarchy

When proposing a fix, prefer earlier moves over later ones — delete beats reduce beats polish:

1. **Delete the animation** (high-frequency / no purpose / keyboard-triggered).
2. **Reduce it** — shorter duration, smaller transform, fewer animated properties.
3. **Fix the easing** — swap `ease-in` for `ease-out`/out-quint; use a strong cubic-bezier.
4. **Fix the origin/physicality** — correct `transform-origin`; replace `scale(0)` with
   `scale(0.9-0.97)` + opacity.
5. **Make it interruptible** — `@keyframes` to transitions, or a spring for gesture-driven
   motion.
6. **Move it to the GPU** — layout props to `transform`/`opacity`; shorthand to a full
   `transform` string; WAAPI for programmatic CSS.
7. **Asymmetric timing** — slow the deliberate phase, snap the response.
8. **Polish** — `filter: blur(2px)` to mask a crossfade, 30-80ms stagger for groups,
   `@starting-style` for entry-without-JS, a spring for "alive" elements.
9. **Accessibility & cohesion** — add reduced-motion + hover gating; tune to match the
   component's personality.

## Performance rules

- **`transition: all` is always a finding** — it animates unintended properties off-GPU.
- **Framer Motion `x`/`y`/`scale` shorthands are NOT hardware-accelerated** — they run on the
  main thread via `requestAnimationFrame` and drop frames under load. Use the full transform
  string: `animate={{ transform: "translateX(100px)" }}`, not `animate={{ x: 100 }}`.
- **Don't drive child transforms via a CSS variable on the parent** — `setProperty('--x', …)`
  forces a style recalc on every child. Set `transform` directly on the animating element.
- CSS/WAAPI beat rAF-based JS under load (off the main thread); reserve JS/springs for dynamic,
  interruptible, gesture-driven motion.
- Keep transition-time `filter: blur()` **under 20px** — heavy blur is expensive, especially in
  Safari.

## Gesture values

- **Velocity dismissal**: don't require crossing a distance threshold — compute velocity
  (`Math.abs(distance) / elapsedMs`) and dismiss when it exceeds **~0.11**. A flick should be
  enough on its own.
- **Multi-touch guard**: ignore extra touch points once a drag has started
  (`if (isDragging) return`) — prevents jumps from a second finger landing mid-gesture.
- **Rubber-banding**: dragging past a natural edge should move less the further past it the user
  drags (see `apple-fluid-interfaces.md` § Rubber-banding for the exact function) — friction
  over hard stops.
- **Pointer capture**: `setPointerCapture` once a drag starts, so tracking continues even when
  the pointer leaves the element's bounds.

## mifb interaction craft (two rows)

> Source: `github.com/jakubkrehel/make-interfaces-feel-better` `animations.md` lines 281-379
> (MIT), fetched 2026-07-16 — a third source alongside the Emil Kowalski skills and
> `evals/canon-eval/REPORT.md` cited at the top of this file. Checked against this file's own
> decision gate and review-bar sections first: scale-on-press and page-load-skip were not
> previously adjudicated anywhere above, so both land as genuine additions, not competing values
> for an existing row.

### Scale-on-press

Press feedback (this file's own **Feedback** purpose value, § the decision gate item 2) scales
the target to `scale(0.96)` on press — never go below `0.95`, since a smaller scale reads as the
element shrinking away rather than compressing under a finger. Use a CSS `transition`, not a
JS-driven spring: a transition retargets instantly from wherever the current scale sits, so a
press interrupted mid-gesture (finger lifts before the down-transition finishes) doesn't fight an
in-flight spring trying to reverse (this file's own **Interruptibility** review standard, § the
ten review standards item 6). Ship a `static` opt-out prop for contexts where the scale motion
would distract — dense tables, high-frequency toggles, and other elements this file's own
frequency table (§ the decision gate item 1) already pushes toward Reject or
near-imperceptible-only.

### Page-load animation skip

Set `initial={false}` on Framer Motion's `AnimatePresence` for elements that render in their
default/steady state on load — icon swaps (see `assets/icon-swap.css` for the CSS-only version of
the same swap pattern), toggles, tabs — so they don't play an unwanted entrance animation on
first paint. Caveat: any component that legitimately relies on `initial` for a staged first-time
entrance animation (onboarding reveals, empty-state intros) must NOT use `initial={false}` —
verify behavior on a full page refresh before applying it broadly, since a blanket
`initial={false}` across a codebase silently kills a real entrance animation along with the
unwanted ones.

## New tokens added by this reference

`--ease-drawer: cubic-bezier(0.32, 0.72, 0, 1)` — an iOS-like drawer curve, genuinely absent from
`SKILL.md`'s three-curve table (out-quint / spring overshoot / ease-in-out-closing cover
ambient / attention / closing; none is drawer-specific). Reach for it on sheet/drawer motion
where the existing three curves feel too generic. This is an ADDITION, not a competing value for
an existing token — see `evals/canon-eval/REPORT.md`'s verdict table for why out-quint itself did
not flip.

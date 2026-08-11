
# Motion Performance Triage

> Adapted from `ibelick/ui-skills` `fixing-motion-performance` skill (MIT license).

The nine CSS patterns in this skill are already compositor-safe (transform/opacity only, see
below). This reference is for diagnosing and fixing jank in **custom** motion code — scroll-driven
effects, imperative JS animation, and anything that didn't come from `assets/`.

For the compositor-only animation rule (never animate `width`/`height`/`top`/`left`/`margin`) and
`will-change` scoping discipline, see `frontend-design` skill's `anti-slop-canon.md` § Mechanics
Deslop rows #10–12 — those rows are the summary; this doc covers the deeper rendering-pipeline
mechanics and the specific scroll/rAF/blur failure patterns below, and does not restate them.

## Rendering-steps glossary

Every frame the browser paints, up to three pipeline stages can run, in this cost order
(cheapest to most expensive):

| Step | What it does | Cost | Triggered by |
|---|---|---|---|
| **Composite** | Combines already-painted layers (moves/fades pixels the GPU already has) | Cheapest — can run on its own thread, off the main thread | `transform`, `opacity` |
| **Paint** | Rasterizes pixels for a layer (fills in color, text, shadows, blur) | Moderate — repaints the affected layer's pixels every frame it changes | `background`, `box-shadow`, `color`, `filter`/`backdrop-filter` |
| **Layout** (a.k.a. reflow) | Recomputes geometry — size and position of every affected element and its neighbors | Most expensive — a layout change can cascade to siblings, parents, and the whole subtree | `width`, `height`, `top`/`left`, `margin`, `padding`, adding/removing DOM nodes |

An animation that only triggers composite is the goal: the main thread stays free, so
scrolling/input/JS keep running smoothly while the animation plays. An animation that triggers
layout every frame forces the browser to redo layout -> paint -> composite in sequence, 60+
times a second — this is what "jank" is, mechanically.

## Never-patterns

### 1. Scroll-driven animation via polling `scrollTop`/`scroll` events

Reading scroll position on every `scroll` event and writing an inline style forces a
layout/paint cycle synchronously on the main thread, competing with the browser's own scroll
handling — the classic cause of scroll-jank.

**Before** (polling + inline style mutation):

```javascript
window.addEventListener('scroll', () => {
  const progress = window.scrollY / (document.body.scrollHeight - window.innerHeight);
  el.style.transform = `translateY(${progress * 100}px)`; // main-thread write, every scroll event
});
```

**After** (CSS scroll-driven animation — no JS, no main-thread work):

```css
@keyframes reveal {
  from { transform: translateY(100px); opacity: 0; }
  to   { transform: translateY(0);     opacity: 1; }
}

.t-scroll-reveal {
  animation: reveal linear;
  animation-timeline: scroll(); /* driven by the compositor, not JS */
  animation-range: 0% 40%;
}
```

Where `animation-timeline: scroll()` isn't viable (older browsers, or a threshold-based
trigger rather than continuous scrubbing), use a throttled `IntersectionObserver` instead of a
`scroll` listener — it fires only on visibility-threshold crossings, off the scroll event
entirely:

```javascript
const io = new IntersectionObserver(
  ([entry]) => entry.target.classList.toggle('is-visible', entry.isIntersecting),
  { threshold: 0.4 }
);
io.observe(el); // el.is-visible toggles a transform/opacity transition in CSS — no polling
```

### 2. Unbounded `requestAnimationFrame` loops

A rAF loop with no exit condition keeps running (and keeps painting) even after its element is
gone, off-screen, or the interaction that started it has ended — a standing CPU/battery cost
that compounds with every uncancelled loop left running.

**Before** (no cleanup — loop outlives the component):

```javascript
function tick() {
  el.style.transform = `translateX(${Math.sin(Date.now() / 500) * 20}px)`;
  requestAnimationFrame(tick);
}
requestAnimationFrame(tick);
```

**After** (cancel on unmount/condition):

```javascript
let rafId;
function tick() {
  el.style.transform = `translateX(${Math.sin(Date.now() / 500) * 20}px)`;
  rafId = requestAnimationFrame(tick);
}
rafId = requestAnimationFrame(tick);

// on unmount / when the effect should stop:
cancelAnimationFrame(rafId);
```

In React, this means returning a cleanup function from the effect that scheduled the loop —
`useEffect(() => { rafId = requestAnimationFrame(tick); return () => cancelAnimationFrame(rafId); }, [])`.

### 3. Animating large `blur()`/`backdrop-filter` surfaces

Both are **paint**-stage operations (see glossary above), and their cost scales with the pixel
area being blurred. Animating blur radius on a large surface (a full-bleed hero backdrop, a
full-screen modal scrim) repaints that entire area every frame — expensive even though no
layout is involved.

**Before** (animating blur radius directly on a large element):

```css
.t-hero-backdrop {
  transition: backdrop-filter 300ms ease;
}
.t-hero-backdrop.is-focused {
  backdrop-filter: blur(20px); /* repaints the full backdrop area every frame of the transition */
}
```

**After** — prefer animating opacity on a pre-blurred layer (compositor-only) over animating
the blur radius itself:

```css
.t-hero-backdrop {
  backdrop-filter: blur(20px); /* set once, not animated */
  opacity: 0;
  transition: opacity 300ms ease; /* composite-only */
}
.t-hero-backdrop.is-focused {
  opacity: 1;
}
```

If the blur radius itself must change (not just fade in/out), keep the animated surface as
small as possible and prefer a shorter duration — there is no compositor-only equivalent for an
animated blur *radius*, only for its presence.

## FLIP technique (for layout-triggering changes that can't be avoided)

Some visual changes are inherently layout changes — resizing a card to its expanded state,
reordering a list. FLIP (First, Last, Invert, Play) makes these feel like compositor-cheap
animations by doing the expensive layout work **once**, then animating the *visual delta* with
transform (composite-only):

1. **First** — record the element's current position/size (`getBoundingClientRect()`).
2. **Last** — apply the layout change (the class/style that causes the resize/reorder), then
   immediately record the new position/size.
3. **Invert** — compute the delta between First and Last, and apply a `transform` that visually
   snaps the element back to its First position/size (so the user still sees the old layout).
4. **Play** — remove the invert transform with a transition; the browser only has to animate a
   `transform`, even though the underlying change was layout.

```javascript
const first = el.getBoundingClientRect();
el.classList.add('is-expanded'); // triggers the real layout change
const last = el.getBoundingClientRect();

const deltaX = first.left - last.left;
const deltaY = first.top - last.top;
const deltaW = first.width / last.width;
const deltaH = first.height / last.height;

el.style.transformOrigin = 'top left';
el.style.transform = `translate(${deltaX}px, ${deltaY}px) scale(${deltaW}, ${deltaH})`;
el.style.transition = 'transform 0s'; // invert with no transition

requestAnimationFrame(() => {
  el.style.transition = 'transform 300ms cubic-bezier(0.22, 1, 0.36, 1)'; // out-quint, see SKILL.md
  el.style.transform = ''; // play — animates back to the real (Last) layout
});
```

The layout recalculation still happens once, synchronously, at the `is-expanded` toggle — FLIP
doesn't eliminate that cost, it eliminates paying it **every frame** of the transition.

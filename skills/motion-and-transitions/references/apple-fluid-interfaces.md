
# Apple Fluid Interfaces — Spring Physics & Materials

> Adapted from Apple's WWDC design talks (*Designing Fluid Interfaces* 2018, *UI Typography*
> 2020) as distilled in emilkowalski/skills `apple-design/SKILL.md` (MIT, fetched 2026-07-16),
> translated for the web (CSS, Pointer Events, spring libraries). The corpus had **zero** coverage of
> spring physics, momentum, rubber-banding, or translucent materials before this reference — see
> `docs/recon/emilkowalski-skills.md` § Adapt 2. Use this reference for anything the user can
> **grab and interrupt**: drags, swipes, sheets, drawers. `SKILL.md`'s CSS snippets (predetermined,
> non-interruptible motion) and `emil-craft-canon.md`'s review bar cover everything else.

## The through-line

An interface feels alive when motion starts from the current on-screen value, inherits the
user's velocity, projects momentum forward, and can be grabbed and reversed at any instant.
Springs are the tool that makes this natural — they are inherently interruptible and
velocity-aware, unlike fixed-duration transitions/keyframes.

## Damping & response — the spring model

Apple replaces the physics triplet (mass/stiffness/damping) with two designer-friendly params:

- **Damping ratio** — controls overshoot. `1.0` = critically damped, no bounce, smooth settle.
  `<1.0` = overshoots and oscillates; lower = bouncier.
- **Response** — how quickly the value reaches the target, in seconds. Lower = snappier. This
  is NOT "duration" — a spring has no fixed duration; settle time emerges from the parameters.

**Defaults**: start most UI at damping `1.0` (critically damped) — graceful, non-distracting.
Add bounce (damping ~`0.8`) ONLY when the gesture itself carried momentum (a flick, a throw, a
drag release). Overshoot on a menu that just faded in feels wrong; overshoot on a card you
flicked feels right.

| Interaction | Damping | Response |
| --- | --- | --- |
| Move / reposition (e.g. picture-in-picture) | `1.0` | `0.4` |
| Rotation | `0.8` | `0.4` |
| Drawer / sheet | `0.8` | `0.3` |

Web mapping (Motion / Framer Motion): the `bounce` + `duration` spring API maps closely to
damping + response. Safe house style: `damping: 1.0` springs everywhere by default; reserve
bounce for momentum-driven, physical interactions.

```js
import { animate } from 'motion';

// Critically damped default (no overshoot)
animate(el, { y: 0 }, { type: 'spring', bounce: 0, duration: 0.4 });

// Momentum interaction — a little bounce, only because a flick preceded it
animate(el, { y: target }, { type: 'spring', bounce: 0.2, duration: 0.4 });
```

## Velocity handoff

When a gesture ends, the animation must continue at the finger's exact velocity — no visible
seam between dragging and animating. Some spring APIs want relative velocity, normalized by the
remaining distance to the target:

```
relativeVelocity = gestureVelocity / (targetValue - currentValue)
```

Example: element at `y=50`, target `y=150` (100px to go), finger moving 50px/s -> initial spring
velocity = `50 / 100 = 0.5`. Framer Motion / Motion take absolute px/s velocity directly (the
`velocity` option) — usually hand it the raw value.

## Momentum projection

Don't snap to the nearest boundary from the release point. Use velocity to project the resting
position — like scroll deceleration — then snap to the target nearest that projection. Apple's
exact function (from the *Designing Fluid Interfaces* sample code; the physics-textbook
`v^2/(2*decel)` is NOT what Apple ships):

```js
// decelerationRate ~ 0.998 for normal scroll feel; 0.99 for snappier
function project(initialVelocity /* px/s */, decelerationRate = 0.998) {
  return (initialVelocity / 1000) * decelerationRate / (1 - decelerationRate);
}

const projectedEndpoint = currentPosition + project(releaseVelocity);
const target = nearestSnapPoint(projectedEndpoint);      // choose target from the projection
animateSpringTo(target, { velocity: releaseVelocity });  // then hand off velocity (above)
```

This is the standard behavior in good bottom-sheets and carousels (Vaul, Embla).

## Rubber-banding — soft boundaries

At an edge, resist progressively instead of stopping hard. A hard stop reads as "frozen";
continuous resistance reads as "responsive, but there's nothing more here."

```js
// The further past the bound, the less the element follows
function rubberband(overshoot, dimension, constant = 0.55) {
  return (overshoot * dimension * constant) / (dimension + constant * Math.abs(overshoot));
}
```

## Presentation-value interruptibility

Every animation must be interruptible and redirectable at any moment — a user grabbing a moving
element mid-flight must be able to reverse it without waiting for the animation to finish.

- **Always animate from the presentation (current) value, never the target value.** On
  interrupt, read the element's live on-screen transform and start the new animation from
  there. Starting from the logical/target value causes a visible jump.
- Avoid CSS transitions and `@keyframes` for gesture-driven motion — they can't be smoothly
  grabbed and reversed mid-flight. Springs animate from the current value by default.
- **When a gesture reverses, blend velocity — don't hard-cut it.** Replacing one animation with
  another at a reversal creates a velocity discontinuity (a "brick wall"). Use a spring library
  that re-targets from the current velocity.
- **Decompose 2D motion into independent X and Y springs.** A single spring on a 2D distance
  desyncs when X and Y have different velocities.

## Materials & depth

Apple uses translucent materials as a floating functional layer that adds structure without
stealing focus. Approximate on the web with `backdrop-filter`.

- Build nav/toolbars/sheets as translucent layers (`backdrop-filter: blur()` + a semi-transparent
  background) with content scrolling underneath — not opaque bars.
- Material weight encodes hierarchy: darker/heavier materials separate structural regions
  (sidebars); lighter materials draw attention to interactive elements (buttons). **Never stack
  a light translucent surface on another** — legibility collapses.
- Bigger surfaces read as thicker: stronger blur + a deeper shadow than small chips.
- Dim to focus (a modal task pairs the surface with a dimming scrim), separate to keep flow (a
  parallel non-blocking panel uses translucency + offset, no scrim).
- Vibrancy keeps text legible over changing backgrounds — higher-contrast, slightly heavier
  weight, small letter-spacing bump over blurred/translucent surfaces; put color on a solid
  layer, not the translucent foreground.
- Scroll edge effects, not hard dividers — fade a small blur/gradient mask where floating chrome
  overlaps content, instead of a 1px border under a sticky header.
- Materialize, don't just fade — animate blur radius and scale together on enter/exit for glass
  surfaces, so the surface reads as arriving material, not a plain opacity fade.

```css
.toolbar {
  background: rgba(255, 255, 255, 0.6);
  backdrop-filter: blur(20px) saturate(180%);
  border-top: 1px solid rgba(255, 255, 255, 0.4); /* bright top edge = light catching the material */
}
```

## Typography — size-specific tracking

- **Tracking (letter-spacing) is size-specific — never one value for all sizes.** Large display
  text wants negative tracking (`-0.02em`); small/body text wants near-zero or slightly positive
  tracking for legibility.
- **Leading (line-height) tracks size inversely.** Tight on large headings, looser on body copy.
- Build hierarchy from weight + size + leading as a set, not size alone.
- Respect the user's text-size setting (Dynamic Type / OS font scaling) — spacing in `rem`/`em`,
  not fixed px.

```css
.display {
  font-size: clamp(2rem, 5vw, 4rem);
  line-height: 1.05;        /* tight leading for large text */
  letter-spacing: -0.02em;  /* negative tracking as it grows */
}
```

## Three reduced-* signals

Reduced motion means fewer and gentler animations, **not zero** — respond to three independent
signals:

- **`prefers-reduced-motion: reduce`** — replace slides/springs/parallax with short opacity
  cross-fades or static transitions. Drop elastic/overshoot. Keep opacity/color changes that aid
  comprehension.
- **`prefers-reduced-transparency: reduce`** — raise translucent-surface background opacity,
  drop the blur (frostier/solid).
- **`prefers-contrast: more`** — near-solid backgrounds with a defined, contrasting border.

```css
@media (prefers-reduced-motion: reduce) {
  .sheet { transition: opacity 200ms ease; transform: none !important; }
}
@media (prefers-reduced-transparency: reduce) {
  .toolbar { background: white; backdrop-filter: none; }
}
```

## The eight design foundations

Names to reason with, from *Principles of Great Design* (WWDC 2026): **Purpose** (decide what
NOT to build) · **Agency** (keep people in control; forgiveness over confirmation dialogs) ·
**Responsibility** (act in the user's interest — privacy, safety, anticipate AI misuse) ·
**Familiarity** (build on known metaphors, stay consistent) · **Flexibility** (adapt to context/
device/ability, let people personalize) · **Simplicity — not minimalism** (strip the
unnecessary so purpose shines; concise + clear, not just sparse) · **Craft** (uncompromising
detail — nothing is random, every value is a deliberate, defensible choice) · **Delight** (the
result of the other seven, not confetti tacked on).

## Quick reference

| Need | Technique | Concrete value |
| --- | --- | --- |
| Default UI spring | Critically damped, no overshoot | damping `1.0`, response `0.3-0.4` |
| Momentum / flick spring | Under-damped, slight bounce | damping `~0.8`, response `0.3-0.4` |
| Gesture -> spring velocity | Hand off release velocity | `gestureVelocity / (target - current)` if normalized |
| Flick landing point | Project momentum | `current + (v/1000)*d/(1-d)`, `d ~ 0.998` |
| Interrupt cleanly | Start from presentation (live) value | read the on-screen transform |
| Boundary | Rubber-band, don't hard-stop | progressive resistance |
| Translucent chrome | `backdrop-filter` layer | content scrolls under |
| Type tracking | Size-specific, never fixed | tighten large text (`-0.02em`), body near `0` |
| Reduced motion | Cross-fade, not slide/spring | `@media (prefers-reduced-motion)` |

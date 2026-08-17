
# Text Effects — scrambleText / splitText (v4.4.0+)

> Source: https://animejs.com/documentation/text/scrambletext (2026-07-12 recon:
> `docs/recon/animejs-text-scrambletext.md`)

anime.js v4.4.0+ ships a first-party **text module** — `scrambleText()` (character-by-character
scramble-and-reveal) and `splitText()` (line/word/char splitting). Together they are an
MIT-licensed equivalent of GSAP's paid `SplitText` + `ScrambleTextPlugin` (GSAP Club).

## scrambleText

`scrambleText()` does not run standalone — it composes with `animate()`'s normal timeline
machinery, driven through the `innerHTML` property:

```javascript
import { animate, scrambleText } from 'animejs'; // or 'animejs/text'

animate('p', {
  innerHTML: scrambleText(), // NOT textContent
  loop: true,
  loopDelay: 1000,
});
```

### Footgun: `innerHTML`, not `textContent`

`scrambleText()` MUST be assigned to the `innerHTML` animation property. Targeting `textContent`
instead silently no-ops — no error, no console warning, the element just never scrambles. This
is the single most common mistake when adopting this API; check the property name first if a
scramble animation appears to do nothing.

### Parameter table

| Param | Type | Purpose |
|---|---|---|
| `text` | `string` | Override the text to scramble through (defaults to the target's existing text) |
| `chars` | `string` | Character pool to scramble through (defaults to a mixed alphanumeric set) |
| `ease` | easing string | Easing applied to the scramble-to-reveal transition |
| `cursor` | `boolean` \| `string` | Optional cursor character shown during the scramble |
| `revealRate` | `number` | How quickly characters lock in to their final value |
| `revealDelay` | `number` | Delay before reveal begins after the scramble starts |
| `settleRate` | `number` | How quickly a scrambling character settles once revealed |
| `settleDuration` | `number` | Duration of the per-character settle transition |
| `delay` | `number` | Delay before the whole effect starts |
| `duration` | `number` | Total effect duration |
| `perturbation` | `number` | Amount of randomness injected into the scramble |
| `from` | `'start'` \| `'end'` \| `'center'` \| `number` | Where the reveal originates from across the string |
| `reversed` | `boolean` | Reverses the reveal direction |
| `seed` | `number` | Fixes the RNG seed — see § Deterministic replay below |
| `onChange` | `(self) => void` | Callback fired on every scramble tick |

### Deterministic replay via `seed`

Pass a fixed `seed` to get byte-identical scramble sequences across runs:

```javascript
animate('h1', {
  innerHTML: scrambleText({ seed: 1 }),
  duration: 2000,
});
```

This matters for demos, evals, or any runtime-evidence capture where the same input must
produce the same visual output on repeat — an unseeded scramble is different every render,
which breaks screenshot-diff or eval-based verification.

## splitText

```javascript
import { createTimeline, stagger, splitText } from 'animejs';

const { words, chars } = splitText('p', {
  words: { wrap: 'clip' },
  chars: true,
});

createTimeline({ loop: true, defaults: { ease: 'inOut(3)', duration: 650 } })
  .add(words, { y: [$el => +$el.dataset.line % 2 ? '100%' : '-100%', '0%'] }, stagger(125))
  .init();
```

`splitText()` returns handles (`words`, `chars`, `lines` depending on config) that are valid
`animate`/timeline targets — same as any DOM node array.

### `accessible` setting

`splitText`'s `accessible` option controls how the split DOM is exposed to screen readers
(the underlying text gets ARIA-labeled so assistive tech reads the original string, not the
per-character/per-word wrapper spans). Keep `accessible: true` for any production text-reveal —
splitting text into individual spans without it is a common accessibility regression.

## v3 vs v4

v4 is a full API rewrite — named ESM exports (`animate`, `createTimeline`, `stagger`,
`scrambleText`, `splitText`) replace v3's global `anime({...})` function call. Do not port v3
snippets directly; `wayfinder/references/libraries.md`'s pinned v3.2.2 CDN snippet uses
the old API and has no text module at all (added in 4.4.0).

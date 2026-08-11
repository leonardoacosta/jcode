
# Dithering, Halftone & ASCII Art Recipes

> Reference for the `frontend-design` skill — retro/CRT/1-bit aesthetics: Bayer matrices,
> Floyd-Steinberg error diffusion, ASCII conversion, and GLSL dither shaders.
>
> Consolidated from the `dithering` skill (score-and-remediate-skill-quality-floor, 2.5). That
> skill now routes here for its deep content — this file is the canonical home.

Dithering converts continuous-tone images into patterns of discrete colors (often pure black
and white) using spatial arrangement to preserve perceived brightness. Combined with ASCII
rendering or halftone grids, it produces the retro/CRT/1-bit aesthetic behind brutalist UI,
zine layouts, and decorative cards.

## When to reach for this

Retro, CRT, terminal, or 1-bit aesthetic; brutalist or zine-style layouts; monochrome hero
imagery that needs texture instead of flat color; ASCII art cards, text-portraits, quote
cards; decorative cards with image content matching a mono palette; replicating effects from
Efecto, Photoshop's Bitmap mode, or vintage Mac OS.

## Approach-selection table

The choice depends on **what you need to dither** and **how interactive it must be**. Start
at the top of the table; only move down if the previous approach can't meet the requirement.

| Approach | Best for | Cost | Motion? | Difficulty |
|---|---|---|---|---|
| **`react-ascii-ui`** drop-in | ASCII cards, quote cards, static image -> ASCII conversion | Add 1 package | Static | Low |
| **React Bits `Dither`** component | Dithered hero backgrounds, animated noise/gradient fields | Copy-paste | Animated | Low |
| **Canvas 2D + Bayer matrix** | Static image dithering baked at build or runtime | Zero deps | Static | Medium |
| **React Three Fiber + GLSL shader** | Full-motion dither over video, 3D scenes, scroll-driven effects | R3F + postprocessing | Real-time | High |

### Skip-to-shader / skip-to-canvas signal table

Some briefs signal texture-fidelity intent strongly enough to skip past the library tier
directly to Canvas 2D (Approach 3) or the shader (Approach 4):

| Signal in the brief | Skip to | Why |
|---|---|---|
| "brutalist", "zine", "handcrafted", "artisanal" | Approach 3 | The user is asking for a *look*, not a feature — library defaults won't satisfy |
| Specific algorithm named (Floyd-Steinberg, Atkinson, Jarvis-Judice-Ninke) | Approach 3 | They know what they want and need control over the error-diffusion weights |
| "CRT", "scanlines", "bloom", "chromatic aberration" | Approach 4 | CRT post-processing needs a shader pipeline; canvas can't do bloom |
| "video", "webcam", "60fps", "real-time", "interactive" | Approach 4 | Motion content must run in a shader — Canvas 2D per-frame is too slow |
| MacPaint, HyperCard, classic Mac, Amiga, C64 reference | Approach 3 | Period-correct output needs tunable parameters (palette, dither matrix size) |

When none of these signals are present, Approach 1 (`react-ascii-ui`) remains the default.

## Approach 1: `react-ascii-ui` (fastest drop-in)

A component library with 50+ ASCII-styled primitives and a built-in `AsciiArtGenerator` that
converts images to ASCII with selectable dithering algorithms.

```bash
pnpm add react-ascii-ui
```

Use for: ASCII cards (portrait rendered in characters), static image conversion, retro form
chrome, ASCII buttons/tables for a zine look.

```tsx
"use client";
import { AsciiCard, AsciiArtGenerator } from "react-ascii-ui";

export function QuoteCard({ imageSrc, title, body }: Props) {
  return (
    <AsciiCard className="w-[360px]">
      <AsciiArtGenerator
        src={imageSrc}
        charSet="detailed"   // "detailed" | "simple" | "blocks" | "dots" | "classic"
        dithering="floyd-steinberg"
        width={64}
      />
      <h3 className="mt-4 text-lg font-semibold">{title}</h3>
      <p className="mt-2 text-sm text-neutral-600">{body}</p>
    </AsciiCard>
  );
}
```

Character-set presets: `detailed` uses a long density ramp (`@#8&o:*. `) for photos; `blocks`
uses Unicode blocks (`█▓▒░`) for chunky graphics; `dots` uses Braille patterns for a stippled
look.

## Approach 2: React Bits `Dither` background

A copy-paste `Dither` component from React Bits' backgrounds collection — a WebGL-rendered
animated noise field for hero backgrounds and section dividers.

```bash
# React Bits uses a jsrepo-style CLI — fetch into your components dir
pnpm dlx jsrepo add https://reactbits.dev/default/Backgrounds/Dither
```

Or copy the source from https://www.reactbits.dev/backgrounds/dither directly into
`components/ui/dither.tsx`. It's a React Three Fiber canvas with a fragment shader — needs
`three`, `@react-three/fiber`, and `@react-three/postprocessing`.

Use for: animated dithered hero backgrounds, section breaks, "hold my place while I think"
states, interstitials.

## Approach 3: Canvas 2D + Bayer matrix (zero deps)

For static image dithering with no dependencies, Bayer ordered dithering is the simplest
algorithm — produces the cross-hatched pattern typical of old Mac OS screenshots.

```tsx
"use client";
import { useEffect, useRef } from "react";

const BAYER_8 = [
  [ 0,32, 8,40, 2,34,10,42],
  [48,16,56,24,50,18,58,26],
  [12,44, 4,36,14,46, 6,38],
  [60,28,52,20,62,30,54,22],
  [ 3,35,11,43, 1,33, 9,41],
  [51,19,59,27,49,17,57,25],
  [15,47, 7,39,13,45, 5,37],
  [63,31,55,23,61,29,53,21],
];

export function DitheredImage({ src }: { src: string }) {
  const canvasRef = useRef<HTMLCanvasElement>(null);

  useEffect(() => {
    const canvas = canvasRef.current;
    if (!canvas) return;
    const ctx = canvas.getContext("2d");
    if (!ctx) return;

    // Cancel-on-unmount guard: if `src` changes or the component
    // unmounts mid-load, the onload handler becomes a no-op.
    let cancelled = false;

    const img = new Image();
    img.crossOrigin = "anonymous";
    img.onload = () => {
      if (cancelled) return;
      canvas.width = img.width;
      canvas.height = img.height;
      ctx.drawImage(img, 0, 0);
      const { data, width, height } = ctx.getImageData(0, 0, img.width, img.height);

      for (let y = 0; y < height; y++) {
        for (let x = 0; x < width; x++) {
          const i = (y * width + x) * 4;
          // Luminance per ITU-R BT.601
          const lum = 0.299 * data[i]! + 0.587 * data[i + 1]! + 0.114 * data[i + 2]!;
          const threshold = (BAYER_8[y % 8]![x % 8]! / 64) * 255;
          const v = lum > threshold ? 255 : 0;
          data[i] = data[i + 1] = data[i + 2] = v;
        }
      }
      ctx.putImageData(new ImageData(data, width, height), 0, 0);
    };
    img.src = src;

    return () => {
      cancelled = true;
    };
  }, [src]);

  return <canvas ref={canvasRef} className="w-full" />;
}
```

Use for: blog post hero images, decorative portraits, static content that benefits from being
baked once and cached.

### Floyd-Steinberg error diffusion (higher quality, more complex)

For higher quality at the cost of complexity, swap Bayer for Floyd-Steinberg error diffusion —
distribute quantization error to neighbor pixels as you scan, using these weights:

```
        X   7/16
3/16  5/16  1/16
```

(`X` is the current pixel; the error from quantizing it spreads right, and to the three
pixels on the row below, in those proportions.) The result is the classic
photograph-in-newsprint look.

## Approach 4: React Three Fiber + GLSL shader (full-motion)

For dithering video, 3D scenes, or any content that needs to run at 60 FPS, do the work in a
fragment shader. This is the Efecto approach (https://efecto.app).

```bash
pnpm add three @react-three/fiber @react-three/postprocessing
```

Technique summary (see the Codrops Efecto article for full GLSL):

- Render scene to an offscreen texture.
- In a post-process pass, sample the texture per fragment.
- Compute luminance: `0.299*R + 0.587*G + 0.114*B`.
- For Bayer dither: compare luminance against a Bayer matrix sampled by `gl_FragCoord % 8`.
- For ASCII: divide screen into cells, sample brightness at cell center, pick glyph from a
  density ramp drawn procedurally on a 5x7 pixel grid using GLSL math.
- Optionally add CRT effects: scanlines, chromatic aberration, bloom, screen curvature.

Use for: full-motion backgrounds behind scroll sections, dithered video players, decorative 3D
hero scenes. Budget carefully — shader post-processing is the heaviest option.

## Gotchas

- **Image CORS** — Canvas 2D and shader approaches both need `crossOrigin="anonymous"` on the
  source image and CORS headers on the server, otherwise `getImageData`/texture uploads throw.
  For Next.js `<Image>`, prefer hosting assets under `/public` or a same-origin CDN.
- **Device pixel ratio** — dither patterns are resolution-dependent. On retina displays a 1:1
  canvas looks twice as dense. Decide whether to match DPR (crisper but slower) or clamp to 1
  (retro-chunky but faster).
- **Mono vs palette quantizer** — every recipe above assumes a 1-bit or 2-bit palette. For
  4-color or palette-indexed dithering you need a quantizer (e.g. the `image-q` npm package)
  before running error diffusion.
- **ASCII readability** — match glyph density to the available width. Too few characters and
  you lose detail; too many and the card becomes illegible. 64 chars wide is a good starting
  point for card-sized content.
- **Animation cost** — dithering a static image once is cheap. Re-running the CPU dither per
  frame is not. If the content moves, use a shader, not Canvas 2D.
- **SSR** — all approaches that touch `canvas`, `WebGL`, or `Image` must be client-only. Mark
  wrapper components `"use client"` in App Router.

## Related skills

- **`references/aceternity/`** — if the dithering is decoration for an otherwise animated page,
  Aceternity handles the motion, dithering handles the texture.
- **`awesome-design-md`** (`references/brand-design-catalogue.md`) — when cloning a specific
  brand aesthetic that happens to use dither (e.g. some terminal-native brands).
- **`vercel-react-best-practices`** — before shipping, audit for waterfall-free loading,
  module-hoisted constants, and proper unmount cleanup (dither components touch imperative
  canvas/WebGL APIs and `useEffect`).

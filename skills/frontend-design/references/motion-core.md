
# Motion Core — React Animation Component Reference

> Source: [motion-core/motion-core](https://github.com/Motion-Core/motion-core) | MIT License
> Local recon: `~/.claude/recon/motion-core.md`
> Original: Svelte 5 + Threlte + GSAP | **This reference: React porting patterns**

36 production-ready animation components. Shaders copy verbatim. GSAP works natively in React.
Three.js scenes use React Three Fiber (R3F) instead of Threlte.

## Porting Stack

| Svelte (Original) | React Equivalent | Notes |
|--------------------|------------------|-------|
| `<Canvas>` (Threlte) | `<Canvas>` (R3F) | `@react-three/fiber` |
| `<T.Mesh>`, `<T.PlaneGeometry>` | `<mesh>`, `<planeGeometry>` | Lowercase JSX intrinsics |
| `useTask((delta) => {})` | `useFrame((state, delta) => {})` | R3F frame loop |
| `useTexture()` (Threlte extras) | `useTexture()` (`@react-three/drei`) | Same API surface |
| `$props()` destructuring | React `props` / destructuring | Mechanical |
| `$state()` | `useState` or `useRef` | `useRef` for animation values |
| `$effect()` / `onMount` | `useEffect` | Cleanup pattern identical |
| `bind:this` | `useRef<HTMLElement>()` | DOM refs |
| Svelte `{#snippet}` | React `children` or render props | JSX children |
| `class:` directive | `className={cn(...)}` | clsx + tailwind-merge |
| GSAP (same) | GSAP (same) | Use `useRef` + `useGSAP` hook |
| GLSL shaders (same) | GLSL shaders (same) | Copy verbatim |
| `use:portal` action | React portal (`createPortal`) | `react-dom` |

### GSAP in React Pattern

```tsx
import { useRef } from "react";
import { useGSAP } from "@gsap/react";
import gsap from "gsap";

function Component() {
  const container = useRef<HTMLDivElement>(null);

  useGSAP(() => {
    // All GSAP animations scoped to container
    gsap.to(".target", { x: 100, duration: 1 });
    // Cleanup automatic via useGSAP
  }, { scope: container });

  return <div ref={container}>...</div>;
}
```

### R3F Scene Pattern

```tsx
import { Canvas, useFrame } from "@react-three/fiber";
import { useTexture } from "@react-three/drei";
import { useRef } from "react";
import * as THREE from "three";

function Scene({ src }: { src: string }) {
  const meshRef = useRef<THREE.Mesh>(null);
  const texture = useTexture(src);
  const uniforms = useRef({
    uTime: { value: 0 },
    uTexture: { value: texture },
  });

  useFrame((_, delta) => {
    uniforms.current.uTime.value += delta;
  });

  return (
    <mesh ref={meshRef}>
      <planeGeometry args={[2, 2]} />
      <shaderMaterial
        uniforms={uniforms.current}
        vertexShader={vertexShader}
        fragmentShader={fragmentShader}
      />
    </mesh>
  );
}

export function Effect({ src, className }: Props) {
  return (
    <div className={cn("relative", className)}>
      <Canvas dpr={[1, 2]} gl={{ toneMapping: THREE.NoToneMapping }}>
        <Scene src={src} />
      </Canvas>
    </div>
  );
}
```

---

## GSAP Plugins Required

| Plugin | Components | npm |
|--------|-----------|-----|
| `SplitText` | SplitHover, SplitReveal, TextScramble, FloatingMenu | GSAP Club (paid) |
| `ScrollTrigger` | SplitReveal, Marquee, CardStack | `gsap/ScrollTrigger` |
| `Flip` | FlipGrid, VideoPlayer | `gsap/Flip` |
| `CustomEase` | SplitHover, SplitReveal, FloatingMenu | `gsap/CustomEase` |
| `MorphSVGPlugin` | VideoPlayer | GSAP Club (paid) |

**Free alternative for the SplitText row:** anime.js `scrambleText`/`splitText` (MIT, >= 4.4.0)
cover TextScramble and the SplitText-derived reveals with no GSAP Club license — see § Free
Path (anime.js) below.

**Custom ease used throughout:** `"motion-core-ease"` = `cubic-bezier(0.625, 0.05, 0, 1)`

Register once at app root:
```tsx
gsap.registerPlugin(ScrollTrigger, Flip, CustomEase, SplitText);
CustomEase.create("motion-core-ease", "M0,0 C0.625,0.05 0,1 1,1");
```

---

## Category 1: GSAP Text & DOM (16 components)

### SplitHover
**Effect:** Characters slide up on hover, clones slide in from below.
**React approach:** `useGSAP` + `SplitText.create()` on ref. Duplicate spans for clone layer.
**Props:** `children`, `className`, `hoverTarget?: HTMLElement`
**Key GSAP:** `gsap.timeline()`, stagger 0.02s, `yPercent: -100` exit / `yPercent: 0` enter

### SplitReveal
**Effect:** Text reveals on mount or scroll — lines, words, or characters slide up.
**React approach:** `useGSAP` + `SplitText` + optional `ScrollTrigger`.
**Props:**
```tsx
interface SplitRevealProps {
  children: React.ReactNode;
  className?: string;
  mode?: "lines" | "words" | "chars";
  config?: { duration?: number; stagger?: number };
  delay?: number;
  triggerOnScroll?: boolean;
  scrollElement?: string | HTMLElement;
  as?: keyof JSX.IntrinsicElements;
}
```
**Defaults:** lines=0.8s/0.08s stagger, words=0.6s/0.06s, chars=0.4s/0.008s
**Key GSAP:** `gsap.to()` from `yPercent: 110` → `0`, ScrollTrigger `start: "top 85%"`

### ImageTrail
**Effect:** Images spawn along mouse path and fade out.
**React approach:** `useRef` for image pool (max 24), `onPointerMove` handler, RAF loop.
**Props:**
```tsx
interface ImageTrailProps {
  images: string[];
  className?: string;
  imageLifespan?: number;     // 600ms
  mouseThreshold?: number;    // 40px movement to spawn
  minImageSize?: number;      // 260px
  maxImageSize?: number;      // 340px
  maxRotationFactor?: number; // 3x speed multiplier
}
```
**Key GSAP:** `gsap.to()` for scale/opacity fade-in/out. Pool-based recycling prevents GC.

### TextLoop
**Effect:** Cycling text with blur transitions.
**React approach:** `useState` for active index, `useGSAP` for entry/exit animations, `setInterval`.
**Props:** `texts: string[]`, `interval?: number` (2000ms), `className?`
**Key GSAP:** Entry: `yPercent: 50→0, blur: 8px→0`, Exit: `yPercent: -50, blur: 6px`
**Note:** Animate parent width to match current text width (0.35s)

### TextScramble
**Effect:** Characters scramble through random pool on hover, settle to original.
**React approach:** `useGSAP` + `SplitText`. Timeline with staggered `gsap.call()`.
**Props:** `children`, `className`, `scrambleDuration?: 0.6`, `stagger?: 0.03`, `cycles?: 12`,
  `characters?: "ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789!@#$%^&*"`
**Key GSAP:** `gsap.timeline()`, `gsap.call()` for per-character scramble loop
**Free alternative:** anime.js `scrambleText` — see § Free Path (anime.js) below; MIT, no
GSAP Club license required.

### Free Path (anime.js) — MIT alternative to the 4 GSAP Club text recipes

anime.js v4.4.0+ ships a first-party text module (`scrambleText`/`splitText`) that covers
TextScramble and the SplitText-derived reveals (SplitHover, SplitReveal, FloatingMenu) without
a GSAP Club license. Full param table and deeper detail: `animejs` skill §
`references/text-effects.md`. Keep the GSAP recipes above as the GSAP-stack variant — this is
an additive alternative, not a replacement.

```javascript
import { animate, scrambleText } from 'animejs'; // or 'animejs/text'

animate('p', {
  innerHTML: scrambleText(), // NOT textContent — footgun: textContent silently no-ops
  loop: true,
  loopDelay: 1000,
});
```

```javascript
import { createTimeline, stagger, splitText } from 'animejs';

const { words, chars } = splitText('p', {
  words: { wrap: 'clip' },
  chars: true,
  accessible: true, // screen-reader handling of split DOM — keep on unless you have a reason not to
});

createTimeline({ loop: true, defaults: { ease: 'inOut(3)', duration: 650 } })
  .add(words, { y: [$el => +$el.dataset.line % 2 ? '100%' : '-100%', '0%'] }, stagger(125))
  .init();
```

- **Footgun:** `scrambleText()` drives through `animate(target, { innerHTML: scrambleText() })`
  — targeting `textContent` instead silently no-ops (no scramble, no error).
- **`seed`:** pass a fixed `seed` to `scrambleText({ seed: 1 })` for deterministic replay
  (useful for demos/evals where the scramble sequence must be reproducible).
- **`accessible`:** `splitText`'s `accessible` setting controls screen-reader handling of the
  split DOM — keep it on for any production text-reveal.

### WeightWave
**Effect:** Font weight changes based on mouse proximity to each character.
**React approach:** Split text into spans via ref, `onPointerMove` calculates distance per char.
**Props:** `children`, `baseWeight?: 350`, `hoverWeight?: 750`, `influenceRadius?: 3`,
  `falloffPower?: 1.5`, `duration?: 1.0`, `ease?: "power3.out"`
**Key GSAP:** `gsap.to()` per character, targeting `fontWeight` + `fontVariationSettings`
**Formula:** `weight = base + (hover - base) * pow(normalized_distance, falloffPower)`

### Magnetic
**Effect:** Element follows cursor with elastic spring.
**React approach:** `useRef` for element, `gsap.quickTo()` for x/y.
**Props:** `children`, `duration?: 1`, `ease?: "elastic.out(1, 0.3)"`, `className?`
**Key GSAP:** `gsap.quickTo(el, "x", { duration, ease })` — high-perf tween reuse
**Events:** `onPointerMove` → offset from center, `onPointerLeave` → animate to (0,0)

### Marquee
**Effect:** Continuous scroll with velocity-synced speed.
**React approach:** Repeat children N times, `useGSAP` timeline `repeat: -1`, ScrollTrigger velocity.
**Props:**
```tsx
interface MarqueeProps {
  children: React.ReactNode;
  className?: string;
  gap?: number;        // 32px
  repeat?: number;     // 3 (seamless copies)
  duration?: number;   // 5s per loop
  velocity?: number;   // 0.5x scroll multiplier
  reversed?: boolean;
  scrollElement?: string | HTMLElement;
}
```
**Key GSAP:** `gsap.timeline({ repeat: -1 })`, `ScrollTrigger` velocity modifier on `timeScale`

### Slideshow
**Effect:** Image carousel with parallax inner movement.
**React approach:** `useState` for active index, `useGSAP` timeline per transition.
**Props:** `images: { src: string; alt?: string }[]`, `className?`
**Key GSAP:** Outgoing: `xPercent: -100`, inner `xPercent: +75` (parallax). Incoming: reverse.
Duration 1.5s, ease "motion-core-ease". zIndex managed per slide.

### Preloader
**Effect:** Full-screen image reveal sequence → zoom to center.
**React approach:** `useGSAP` master timeline with chained `.to()` calls.
**Props:** `images: { src: string; alt?: string }[]`, `className?`, `onComplete?: () => void`
**Sequence:** Reveal L→R (2.5s) → scale non-center (2s) → center expand to viewport (2s)

### RadialGallery
**Effect:** Items arranged in circle, continuously rotating.
**React approach:** CSS transforms for circular placement, `gsap.to()` for rotation.
**Props:** `items: T[]`, `children: (item: T, index: number) => ReactNode`,
  `radius?: 600`, `duration?: 20`, `reversed?`, `elementSize?: 100`
**Placement:** `rotate(${i * 360/n}deg) translate(0, -${radius}px) rotate(90deg)`

### MacosDock
**Effect:** Icons magnify based on cursor distance (macOS dock).
**React approach:** `onPointerMove` on container, calculate distance per item, `gsap.to()` width.
**Props:**
```tsx
interface DockProps {
  items: { src: string; alt: string; label?: string; href?: string }[];
  className?: string;
  baseWidth?: number;      // 4em
  magnification?: number;  // 1.5x
  distance?: number;       // 3 items influence radius
}
```
**Formula:** Gaussian: `ratio = (distance - dist) / distance`, `width = base + (max - base) * ratio`

### FlipGrid
**Effect:** Grid items animate position/size on layout change (FLIP).
**React approach:** `Flip.getState()` before update, `Flip.from()` after React re-render (use `useLayoutEffect`).
**Props:** `children`, `className`, `duration?: 0.5`, `ease?: "power2.inOut"`, `stagger?`, `columns?`
**Key GSAP:** `Flip.getState()`, `Flip.from()` — capture state pre-render, animate post-render

### CardStack
**Effect:** Stacked cards reveal on scroll with scale reduction.
**React approach:** Sticky positioning + `ScrollTrigger` scrub per card.
**Props:** `children`, `className`, `scaleFactor?: 0.05`, `offset?: 10`,
  `topOffset?: 0`, `scrollElement?`
**Key GSAP:** Per-card `ScrollTrigger` with `scrub: true`, `scale: 1 - (n - 1 - i) * factor`

### FloatingMenu
**Effect:** Expanding nav menu with portal and staggered link reveals.
**React approach:** `createPortal`, `useGSAP` paused timeline, toggle plays/reverses.
**Props:** `menuGroups: MenuGroup[]`, `logo?`, `primaryButton?`, `secondaryButton?`, `className?`
**Sequence:** Width expand → overlay fade → height expand → toggle lines rotate 45° → links stagger up
**Responsive:** 768px / 1024px breakpoints

### VideoPlayer
**Effect:** Custom player with SVG morph icons and FLIP fullscreen.
**React approach:** `useRef` for video element, `MorphSVGPlugin` for icon transitions, `Flip` for fullscreen.
**Props:** `src`, `poster?`, `autoplay?`, `muted?`, `loop?`, `hideControls?`, `className?`
**Key GSAP:** `MorphSVGPlugin` (play↔pause, volume↔mute, enter↔exit fullscreen), `Flip.fit()` for fullscreen

---

## Category 2: Three.js / WebGL Shaders (14 components)

All components follow this React Three Fiber pattern:
```tsx
<Canvas dpr={[1, 2]} gl={{ toneMapping: THREE.NoToneMapping }}>
  <PerspectiveCamera makeDefault position={[0, 0, 5]} fov={50} />
  <SceneComponent {...props} />
</Canvas>
```

### AsciiRenderer
**Effect:** Image rendered as ASCII art characters.
**Props:** `src`, `className?`, `density?: 25`, `strength?: 25`, `color?: "#00ff00"`, `backgroundColor?: "#000000"`
**Shader:** Fragment shader converts texture to luminance, maps to 5x5 ASCII digit patterns with CRT scanlines.
**Uniforms:** `uDensity`, `uStrength`, `uColor`, `uBackgroundColor`, `uTime`, `uResolution`, `uTexture`
**R3F:** `<mesh><planeGeometry args={[2,2]} /><shaderMaterial ... /></mesh>`

### Card3D
**Effect:** 3D card with head-tracking parallax via MediaPipe face detection.
**Props:** `image`, `className?`, `width?: 3.2`, `height?: 2`, `depth?: 0.08`, `radius?: 0.15`, `showPreview?`
**Three.js:** `ExtrudeGeometry` from `THREE.Shape` with `quadraticCurveTo` rounded corners.
**Face tracking:** `@mediapipe/tasks-vision` `FaceLandmarker` → nose position → card rotation.
Lerp 0.1, `rotationX = headY * 0.4`, `rotationY = -headX * 0.5`
**Dep:** `@mediapipe/tasks-vision` (heavy — lazy-load recommended)

### GlassPane
**Effect:** Glass rod refraction with chromatic aberration.
**Props:** `image`, `className?`, `distortion?: 1.0`, `chromaticAberration?: 0.005`, `speed?: 1.0`,
  `waviness?: 0.05`, `frequency?: 6.0`, `rods?: 5.0`
**Shader:** Rotated 45° quad, sine wave "rods" create half-circle cross-sections.
Refraction ray: `mix(normal, rayDirection, refractiveIndex)`.
Chromatic aberration: R offset +, G none, B offset -.
Flow: `sin(time)`, `cos(time * 0.8)` for organic wave motion.

### DitheredImage
**Effect:** Ordered dithering (Bayer, halftone, void-and-cluster).
**Props:** `src`, `className?`, `ditherMap?: "bayer4x4" | "bayer8x8" | "halftone" | "voidAndCluster"`,
  `pixelSize?: 1`, `color?: "#ff6900"`, `backgroundColor?: "#111113"`, `threshold?: 0.0`
**Shader:** `DataTexture` from dithering matrices. Pixel grid via `floor(fragCoord / pixelSize)`.
Compare luminance vs threshold map → binary output → `mix(bgColor, fgColor, dither)`.
**R3F note:** Create `DataTexture` in `useMemo`, pass as uniform.

### PixelatedImage
**Effect:** Progressive depixelation animation (coarse → sharp).
**Props:** `src`, `className?`, `initialGridSize?: 6.0`, `stepDuration?: 0.15`
**Technique:** Grid size transitions: 6.0 → 4.5 → 3.0 → 1.5 → 0.5 on interval.
Shader quantizes UV to grid cells. Animate `uGridSize` uniform.

### WaterRipple
**Effect:** Interactive water distortion on mouse.
**Props:** `src`, `className?`, `brushSize?: 100`
**Technique:** Height map texture for wave propagation. Mouse adds displacement.
Render-to-texture feedback loop for ripple physics.

### InteractiveGrid
**Effect:** Grid mesh distorts toward mouse position.
**Props:** `image`, `className?`, `grid?: 15`, `mouseSize?: 0.15`, `strength?: 0.35`, `relaxation?: 0.9`
**Three.js:** Subdivided `PlaneGeometry`. Per-vertex displacement in `useFrame` based on
  normalized mouse distance. Relaxation factor for smooth return to rest.

### InfiniteGallery
**Effect:** 3D tunnel of images receding into depth.
**Props:** `images`, `className?`, `speed?: 1`, `visibleCount?: 8`,
  `fadeSettings?`, `blurSettings?`
**Three.js:** Images as textured planes arranged along Z axis. Opacity/blur fade by depth.
Camera at origin, FOV 55°. Time-based offset for scroll feeling.

### LavaLamp
**Effect:** Organic metaball blobs with Fresnel edge glow.
**Props:** `className?`, `color?: "#18181b"`, `fresnelColor?: "#ff6900"`, `speed?: 1.0`,
  `fresnelPower?: 3.0`, `radius?: 1`, `smoothness?: 0.1`
**Technique:** Noise-based vertex deformation on sphere. Fresnel shader for edge lighting.
Time-animated Perlin noise for organic movement.

### NeuralNoise
**Effect:** Animated Perlin/Simplex noise texture.
**Props:** `className?`, `speed?: 1.0`
**Technique:** 3D noise sampled at `(x, y, time * speed)`. Full-screen quad shader.

### RubiksCube
**Effect:** 3D Rubik's cube with face rotations.
**Props:** `className?`, `size?: 1`, `duration?: 1.5`, `gap?: 0.015`, `radius?: 0.125`, `fresnelConfig?`
**Three.js:** 27 `BoxGeometry` cubelets. Sequential axis rotations. Fresnel material.
**R3F:** Use `useFrame` for rotation animation, `<group>` for face grouping.

### PlasmaGrid
**Effect:** Cellular plasma noise pattern.
**Props:** `className?`, `color?: "#111113"`, `highlightColor?: "#FF6900"`
**Shader:** Simplex noise grid with `smoothstep` for sharp cell transitions.

### GlitterCloth
**Effect:** Silk cloth with glitter particles and vignette.
**Props:** `className?`, `color?: "#FF6900"`, `speed?: 1.0`, `brightness?: 1.0`,
  `blendStrength?: 0.02`, `noiseScale?: 4.0`, `vignetteStrength?: 15.0`
**Shader:** Simplex noise surface + vivid-light blend for glitter. Vignette: `pow(distance, power)`.

### GlassSlideshow
**Effect:** Image transitions with glass refraction blend.
**Props:** `images: string[]`, `index?: 0`, `className?`, `transitionDuration?: 2000`,
  `intensity?: 1.0`, `distortion?: 1.0`, `chromaticAberration?: 1.0`,
  `autoplay?`, `autoplayInterval?: 5000`
**Technique:** Two-texture blend during transition with GlassPane-style refraction shader.

---

## Category 3: Standalone Effects (6 components)

### Globe
**Effect:** Interactive 3D globe with point cloud land and markers.
**Props:** `className?`, `radius?: 2`, `pointCount?: 15000`, `landPointColor?: "#f77114"`,
  `pointSize?: 0.05`, `autoRotate?`, `markers?: GlobeMarker[]`, `focusOn?: [lat, lon]`
**Three.js:** `IcosahedronGeometry` + Points for land. `OrbitControls` with polar lock.
Fresnel atmosphere layer. Lat/lon → spherical coordinate conversion for markers.
**R3F:** Use `@react-three/drei` `<OrbitControls>`, `<Points>`, `<Billboard>` for markers.

### GodRays
**Effect:** Volumetric light rays from anchor point.
**Props:** `className?`, `color?: "#FFFFFF"`, `backgroundColor?: "#000000"`,
  `anchorX?: 0.5`, `anchorY?: 1.2`, `speed?: 1.0`, `lightSpread?: 1.0`,
  `rayLength?: 1.0`, `pulsating?`, `noiseAmount?: 0.0`, `distortion?: 0.0`
**Shader:** Radial rays from anchor, per-ray fade, optional noise/wave distortion, pulsation.

### SpecularBand
**Effect:** Animated specular light band with lens distortion.
**Props:** `className?`, `color?: "#FF6900"`, `backgroundColor?: "#000000"`,
  `speed?: 1.0`, `distortion?: 0.2`, `hueShift?: 30.0`, `intensity?: 1.0`
**Shader:** Time-animated band position, barrel/pincushion distortion, HSV hue shift.

### Halo
**Effect:** Atmospheric scattering / halo around light source.
**Props:** `className?`, `rotationSpeed?: 0.5`, `backgroundColor?: "#000000"`,
  `cameraDistance?: 3.0`, `fov?: 55.0`, `sunX/Y/Z`, `intensity?: 1.0`
**Technique:** Rayleigh + Mie scattering simulation. Camera orbits center.

### FluidSimulation
**Effect:** Interactive SPH fluid with pointer splats.
**Props:** `className?`, `dissipation?: 0.96`, `pointerSize?: 0.005`, `color?: "#ff6900"`,
  `velocityDissipation?: 0.96`, `pressureIterations?: 10`
**Technique:** Texture-based velocity/pressure fields. Pointer adds velocity splats.
Render-to-texture feedback. Pressure solver iterations for stability.

### LogoCarousel
**Effect:** Auto-scrolling vertical logo columns.
**Props:** `logos: { name: string; component: React.ComponentType }[]`,
  `columnCount?: 2`, `cycleInterval?: 2000`, `className?`
**React approach:** Pure CSS/GSAP. Distribute logos across N columns, each auto-scrolling.

---

## React Dependency Matrix

| Package | npm | Components |
|---------|-----|-----------|
| `gsap` | `gsap` | All DOM components (16) |
| `@gsap/react` | `@gsap/react` | `useGSAP` hook for React |
| `gsap/SplitText` | GSAP Club | SplitHover, SplitReveal, TextScramble, FloatingMenu (free alt: anime.js `scrambleText`/`splitText`, MIT — § Free Path) |
| `gsap/ScrollTrigger` | `gsap` | SplitReveal, Marquee, CardStack |
| `gsap/Flip` | `gsap` | FlipGrid, VideoPlayer |
| `gsap/MorphSVGPlugin` | GSAP Club | VideoPlayer |
| `three` | `three` | All WebGL components (14+6) |
| `@react-three/fiber` | `@react-three/fiber` | All WebGL components |
| `@react-three/drei` | `@react-three/drei` | Globe (OrbitControls, Points), textures |
| `@mediapipe/tasks-vision` | `@mediapipe/tasks-vision` | Card3D only |
| `clsx` | `clsx` | All (class merging) |
| `tailwind-merge` | `tailwind-merge` | All (Tailwind dedup) |

### GSAP Club vs Free

| Free (included in `gsap`) | Club (paid license) |
|---------------------------|---------------------|
| `gsap.to/from/fromTo/set` | `SplitText` |
| `gsap.timeline` | `MorphSVGPlugin` |
| `gsap.quickTo` | `DrawSVGPlugin` |
| `ScrollTrigger` | `ScrambleTextPlugin` (free alt: anime.js `scrambleText`, MIT) |
| `Flip` | `InertiaPlugin` |
| `CustomEase` | |

**4 components require GSAP Club:** SplitHover, SplitReveal, TextScramble, FloatingMenu (all need SplitText).
VideoPlayer also needs MorphSVGPlugin (Club) but can be reimplemented with CSS transitions.
**Free alternative for all 4 text components:** anime.js `scrambleText`/`splitText` (MIT,
>= 4.4.0) — see § Free Path (anime.js) above.

## When to Suggest (for frontend-design skill)

| Design Goal | Components to Recommend |
|-------------|------------------------|
| Hero section with depth | GlassPane, GodRays, Halo, GlitterCloth |
| Interactive background | FluidSimulation, WaterRipple, InteractiveGrid, PlasmaGrid |
| Text animation | SplitReveal (scroll), SplitHover (hover), TextScramble, TextLoop, WeightWave (or anime.js `scrambleText`/`splitText`, MIT, no GSAP Club needed — § Free Path) |
| Image showcase | InfiniteGallery, Slideshow, GlassSlideshow, Preloader |
| Navigation | FloatingMenu, MacosDock, Magnetic |
| Data visualization | Globe (with markers), AsciiRenderer |
| Card layouts | CardStack (scroll), FlipGrid (filter), Card3D (parallax) |
| Loading/transitions | Preloader, PixelatedImage |
| Scrolling content | Marquee, CardStack, RadialGallery |
| Branding | LogoCarousel, DitheredImage (retro), NeuralNoise (texture) |
| Video | VideoPlayer (custom controls with morph icons) |
| Luxury/premium feel | GlassPane, GlitterCloth, SpecularBand, Halo |
| Playful/interactive | Magnetic, ImageTrail, MacosDock, FluidSimulation |
| Dark theme hero | GodRays, NeuralNoise, PlasmaGrid, LavaLamp, Halo |
| 3D showcase | Globe, RubiksCube, Card3D, InfiniteGallery |

## Performance Notes

- WebGL components: cap DPR at 2x, use `<Canvas dpr={[1, 2]}>` (R3F auto-handles)
- Lazy-load heavy deps: `@mediapipe/tasks-vision` (Card3D), `three` (all WebGL)
- One full-viewport WebGL scene is fine; 3+ simultaneous canvases strain mobile GPUs
- GSAP animations: prefer `transform` and `opacity` (GPU-composited properties)
- `gsap.quickTo()` (Magnetic, WeightWave) is significantly faster than repeated `gsap.to()` calls
- For SSR (Next.js): all Canvas/WebGL components need `"use client"` + dynamic import with `ssr: false`

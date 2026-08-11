
# Radiant Shaders — Generative Visual Effects

> Source: https://github.com/pbakaus/radiant | MIT License
> Local recon: `~/.claude/recon/radiant-shaders.md`
> React wrapper: `~/.claude/recon/radiant-react-wrapper/`

91 production-ready, zero-dependency shader effects for web backgrounds, heroes, and accents.
Each shader is a self-contained HTML file — embed via `<iframe srcDoc={html} />`.

## When to Recommend

- Hero sections needing visual impact beyond static gradients
- Full-viewport animated backgrounds
- Accent elements adding depth and motion
- Dark-themed interfaces (shaders default to `#0a0a0a` background)
- Any design where generative art elevates the aesthetic

## Available Categories

| Tag | Count | Best For |
|-----|-------|----------|
| `fill` | ~40 | Full-screen backgrounds, hero sections |
| `particles` | ~20 | Dynamic, flowing backgrounds |
| `noise` | ~25 | Organic textures, terrain-like effects |
| `geometric` | ~15 | Structured, techy aesthetics |
| `organic` | ~15 | Natural, fluid motion |
| `physics` | ~10 | Interactive, responsive to mouse |
| `object` | ~10 | Standalone animated elements |

### Techniques

- **Canvas 2D**: Particle systems, flow fields, trails — lighter on GPU
- **WebGL**: Fragment shaders, ray marching — higher visual fidelity

## Color Schemes (CSS Filter-Based)

Apply via CSS `filter` on the iframe — no shader modification needed:

| Scheme | Filter | Vibe |
|--------|--------|------|
| Amber | `none` (native) | Warm, golden, default |
| Mono | `grayscale(1)` | Elegant, minimal |
| Blue | `hue-rotate(175deg)` | Cool, tech, corporate |
| Rose | `hue-rotate(300deg) saturate(1.1)` | Soft, feminine, warm-pink |
| Emerald | `hue-rotate(90deg) saturate(1.2)` | Nature, growth, fresh |
| Arctic | `hue-rotate(180deg) saturate(0.5) brightness(1.1)` | Ice, clean, subtle |

## Integration Pattern (React / Next.js)

```tsx
"use client";

import { RadiantShader } from "~/.claude/recon/radiant-react-wrapper";
// Or copy the component into the project

// Import shader HTML as raw string
import shaderHtml from "./shaders/flow-field.html?raw";

// Full-screen background
<RadiantShader
  html={shaderHtml}
  colorScheme="blue"
  params={{ SPEED: 2.0 }}
  loading="lazy"
  className="absolute inset-0"
/>
```

### Key Props

| Prop | Type | Purpose |
|------|------|---------|
| `html` | `string` | Raw shader HTML (import with `?raw`) |
| `colorScheme` | `ColorScheme` | CSS filter color scheme |
| `params` | `Record<string, number>` | Shader parameters via postMessage |
| `loading` | `"eager" \| "lazy" \| "manual"` | When to mount the iframe |
| `paused` | `boolean` | Freeze animation |
| `stripLabel` | `boolean` | Remove shader name overlay (default: true) |

### Layout Patterns

1. **Full Background**: Shader fills viewport, content overlaid with semi-transparent backdrop
2. **Hero Split**: Content left, shader right in contained box
3. **Accent Strip**: Shader as a narrow band/divider between sections
4. **Card Background**: Small shader behind card content with border-radius + overflow:hidden

### SSR Considerations

- Always use `"use client"` or `dynamic(() => import(...), { ssr: false })`
- Canvas/WebGL is client-only — no server rendering possible
- Use `loading="lazy"` for below-fold shaders to avoid GPU waste

## Shader Selection Guide

| Design Goal | Recommended Shaders | Technique |
|-------------|-------------------|-----------|
| Calm, flowing backgrounds | flow-field, aurora-veil, aurora-curtain | Canvas 2D |
| Techy/futuristic | analog-drift, artpop-iridescence | WebGL |
| Interactive hero | Any shader with `params` + mouse support | Either |
| Subtle texture | Noise-tagged shaders at low opacity | Canvas 2D |
| High-impact landing | bass-ripple, eclipse-glow | WebGL |

## Performance Notes

- Cap DPR at 2x (shaders do this internally)
- Use `loading="lazy"` + IntersectionObserver for galleries
- One full-viewport shader is fine; 5+ simultaneous iframes will strain mobile GPUs
- `sandbox="allow-scripts"` on iframe for security

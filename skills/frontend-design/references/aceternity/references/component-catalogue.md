
# Aceternity Component Catalogue

> Load WHEN you need to pick a specific Aceternity component for a section and want the
> category → component mapping. This is a routing table, not an inventory — the live, complete
> list is at https://ui.aceternity.com/components. Find the component, copy its slug from the
> URL, append `.json` to install via the shadcn registry.

## Decide before you browse the table: aceternity vs shadcn vs motion-and-transitions

The table below answers "which component" — this answers "which skill" first, so you don't
install a heavyweight component for a job a plain primitive already does.

| The UI need is... | Reach for | Why |
|---|---|---|
| A "wow moment" — hero, landing section, marketing showcase, a card that should feel premium | **aceternity** (this table) | Purpose-built motion components with the visual complexity already solved (3D transforms, particle fields, scroll choreography) — hand-rolling these in raw Framer Motion is a multi-hour sink for a worse result |
| Utilitarian chrome — forms, tables, dialogs, dropdowns, nav that just needs to work | **shadcn** | Faster to ship, smaller bundle, no motion budget to manage; aceternity here is over-engineering for CRUD |
| A single existing element needs to feel more alive — a modal that pops instead of snapping in, a badge that pulses, a menu that opens smoothly | **motion-and-transitions** | Nine copy-paste CSS snippets for exactly this scale of interaction; installing a full aceternity component (with its own state, refs, and JS dependency) to animate one open/close is the wrong tool for a timing/easing problem |

**What the registry install actually costs** — every aceternity component pulls in Framer
Motion (or `motion/react`) as a real dependency, not just CSS, so the bundle cost compounds
across a page: budget one motion component per "moment" (see below) rather than stacking
several because they're each individually cheap to install. Most components also require a
`"use client"` boundary — they read `window`/`document`/scroll position, so they cannot render
as a Server Component. Placing one deep in a server-rendered tree forces the client boundary up
to wherever you mount it; decide that boundary before installing, not after the hydration error
shows up.

Pick a category, then pick a component:

| Category | Representative components | Best for |
|---|---|---|
| **Backgrounds** | Aurora, Sparkles, Meteors, Shooting Stars, Vortex, Grid/Dot patterns, Canvas Reveal, SVG Mask | Hero sections, section dividers |
| **Cards** | 3D Card, Hover Effect, Spotlight, Focus, Glare, Wobble, Card Stack, Direction-Aware, Expandable | Feature grids, product cards |
| **Text effects** | Text Generate, Typewriter, Flip Words, Text Reveal, Colourful Text, Hover Effect, Text Hover Effect | Headlines, taglines, reveals |
| **Scroll & parallax** | Container Scroll, Sticky Scroll Reveal, MacBook Scroll, Hero Parallax, Parallax Grid, Tracing Beam | Long-form landing pages |
| **Navigation** | Floating Dock, Floating Navbar, Navbar Menu, Sidebar, Resizable Navbar, Tabs, Sticky Banner | Global chrome |
| **Buttons & loaders** | Hover Border Gradient, Moving Border, Stateful Button, Multi-Step Loader | CTAs, form submission |
| **Overlays** | Animated Modal, Animated Tooltip, Link Preview, Lamp Effect | Detail reveals |
| **Data-viz** | GitHub Globe, World Map, Timeline, Comparison Slider, Code Block, 3D Globe | Marketing proof/stats |
| **Forms** | Signup Form, File Upload, Placeholder-Vanish Input, Gooey Input | Playful onboarding |
| **Layout** | Bento Grid, Layout Grid, Container Cover | Feature showcases |
| **Carousels** | Apple Carousel, Animated Testimonials, Image Slider | Testimonials, galleries |

## Composition patterns

### Layering backgrounds under content

Aceternity backgrounds are absolute-positioned full-bleed canvases. Wrap them in a relative
container and put content on top with `relative z-10`:

```tsx
import { AuroraBackground } from "~/components/ui/aurora-background";
import { TextGenerateEffect } from "~/components/ui/text-generate-effect";

export function Hero() {
  return (
    <AuroraBackground>
      <div className="relative z-10 flex flex-col items-center justify-center px-4">
        <TextGenerateEffect words="Ship faster. Ship prettier." />
        <button className="mt-6 rounded-full bg-black px-6 py-3 text-white">
          Get started
        </button>
      </div>
    </AuroraBackground>
  );
}
```

### Combining motion components

Aceternity components compose — a 3D Card inside a Bento Grid inside a Hero Parallax all works.
Budget **one motion component per "moment"** so the page doesn't feel busy.

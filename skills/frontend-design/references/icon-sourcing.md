
# Icon Sourcing

> Reference for the `frontend-design` skill — one consolidated canon for UI icons, brand
> logos, and cloud/infra service marks.
>
> Consolidated from the `phosphor-icons`, `boxicons`, `svgl`, and `thesvg` skills
> (score-and-remediate-skill-quality-floor, 2.5). Those skills now route here for their deep
> content — this file is the canonical home.

## Decision ladder

Icon sourcing splits into three orthogonal questions — generic UI chrome, brand logos, and
cloud/infra service marks. Each has its own library and its own selection rule. Work top to
bottom; only drop to the next rung when the current one doesn't have what you need.

| Need | Library | Fallback |
|---|---|---|
| Generic UI chrome (nav, buttons, status, forms) | **Phosphor Icons** (project default) | Boxicons regular/solid (rarely needed — overlaps Phosphor without adding value) |
| Brand logo (dev/SaaS: Stripe, GitHub, Vercel, Figma...) | **svgl.app** (official press-kit fidelity) | Boxicons `bxl-*` (community-drawn) -> brand's own asset page |
| Brand logo (real-world consumer/enterprise: BMW, Delta, Coca-Cola, Costco, Visa...) | **thesvg** (svgl's SaaS-focused ~660 logos won't have these) | thesvg's own brand-logo long tail (~4,500) |
| Cloud/infra service icon (AWS, Azure, GCP marks; architecture/infra diagrams) | **thesvg** (739 AWS + 627 Azure + 214 GCP — svgl carries none of these) | registry search — never guess the slug |

**Never skip the fidelity check on brand logos.** If svgl has the logo, use it — don't reach
for Boxicons just because the import is shorter. svgl's SVGs come from the brand's own press
kit, so they track redesigns; Boxicons drifts behind them.

## Generic UI chrome — Phosphor Icons

1,248 icons across 6 weights, designed at 16x16px. MIT licensed. Browse: https://phosphoricons.com

### Packages

| Package | Framework | Install |
|---------|-----------|---------|
| `@phosphor-icons/react` | React / Next.js | `pnpm add @phosphor-icons/react` |
| `@phosphor-icons/web` | Vanilla HTML/CSS | CDN or `pnpm add @phosphor-icons/web` |
| `@phosphor-icons/vue` | Vue | `pnpm add @phosphor-icons/vue` |
| `@phosphor-icons/core` | Raw SVG assets | `pnpm add @phosphor-icons/core` |

### Weights — one weight per region

Every icon ships in 6 weights. Pick one weight per UI region for visual consistency — don't
mix regular and bold in the same nav. The only acceptable mix is fill for active states
alongside regular for inactive.

| Weight | Class (web) | Prop (React) | When to use |
|--------|-------------|--------------|-------------|
| **Thin** | `ph-thin` | `weight="thin"` | Elegant, minimal UI with lots of whitespace |
| **Light** | `ph-light` | `weight="light"` | Clean dashboards, subtle secondary icons |
| **Regular** | `ph` | `weight="regular"` | Default — most UI contexts |
| **Bold** | `ph-bold` | `weight="bold"` | Emphasis, primary actions, nav items |
| **Fill** | `ph-fill` | `weight="fill"` | Active/selected states, solid indicators |
| **Duotone** | `ph-duotone` | `weight="duotone"` | Decorative, feature sections, illustrations |

Duotone renders a two-tone version with one layer at 20% opacity — the most visually rich
weight. Use it for feature showcases, onboarding, and marketing sections. In React the
secondary layer inherits the icon's `color` at reduced opacity; target it via CSS if needed:

```css
/* Target the background layer of duotone icons */
.ph-duotone::before { opacity: 0.15; }
```

### React usage

```tsx
import { HouseIcon, GearIcon, BellIcon } from "@phosphor-icons/react";

<HouseIcon size={24} />
<GearIcon size={24} weight="bold" />
<BellIcon size={24} color="var(--muted-foreground)" />
```

Naming: PascalCase + `Icon` suffix — House -> `HouseIcon`, Gear Six -> `GearSixIcon`, Chat
Circle Dots -> `ChatCircleDotsIcon`. When unsure, search https://phosphoricons.com.

```typescript
interface IconProps extends SVGAttributes<SVGSVGElement> {
  color?: string;           // Any CSS color or "currentColor" (default)
  size?: number | string;   // px, em, rem, % — default 16
  weight?: "thin" | "light" | "regular" | "bold" | "fill" | "duotone";
  mirrored?: boolean;       // Flip for RTL
  alt?: string;             // Accessible label
}
```

Icons inherit `color` from parent text color via `currentColor` by default — prefer that over
setting an explicit color, so icons stay consistent across theme changes.

`IconContext.Provider` sets defaults for all icons in a subtree (`size`, `weight`, `color`) —
**not available in Server Components**, use explicit props there.

**Server Components / SSR — the `/ssr` submodule rule:** import from `/ssr`, which doesn't use
React Context (pass all props explicitly):

```tsx
import { HouseIcon } from "@phosphor-icons/react/ssr";

export default function Nav() {
  return <HouseIcon weight="bold" size={20} />;
}
```

**Next.js `optimizePackageImports` fix** — without this, dev server startup and HMR are
noticeably slower because the bundler pulls in all 9,000+ icon modules:

```js
module.exports = {
  experimental: {
    optimizePackageImports: ["@phosphor-icons/react"],
  },
};
```

Alternative for environments without it: direct path import,
`import { BellSimpleIcon } from "@phosphor-icons/react/dist/csr/BellSimple"`.

### Vanilla HTML (web package)

```html
<link rel="stylesheet" href="https://cdn.jsdelivr.net/npm/@phosphor-icons/web@2.1.2/src/regular/style.css" />
<link rel="stylesheet" href="https://cdn.jsdelivr.net/npm/@phosphor-icons/web@2.1.2/src/bold/style.css" />

<i class="ph ph-house"></i>           <!-- regular -->
<i class="ph-bold ph-heart"></i>      <!-- bold -->
```

Icons respond to `font-size` and `color`. **Do NOT override** `font-family`, `font-style`,
`font-weight`, `font-variant`, or `text-transform` — these break icon rendering. Icon classes
are lowercase kebab-case: `ph-house`, `ph-gear-six`, `ph-arrow-right`, `ph-chat-circle-dots`.

### Context -> size table

| Context | Size | Why |
|---------|------|-----|
| Inline with body text | 16px | Matches text baseline |
| Buttons | 16-20px | Proportional to button text |
| Navigation items | 20-24px | Visible at sidebar scale |
| Card headers | 24-32px | Visual anchor for content |
| Feature/hero sections | 32-48px | Decorative emphasis |
| Empty states | 48-64px | Large illustration role |

### Common icon picks

| Context | Icon | React | HTML |
|---------|------|-------|------|
| Home/Dashboard | House | `HouseIcon` | `ph-house` |
| Settings | Gear Six | `GearSixIcon` | `ph-gear-six` |
| User/Profile | User | `UserIcon` | `ph-user` |
| Search | MagnifyingGlass | `MagnifyingGlassIcon` | `ph-magnifying-glass` |
| Notifications | Bell | `BellIcon` | `ph-bell` |
| Menu/Hamburger | List | `ListIcon` | `ph-list` |
| Close | X | `XIcon` | `ph-x` |
| Back/Left | ArrowLeft | `ArrowLeftIcon` | `ph-arrow-left` |
| Forward/Right | ArrowRight | `ArrowRightIcon` | `ph-arrow-right` |
| Add/Create | Plus | `PlusIcon` | `ph-plus` |
| Delete/Remove | Trash | `TrashIcon` | `ph-trash` |
| Edit | PencilSimple | `PencilSimpleIcon` | `ph-pencil-simple` |
| Save | FloppyDisk | `FloppyDiskIcon` | `ph-floppy-disk` |
| Download | DownloadSimple | `DownloadSimpleIcon` | `ph-download-simple` |
| Upload | UploadSimple | `UploadSimpleIcon` | `ph-upload-simple` |
| Link | Link | `LinkIcon` | `ph-link` |
| Copy | Copy | `CopyIcon` | `ph-copy` |
| Check/Success | Check | `CheckIcon` | `ph-check` |
| Warning | Warning | `WarningIcon` | `ph-warning` |
| Error | XCircle | `XCircleIcon` | `ph-x-circle` |
| Info | Info | `InfoIcon` | `ph-info` |
| Eye/Show | Eye | `EyeIcon` | `ph-eye` |
| Hide | EyeSlash | `EyeSlashIcon` | `ph-eye-slash` |
| Filter | Funnel | `FunnelIcon` | `ph-funnel` |
| Sort | SortAscending | `SortAscendingIcon` | `ph-sort-ascending` |
| Calendar | Calendar | `CalendarIcon` | `ph-calendar` |
| Clock/Time | Clock | `ClockIcon` | `ph-clock` |
| Email | Envelope | `EnvelopeIcon` | `ph-envelope` |
| Phone | Phone | `PhoneIcon` | `ph-phone` |
| Chat | ChatCircle | `ChatCircleIcon` | `ph-chat-circle` |
| Lock/Auth | Lock | `LockIcon` | `ph-lock` |
| Logout | SignOut | `SignOutIcon` | `ph-sign-out` |

### Phosphor rules

1. One weight per region — mixing regular/bold in the same nav reads as inconsistent.
2. Prefer `currentColor` over explicit colors.
3. Always add `alt`/`aria-label` for icons that convey meaning without adjacent text;
   decorative icons next to a label can use `aria-hidden="true"`.
4. Use `optimizePackageImports` in Next.js.
5. SSR = `/ssr` import — Server Components cannot use `IconContext`.
6. Brand logos go through svgl/thesvg (below), not Phosphor.

### Phosphor never

- **Never import the default `@phosphor-icons/react` entry inside a Server Component.** The
  default entry wires every icon through `IconContext`, which is React Context — Server
  Components have no context tree to read from, so the import silently falls back to
  hardcoded defaults instead of your theme's size/weight/color. Import from `/ssr` there,
  every time, not just when you notice the mismatch.
- **Never mix weights within one visual region** (a single nav, toolbar, or card row). Each
  weight is drawn at a different stroke thickness — a `regular` icon next to a `bold` one in
  the same row reads as a rendering bug, not a design choice, because the optical weight
  mismatch is more visually jarring than an outright wrong icon would be. Pick one weight per
  region; `fill` for the active state alongside `regular` for inactive siblings is the only
  sanctioned exception.
- **Never inline-copy an icon's SVG path when the icon already exists in the package.** A
  pasted-in `<svg>` loses every theming hook the component gives you for free — `size`,
  `weight`, `color`/`currentColor`, and RTL `mirrored` all stop working, and a later weight or
  size change now requires hand-editing SVG markup instead of changing a prop. Import the
  component; only inline a raw SVG for an icon that genuinely isn't in Phosphor's 1,248-icon
  set.

## Brand logos — dev/SaaS: svgl.app

svgl.app is a free, open API serving 760+ SVG brand logos across 40 categories, no auth
required. Replaces emoji, placeholder text, and guessed SVG paths with real brand assets.

### API reference

Base: `https://api.svgl.app`

| Endpoint | Returns |
|----------|---------|
| `GET /` | All SVGs (array) |
| `GET /?search={term}` | Search by title |
| `GET /?limit={n}` | Limit results |
| `GET /category/{name}` | Filter by category |
| `GET /categories` | All categories with counts |
| `GET /svg/{name}.svg` | Raw SVG markup (SVGO-optimized) |
| `GET /svg/{name}.svg?no-optimize` | Raw SVG markup (original) |

```typescript
interface SVG {
  id: number;
  title: string;                    // e.g. "Vercel", "Stripe"
  category: string | string[];      // single or multi-category
  route: string | ThemeOptions;     // logo URL(s)
  wordmark?: string | ThemeOptions; // wider logo with text
  url: string;                      // official website
  brandUrl?: string;                // brand guidelines
}

interface ThemeOptions {
  light: string;  // for light backgrounds
  dark: string;   // for dark backgrounds
}
```

### The never-guess-CDN-paths rule

**Filenames are inconsistent across the library** — don't guess. Confirmed inconsistent
examples: `github-light.svg` / `github-dark.svg` (hyphenated) vs `nextjs_icon_dark.svg` /
`nextjs_icon_light.svg` (underscored, `icon` infix) vs plain `stripe.svg` (no variant at all).
If the brand isn't in the quick-reference table below, search the API
(`https://api.svgl.app?search={brand}`) rather than constructing the path yourself.

### Quick reference (skip the API call for these)

| Brand | Light | Dark |
|-------|-------|------|
| Vercel | `svgl.app/library/vercel.svg` | `svgl.app/library/vercel_dark.svg` |
| GitHub | `svgl.app/library/github-light.svg` | `svgl.app/library/github-dark.svg` |
| Stripe | `svgl.app/library/stripe.svg` | — |
| Next.js | `svgl.app/library/nextjs_icon_dark.svg` | `svgl.app/library/nextjs_icon_light.svg` |
| React | `svgl.app/library/react_dark.svg` | `svgl.app/library/react_light.svg` |
| TypeScript | `svgl.app/library/typescript.svg` | — |
| Tailwind | `svgl.app/library/tailwindcss.svg` | — |
| Node.js | `svgl.app/library/nodejs.svg` | — |
| PostgreSQL | `svgl.app/library/postgresql.svg` | — |
| Docker | `svgl.app/library/docker.svg` | — |

Prefix all paths with `https://` when embedding.

### Theme-variant + wordmark-vs-route selection

| Context | Use |
|---------|-----|
| Icon in a card/button | `route` (compact logo) |
| Header or hero section | `wordmark` (logo + text) |
| Dark background / dark mode | `.dark` variant |
| Light background / light mode | `.light` variant |
| No theme variants available | Use the string URL directly |

**Embed:**

```html
<img src="https://svgl.app/library/stripe.svg" alt="Stripe" width="24" height="24" />
```

Inline SVG (fetch `https://api.svgl.app/svg/stripe.svg` and paste the markup directly) when
you need `fill`/`stroke`/CSS control over the paths — otherwise prefer `<img>`, it's simpler
and cacheable.

### svgl categories (40)

AI, Analytics, Authentication, Automation, Browser, CMS, Community, Compiler, Config, Crypto,
Cybersecurity, Database, Design, Devtool, Education, Entertainment, Framework, Google,
Hardware, Hosting, IaC, IoT, Language, Library, Marketplace, Microsoft, Monorepo, Music, Nuxt,
Payment, Platform, Privacy, Secrets, Social, Software, Sync Engine, Themes, Vercel, VoidZero

### svgl rules

1. Never guess CDN paths — search the API when the brand isn't in the quick reference.
2. Always include `alt` text with the brand name.
3. Pick the right theme variant for the background it sits on.
4. Prefer `<img>` over inline SVG unless you need CSS control.
5. Sizing: nav 20-24px, card icons 32-40px, hero/feature 48-64px. Wordmarks scale by width.
6. Don't fabricate a URL if the API search returns empty — fall back to Phosphor or plain text.
7. No emojis as brand placeholders.

## Brand logos — community fallback: Boxicons `bxl-*`

1500+ icons across Regular (`bx-`), Solid (`bxs-`), and Logos (`bxl-*`) styles. MIT code +
CC-BY 4.0 SVGs. Browse: https://boxicons.com

Reach for Boxicons specifically when svgl doesn't have the brand logo, or when a legacy
project already uses it. Boxicons' regular/solid generic sets overlap Phosphor without adding
value — skip them for chrome; Phosphor stays the default there.

### Why svgl outranks Boxicons for the same logo

The fidelity ladder (§ Decision ladder above) isn't a licensing preference — it's about where
the SVG originated. svgl's marks are pulled from each brand's own press kit, so a redesign
(new wordmark, updated glyph proportions, a color-system refresh) reaches svgl on the brand's
timeline. Boxicons' `bxl-*` set is community-redrawn: a contributor traces the mark by eye and
submits it, so it lags the live brand by however long it takes someone to notice and redraw it
— and a redraw can drift on stroke weight, corner radius, or proportions in ways that are
individually subtle but read as "off" in aggregate. Shipping a stale or slightly-wrong logo
next to a customer's or partner's name is a brand-trust bug, not a cosmetic one — treat the
ladder as load-bearing, not a shortcut you can skip because the Boxicons import is one line
shorter.

### Install

```bash
pnpm add boxicons
```

```ts
// app/layout.tsx or global client entry — pin the import here, not a component file,
// or Next.js may dedupe incorrectly and double-load the stylesheet on client nav
import "boxicons/css/boxicons.min.css";
```

```tsx
<i className="bx bx-home" />          // regular outline
<i className="bx bxs-home" />         // solid filled
<i className="bx bxl-github" />       // logo
```

CDN zero-install alternative: `<link href="https://cdn.jsdelivr.net/npm/boxicons@latest/css/boxicons.min.css" rel="stylesheet" />`.
A `<box-icon>` web component and a community `boxicons-react` wrapper also exist — prefer the
CSS-class approach for a large project (always current, predictable bundle size).

### Brand logo catalogue (non-exhaustive — check boxicons.com for the full set)

Grouped by domain, `bxl-` prefix implied: **dev/cloud** — github, gitlab, bitbucket, vercel
(not always shipped), aws, google-cloud, docker, kubernetes. **payments** — stripe, paypal,
visa, mastercard, venmo. **social** — twitter, facebook, instagram, linkedin, youtube, tiktok,
reddit, discord, slack, telegram, whatsapp. **tools** — figma, sketch, adobe, notion, dropbox.
**stacks** — react, vuejs, angular, nodejs, typescript, tailwind-css, nextjs.

When a specific logo is missing, fall back to the company's own SVG from their brand assets
page — don't substitute a generic icon that only approximates the brand.

### Boxicons gotchas

- **`bxl-*` coverage is uneven** — grep the icon list before committing to a logo choice.
- **No tree-shaking for the CSS approach** — all 1500+ classes load (~50KB gzipped). If bundle
  size matters, use the React wrapper or import individual SVGs from `@boxicons/svg`.
- **Never import the stylesheet from a component file.** Pin `import
  "boxicons/css/boxicons.min.css"` once in `app/layout.tsx` or a global client entry (see
  Install above). Importing it from a component instead lets Next.js's module dedup miss —
  the same 50KB stylesheet gets fetched again on client-side navigation, so the page silently
  ships double the CSS bytes with no visible difference until someone checks the network tab.
- Icon-only `<i>` tags are invisible to screen readers — wrap in a button or add `aria-label`.
- Don't mix `bx-`/`bxs-` in the same toolbar, and don't mix Boxicons with Phosphor in the same
  cluster (fine at the page level; pick one per toolbar/row).

### Parallel fuzzy-find across libraries

When it's unclear which library has the best match, or the user asks "what are my options for
X?", search svgl + Phosphor + Boxicons in parallel and **return every candidate with its
fidelity tier** rather than committing to one — lead with the highest-fidelity match as the
recommendation, list lower tiers as alternates:

```
User: "What are the options for a 'database' icon?"

  **svgl** (brand — not applicable for generic shapes)
    — (no entries for generic "database")

  **Phosphor** (generic, preferred for chrome)
    — Database (regular/bold/fill/duotone/light/thin)
    — HardDrive
    — Cylinder

  **Boxicons** (fallback generic)
    — bx-data
    — bxs-data

  Recommendation: Phosphor `Database` (fill weight) for chrome consistency.
```

For brand queries the same fuzzy-find runs in fidelity order (svgl -> boxicons `bxl-*` ->
brand's own asset page). None of the three icon sets is strictly a superset of another — the
project's context (consistency, available weights, brand freshness) is the tiebreaker.

**Worked example — a brand logo, all three libraries checked:**

```
User: "I need a Discord logo for the community-links section."

  **svgl** (dev/SaaS brand — check first)
    — /?search=discord -> present, press-kit SVG, light + dark variants
    — TIER: use this. Official source, themed variants match either background.

  **Boxicons** (fallback — checked anyway to confirm svgl isn't stale)
    — bxl-discord present, single color, no light/dark split
    — TIER: skip. svgl already covers it at higher fidelity; only fall back here
      if svgl's entry 404s or the project already has Boxicons wired in.

  **thesvg** (long-tail fallback — not needed here)
    — not queried; svgl already resolved the request

  Recommendation: svgl `discord` (theme-variant route). Boxicons stays on the bench —
  it's a legitimate fallback, not a tiebreaker, when a higher-fidelity source exists.
```

The tier verdict is the deliverable, not just the icon — it tells the caller *why* one source
won so the next similar request doesn't have to re-run the full three-way search from scratch.

## Cloud/infra + long-tail brand logos: thesvg

theSVG hosts 6,000+ SVGs behind one predictable, no-auth CDN. Its unique value over svgl is
the **cloud architecture collections** — 739 AWS, 627 Azure, 214 GCP service icons svgl does
not carry — plus a ~4,500-logo brand long tail for real-world consumer/enterprise names svgl's
SaaS-focused set lacks. MIT-licensed codebase; icon marks carry their own source licenses (see
§ Licensing below — this is load-bearing, not boilerplate).

### Boundary with svgl

| The request is about... | Use |
|---|---|
| AWS / Azure / GCP service icons, architecture or infra diagrams | **thesvg** |
| A dev/SaaS brand logo (GitHub, Stripe, Vercel, Notion, Figma...) | **svgl** first |
| A real-world consumer/enterprise brand (car, airline, bank, retailer, food, crypto, consumer hardware) | **thesvg** — svgl's set won't have it |
| Any brand svgl doesn't have (searched, came up empty) | **thesvg** as fallback |
| Generic UI glyphs (chevrons, menus, arrows) | neither — Phosphor |

### Resolve-the-slug-never-guess rule

theSVG's cloud slugs are **not** what you'd predict — they carry the provider's own verbose
product names and sometimes a doubled prefix:

| Service | What you'd guess | Actual slug |
|---|---|---|
| AWS S3 | `aws-s3` | `aws-amazon-simple-storage-service` |
| AWS Lambda | `aws-lambda` | `aws-aws-lambda` (yes, doubled) |
| AWS EKS | `aws-eks` | `aws-amazon-elastic-kubernetes-service` |
| Azure Cosmos DB | `azure-cosmos-db` | `azure-azure-cosmos-db` |
| GCP GKE | `google-cloud-gke` | `gcp-google-kubernetes-engine` |

Guessing wastes turns on 404s. Fetch the registry once per session and search it — it changes
on the order of days, so cache it for the whole session:

```
GET https://cdn.jsdelivr.net/gh/glincker/thesvg@main/src/data/icons.json
```

It's a JSON array of `{ slug, title, aliases, variants, license, ... }`. Match the user's words
against `title`/`aliases`, then read the **`variants` map values** for the real file paths —
don't reconstruct filenames yourself (e.g. the `wordmarkLight` variant lives at
`wordmark-light.svg`, so the key and filename differ):

```bash
curl -sL https://cdn.jsdelivr.net/gh/glincker/thesvg@main/src/data/icons.json \
 | jq -r '.[] | select(.title|test("cosmos";"i")) | "\(.slug)\t\(.title)"'
# azure-azure-cosmos-db    Azure Cosmos DB
```

The quick-reference tables below are verified-present — trust them, don't re-derive or
second-guess a listed slug as "missing." Only search the registry when a needed service isn't
already in the tables.

### URL contract

No auth, no rate limits. Two equivalent endpoints, pick by who fetches:

```
# Agent / automation / batch fetch (global CDN absorbs bursty load):
https://cdn.jsdelivr.net/gh/glincker/thesvg@main/public/icons/{slug}/{variant}.svg

# Embedded in a user-facing page (README, HTML, blog) — light per-visitor traffic:
https://thesvg.org/icons/{slug}/{variant}.svg
```

### Cloud icons have size variants, not color variants

Cloud icons have **no color variants** — asking for `mono`/`light`/`dark` on a cloud slug
404s. Always confirm against the registry's `variants` map; `default` is always present.

| Collection | Variants that exist | Notes |
|---|---|---|
| AWS (`aws-*`) | `default`, `16`, `32`, `64` | Size variants, not color. Use `default` for diagrams. |
| Azure (`azure-*`) | `default` only | — |
| GCP (`gcp-*`) | `default` only | — |
| Brand (`github`, `stripe`...) | `default`, `mono`, `light`, `dark`, `wordmark`, `wordmarkLight`, `wordmarkDark`, `color` | Pick `mono`/`light` on dark backgrounds. |

### Quick reference — common cloud slugs (verified)

**AWS** (prefix `aws-`; AWS-branded services double to `aws-aws-`, Amazon-branded use `aws-amazon-`):

| Service | Slug |
|---|---|
| Lambda | `aws-aws-lambda` |
| Fargate | `aws-aws-fargate` |
| Step Functions | `aws-aws-step-functions` |
| S3 | `aws-amazon-simple-storage-service` |
| EC2 | `aws-amazon-ec2` |
| RDS | `aws-amazon-rds` |
| Aurora | `aws-amazon-aurora` |
| DynamoDB | `aws-amazon-dynamodb` |
| CloudFront | `aws-amazon-cloudfront` |
| CloudWatch | `aws-amazon-cloudwatch` |
| API Gateway | `aws-amazon-api-gateway` |
| EKS | `aws-amazon-elastic-kubernetes-service` |
| Redshift | `aws-amazon-redshift` |
| Route 53 | `aws-amazon-route-53` |

AWS also ships resource-level (`aws-res-*`) and grouping (`aws-group-*`) icons for detailed
architecture diagrams — search the registry for those.

**Azure** (prefix `azure-`; Azure-branded services double to `azure-azure-`):

| Service | Slug |
|---|---|
| Function Apps | `azure-function-apps` |
| App Services | `azure-app-services` |
| AKS (Kubernetes) | `azure-kubernetes-services` |
| Cosmos DB | `azure-azure-cosmos-db` |
| Blob Storage | `azure-blob` |
| Container Instances | `azure-container-instances` |
| Container Registries | `azure-container-registries` |
| Key Vaults | `azure-key-vaults` |
| Monitor | `azure-monitor` |

**GCP** (prefix `gcp-`):

| Service | Slug |
|---|---|
| Cloud Run | `gcp-cloud-run` |
| Cloud Functions | `gcp-cloud-functions` |
| GKE (Kubernetes) | `gcp-google-kubernetes-engine` |
| Compute Engine | `gcp-compute-engine` |
| Cloud Storage | `gcp-cloud-storage` |
| Cloud SQL | `gcp-cloud-sql` |
| BigQuery | `gcp-bigquery` |
| Firestore | `gcp-firestore` |
| Cloud Spanner | `gcp-cloud-spanner` |
| Vertex AI | `gcp-vertexai` |

### How to deliver

| Context | What to return |
|---|---|
| Architecture diagram (Excalidraw / draw.io / Figma) | The CDN URL per service, **grouped by tier** (compute / storage / network / data) so they drop onto the canvas in order |
| Mermaid / C4 diagram with image nodes | `img` node referencing the CDN URL, or fetch + inline the SVG body |
| Markdown / README | `![Lambda](https://thesvg.org/icons/aws-aws-lambda/default.svg)` |
| HTML / web artifact | `<img src="https://thesvg.org/icons/aws-aws-lambda/default.svg" width="40" height="40" alt="AWS Lambda" />` |
| Agent fetching raw SVG to inline | GET the jsDelivr URL, paste the `<svg>...</svg>` body |

### Licensing hierarchy that bites

Treat this as load-bearing, not boilerplate. The theSVG codebase is MIT, but the icon marks
carry their source licenses:

- **AWS icons: `CC-BY-ND-2.0` — No Derivatives** (all 739). Distribute *unmodified*. Do **not**
  recolor, redraw, restyle, or compose AWS marks into something new — the strictest of the
  four collections.
- **GCP icons: `Apache-2.0`** (all 214). Permissive — recolor/modify OK.
- **Azure icons: `MIT`** (626 of 627; a single entry is tagged `Custom`). Treat as permissive,
  but glance at the entry's `license` on the rare oddball before restyling.
- **Brand logos: it genuinely varies — read the entry.** About three-quarters are `CC0-1.0`
  (recolor-safe), but the rest carry `MIT`, `brand-use`, `Fair Use`, `Custom`, `CC-BY-SA-4.0`,
  or `Trademark` terms — e.g. Visa and Mastercard are CC0 (free to tint), but Affirm, Capital
  One, and the Google Workspace marks are Fair Use/Trademark (use unmodified). Never infer CC0
  from the file format alone.

Brand marks remain trademarks of their owners regardless of file license. If asked to recolor,
distort, or merge a brand/AWS mark into a new logo, flag the constraint before proceeding.

### Brand logos — the long tail svgl lacks

svgl.app is SaaS/devtool-focused (~660 logos). theSVG carries ~4,500 brand logos — all but two
of svgl's, plus ~3,800 svgl doesn't have. When the brand is a real-world consumer or enterprise
name rather than a dev tool, come straight here:

| Category | Examples theSVG has that svgl does not |
|---|---|
| Automotive | BMW, Audi, Toyota, Tesla, Ford, Porsche, Ferrari, Mercedes-Benz |
| Airlines | Delta, United, American Airlines, Air France, Lufthansa, Emirates, Airbus |
| Banking / Finance | Bank of America, BBVA, AXA, Aviva, Affirm, Afterpay, Alipay |
| Food / Beverage | Coca-Cola, Burger King, Chick-fil-A, Domino's, DoorDash, Deliveroo |
| Retail | Costco, IKEA, Best Buy, Aldi, ASDA, The Home Depot, Etsy |
| Crypto | Cardano, Polkadot, Chainlink, Avalanche, Base, Bybit |
| Consumer hardware | AMD, ASUS, Anker, Bose, Arduino, Acer, Alienware |
| Entertainment / Media | Amazon Prime, Cartoon Network, Deezer, DreamWorks, Bravo |

Brand slugs are simpler than cloud slugs — usually just the lowercased name (`bmw`,
`coca-cola`, `bank-of-america`) — but still resolve via the registry when unsure. Unlike cloud
marks, brand icons **do** carry color variants (`mono`, often `light`/`dark`/`wordmark`), and
many are `CC0-1.0` (recolor-safe — the opposite of the AWS constraint). Always read the
entry's `license` before restyling; the mark stays a trademark regardless of file license.

Missing icon: submit at `https://thesvg.org/submit` (accepts brands whose domain is 30+ days
old). Cloud service registry updates quarterly from the official provider icon packs.

## Related skills

- **`wayfinder`** — when building an HTML architecture diagram, pull cloud marks from thesvg.
- **`c4-architecture`** / `documentation-writer`'s **`references/mermaid-diagrams/`** — embed CDN
  URLs as image nodes for real service icons instead of plain boxes.
- **`shadcn`** — when icons live inside buttons, form fields, or dropdowns.

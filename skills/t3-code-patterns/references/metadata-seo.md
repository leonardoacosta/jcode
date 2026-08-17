
# Metadata & SEO (Next.js App Router)

> Adapted from ui-skills' `fixing-metadata` skill (github.com/ibelick/ui-skills, MIT). Rewritten
> for Next.js App Router `generateMetadata` conventions and T3 Turbo fleet placement — full
> ruleset backing `SKILL.md` § Metadata/SEO.

## Agreement Rule (MUST)

Title, description, canonical URL, and `og:url` MUST all agree with each other and with the
actual served URL. The most common regression is a stale canonical/OG URL left pointing at the
old path after a route rename — the page renders fine, but every crawler and social-card
consumer resolves the wrong URL.

```typescript
// apps/nextjs/src/app/events/[slug]/page.tsx
export async function generateMetadata({ params }: Props): Promise<Metadata> {
  const { slug } = await params;
  const event = await getEvent(slug);
  const url = `${env.NEXT_PUBLIC_APP_URL}/events/${slug}`; // ✅ derived from the SAME slug used to render the page

  return {
    title: event.title,
    description: event.summary,
    alternates: { canonical: url },
    openGraph: { url, title: event.title, description: event.summary },
  };
}
```

**Verify social cards against a REAL deployed/preview URL, never `localhost`.** Social crawlers
(Slack, Twitter/X, iMessage, Facebook) cannot reach `localhost` — a card that "looks right" in a
local OG-image preview tool is not verified. Paste the preview-URL debugger result (e.g.
Slack's unfurl, or `curl -s <preview-url> | grep 'og:image'` confirming the tag resolves to an
absolute, publicly reachable image URL) before calling metadata work done.

## Priority x Impact

| Priority | Issue | User-facing impact | Fix effort |
| --- | --- | --- | --- |
| Critical | Missing canonical | Duplicate-content penalty, wrong URL indexed/shared | Low |
| Critical | Missing OG image | Bare link preview in every chat app / social share | Low |
| High | Duplicate `<title>` across routes | Search results and browser tabs indistinguishable | Low |
| High | Canonical/OG URL mismatch (route rename) | Shares resolve to a dead or wrong page | Low |
| Medium | Missing JSON-LD | Lost rich-result eligibility (ratings, breadcrumbs, price) | Medium |
| Medium | Missing robots directive | Staging/preview or gated pages get indexed | Low |
| Low | Missing favicon / apple-touch-icon | Generic icon in tabs, bookmarks, iOS home screen | Low |

## Coverage Checklist

| Surface | Tag(s) | `generateMetadata` field |
| --- | --- | --- |
| Title | `<title>` | `title` |
| Description | `<meta name="description">` | `description` |
| Canonical | `<link rel="canonical">` | `alternates.canonical` |
| Open Graph | `og:title`, `og:description`, `og:image`, `og:url`, `og:type` | `openGraph.{title,description,images,url,type}` |
| Twitter Card | `twitter:card`, `twitter:title`, `twitter:image` | `twitter.{card,title,images}` |
| Structured data | JSON-LD (`Organization`/`Product`/`Article`, where applicable) | `<script type="application/ld+json">` in the page body — no `generateMetadata` field owns this |
| Crawl directives | `robots.txt`, meta-robots | `robots` (per-route) or `app/robots.ts` (site-wide) |
| Favicon set | standard favicon, `apple-touch-icon`, manifest icons | `icons` (or `app/icon.tsx` / `app/apple-icon.tsx`) |

A route missing any row above is a finding at the priority listed in the table — cite the
missing tag, not just "SEO is incomplete."

## `generateMetadata` Conventions

### `metadataBase` — set once, root layout only

`metadataBase` resolves every relative URL (`openGraph.images`, `alternates.canonical`, etc.)
into an absolute one. Set it exactly once in the root layout; a child layout or page that also
sets it silently overrides the resolution base for its whole subtree.

```typescript
// apps/nextjs/src/app/layout.tsx
export const metadata: Metadata = {
  metadataBase: new URL(env.NEXT_PUBLIC_APP_URL),
};
```

### Nested layout metadata merging

App Router merges `metadata`/`generateMetadata` from the root layout down to the page,
**shallow-merged per field** — a child does not need to repeat fields it isn't changing, but
`openGraph`/`twitter` objects replace their parent wholesale rather than deep-merging. A page
that sets `openGraph.title` but omits `openGraph.images` loses the parent's OG image entirely.

```typescript
// apps/nextjs/src/app/events/layout.tsx
export const metadata: Metadata = {
  openGraph: { siteName: "Events" }, // shallow field, inherited by every child page
};

// apps/nextjs/src/app/events/[slug]/page.tsx
export async function generateMetadata({ params }: Props): Promise<Metadata> {
  const event = await getEvent((await params).slug);
  return {
    openGraph: {
      title: event.title,      // ✅ page-specific
      images: [event.ogImage], // MUST repeat — layout's openGraph object does not deep-merge
    },
  };
}
```

### Static vs dynamic — `Metadata` object vs async function

Use the static `export const metadata: Metadata` object when nothing depends on route params or
a data fetch. Use the async `generateMetadata({ params, searchParams }, parent)` function only
when title/description/OG content is derived from fetched data — the async form adds a data
fetch to every request unless the underlying query is already cached (`React.cache` /
`fetch` dedup), so don't reach for it by default.

```typescript
// Static — no params, no fetch
export const metadata: Metadata = {
  title: "Pricing",
  description: "Plans and pricing for {product}.",
};
```

### Robots — per-route vs site-wide

Per-route directive lives in that route's `metadata.robots`; site-wide crawl policy lives in
`app/robots.ts` (generates `/robots.txt`). Prefer `app/robots.ts` for blanket rules (disallow
`/admin`, `/api`) and per-route `robots` only for a specific page that must diverge from the
site-wide policy (e.g. a preview/staging route that must never be indexed regardless of
environment).

```typescript
// apps/nextjs/src/app/robots.ts
import type { MetadataRoute } from "next";

export default function robots(): MetadataRoute.Robots {
  return {
    rules: { userAgent: "*", disallow: ["/admin", "/api"] },
    sitemap: `${env.NEXT_PUBLIC_APP_URL}/sitemap.xml`,
  };
}
```

### Favicon set — `app/icon.tsx` vs static files

Next.js App Router resolves favicons from special files colocated in `app/` — `icon.png` /
`icon.tsx` (favicon), `apple-icon.png` (apple-touch-icon), and `manifest.ts` /`manifest.json`
(PWA/manifest icons) — no explicit `<link>` tags or `metadata.icons` entry needed for the
default set. Only set `metadata.icons` explicitly when serving a non-default path or a
per-route icon override.

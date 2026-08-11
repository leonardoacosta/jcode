---
name: nextjs-app-router
description: Next.js 16 App Router patterns including route handlers, streaming, Server Actions, the 'use cache' caching model, Cache Components, Instant Navigations, and proxy.ts. Use when building Next.js pages, implementing data fetching, configuring route protection, debugging stale or over-cached data, adopting Cache Components (cacheComponents, partialPrefetching), or migrating off Next.js 15 (proxy.ts rename, catchError/retry, root params).
source: ~/.agents/skills@2026-07-13
user-invocable: false
paths: ["apps/*/src/app/**"]
---


# Next.js 16 App Router Patterns

> Covers App Router-specific patterns for Next.js 16, React 19, TypeScript, Tailwind, tRPC, Better Auth, Vercel.
> This skill fills the gap left by `react-dev` — focus here is on routing, caching, and server/client boundaries.

## Data Fetching Hierarchy

The mental model: **where data lives determines how you fetch it.**

| Layer | Pattern |
|-------|---------|
| Server Component | `await fetch()` or `await createCaller(ctx).router.procedure()` directly |
| Client Component | tRPC React Query hooks (`trpc.router.procedure.useQuery()`) |
| Layout | Fetch shared data once (session, org context); pass via React Context or props |
| `cache()` from React | Deduplicate identical fetches within a single request — NOT across requests |

```typescript
// Server Component — fetch directly, no hooks
export default async function ProductPage({ params }: { params: Promise<{ id: string }> }) {
  const { id } = await params;  // Next.js 15: params is a Promise
  const product = await db.query.product.findFirst({ where: eq(schema.product.id, id) });
  return <ProductView product={product} />;
}

// Deduplicate within a request using cache()
import { cache } from "react";
export const getSession = cache(async () => auth());
```

### Stale Data Triage

Next.js 16 is **dynamic by default — no hidden or implicit caching.** A route only serves stale
data when something explicitly opted into caching. Work through this:

```
Data not updating?
├── Server Component with fetch()?
│   └── fetch() is NOT cached by default in Next.js 16 — check for:
│       ├── { next: { revalidate: N } } or { cache: "force-cache" } on the call
│       └── an ancestor route/function wrapped in 'use cache'
│           Fix: remove the cache config, or set { next: { revalidate: 0 } }
├── Route wrapped in 'use cache', or calling a function that is?
│   └── 'use cache' holds the result until revalidateTag()/revalidatePath(), or its
│       own revalidate window, fires
│       Fix: revalidateTag("your-tag") after the mutation, or shorten `revalidate`
├── tRPC query returning old data?
│   └── React Query staleTime or gcTime too high
│       Fix: staleTime: 0 for real-time data
├── Mutated data not reflected?
│   └── Missing revalidation after Server Action
│       Fix: revalidatePath() or revalidateTag() after mutation
└── Page not updating on Vercel specifically?
    └── Only relevant if the route opted into 'use cache' with a revalidate window
        Fix: shorten the window, or revalidateTag() the route's cache
```

## Streaming + Suspense

- Wrap slow data in `<Suspense fallback={<Skeleton />}>` — never block the full page on slow queries
- `loading.tsx` is an implicit Suspense boundary for the entire route segment
- Never `await` independent fetches sequentially — fan out with `Promise.all()`
- Use the `use()` hook (React 19) to consume a promise passed from a Server Component into a Client Component

```typescript
// Parallel fetches — not sequential
const [user, orders] = await Promise.all([getUser(id), getOrders(id)]);

// Streaming slow data
export default function Page() {
  return (
    <>
      <FastSection />
      <Suspense fallback={<OrdersSkeleton />}>
        <SlowOrdersSection />
      </Suspense>
    </>
  );
}

// React 19 use() in Client Component
"use client";
import { use } from "react";
export function UserCard({ userPromise }: { userPromise: Promise<User> }) {
  const user = use(userPromise);  // Suspends until resolved
  return <div>{user.name}</div>;
}
```

### Client-component layouts kill RSC SSR + prefetch (gotcha)

A `"use client"` layout — or one that gates `{children}` behind client-only state (e.g. `const [ctx] = useState(null)` populated in a `useEffect` after a query resolves) — renders a fallback during SSR and **never renders `{children}` server-side**. This silently defeats RSC server rendering AND prefetch for the *entire* subtree below it:

- **Nothing in the subtree appears in the raw SSR HTML.** The layout streams its spinner/fallback; children render only after client hydration + the gate resolves. A `page.request.get()` (or any no-JS fetch) sees only the gate. Asserting "content X is in the initial HTML" for a gated route is structurally impossible.
- **`prefetch` + `HydrateClient` still ship settled data** in the Flight/dehydrated payload (the prefetch itself works — the key hashes correctly, status is `success`), but the island never renders server-side, so the data is invisible until the client mounts it.
- **Hydration is deferred to the client (post-gate), so it races the island.** A `useSuspenseQuery`/`useQuery` observer can subscribe *before* the dehydrated cache hydrates → one stray first-paint refetch, defeating "zero client refetch." `refetchOnMount: false` alone does NOT fully close it; you also need `initialData` (or `staleTime: Infinity`) on the first-paint read.

**Tell:** prefetch looks correct (settled entry in the payload, matching queryKey) yet the feed is absent from raw SSR HTML and refetches once on load — and the same symptom appears on *every* route under that layout, not just one.

**Fix for true RSC SSR + zero-refetch:** make the layout a **server component** — resolve the gating data server-side (e.g. read the scope from a cookie/header in the RSC) so children render and hydrate on the server. If the layout must stay client-gated, the achievable invariant is "prefetch ships settled + island consumes cache with no *extra* network," not "content in raw HTML."

> Evidence: a production RSC-prefetch pilot found an `(auth)/layout.tsx` with `"use client"`
> gating on client-only scope state. Every nested route rendered only a loading fallback in raw
> SSR; the prefetch shipped settled data, but the island never server-rendered and raced hydration.
> Decoding the deployed dehydrated state isolated the client gate as the cause.

## Route Organization

| Pattern | Syntax | Use When |
|---------|--------|----------|
| Route group | `(auth)/login` | Share a layout without adding URL segment |
| Parallel route | `@modal/default.tsx` | Render multiple views in the same layout slot |
| Intercepting route | `(.)photo/[id]` | Modal overlay that shows a page inline |

- **Route groups vs separate layouts**: groups share a layout; separate directories get independent layouts.
- `(.)` intercepts same level, `(..)` parent level, `(...)` root level.

```
app/
  (dashboard)/
    layout.tsx        ← shared dashboard shell
    overview/page.tsx
    settings/page.tsx
  (auth)/
    layout.tsx        ← minimal auth shell
    login/page.tsx
  @modal/
    (.)photo/[id]/page.tsx  ← intercepted modal
```

### Shared route chrome belongs in layout.tsx, not per-page (gotcha)

Tabs, section headers, and filter toolbars that are constant across a group of sibling routes MUST live in the group's `layout.tsx`, wrapping `{children}`. App Router preserves the `layout` instance across intra-group navigation and swaps only the `page` slot — so chrome in the layout stays mounted, while chrome rendered *inside* each `page.tsx` **remounts on every navigation**.

**Tell:** route-based tabs (`<Link>` + `usePathname`) where clicking a tab visibly re-renders the whole surrounding card — header, filter bar, and any queries the toolbar fires flash/refetch — instead of only the content under the tabs. The give-away in code is the same chrome component imported into 2+ sibling `page.tsx` files.

```tsx
// ❌ header remounts on every tab click — it's in the page
// list-view/page.tsx, my-shifts/page.tsx, ... each do:
export default function Page() {
  return <SectionHeader>{/* tab body */}</SectionHeader>;
}

// ✅ header persists; only the body swaps — it's in the layout
// layout.tsx
export default function Layout({ children }: { children: ReactNode }) {
  return <SectionHeader>{children}</SectionHeader>;
}
// list-view/page.tsx → returns only its body
```

A `"use client"` chrome component (needs `usePathname` for the active-tab state) wraps RSC `{children}` in a layout fine — the children still stream server-side. Also trim the group's `loading.tsx` to a **body-only** skeleton: it fills the `page` Suspense slot *inside* the persistent layout, so a header skeleton there double-renders the real header.

> Evidence: in a production route group, a shared section header rendered inside every page,
> causing each tab click to remount the card and re-fire toolbar queries. Moving the shared
> chrome to `layout.tsx` preserved it across sibling-route navigation.

### Async `params` (legacy migration note)

`params` and `searchParams` became a `Promise` in Next.js 15 — still true in 16, but this is two
minors old and no longer the headline migration. Relevant only when touching pre-15 code:

```typescript
// Next.js 14 pattern — breaks from 15 onward
export default function Page({ params }: { params: { id: string } }) {
  return <div>{params.id}</div>;  // TS error: params is Promise
}

// Next.js 15+ — await params
export default async function Page({ params }: { params: Promise<{ id: string }> }) {
  const { id } = await params;
  return <div>{id}</div>;
}

// Client component — use React.use()
"use client";
import { use } from "react";
export default function Page({ params }: { params: Promise<{ id: string }> }) {
  const { id } = use(params);
  return <div>{id}</div>;
}
```

Same applies to `searchParams` in page components and `generateMetadata`.

## Server Actions

`"use server"` at the top of a file is preferred over inline — promotes reuse across components.

```typescript
// app/actions/cart.ts
"use server";
import { z } from "zod";
import { revalidatePath } from "next/cache";

const AddItemSchema = z.object({ productId: z.string(), qty: z.number().min(1) });

export async function addToCart(input: unknown) {
  const parsed = AddItemSchema.safeParse(input);  // ALWAYS validate — client can send anything
  if (!parsed.success) return { error: "Invalid input" };

  await db.insert(schema.cartItem).values({ ...parsed.data, userId: session.user.id });
  revalidatePath("/cart");  // Bust the cache for /cart
  // return { error: "..." } on failure — don't throw (unhandled = 500 page)
}
```

- After mutation: `revalidatePath("/route")` or `revalidateTag("tag")` to invalidate cached data
- Server Actions run on the server even when called from a Client Component
- Return `{ error: string }` for handled failures — only throw for truly unexpected errors

## Caching Gotchas

Next.js 16 is **dynamic by default, with no hidden or implicit *data* caching** — that claim is
about `fetch()` and DB queries, not about rendering mode. The static-vs-dynamic rendering model is
unchanged: a route with no dynamic APIs still prerenders at build time, and `cookies()`,
`headers()`, and `searchParams` still de-opt it to per-request rendering, same as before 16. What
changed is that `fetch()` no longer caches its response implicitly, and `unstable_cache()`/ISR's
implicit page-level cache are both replaced by one explicit primitive: `'use cache'`.

| Behavior | Default | Opt into caching |
|----------|---------|-------------------|
| `fetch()` in Server Component | NOT cached — runs every request | `{ next: { revalidate: N } }`, or wrap the caller in `'use cache'` |
| Route using `cookies()`/`headers()`/`searchParams` | Rendered dynamically, per request | `'use cache'` around the specific data/component that doesn't need the dynamic API |
| Route with no dynamic APIs | Prerendered at build (static) | Already as cached as it gets — add `'use cache'` + a `revalidate` window only to update it on a schedule without a full rebuild |
| DB query via tRPC/Drizzle | Not cached | `'use cache'` around the query function |

**`cookies()`/`headers()`/`searchParams` still de-opt a route to request-time rendering.** That
de-opt is exactly the regression the `instant()` Playwright helper (§ Instant Navigations below;
also see `t3-testing-patterns` skill) exists to catch when a shared header component grows one of
these calls.

```typescript
// Cache an expensive DB query — explicit opt-in, not a global default
import { cacheTag } from "next/cache";

async function getCachedProducts() {
  'use cache';
  cacheTag('products');
  return db.query.product.findMany();
}

// Invalidate from a Server Action after a mutation
import { revalidateTag } from "next/cache";
revalidateTag("products");
```

`'use cache'` can wrap a file, a Server Component, or a plain async function — the directive's
scope is the cache entry's scope. Requires `cacheComponents: true` in `next.config.ts` (see
§ Instant Navigations below).

### revalidatePath vs revalidateTag

| Use | When |
|-----|------|
| `revalidatePath("/products")` | Only `/products` page shows this data |
| `revalidatePath("/", "layout")` | Blow away ALL cached data (nuclear option) |
| `revalidateTag("products")` | Multiple routes display product data (`/products`, `/dashboard`, `/search`) |

**Rule of thumb:** If the data appears on >1 route, use tags. Tags are cheaper (invalidate specific cache entries) vs path (re-renders entire route segment).

## Instant Navigations

Every server `await` on a navigation is now a three-way choice, not just "block until done":

```
Route awaits data on navigation?
├── Show a loading state instantly, stream the rest?
│   └── Stream — wrap in <Suspense fallback={<Skeleton />}>
├── Show previously-cached UI instantly, update in the background?
│   └── Cache — wrap the data/component in 'use cache'
└── Must this navigation wait for fresh server data, every time?
    └── Block — export const instant = false on the route
        (e.g. a blog that must never show a stale post while a new one publishes)
```

Stream and Cache both feel instant/SPA-like to the user; Block is a deliberate, explicit opt-out
— not the default.

### Enable it

```ts
// next.config.ts
const nextConfig: NextConfig = { cacheComponents: true, partialPrefetching: true };
```

- `cacheComponents` turns on the `'use cache'` model (§ Caching Gotchas above).
- `partialPrefetching` extends prefetching from "the whole page" to "the reusable shell" — below.

### Prefetching is per-route, not per-link

Pre-16.3, Next.js sent one prefetch request per `<Link>` in the viewport — a sidebar with twenty
links sent twenty requests. With `partialPrefetching`, Next.js prefetches a reusable **shell**
once per route (one shell for `/chat/[id]`, one for `/dashboard`), cached on the client, instead
of once per link.

This isn't all-or-nothing: `<Link prefetch={true}>` still works and, combined with `'use cache'`,
adds deeper per-link prefetching on top of the route-shell baseline.

```tsx
// Baseline: route shell prefetched automatically, no per-link config needed
<Link href={`/chat/${id}`}>{title}</Link>

// Opt a specific link into deeper prefetching
<Link href={`/chat/${id}`} prefetch={true}>{title}</Link>
```

### Block a route deliberately

```tsx
// A route that must never show a stale/loading shell
export const instant = false;
```

## Error Boundaries

`catchError` (stable as of Next.js 16.3; shipped as `unstable_catchError`/`unstable_retry` in
16.2 — rename only, same shape) is preferred over `error.tsx`'s `reset()` for most recovery
scenarios:

```tsx
import { catchError, type ErrorInfo } from "next/error";

function CustomErrorBoundary(props: { title: string }, { error, retry }: ErrorInfo) {
  return (
    <div>
      <p>{props.title}: {error.message}</p>
      <button onClick={() => retry()}>Try again</button>
    </div>
  );
}
export default catchError(CustomErrorBoundary);
```

- **Framework-aware**: `redirect()`/`notFound()` throw special errors under the hood — `catchError`
  handles those without your boundary accidentally catching them.
- **`retry()` over `reset()`**: `reset()` only clears error state and re-renders children — it
  doesn't help when the error originated in data fetching or the RSC phase. `retry()` calls
  `router.refresh()` and `reset()` inside a transition, re-fetching data and re-rendering the
  segment.
- If you see `unstable_catchError`/`unstable_retry` imported from `next/error`, that's the 16.2
  spelling — rename to `catchError`/`retry` on Next.js ≥ 16.3.

## Root Params

`next/root-params` reads a root-level dynamic segment (e.g. `[lang]`) without prop-drilling it
through every layout and page below it:

```tsx
import { lang } from "next/root-params";

export default async function Page() {
  const locale = await lang();
  // ...
}
```

Server Components only today — no Client Component equivalent yet.

## Better Auth + App Router

```typescript
// Server Component or Server Action — synchronous, no await
import { auth } from "@/lib/auth";
const session = auth();
if (!session) redirect("/login");

// Pass only needed fields to Client Components — never full session
<ClientComponent userId={session.user.id} role={session.user.role} />
```

- **Bulk route protection**: `proxy.ts` — runs at edge before render. Renamed from
  `middleware.ts` in Next.js 16 (file and exported function both renamed; logic is identical —
  see `vercel-platforms` skill § Gotchas for the same rename)
- **Per-route logic**: layout-level `auth()` check with `redirect()`
- Never expose session tokens or full user objects to Client Components — they ship to the browser

```typescript
// proxy.ts — protect all /dashboard/* routes (Next.js 16; was middleware.ts pre-16)
import { NextResponse } from "next/server";
import type { NextRequest } from "next/server";

export function proxy(req: NextRequest) {
  const token = req.cookies.get("session")?.value;
  if (!token && req.nextUrl.pathname.startsWith("/dashboard")) {
    return NextResponse.redirect(new URL("/login", req.url));
  }
}
export const config = { matcher: ["/dashboard/:path*"] };
```

### Auth Strategy Decision

```
Protecting routes?
├── Bulk route protection (all /dashboard/*)?
│   └── proxy.ts — runs at edge, before any rendering
├── Per-page with different logic per role?
│   └── Layout-level auth() check + redirect()
├── Protecting a Server Action?
│   └── auth() check inside the action — proxy.ts doesn't cover actions
└── Protecting an API route handler?
    └── auth() check inside the handler — same as Server Actions
```

## Never Do This

- **NEVER** use `useEffect` for initial data fetching — **WHY:** Server Components fetch data during render with zero client JS. useEffect fetches AFTER hydration, causing a loading flash and doubling time-to-data.
- **NEVER** `await` sequential independent fetches — **WHY:** `const a = await getA(); const b = await getB();` takes `timeA + timeB`. `Promise.all([getA(), getB()])` takes `max(timeA, timeB)`.
- **NEVER** put secrets in Client Components — **WHY:** `"use client"` components ship their entire module to the browser. Environment variables, API keys, and session tokens become visible in the JS bundle.
- **NEVER** use `{ params }` without `await` in Next.js 15+ — **WHY:** `params` is a `Promise`. Accessing `.id` directly gives `undefined` or a TS error. Always `const { id } = await params;` in Server Components or `use(params)` in Client Components.
- **NEVER** assume `fetch()` or a route is cached by default — **WHY:** Next.js 16 is dynamic by default with no hidden caching. Data goes stale only when something explicitly opted in — a `{ next: { revalidate } }` fetch, or a `'use cache'` function/route. Check for those before assuming a cache is the problem.
- **NEVER** use `revalidatePath("/", "layout")` as a default — **WHY:** It invalidates ALL cached data across ALL routes. Use targeted `revalidateTag()` for specific data, or `revalidatePath("/specific-route")` for one page.
- **NEVER** mix `"use server"` and `"use client"` in the same file — **WHY:** A file is either a Server Module or Client Module. Mixing directives is a build error.

## View Transitions

Opt in via `next.config.ts` (experimental flag, Next.js 16+):

```ts
const nextConfig: NextConfig = {
  experimental: { viewTransition: true },
}
```

Then import `ViewTransition` from React 19:

```tsx
import { ViewTransition } from 'react'
```

Animations only fire inside React Transitions (`useTransition`, `<Suspense>`, `useDeferredValue`). Route navigations in Next.js are already Transitions — `<ViewTransition>` activates automatically during `<Link>` clicks.

### Four canonical patterns

| Pattern | How | What it communicates |
|---|---|---|
| Shared element morph | Same `name` prop on both pages | "Same object, different view" |
| Suspense reveal | `exit` on skeleton, `enter` on content | "Data loaded" |
| Directional navigation | `transitionTypes` on `<Link>` + mapped `enter`/`exit` props | "Going forward / going back" |
| Same-route crossfade | Change `key` on `<ViewTransition name="…" share="auto">` | "Same place, different content" |

```tsx
// Shared element morph — wrap matching elements on both pages with same name
<ViewTransition name={`photo-${photo.id}`}>
  <Image src={photo.src} alt={photo.title} />
</ViewTransition>

// Directional navigation — tag links with a type
<Link href={`/photo/${id}`} transitionTypes={['nav-forward']}>…</Link>
<Link href="/" transitionTypes={['nav-back']}>…</Link>

// Map types to CSS class names on the receiving page
<ViewTransition
  enter={{ 'nav-forward': 'nav-forward', 'nav-back': 'nav-back', default: 'none' }}
  exit={{ 'nav-forward': 'nav-forward', 'nav-back': 'nav-back', default: 'none' }}
  default="none"
>
  {/* page content */}
</ViewTransition>
```

CSS targets `::view-transition-old(.class)` / `::view-transition-new(.class)` / `::view-transition-group(.class)`.

**Always add a reduced-motion guard:**

```css
@media (prefers-reduced-motion: reduce) {
  ::view-transition-old(*), ::view-transition-new(*), ::view-transition-group(*) {
    animation-duration: 0s !important;
    animation-delay: 0s !important;
  }
}
```

**Caveats:**
- Browser back/swipe gestures do NOT carry `transitionTypes` — shared element morph still applies
- `useRouter().push()` and `.replace()` also accept `transitionTypes`
- Without browser support the app works normally (progressive enhancement)
- Safari may animate differently for some patterns
- `transitionTypes` is App Router-only — the Pages Router doesn't use React Transitions for
  navigation, so the prop is silently ignored there. A shared `<Link>` wrapper passing
  `transitionTypes` is safe to use across both routers without a router check.

See also: `motion-and-transitions` skill for component-local CSS transitions (modals, dropdowns, badges).

## References

**MANDATORY** — load when debugging stale data or building data-fetching pages:
- [caching-gotchas.md](references/caching-gotchas.md) — extended caching traps with examples

**Do NOT load** for purely client-side component work (forms, modals, state) — use react-dev skill instead.

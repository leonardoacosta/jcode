
# Caching Gotchas — Extended Reference

## The Caching Layers

Next.js 16 is **dynamic by default, with no hidden or implicit *data* caching.** That claim is
about `fetch()` and DB queries — the static-vs-dynamic rendering model is unchanged. A route with
no dynamic APIs still prerenders at build time; `cookies()`, `headers()`, and `searchParams` still
de-opt it to per-request rendering, same as before 16. Once a route or function has opted into
caching, there are two layers left, and both require an explicit opt-in:

| Layer | Scope | Default | Invalidation |
|-------|-------|---------|--------------|
| **React cache()** | Per-request | Deduplicates within single render | Automatic (request-scoped) |
| **fetch() (no cache config)** | Across requests | NOT cached — runs every request | n/a — pass `{ next: { revalidate: N } }` to opt in |
| **`'use cache'`** | Server + client (route/component/function) | NOT cached unless the directive is present; once present, cached until invalidated/expired | `revalidateTag()`, `revalidatePath()`, or the entry's `revalidate` window |
| **Router Cache (client)** | Client browser | Caches instant-navigation shells, not stale application data | Navigation, `router.refresh()`, `revalidatePath()` from a Server Action |

`unstable_cache()` and the old implicit Full Route Cache (ISR-by-default) are both superseded by
`'use cache'` — one primitive replaces both.

## Common Traps

### Trap 1: assuming fetch() is cached (it isn't)

```typescript
// This runs on every single request — nothing is cached
const res = await fetch("https://api.example.com/data");

// Opt in to caching explicitly
const res = await fetch("https://api.example.com/data", { next: { revalidate: 60 } });
```

Defensive `{ cache: "no-store" }` calls left over from a Next.js 15 app are now a no-op — dynamic
is already the default. Harmless, but worth deleting as noise once you've confirmed nothing else
in the call chain wraps the fetch in `'use cache'`.

### Trap 2: a page silently stays cached because of `'use cache'`

```typescript
// Fresh DB query... but cached anyway, because of the directive
async function getProducts() {
  'use cache';
  return db.query.product.findMany();
}

export default async function ProductsPage() {
  const products = await getProducts();     // served from cache, not the DB
  return <ProductList products={products} />;
}

// Fix: shorten or remove the cache window, or invalidate explicitly
import { revalidateTag } from "next/cache";
revalidateTag("products");
```

Unlike Next.js 15's implicit ISR, this trap only exists where a `'use cache'` directive was
written on purpose — grep for `'use cache'` when triaging "why is this stale" against a 16 app;
absence of the directive means the page is not the cause.

### Trap 3: Router Cache shows a stale shell after navigation

```typescript
// User navigates /products -> /products/123 -> back to /products
// Client-cached shell reappears before the fresh server render lands

// Fix 1: In Server Action after mutation
"use server";
export async function deleteProduct(id: string) {
  await db.delete(product).where(eq(product.id, id));
  revalidatePath("/products");  // busts both server and client cache
}

// Fix 2: Client-side forced refresh
router.refresh();  // refetches current route from server
```

### Trap 4: `'use cache'` without a tag has no invalidation path

```typescript
// Cached, but nothing can ever invalidate it by name
async function getProducts() {
  'use cache';
  return db.query.product.findMany();
}

// Fix: tag it so a Server Action can target it
import { cacheTag } from "next/cache";

async function getProducts() {
  'use cache';
  cacheTag("products");
  return db.query.product.findMany();
}

// Invalidate from a Server Action
import { revalidateTag } from "next/cache";
revalidateTag("products");
```

### Trap 5: React.cache() doesn't cache across requests

```typescript
// This only deduplicates within a SINGLE request/render
import { cache } from "react";
const getUser = cache(async (id: string) => fetchUser(id));

// Request 1: getUser("123") -> fetches from DB
// Request 1: getUser("123") -> returns cached (same request!)
// Request 2: getUser("123") -> fetches from DB again (new request!)

// For cross-request caching, wrap in 'use cache' instead:
import { cacheTag } from "next/cache";

async function getUser(id: string) {
  'use cache';
  cacheTag(`user-${id}`);
  return fetchUser(id);
}
```

## Decision: Which Cache to Use

```
Need to cache?
├── Same data used by multiple components in one render?
│   └── React.cache() — per-request dedup, automatic cleanup
├── Same data across multiple user requests?
│   ├── External API with fetch()? -> { next: { revalidate: N, tags: [...] } }
│   └── DB query, page, or component? -> 'use cache' + cacheTag()
└── Data must always be fresh?
    └── Do nothing — dynamic (no cache directive, no cache config) is the Next.js 16 default
```

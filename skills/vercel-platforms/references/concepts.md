
# Multi-Tenant Concepts

> Derived from `vercel.com/docs/platforms/multi-tenant-platforms/concepts` (last_updated
> 2026-06-26).

## Tenants

A tenant is a customer, workspace, or organization inside your application. Each has its own
data, configuration, and branding; all share one codebase and one deployment.

- Blog platform: each writer's blog is a tenant
- Docs platform: each company's docs site is a tenant
- E-commerce platform: each store owner is a tenant

## Tenant identification

Three strategies. They are not exclusive — production platforms usually run custom-domain
lookup first and fall back to subdomain.

**Subdomain-based** — `tenant1.yourapp.com`:

```ts
const hostname = request.headers.get('host');
const subdomain = hostname.split('.')[0]; // "tenant1"
```

Naive: this treats `www` as a tenant. See `routing.md` for a resolver that handles the real
cases (local dev, preview URLs, `www` exclusion).

**Custom domain-based** — `tenant1.com` → Tenant 1:

```ts
const tenant = await db.tenant.findFirst({
  where: { customDomain: hostname },
});
```

**Path-based** — `/tenant1/dashboard`:

```ts
const pathname = request.nextUrl.pathname;
const tenantSlug = pathname.split('/')[1]; // "tenant1"
```

Path-based avoids all DNS and certificate work, which makes it the cheapest option — but tenants
cannot use their own branding at the domain level.

## Data isolation

Multi-tenant apps must keep tenant data separated. There is no infrastructure boundary doing it
for you; it is entirely your discipline.

**Database-level** — tenant ID in every query:

```ts
const posts = await db.post.findMany({
  where: { tenantId: tenant.id },
});
```

**Application-level** — middleware guarantees a request can only reach its own tenant's data.

**Edge Config** — store tenant configuration for fast reads at the edge:

```ts
import { get } from '@vercel/edge-config';

const tenant = await get(`tenant_${hostname}`);
```

### Redis/Upstash tenant store

> **Extension (recon A1, not in upstream docs).** Upstream names Edge Config and
> `db.tenant.findFirst` but gives no concrete key-value schema or read layer. This is the
> `vercel/platforms` starter's store, rewritten to fix two real defects in it.

The client is driven by **Vercel KV env names**, which Upstash also accepts — so the same code
runs against either provider with no changes:

```ts
// lib/redis.ts
import { Redis } from '@upstash/redis';

export const redis = new Redis({
  url: process.env.KV_REST_API_URL!,
  token: process.env.KV_REST_API_TOKEN!,
});
```

Schema — one key per tenant, plus a set used as an index:

```
subdomain:{name}   ->  { emoji, createdAt }   // tenant record
subdomains         ->  SET of tenant names    // index
```

```ts
// lib/tenants.ts
import { redis } from './redis';

type Tenant = { emoji: string; createdAt: number };

const key = (name: string) => `subdomain:${name}`;
const INDEX = 'subdomains';

export async function getTenant(name: string): Promise<Tenant | null> {
  try {
    return await redis.get<Tenant>(key(name));
  } catch (err) {
    console.error('tenant lookup failed', { name, err });
    return null; // fail closed -- caller renders 404, not a 500
  }
}

export async function createTenant(name: string, tenant: Tenant): Promise<boolean> {
  try {
    // NX so a race cannot clobber an existing tenant
    const created = await redis.set(key(name), tenant, { nx: true });
    if (!created) return false;
    await redis.sadd(INDEX, name);
    return true;
  } catch (err) {
    console.error('tenant create failed', { name, err });
    return false;
  }
}

export async function listTenants(): Promise<Array<Tenant & { name: string }>> {
  try {
    const names = await redis.smembers(INDEX);
    if (names.length === 0) return [];
    const records = await redis.mget<Tenant[]>(...names.map(key));
    return names
      .map((name, i) => (records[i] ? { name, ...records[i] } : null))
      .filter((t): t is Tenant & { name: string } => t !== null);
  } catch (err) {
    console.error('tenant list failed', err);
    return [];
  }
}
```

Two deliberate departures from the starter kit:

1. **SET index instead of `keys('subdomain:*')`.** `keys()` is an O(N) scan across the entire
   keyspace and blocks the server on large databases — fine for a demo, not for production. The
   `subdomains` set gives an O(1) membership list, and `mget` still fetches every record in one
   round trip.
2. **try/catch on every Redis call.** The starter wraps none of them, so a Redis outage surfaces
   as a 500 on every page. Failing closed (`null` / `[]`) lets the caller render a 404 or an
   empty state instead.

Keep the defensive default the starter gets right — `emoji ?? '❓'` at the render site — since a
partially-written record should not crash a page.


## Tenant context propagation

Resolve once in middleware, then read downstream from headers.

In middleware — set:

```ts
response.headers.set('x-tenant-id', tenant.id);
```

In server components — read:

```ts
import { headers } from 'next/headers';

const tenantId = headers().get('x-tenant-id');
```

In API routes — read:

```ts
const tenantId = request.headers.get('x-tenant-id');
```

## Request flow

1. User visits `tenant1.yourapp.com`
2. Request hits Vercel's edge network
3. Middleware extracts the subdomain (`tenant1`)
4. Middleware looks the tenant up in a database or Edge Config
5. Middleware attaches tenant context to request headers
6. The page component reads the tenant from headers
7. The page renders tenant-specific data

## Performance

- **Edge Config** for tenant configuration — sub-10ms lookups at the edge.
- **Cache tenant lookups** in middleware so you are not hitting the database per request.
- **Connection pooling** for database access, since every tenant shares the same pool.

## Architecture

One codebase, one deployment, many domains (subdomains plus customer custom domains), shared
infrastructure, tenant-aware routing and data access.

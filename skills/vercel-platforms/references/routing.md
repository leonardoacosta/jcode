
# Routing

> Derived from `vercel.com/docs/platforms/multi-tenant-platforms/middleware-and-routing`,
> `/custom-subpaths`, `/preview-url-prefixes`, and `/examples/multi-tenant-template`
> (last_updated 2026-06-26).

## File convention: `middleware.ts` vs `proxy.ts`

Next.js 16 renames the middleware convention — the file becomes `proxy.ts` and the export
becomes `proxy`:

```ts
// Next 16
export async function proxy(request: NextRequest) { /* ... */ }
```

```ts
// Next 15 and earlier
export async function middleware(request: NextRequest) { /* ... */ }
```

The logic is unchanged. Vercel's docs are mid-migration and inconsistent between pages, so check
your Next major rather than trusting any single snippet. Examples below use `middleware` for
consistency with the majority of upstream pages.

## Subdomain resolution

Extract tenant identity from `tenant1.yourapp.com`:

```ts
// middleware.ts
import { NextRequest, NextResponse } from 'next/server';

export async function middleware(request: NextRequest) {
  const hostname = request.headers.get('host') || '';

  const subdomain = hostname.split('.')[0];

  // Skip processing for main app domains
  if (subdomain === 'www' || subdomain === 'app' || subdomain === 'admin') {
    return NextResponse.next();
  }

  const tenant = await validateTenant(subdomain);

  if (!tenant) {
    return NextResponse.redirect(new URL('/not-found', request.url));
  }

  const response = NextResponse.next();
  response.headers.set('x-tenant-id', tenant.id);
  response.headers.set('x-tenant-subdomain', subdomain);

  return response;
}

export const config = {
  matcher: ['/((?!api|_next/static|_next/image|favicon.ico).*)'],
};
```

#### Matcher variant: exclude every dotted file

> **Correction (recon A3, not in upstream docs).** The matcher above enumerates specific
> exclusions, so any *other* dotted path — `robots.txt`, `sitemap.xml`, `.well-known/*`,
> `manifest.webmanifest` — still enters tenant resolution and gets rewritten. The
> `vercel/platforms` starter uses one regex that excludes any file with an extension:

```ts
export const config = {
  matcher: ['/((?!api|_next|[\\w-]+\\.\\w+).*)'],
};
```

Why: `[\w-]+\.\w+` matches any `name.ext` segment, so static-looking requests skip middleware
without you maintaining a filename allowlist. Use the enumerated form only when you deliberately
want a dotted path (a per-tenant `robots.txt`) to reach the resolver — see § Serving per-tenant
static files, which is exactly that case.

### Battle-tested subdomain extractor

Handles local dev, Vercel preview URLs, and production in one function:

```ts
function extractSubdomain(request: NextRequest): string | null {
  const url = request.url;
  const host = request.headers.get('host') || '';
  const hostname = host.split(':')[0];

  // Local development
  if (url.includes('localhost') || url.includes('127.0.0.1')) {
    const fullUrlMatch = url.match(/http:\/\/([^.]+)\.localhost/);
    if (fullUrlMatch && fullUrlMatch[1]) {
      return fullUrlMatch[1];
    }
    if (hostname.includes('.localhost')) {
      return hostname.split('.')[0];
    }
    return null;
  }

  const rootDomainFormatted = rootDomain.split(':')[0];

  // Preview deployment URLs (tenant---branch-name.vercel.app)
  if (hostname.includes('---') && hostname.endsWith('.vercel.app')) {
    const parts = hostname.split('---');
    return parts.length > 0 ? parts[0] : null;
  }

  // Regular subdomain detection
  const isSubdomain =
    hostname !== rootDomainFormatted &&
    hostname !== `www.${rootDomainFormatted}` &&
    hostname.endsWith(`.${rootDomainFormatted}`);

  return isSubdomain ? hostname.replace(`.${rootDomainFormatted}`, '') : null;
}
```

Note the explicit `www` exclusion — without it, `www` resolves as a tenant name.

## Custom domain resolution

Check registered custom domains before falling back to subdomains. Edge Config keeps the lookup
off the database hot path:

```ts
import { get } from '@vercel/edge-config';

export async function middleware(request: NextRequest) {
  const hostname = request.headers.get('host') || '';

  const customDomainTenant = await get(`domain_${hostname}`);
  if (customDomainTenant) {
    const response = NextResponse.next();
    response.headers.set('x-tenant-id', customDomainTenant.id);
    response.headers.set('x-tenant-type', 'custom-domain');
    return response;
  }

  const subdomain = hostname.split('.')[0];
  const subdomainTenant = await get(`subdomain_${subdomain}`);
  if (subdomainTenant) {
    const response = NextResponse.next();
    response.headers.set('x-tenant-id', subdomainTenant.id);
    response.headers.set('x-tenant-type', 'subdomain');
    return response;
  }

  return NextResponse.redirect(new URL('/not-found', request.url));
}
```

## Path-based resolution

Extract the tenant from the first path segment and rewrite it away, so the app sees a clean path:

```ts
export async function middleware(request: NextRequest) {
  const { pathname } = request.nextUrl;
  const pathSegments = pathname.split('/');
  const tenantSlug = pathSegments[1];

  if (!tenantSlug || tenantSlug.startsWith('_')) {
    return NextResponse.next();
  }

  const tenant = await validateTenant(tenantSlug);
  if (!tenant) {
    return NextResponse.redirect(new URL('/not-found', request.url));
  }

  const newPath = `/${pathSegments.slice(2).join('/')}`;
  const response = NextResponse.rewrite(new URL(newPath, request.url));
  response.headers.set('x-tenant-id', tenant.id);
  response.headers.set('x-tenant-slug', tenantSlug);
  return response;
}
```

## Platforms Starter Kit pattern

The official starter (`vercel/platforms`) resolves the subdomain, then routes tenant traffic to
an internal `/s/{subdomain}` segment while leaving the marketing root and admin surface alone:

```ts
export async function middleware(request: NextRequest) {
  const { pathname } = request.nextUrl;
  const subdomain = extractSubdomain(request);

  if (subdomain) {
    return NextResponse.rewrite(new URL(`/s/${subdomain}${pathname}`, request.url));
  }

  return NextResponse.next();
}
```

Route-shape variants across the docs: `/s/[subdomain]` (starter kit), `/domains/[domain]`
(multi-tenant template), `/tenant/[id]` (middleware-and-routing page). Only the folder name
differs; pick one and stay consistent.

### Rewrite vs redirect

> **Correction (recon A3, not called out upstream).** These are different operations with
> different user-visible results, and the starter kit deliberately uses one of each.

- **Tenant root → rewrite.** `tenant.domain.com/` serves `/s/tenant` *internally*; the URL bar
  keeps showing `tenant.domain.com/`. The tenant never sees your internal route shape. Using a
  redirect here would expose `/s/tenant` in the address bar and break the white-label illusion.
- **Admin on a subdomain → redirect.** A request for `tenant.domain.com/admin` is bounced to the
  root domain's admin. The admin surface has exactly one canonical home; rewriting it would serve
  the same panel from every tenant hostname, giving you N URLs for one resource — bad for
  sessions, cookies, and SEO.

```ts
const subdomain = extractSubdomain(request);

if (subdomain) {
  // admin never lives on a tenant subdomain -> bounce to the root domain
  if (pathname.startsWith('/admin')) {
    return NextResponse.redirect(new URL('/admin', `https://${rootDomain}`));
  }
  // tenant content -> rewrite, URL bar unchanged
  return NextResponse.rewrite(new URL(`/s/${subdomain}${pathname}`, request.url));
}
```

Rule of thumb: rewrite when the URL the user typed *is* the right URL and only the internal
resolution differs; redirect when the URL the user typed is the wrong place for that resource.

## Tenant-aware page routing

Send different URL shapes to different tenant-scoped pages:

```ts
if (pathname === '/') {
  const url = request.nextUrl.clone();
  url.pathname = `/tenant/${tenantId}`;
  return NextResponse.rewrite(url);
}

if (pathname.startsWith('/blog')) {
  const url = request.nextUrl.clone();
  url.pathname = `/tenant/${tenantId}/blog${pathname.replace('/blog', '')}`;
  return NextResponse.rewrite(url);
}
```

## Custom subpaths on a customer domain

Serve platform content under a path on the customer's own domain (`customer.com/docs/*` →
your `/sites/customer-slug/*`):

```ts
export async function proxy(request: NextRequest) {
  const { pathname } = request.nextUrl;

  if (pathname.startsWith('/docs/')) {
    const targetPath = pathname.replace('/docs/', '/sites/customer-slug/');
    return NextResponse.rewrite(new URL(`https://yourapp.com${targetPath}`));
  }

  if (pathname.startsWith('/your-platform-assets/')) {
    return NextResponse.rewrite(new URL(`https://yourapp.com${pathname}`));
  }

  return NextResponse.next();
}

export const config = {
  matcher: ['/docs/:path*', '/your-platform-assets/:path*'],
};
```

Pair it with a unique `assetPrefix` so your `_next` assets do not collide with the customer's:

```js
/** @type {import('next').NextConfig} */
const nextConfig = {
  assetPrefix: '/your-platform-assets',
  async rewrites() {
    return [
      { source: '/your-platform-assets/_next/:path*', destination: '/_next/:path*' },
    ];
  },
};
```

## Preview URL prefixes

Test a tenant's experience on a preview deployment without assigning it a domain. Any prefix
before `---` routes to the same deployment while your code receives the full hostname:

```
{prefix}---{preview-url}
```

| URL | Routes to | Your code receives |
| --- | --- | --- |
| `acme---preview-xyz.vercel.dev` | `preview-xyz.vercel.dev` | `host: acme---preview-xyz.vercel.dev` |
| `globex---my-app-git-feature.vercel.dev` | `my-app-git-feature.vercel.dev` | `host: globex---my-app-git-feature.vercel.dev` |

```ts
import { getSubdomain } from 'tldts';

export async function middleware(request: NextRequest) {
  const hostname = request.headers.get('host') || request.nextUrl.hostname;
  const subdomain = getSubdomain(hostname) || '';
  const [tenantPart] = subdomain.includes('---') ? subdomain.split('---') : [];

  if (!tenantPart) return NextResponse.next();

  const url = request.nextUrl.clone();
  url.pathname = `/${tenantPart}${url.pathname}`;
  return NextResponse.rewrite(url);
}
```

Limitations:

- Only works with a custom deployment suffix, never the default `.vercel.app` (see
  `domains.md` § Plan gates — it is a billing add-on).
- The prefix must appear before `---`.
- Total hostname length must stay within the DNS limit of 253 characters.

## Local development

> **Correction (recon A3).** Upstream docs prescribe adding `127.0.0.1 tenant1.localhost` entries
> to your hosts file. That is unnecessary for browser testing: **browsers resolve `*.localhost`
> natively** — every subdomain of `localhost` is required to resolve to the loopback address
> (RFC 6761), and Chrome, Firefox, and Safari all implement this. Just open
> `http://tenant1.localhost:3000` and it works, with no hosts-file editing and no sudo.

Hosts-file entries are still needed for **non-browser clients** — `curl`, Node `fetch`, Docker
containers, and most HTTP libraries do not implement the `*.localhost` rule and will fail to
resolve. Add entries only for the tenants those clients need:

```text
127.0.0.1 tenant1.localhost
127.0.0.1 tenant2.localhost
127.0.0.1 custom.localhost
```

The `extractSubdomain` helper above already handles both paths — it matches the subdomain out of
the full URL first, then falls back to the `host` header.

## Serving per-tenant static files

`robots.txt`, `sitemap.xml`, and similar need to resolve per tenant — use a route handler, not a
static file:

```ts
export async function GET(
  request: NextRequest,
  { params }: { params: { domain: string } },
) {
  const { domain } = params;
  const tenant = await getTenant(domain);

  if (!tenant) {
    return new NextResponse('Not found', { status: 404 });
  }

  const content = `User-agent: *
Allow: /
Sitemap: https://${tenant.domain}/sitemap.xml`;

  return new NextResponse(content, {
    headers: {
      'Content-Type': 'text/plain',
      'Cache-Control': 'public, max-age=3600',
    },
  });
}
```

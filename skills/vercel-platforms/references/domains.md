
# Domains

> Derived from `vercel.com/docs/platforms/multi-tenant-platforms/concepts` § Domains,
> `vercel.com/docs/domains/*`, and `vercel.com/docs/rest-api/*` domain endpoints
> (last_updated 2026-06-26; plan-gate detail re-verified 2026-07-25).

## Wildcard domains

Serve every subdomain from one project.

- Add `*.yourapp.com` to the project
- Point the apex domain at Vercel's nameservers
- Any subdomain (`tenant1.yourapp.com`, …) routes to your app automatically
- Vercel issues SSL per subdomain on the fly

**Hard requirement**: Vercel's nameservers — `ns1.vercel-dns.com`, `ns2.vercel-dns.com`. A CNAME
record is not sufficient for wildcards; wildcard certificate issuance needs nameserver control.
This is the most common setup failure on multi-tenant projects, and the README of the official
starter kit omits it.

## Customer custom domains

Letting a tenant bring `tenant1.com`:

1. Add the domain to your Vercel project via the SDK
2. The tenant configures DNS (CNAME, or delegate nameservers)
3. Verify ownership via a TXT record
4. Vercel issues the certificate automatically

```ts
import { Vercel } from '@vercel/sdk';

const vercel = new Vercel({ bearerToken: process.env.VERCEL_TOKEN });

await vercel.projects.addProjectDomain({
  idOrName: projectId,
  requestBody: { name: 'tenant1.com' },
});
```

The response carries `verified` plus a `verification` array of challenges — one must be
completed before the domain serves. Poll or re-check with:

```ts
const config = await vercel.domains.getDomainConfig({ domain: 'tenant1.com' });
// config.misconfigured tells you whether DNS is still wrong
```

Adding a subdomain with a redirect to the apex, once the apex verifies:

```ts
await vercel.projects.addProjectDomain({
  idOrName: projectName,
  requestBody: {
    name: 'hello.example.com',
    redirect: 'https://example.com',
    redirectStatusCode: 301,
  },
});
```

## SSL

Issued automatically via Let's Encrypt. Wildcard domains get one wildcard certificate covering
all subdomains; custom domains get an individual certificate each. Renewal is automatic, no
configuration.

## Plan gates

- **Custom domains require a paid plan.** On Hobby, the API returns
  `{"error":{"code":"custom_domain_needs_upgrade","message":"Domain name creation requires a
  premium account."}}`. Design bring-your-own-domain flows around this.
- **Preview Deployment Suffix** (needed for per-tenant preview URLs) is a billing add-on —
  included and enabled by default on Enterprise, enable-able on Pro under Settings → Billing →
  Add-Ons. The suffix domain must be active in the same team and use Vercel nameservers, and
  needs an active wildcard certificate. The reliable way to satisfy all three is to also attach
  that domain to a project in the same team serving a single `index.html`.

Vercel does not publish a fixed per-project domain cap; the practical limits you will meet are
API pagination defaults (`GET /v5/domains` returns 20 without a `limit`; `vercel domains ls`
caps at 100, search at 200).

## Relevant REST endpoints

| Endpoint | Purpose |
| --- | --- |
| `POST /v10/projects/{idOrName}/domains` | Add a domain to a project |
| `GET /v9/projects/{idOrName}/domains/{domain}` | Get a project domain (incl. `verified`, `verification[]`) |
| `GET /v6/domains/{domain}/config` | Get DNS configuration / `misconfigured` |
| `GET /v9/domains/{domain}/verification` | Get the verification record |
| `POST /v9/domains/{domain}/claim` | Claim domain ownership |
| `GET /v1/domains/{domain}/project-domains` | List project domains under an apex |
| `DELETE /v6/domains/{domain}` | Remove a domain |

## CLI equivalents

```bash
vercel domains add example.com
vercel domains add example.com --force      # reassign from another project
vercel dns add [domain] [subdomain] CNAME [value]
vercel alias set [deployment-url] [custom-domain]
```

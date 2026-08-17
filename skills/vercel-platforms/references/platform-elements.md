
# Platform Elements

> Derived from `vercel.com/docs/platforms/platform-elements`, `/actions`, `/blocks` and their
> child pages (last_updated 2026-06-26).

Prebuilt building blocks for platform flows, so you do not hand-roll the Vercel API integration
for common tasks. Two families:

- **Actions** — server-side functions wrapping Vercel API calls
- **Blocks** — UI components for platform flows

Both are installed into your app via the Platform Elements installer rather than consumed as a
runtime dependency, so the code lands in your repo and stays yours to edit.

## Actions

| Action | Purpose |
| --- | --- |
| `add-custom-domain` | Programmatically add a custom domain to a project and check its configuration status |
| `deploy-files` | Deploy files to a Vercel project on behalf of a user |

### add-custom-domain

Wraps the domain-add plus status-check cycle: adds the domain to the project, then reports
verification and DNS-configuration state so your UI can tell the tenant what to fix. Pairs with
the **Custom domain** and **DNS table** blocks — the action does the work, the blocks render the
state.

Underlying API surface is `POST /v10/projects/{idOrName}/domains` plus
`GET /v6/domains/{domain}/config`; see `domains.md` for the raw calls and the
`custom_domain_needs_upgrade` plan gate.

### deploy-files

Deploys a set of files into a tenant's project — the multi-project provisioning path in
`multi-project.md`, packaged. Accepts a custom deployment name so the deployment is identifiable
in the dashboard:

```tsx
deployFiles(arg.paths, {
  projectId: arg.projectId ?? undefined,
  deploymentName: 'your-custom-deployment-name',
});
```

## Blocks

| Block | Purpose |
| --- | --- |
| `claim-deployment` | Let users take ownership of deployments your platform created for them |
| `custom-domain` | UI for adding and verifying a tenant's custom domain |
| `deploy-popover` | Popover showing deployment progress and status |
| `dns-table` | Display the DNS records a tenant must configure |
| `report-abuse` | Flow for users to report abusive tenant sites |

### Choosing blocks

- **Bring-your-own-domain flow** → `custom-domain` + `dns-table`, backed by the
  `add-custom-domain` action. This is the highest-value combination; domain verification UX is
  fiddly and error-prone to build from scratch.
- **AI/generated-app platform** → `deploy-popover` for build feedback, `claim-deployment` so
  users can take the deployment into their own Vercel account.
- **User-generated content at any scale** → `report-abuse`. If tenants can publish public sites
  under your apex domain, abuse reporting is not optional — your domain's reputation is shared
  across every tenant.

## Where this fits

Platform Elements are additive convenience over the SDK, not a separate API. Anything a block or
action does can be done directly with `@vercel/sdk` (see `domains.md` and `multi-project.md`).
Reach for Elements when you want the flow and its edge cases handled; reach for the SDK when you
need control over the exact call sequence.

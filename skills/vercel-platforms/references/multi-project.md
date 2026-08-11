
# Multi-Project Platforms

> Derived from `vercel.com/docs/platforms/multi-project-platforms/{concepts,quickstart,reference}`
> and `vercel.com/docs/projects/managing-projects` (last_updated 2026-06-26).

One Vercel project per tenant. Use this when tenants ship their own code — not when they merely
have their own content (that is multi-tenant; see `concepts.md`).

## When to choose it

**Multi-project when**:

- Tenants deploy their own code
- Each tenant needs custom functionality
- Complete isolation is required (security, compliance)
- AI agents generate and deploy code per user
- Users create applications from templates

**Multi-tenant when**:

- All tenants run the same application
- Content differs but code does not
- You want to deploy once and update every tenant
- You want lower operational overhead

The canonical multi-project shape is an AI app-builder (v0, Orchids, Spawn): the generated code
differs per user, so a shared deployment cannot serve them.

## What a project buys you

- Unique project ID
- Independent configuration and environment variables
- Separate deployment history
- Isolated builds and functions

The cost is management overhead that scales linearly with tenant count — N projects to monitor,
N sets of env vars, N deployment lifecycles.

## Programmatic project creation

```ts
import { Vercel } from '@vercel/sdk';

const vercel = new Vercel({ bearerToken: process.env.VERCEL_TOKEN });

const { value: project } = await vercel.projects.createProject({
  teamId: 'team_1234',
  requestBody: {
    name: `tenant-${tenantId}`,
    framework: 'nextjs',
  },
});
```

### From a template repository

Keeps every tenant project consistent:

```ts
const { value: project } = await vercel.projects.createProject({
  teamId: 'team_1234',
  requestBody: {
    name: `tenant-${tenantId}`,
    gitRepository: {
      type: 'github',
      repo: 'your-org/tenant-template',
    },
  },
});
```

### With environment variables

```ts
await vercel.projects.createProject({
  requestBody: {
    name: `tenant-${tenantId}`,
    framework: 'nextjs',
    environmentVariables: [
      { key: 'TENANT_ID', value: tenantId, target: 'production', type: 'system' },
    ],
    installCommand: 'pnpm install',
    rootDirectory: './',
  },
});
```

REST equivalent: `POST /v11/projects`.

## Deploying on behalf of a user

```ts
const result = await vercel.deployments.createDeployment({
  teamId: 'team_1a2b3c4d5e6f7g8h9i0j1k2l',
  requestBody: {
    name: 'my-instant-deployment',
    project: project.id,
    target: 'production',
    files: [{ file: 'index.js', data: '...' }],
    projectSettings: {
      buildCommand: 'next build',
      installCommand: 'pnpm install',
    },
  },
});
```

Files are uploaded first via `vercel.deployments.uploadFile`, then referenced in the
`files` array. For git-backed tenants, pass `gitSource` instead:

```ts
gitSource: { type: 'github', org: 'vercel', repo: 'next.js', ref: 'main' },
```

## Deployment lifecycle

1. **Create** — deploy code to the project
2. **Build** — Vercel builds it
3. **Preview** — test before promoting
4. **Production** — promote or deploy directly
5. **Rollback** — revert if needed

Promote an existing deployment without rebuilding:

```ts
await vercel.projects.requestPromote({
  projectId: project.id,
  deploymentId: deployment.id,
  teamId: 'team_1234',
});
```

Check status with `vercel.deployments.getDeployment({ idOrUrl })`.

## Per-project domains

Each tenant project carries its own domains:

```ts
await vercel.projects.addProjectDomain({
  idOrName: project.id,
  requestBody: { name: 'tenant1.com' },
});
```

Every project also gets automatic URLs — `project-name.vercel.app` for production and
`project-name-git-branch.vercel.app` for previews. Verification, DNS, and SSL work exactly as in
`domains.md`; the only difference is that the domain attaches to the tenant's project rather than
your single shared one.

## Operational notes

- Deployments and domain adds are API calls against your team — they consume team-level rate
  limits, so batch tenant onboarding rather than looping without backoff.
- Store the returned `project.id` against your tenant record; nearly every later SDK call keys
  off it rather than the name.
- Token scope matters: the bearer token must belong to (or be authorized for) the team named in
  `teamId`.

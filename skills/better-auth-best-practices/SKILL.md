---
name: better-auth-best-practices
description: "Better Auth (TypeScript auth framework) integration gotchas and config reference — VERCEL_ENV vs NODE_ENV rate-limit misconfiguration causing e2e auth flakes, session-shape cross-package type leaks, beta-pin supply-chain risk, model-vs-table-name mismatches, cookie-cache staleness. Trigger on: better-auth, betterAuth(), auth.ts config, rate limit / auth flake in e2e, session shape leaking to client, @better-auth/* version pin, passkey/magicLink/twoFactor plugin setup."
source: ~/.agents/skills@2026-07-13
license: MIT
---


# Better Auth

TypeScript-first, framework-agnostic auth (email/password, OAuth, magic links, passkeys, plugins).
**Always consult [better-auth.com/docs](https://better-auth.com/docs)** for current API surface —
this file exists for the failures the docs don't warn you about, not as a docs mirror.

---

## Hard-Won Gotchas (fix these before they bite in review or CI)

### 1. Rate limiter fires on Vercel preview/dev, not just production (e2e-flake root cause)

Better Auth's rate limiter defaults to `enabled: isProduction`, where `isProduction` reads
`NODE_ENV === "production"` — but **Vercel sets `NODE_ENV=production` on every deployment**,
preview and dev included, not just the real production one. Left at the default, a preview
deploy silently inherits the strict `/sign-in*`/`/sign-up*` cap (3 req/10s). A Playwright suite
running parallel workers against a preview deploy bursts past that the moment two workers
authenticate close together — surfacing as auth timeouts that look like "the deployment can't
handle concurrency" when the real cause is a misscoped rate limiter.

**Fix:** scope on `VERCEL_ENV`, not `NODE_ENV`:
```typescript
rateLimit: { enabled: process.env.VERCEL_ENV === "production" }
```
Full fix (including the `*.config.*` ESLint-exemption placement footgun for the bare
`process.env` read) is in `t3-code-patterns` § Better Auth Rate Limiting — load that skill for
the worked example; this entry exists so you recognize the symptom (e2e auth flakes under
concurrency, not on solo runs) and know where to go.

### 2. Raw session object returned to the client is a cross-package type leak

A router procedure that does `return ctx.session` (or `getSession` echoing `ctx.session` as-is)
ships Better Auth's full internal session shape — token timestamps, IP, etc. — over
`RouterOutputs`. The frontend now depends on Better Auth's internal types; upgrading the library
breaks UI type inference with no warning at the auth layer, only at the UI's consumption site.
**Fix:** define an app-owned `SessionDTO` in the router and narrow before returning
(`return { user: dto(ctx.session.user) }`). Never let `ctx.session` cross the router boundary
unshaped — this is the general "narrow at the package boundary" rule, applied to auth
specifically because the session object is the one place every router touches.

### 3. A `@better-auth/*` beta pin is a supply-chain risk, not a stable dependency

If `pnpm-workspace.yaml` (or any lockfile) pins `better-auth`/`@better-auth/*` packages to a
`-beta.N` version, that's either a deliberate mid-migration pin (fine, should have a tracking
note) or accidental drift that never got promoted to stable (a real risk — beta APIs change
without the same compat guarantees). Flag a beta pin in review; don't assume it's intentional
just because it's already in the lockfile.

### 4. Config uses the ORM model name, not the underlying table name

If your Prisma/Drizzle model is `User` mapped to table `users`, Better Auth config wants
`modelName: "user"` (the ORM reference), not `"users"` (the table). Passing the table name here
fails silently in some adapter paths rather than erroring — verify against the adapter's actual
model registration, not by guessing from the DB schema.

### 5. Cookie-cache is a read-through cache — custom session fields are NEVER cached

Any `session.additionalFields`/custom field you add is always re-fetched from the DB (or
`secondaryStorage`) on every request; only the built-in fields get the cookie-cache speedup. If
you're debugging "why is this custom session field always fresh but slow," this is why — it's
working as designed, not a caching bug to chase.

### 6. Plugin schema changes require a CLI re-run — a stale schema fails at write-time, not config-time

Adding/removing a plugin changes the DB schema Better Auth expects. `npx @better-auth/cli@latest
generate` (or `migrate` for the built-in adapter) must re-run after every plugin change — the
app boots fine with a stale schema and only fails the first time a plugin-specific write hits a
missing column/table, which reads like an unrelated runtime bug if you don't know to check this
first.

---

## Quick Reference

**Env vars:** `BETTER_AUTH_SECRET` (32+ chars, `openssl rand -base64 32`), `BETTER_AUTH_URL`
(base URL). Only set `baseURL`/`secret` in config if the env vars are absent.

**CLI:** `npx @better-auth/cli@latest migrate` (built-in adapter) / `generate` (Prisma/Drizzle
schema) — re-run after any plugin change (see Gotcha 6). Looks for `auth.ts` in `./`, `./lib`,
`./utils`, `./src/*`.

**Session storage priority:** `secondaryStorage` (if defined, sessions go there not DB) ->
`session.storeSessionInDatabase: true` to also persist to DB -> no DB + `cookieCache` = fully
stateless (logout on cache expiry).

**Security flags (`advanced`):** `disableCSRFCheck` / `disableOriginCheck` are real security
risks, not convenience toggles — never set true outside a throwaway local repro.

**Plugins:** import from the dedicated path for tree-shaking —
`better-auth/plugins/two-factor`, not the `better-auth/plugins` barrel. Client-side plugins go
in `createAuthClient({ plugins: [...] })` separately from server plugins.

**Type inference:** `typeof auth.$Infer.Session` / `.Session.user`; cross-project client uses
`createAuthClient<typeof auth>()`.

---

## Resources

- [Docs](https://better-auth.com/docs) · [Options Reference](https://better-auth.com/docs/reference/options) · [LLMs.txt](https://better-auth.com/llms.txt) · [GitHub](https://github.com/better-auth/better-auth)
- `t3-code-patterns` skill § Better Auth Rate Limiting — full worked fix for Gotcha 1.

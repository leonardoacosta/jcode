---
name: deploy-and-env
description: >
  Deployment types (Vercel vs Git Hook), environment variable management via Terraform + .env,
  background execution patterns, and notification architecture for T3 Turbo projects. Covers Vercel
  auto-builds, git hook deploys, Terraform-managed env vars, single commit pattern,
  local-only variables, and background execution defaults with --foreground overrides.
  Use when configuring deployments, managing secrets, writing deploy scripts, or understanding
  background execution behavior.
  Triggers on: env vars, environment variables, deploy config, vercel setup, git hook
  deploy, background execution, TTS notification, say_notify, herdr-shepherd, SKIP_DEPLOY, terraform,
  observability, OpenReplay, Grafana, OTLP, Sentry removal, PostHog removal.
user-invocable: false
disable-model-invocation: false
allowed-tools: Read, Glob, Grep, Bash
---

# Deploy, Environment & Background Execution

> Loaded on-demand by devops agents. This body is a router — dense content lives in
> `references/`; read the linked file before editing the thing it covers.
> For core rules: `rules/CORE.md` | For code patterns: `rules/PATTERNS.md`

## Load references

| Working on | Load |
| --- | --- |
| Env var architecture, canonical/blocked/legacy patterns, the `--overload` wrong-DB fix (tc-za5z) | `references/env-vars.md` — **read before editing any with-env/seed/migration script** |
| Spoken-notification pipeline, `say_notify`, herdr-shepherd notify pipe | `references/notifications.md` |
| Migration-on-deploy trap analysis, the one-shot-job correct pattern | `references/migration-on-deploy.md` — **read before wiring a migration into a build/deploy step** |

## Deployment Types

| Type | Trigger | Projects | CI |
|------|---------|----------|----|
| **Vercel** | `git push` → Vercel auto-builds | oo, tc, tl, mv, ss, lv | GitHub Actions + Vercel preview |
| **Git Hook** | `git push` → `pre-push` hook runs deploy script | cl, co, cw | Hook script (e.g. `scripts/deploy/pre-push.sh`) |

## Vercel Projects (oo, tc, tl, mv, ss, lv)

- Push to any branch → Vercel builds a preview deployment
- Push to main → Vercel promotes to production
- Each commit on a push triggers a **separate** Vercel build
- Monitor: `vercel list` or deploy-status statusline
- **Verify a deploy before calling it done**: `vercel inspect <deployment-url>` — confirms
  the promoted build/state actually matches expectations (see `agents/devops/infra-analyst.md`
  for the fuller investigation flow)
- Fix failures: `/ci:gh --fix` (auto-detects deploy failures)
- Env vars: Managed via Vercel dashboard or `vercel env pull .env.local` — full canonical/
  blocked pattern tables + `--overload` fix: `references/env-vars.md`

### Env Var Management

- **Vercel dashboard**: Add/edit env vars per environment (Production, Preview, Development)
- **Local dev**: `vercel env pull .env.local` to sync env vars locally
- **Terraform**: `packages/infra/environments/{dev,prod}/` manages Vercel project env vars
  via `vercel_project_environment_variable` resources (terraform-modules v2.0.0+)
- **GitHub Actions**: Reference secrets directly via `${{ secrets.SECRET_NAME }}`
- **Banned: Vercel Marketplace-managed env vars** (fleet policy, raised 2026-07-09 during
  tc-9o261 Redis purge — tc's `UPSTASH_REDIS_REST_TOKEN` was Marketplace-integration-injected
  and unreadable/unconvertible even to encrypted type, since Vercel — not the project — owns
  the credential lifecycle). For any shared priceless Vercel project: disconnect the
  Marketplace wrapper, obtain direct provider credentials (Upstash, etc.), and manage them as
  normal directly-owned env vars — encrypted via dotenvx once the repo's env-validation
  rollout (`@t3-oss/env-nextjs` + `createEnv`) reaches it. New integrations MUST be provisioned
  direct-from-provider, never via the Vercel Marketplace connect flow.

## Git Hook Projects (cl, co, cw)

- `pre-push` hook on main branch triggers deployment to homelab/server
- Skip with: `SKIP_DEPLOY=1 git push`
- No Vercel involvement
- Env vars: `.env` file on server (gitignored), loaded via `dotenvx run --overload --quiet -f .env --` or `--env-file`

### Docker/Homelab Env Vars

- `.env` file co-located with docker-compose or service
- Docker containers: `--env-file .env` or `env_file:` in compose
- Sensitive values stored in `.env` (gitignored), structure documented in `.env.example`

## Single Commit Pattern

> Canonical: `rules/BEADS.md` § Single Commit Pattern. Every commit pushed = a CI build.

## Environment Variables

All projects use `.env` files (gitignored) as the local source of truth, with Terraform managing
Vercel-hosted env vars for deployed environments. Full architecture diagram, canonical/blocked/
legacy pattern tables, the `--overload` wrong-DB fix (tc-za5z), and the local-only variable list:
`references/env-vars.md` — **read this before editing any with-env script, seed, or migration.**

## Background Execution

Default to background for long-running operations. Use `--foreground` to block.

> **Note:** The `execution:` frontmatter field in command `.md` files is now the canonical source
> for whether a command runs blocking or background. See each command's frontmatter for authoritative
> behavior. The table below covers raw operations outside of commands.

| Operation                            | Default          | Override               |
| ------------------------------------ | ---------------- | ---------------------- |
| `pnpm install`                       | Background       | `--foreground`         |
| `turbo test:e2e`                     | Background       | `--foreground`         |
| `turbo run lint typecheck build test` | Background      | `--foreground`         |
| Spoken notifications (via herdr pipe) | Fire-and-forget  | Never blocks (bounded by timeout) |
| Task agent spawns                    | Foreground       | `run_in_background: true` |

### Notification Architecture (v4)

Claude Code notifications route through herdr-shepherd's notify pipe via the `say_notify` shell
helper — called bare, since `BASH_ENV` preloads it. The pipe synthesizes on the homelab's kokoro
service and plays on the playback host, appending one record per call to its notify board. Full
pipeline diagram, signature, project detection, timeout behavior, and debugging order:
`references/notifications.md`.

Replaced the nexus-agent path in `cc-kokoro-notify-replace-nx-send` (2026-07-26); `nexus-agent`
is being decommissioned by `retire-nexus-agent`.

> **WARNING:** Do NOT use `run_in_background: true` for TTS Bash calls -- it causes an infinite
> notification loop (background task -> CC task-notification -> forced response -> TTS -> repeat).

### When to Use --foreground

- CI/script contexts requiring sequential execution
- Debugging/watching real-time output
- Operations where next step depends on result
- Quality gates that must pass before proceeding

## Quick Reference

| Action | Command |
|--------|---------|
| Check deploy status | Statusline shows icons, or `vercel list` |
| Verify a Vercel deploy | `vercel inspect <deployment-url>` |
| Fix Vercel failure | `/ci:gh --fix` |
| Check runtime logs | `/monitor:logs` |
| Skip git hook deploy | `SKIP_DEPLOY=1 git push` |
| Pull env vars locally | `vercel env pull .env.local` |
| Run with env vars | `dotenvx run --overload --quiet -f .env -- pnpm dev` (`--overload` mandatory — see `references/env-vars.md`) |

## Migration-on-Deploy Anti-Pattern

Codified from tc's pattern (2026-05-17 fleet audit; `tc/vercel.json:64` `DATABASE_SYNC_ENABLED=true`
+ `tc/packages/db/scripts/sync-production-db.ts:342-348` `IGNORE_DB_SYNC_ERRORS=true` escape hatch).

**Rule:** Never run schema migrations on every deploy with silent error suppression. Use one-shot
migration jobs that fail fast and gate the deploy. **Banned config:** `IGNORE_DB_SYNC_ERRORS=true`
in production env — it converts loud migration failures into silent schema corruption.

Full trap analysis (build timeouts, no-rollback-path, build-concurrency races, no human gate) and
the correct one-shot-job pattern (dry-run -> human approval -> migrate -> verify -> deploy ->
`vercel inspect`): `references/migration-on-deploy.md` — **read before wiring a migration into any
build/deploy step.**

## Observability Canon

Fleet observability consolidates onto **Grafana** (hl LGTM stack: Grafana + VictoriaMetrics +
Loki + Tempo + Alloy + Alertmanager) and **OpenReplay** (self-hosted in hl), replacing Sentry,
PostHog, Better Stack, and Vercel Analytics/Speed Insights. Locked 2026-07-09
(`adopt-grafana-openreplay-observability-canon`); infra shipped 2026-07-12 in hl
`deploy-openreplay-and-public-telemetry-ingest`. Deep migrations (Sentry removal, oo's
PostHog product-analytics rebuild) are separate downstream proposals — this section covers
wiring only.

### Recipe 1 — OTel exporter (server metrics/traces/logs)

| Deploy type | `OTEL_EXPORTER_OTLP_ENDPOINT` | Auth |
| --- | --- | --- |
| Vercel (public egress) | `https://otlp.leonardoacosta.dev` | `OTEL_EXPORTER_OTLP_HEADERS=Authorization=Basic <base64(fleet-otlp:PASSWORD)>` — traefik-native basicAuth (not bearer); credential in hl's gitignored `homelab/openreplay/.env.local` |
| systemd/on-host (same Docker network as hl) | `http://localhost:4318` | None — internal path, unauthenticated by design |

Auth is HTTP Basic, not a bearer token — a bearer-style header will 401. `gRPC :4317` is
intentionally not exposed publicly; use OTLP/HTTP.

**Header-padding footgun**: some OTel SDK versions' `OTEL_EXPORTER_OTLP_HEADERS` env-var
parser splits on *every* `=` in the value — not just the `key=value` separator — which
corrupts a base64 Basic-auth value (base64 padding is literally `=` characters). Found
2026-07-11 wiring sj's exporter (OTel JS 0.39.1): the header arrived at the collector
truncated/mangled and every request 401'd until the header was passed as an explicit object
in code instead of via the env var. **Before trusting a stored
`OTEL_EXPORTER_OTLP_HEADERS` value, test it directly**: `curl -H "Authorization: Basic
<the-stored-value>" -X POST https://otlp.leonardoacosta.dev/v1/metrics` — a `200` confirms
the value survived intact; a `401` means the parser (or a copy/paste step) ate a byte.

### Recipe 2 — OpenReplay tracker (session replay + frontend errors + web vitals)

```js
import Tracker from '@openreplay/tracker';
const tracker = new Tracker({
  projectKey: process.env.NEXT_PUBLIC_OPENREPLAY_PROJECT_KEY,
  ingestPoint: 'https://openreplay-ingest.leonardoacosta.dev/ingest',
});
```

Per-app project keys are recorded in hl `docs/openreplay/project-keys.md` — a pointer, never
copy key values into cc. Wire the env var through `env.ts` (`createEnv`, t3-env canon — see
`t3-code-patterns` § Env Validation), not a bare `process.env` read. The OpenReplay dashboard
itself (`https://openreplay.leonardoacosta.dev`) is **tailnet-only** — only the tracker
ingest hostname (`openreplay-ingest.leonardoacosta.dev`) is public.

### Recipe 3 — pino server logs → plain stdout + OTel trace mixin

> **Amended 2026-07-12** (Leo decision, recorded in the wave-plan decisions log). The
> original recipe prescribed a `pino-opentelemetry-transport` worker shipping logs
> directly to Loki over OTLP. That shape is now BANNED — no fleet repo runs it
> successfully, and it fails specifically under Next.js's serverless webpack bundle. ct's
> attempt (`adopt-grafana-openreplay-observability-canon` task 3.3) confirmed the failure
> mode: `pino.transport({ target: "pino-opentelemetry-transport" })` spawns a
> thread-stream worker that needs a resolvable on-disk module path, but under
> `transpilePackages`/webpack bundling that path arrives as a bundled module id (a
> number) instead of a string — `TypeError: The "path" argument must be of type string.
> Received type number (39163)`, `code: 'ERR_INVALID_ARG_TYPE'`, thrown from
> `pino.transport()`/thread-stream inside `.next/server/chunks`. The logger caught the
> attach failure and silently fell back to plain stdout — so ct's logs were never lost,
> but no OTLP delivery ever ran, and nothing surfaced the fallback as an error.

**The proven canon (ss `packages/logger`, reference implementation)**: pino logs to
**plain stdout** in every environment — no worker transport, nothing that can fail to
attach. Trace correlation comes from a `mixin()` that reads the active OTel span's
`traceId`/`spanId` via `@opentelemetry/api`, not from the transport layer:

```ts
// packages/logger/src/index.ts
import pino from "pino";
import { getTraceContext } from "./trace"; // @opentelemetry/api trace.getActiveSpan()

export function createLogger(name: string) {
  return pino({
    name,
    level: env.LOG_LEVEL ?? (isDev ? "debug" : "info"),
    // pino-pretty ONLY in dev — production stays plain JSON stdout, no worker thread
    transport: isDev ? { target: "pino-pretty", options: { colorize: true } } : undefined,
    mixin() {
      return getTraceContext(); // { traceId, spanId } or {} if no active span
    },
  });
}
```

Getting the log lines from Vercel's platform stdout drain into Loki is a **fleet-level
concern, not a per-app one** — one Vercel log drain feeding a single hl receiver
(Alloy/Loki), covering every Vercel repo uniformly, instead of every app independently
attempting (and failing) its own OTLP worker transport. Tracked as `cc-06294` (needs: a
drain endpoint on hl speaking the Vercel log-drain format, auth, Alloy wiring, per-project
drain config). Until that lands, app-side logs are already correct (stdout + trace
correlation) — they simply aren't yet centralized in Loki.

### Anti-Pattern Table

| Anti-pattern | Use instead |
| --- | --- |
| New `@sentry/*` install/init | OTel traces/errors → Grafana Tempo (Recipe 1) |
| New `posthog-js` capture/init | OpenReplay tracker (Recipe 2) — session replay + web vitals cover the same signal |
| New `@logtail/pino` transport | pino plain stdout + OTel trace mixin (Recipe 3); centralized delivery is the fleet log-drain, not a per-app transport |
| A per-app `pino.transport({ target: "pino-opentelemetry-transport" })` (or any pino worker-thread transport) | Plain stdout + `mixin()` trace-context injection (Recipe 3) — worker transports don't survive Next.js serverless bundling (`ERR_INVALID_ARG_TYPE`, ct incident) |
| New `@vercel/analytics` / `@vercel/speed-insights` | OpenReplay tracker (Recipe 2) — covers web vitals + real-user perf |
| Bearer-token header against `otlp.leonardoacosta.dev` | HTTP Basic (`Authorization=Basic <base64>`) — this endpoint is basicAuth, not bearer |
| Trusting a stored `OTEL_EXPORTER_OTLP_HEADERS` value untested | curl it directly first — some SDK versions mangle `=` padding in the header (see footgun note under Recipe 1) |

## Related Skills

- `deploy-detection` — Algorithmic detection of deployment type (Vercel vs Git Hook vs Docker),
  environment mapping, branching strategy detection, and health check URL resolution. Use when
  you need to *discover* how a project deploys; use this skill when you need to *configure or
  operate* a deployment.

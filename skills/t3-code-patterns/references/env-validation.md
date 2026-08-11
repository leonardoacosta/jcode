
# Env Validation (@t3-oss/env) — Deep Dives

Full schema example, placement matrix, and residual-risk detail backing `SKILL.md` § Env Validation.

## `createEnv` schema — full example

`createEnv` from `@t3-oss/env-nextjs` (Next apps) declares a `server` block, a `client` block, and
a `runtimeEnv` map. Client keys MUST be `NEXT_PUBLIC_`-prefixed — the validator rejects any
`client` entry that isn't (they'd never reach the browser bundle otherwise).

```typescript
// apps/nextjs/src/env.ts
import { createEnv } from "@t3-oss/env-nextjs";
import { z } from "zod";

export const env = createEnv({
  server: {
    POSTGRES_URL: z.string().url(),
    STRIPE_SECRET_KEY: z.string().min(1),
  },
  client: {
    NEXT_PUBLIC_APP_URL: z.string().url(), // client keys MUST be NEXT_PUBLIC_-prefixed
  },
  // Next inlines NEXT_PUBLIC_* at build; map each key explicitly.
  runtimeEnv: {
    POSTGRES_URL: process.env.POSTGRES_URL,
    STRIPE_SECRET_KEY: process.env.STRIPE_SECRET_KEY,
    NEXT_PUBLIC_APP_URL: process.env.NEXT_PUBLIC_APP_URL,
  },
});
```

## Placement — full matrix

| Where | env.ts |
|---|---|
| Next app | `apps/nextjs/src/env.ts` |
| Non-Next package that reads env directly (`packages/auth`, `packages/api`) | per-package `src/env.ts` |

A package that reads env directly ships its own `env.ts` — do NOT reach across into the app's
schema.

## Build-time enforcement (next.config) — code

```typescript
// apps/nextjs/next.config.ts
import "./src/env.ts"; // runs createEnv at build time -> bad env fails the build
```

Importing `./src/env.ts` at the top of `next.config` runs validation during `next build` — a
missing or malformed env var **fails the build** instead of the running app.

## Non-Next packages: `@t3-oss/env-core`

Non-Next packages use `createEnv` from `@t3-oss/env-core` (no `NEXT_PUBLIC_`/`client` split —
`server`/`runtimeEnv` only). Same schema-as-source-of-truth contract.

## CI / `skipValidation` caveat

Build-time validation means CI must supply a satisfying env. Either the checked-in `.env.example`
satisfies every schema, OR CI sets `SKIP_ENV_VALIDATION=1` / passes `skipValidation` to `createEnv`
so a build without secrets doesn't fail on schema. Keep `.env.example` in sync with the schemas so
the non-skip path stays green.

## Residual risk: `$(command)` substitution in `.env` — full detail

`dotenvx` executes `$(command)` shell substitution in **unquoted or double-quoted** `.env` values,
and there is **no off-switch**. This is an accepted, documented residual risk — not solved.

- **Mitigation 1** — single-quote any literal value containing `$(`. Single-quoted `.env` values
  never interpolate, so `KEY='literal-$(not-run)'` is inert.
- **Mitigation 2** — the encrypted-`.env` end-state (commit the encrypted `.env`, gitignore
  `.env.keys`) makes every `.env` change PR-reviewable. The threat model shifts from "an invisible
  local file executes code" to "a reviewed diff" — a human sees the `$(...)` in review before it
  lands.

State it plainly: `$(command)` execution is a residual risk of adopting `dotenvx`, mitigated but
not eliminated.

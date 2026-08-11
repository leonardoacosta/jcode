# Environment Variables — Full Reference

> Read this before editing any `with-env` script, seed, or migration. Router: `../SKILL.md`.

All projects use `.env` files (gitignored) as the local source of truth, with Terraform managing
Vercel-hosted env vars for deployed environments.

## Architecture

```
Terraform (packages/infra/)
├── environments/dev/main.tf     <- Dev env vars via Vercel module
├── environments/prod/main.tf    <- Prod env vars via Vercel module
└── terraform-modules v2.0.0     <- Shared Vercel env var resources

Local Development
├── .env                         <- Local secrets (gitignored)
├── .env.example                 <- Template with placeholder values (committed)
└── .env.local                   <- Machine-specific overrides (gitignored)
```

## Canonical vs Blocked Patterns

> `import dotenv` and `dotenv.config()` hand-loading are caught by ESLint (`no-restricted-imports`).
> Env validation canon: see `t3-code-patterns` § Env Validation (@t3-oss/env).

**Canonical (recommended):**

| Pattern                                       | Why                                              |
| ---------------------------------------------- | ------------------------------------------------ |
| `"@dotenvx/dotenvx": "*"` in pkg.json         | Loader of record (replaces `dotenv-cli`)         |
| `dotenvx run --overload --quiet -f .env --`   | With-env form; `--overload` mandatory (see below)|
| `createEnv` from `@t3-oss/env-nextjs`/`-core` | Typed runtime env validation                     |

**Blocked (anti-patterns):**

| Pattern                          | Issue                                 | Fix                                                     |
| --------------------------------- | ------------------------------------- | ------------------------------------------------------ |
| `dotenvx run` **without** `--overload` | Stray global shadows project `.env` | Add `--overload` — see § `--overload` Override below   |
| `dotenv.config()` hand-loading   | Bypasses loader + validation          | Use `dotenvx run --overload ...` + `createEnv`         |
| Bare `process.env.X` reads       | Unvalidated, untyped                  | Read through `createEnv` output (`@t3-oss/env`)        |
| `~/.env` for secrets             | Machine-specific path                 | Use project `.env` file                                |
| Hardcoded secrets in scripts     | Security risk                         | Use env vars from `.env`                               |

**Legacy (being migrated away from — do NOT use in new work):**

| Pattern                          | Replaced by                                            |
| --------------------------------- | ------------------------------------------------------ |
| `"dotenv": "*"` (dotenv-cli)     | `@dotenvx/dotenvx`                                      |
| `dotenv -o -e ../../.env --`     | `dotenvx run --overload --quiet -f ../../.env --`      |
| `dotenv -e .env` (no `-o`)       | `dotenvx run --overload --quiet -f .env --`            |

## `--overload` Override — project `.env` MUST win over a stray global

> Codified from tc (2026-06-01, tc-za5z follow-up). Symptom: the dev server + seeds silently
> connect to the WRONG database and the app renders no data ("nothing shows").

By default a loader does NOT overwrite an env var that is ALREADY set in the process environment —
the pre-existing value wins. On dev machines a global `~/.env` (sourced by the shell — e.g. the
cortex/nexus tooling sets `POSTGRES_URL=postgresql://...@localhost:5436/...`) or any exported shell
var can define keys that then **shadow the project `.env`**. So a with-env form WITHOUT `--overload`
loads the project file but the stray `POSTGRES_URL` already in the environment takes precedence →
app hits the wrong DB.

**Rule:** every `with-env` / seed / migration script MUST pass `--overload` so the project `.env`
is authoritative. `--overload` is the exact `dotenvx` equivalent of the old dotenv-cli `-o` flag —
same tc-za5z wrong-DB protection, restated for the loader of record:

```jsonc
// apps/<app>/package.json  AND  packages/db/package.json
"with-env": "dotenvx run --overload --quiet -f ../../.env --"
// seeds / migrations inherit it via pnpm with-env:
"seed:foo": "pnpm with-env tsx src/seed-foo.ts"
```

`-f a -f b` layers files last-wins if you need a base + override pair. `--quiet` suppresses the
dotenvx banner.

Verify in a shell that has the stray var set:

```bash
pnpm with-env node -e 'console.log(new URL(process.env.POSTGRES_URL).host)'
# MUST print the project DB host, NOT localhost
```

A running dev server caches the bad connection — **RESTART it** after fixing the script.

## Local-Only Variables

Some values are machine-specific and should NOT go in Terraform or Vercel:

| Key                  | Reason               |
| --------------------- | --------------------- |
| `DAEMON_WS_URL`      | localhost WebSocket  |
| `CO_API_URL`         | localhost API        |
| `AUDIO_RECEIVER_URL` | LAN IP addresses     |
| `ROUTER_PASSWORD`    | Home network         |
| `HA_TOKEN`           | Home Assistant       |

These stay in a local `.env.local` (gitignored) or system environment.

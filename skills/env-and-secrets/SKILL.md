---
name: env-and-secrets
description: Portable T3 coding rules for dotenvx, @t3-oss/env, server/client environment boundaries, and secret-safe scripts, tests, logs, and fixtures. Use when adding or changing environment variables, .env files, dotenvx commands, env.ts, migrations, seeds, integration tests, CI environment setup, or secret-bearing configuration in a T3 project.
---

# T3 environment and secrets

This skill owns the application-code boundary. It is intentionally not a vault or
credential-operations guide: use `dotenvx-secrets` for encryption, redaction limits,
credential injection, and vault references; use `secrets-handling` for the universal rules
for discovered credentials and safe reporting.

## One loader boundary per package

Use one package-owned loader command. The canonical shape is:

```json
{
  "scripts": {
    "with-env": "dotenvx run --overload --quiet -f ../../.env --",
    "db:migrate": "pnpm with-env drizzle-kit migrate",
    "seed": "pnpm with-env tsx src/seed.ts"
  }
}
```

- `--overload` is mandatory: the selected project file wins over inherited shell values.
- `--quiet` prevents loader banners from polluting command output.
- `-f` selects the file; repeated `-f` files are last-wins.
- Do not nest dotenv loaders or call `dotenv.config()` inside the program. A package invokes
  its own `with-env` sibling and downstream commands inherit it.

## Validate values where code reads them

`dotenvx` loads values; it does not type or validate them. Every package that reads an
environment variable owns an `env.ts` schema using `@t3-oss/env`:

- Next.js applications use `@t3-oss/env-nextjs` with explicit `server`, `client`, and
  `runtimeEnv` declarations.
- Non-Next packages use `@t3-oss/env-core` with `server` and `runtimeEnv`.
- New variables enter the schema before code reads them. Do not add a bare
  `process.env.MY_VALUE` read outside the schema/config boundary.
- Client variables use the framework's public prefix and are deliberately non-secret.
  A server secret never belongs in the client schema or browser bundle.

Keep `.env.example` synchronized with required schema keys. CI either supplies satisfying
non-secret values or explicitly uses the framework's documented validation escape hatch for
the relevant build; it does not silently bypass validation in production.

## Keep secret material out of code and evidence

- Commit an encrypted application `.env` only when the repository's encryption workflow is
  established. Keep `.env.keys` ignored and never place private keys in source control.
- Never put a secret value in source, command arguments, test fixtures, snapshots, logs,
  screenshots, artifacts, generated reports, or review text.
- Use synthetic values in tests. An integration test needing a real credential must follow
  `dotenvx-secrets` and `secrets-handling`; do not invent a shell, vault, or redaction flow.
- Do not expose a server value through `NEXT_PUBLIC_`, serialized API data, error messages, or
  client-side diagnostic output.

## Review checklist

When reviewing a T3 environment change, verify the loader path and package ownership, schema
entry and server/client placement, `.env.example` parity, test/CI values, and that no literal
secret or `.env.keys` is introduced. For encryption, redaction, vault, service-account, or
live-agent credential questions, stop and load the two `leo-security` skills named above.

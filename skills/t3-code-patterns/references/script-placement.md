
# Script Placement — Deep Dives

Full package shape, wiring template, flag rationale, and worked example backing `SKILL.md`
§ Script Placement (centralized `apps/scripts` canon, 2026-07-06 reversal).

## Package Shape — full

```
apps/scripts/
  package.json      # name @{ws}/scripts; deps: @{ws}/db, @{ws}/api (as needed), tsx; with-env sibling
  tsconfig.json     # extends the repo tsconfig base
  README.md         # inventory table: name -> purpose -> env needs
  src/
    <name>.ts       # one file per script; shared helpers in src/lib/
```

The starter skeleton lives at `skills/t3-code-patterns/templates/scripts-app/` — copy it into
a repo as `apps/scripts/`, substitute `{ws}`, and `pnpm install`.

## Wiring Template — full code

```jsonc
// apps/scripts/package.json — one key per script + a with-env sibling
{
  "name": "@{ws}/scripts",
  "scripts": {
    "with-env": "dotenvx run --overload --quiet -f ../../.env --",
    "with-test-env": "dotenvx run --overload --quiet -f ../../.env -f ../../.env.test --",
    "<name>": "pnpm with-env tsx src/<name>.ts"
  }
}

// <repo>/package.json — root delegation (root-level invocation as `pnpm scripts:<name>`)
{ "scripts": { "scripts:<name>": "pnpm --filter @{ws}/scripts <name> --" } }

// turbo.json — ONLY if another task depends on this script or its output is cacheable
// (e.g. acme's seed:e2e chain that e2e tasks depend on). Not for one-shot ops actions.
{ "tasks": { "<name>": { "cache": false } } }
```

## Env Loader flags — full rationale

The canonical with-env form is `dotenvx run --overload --quiet -f <path> --` (`@dotenvx/dotenvx`
pinned at `^1.34.0`, acme's reference). The package-local `with-env` sibling above owns the flags so
each script key stays terse (`pnpm with-env tsx src/<name>.ts`).

- `--overload` is **mandatory** — the repo `.env` MUST beat already-exported shell vars / a sourced
  `~/.env`. This is exact parity with the old dotenv-cli `-o` flag and preserves the storefront-za5z
  wrong-DB protection. A `dotenvx run` WITHOUT `--overload` is an anti-pattern.
- `--quiet` suppresses the dotenvx stdout banner.
- `-f` (not `-e`) is dotenvx's file flag. Multi-file is last-wins precedence:
  `-f ../../.env -f ../../.env.test` lets `.env.test` override `.env` (the `:local` / test variant).

## Anti-Patterns — full table

| Pattern | Why It Breaks |
|---|---|
| Root `scripts/*.ts` importing `@{ws}/db` schemas | pnpm doesn't hoist workspace internals; tsx resolution fails — move the script into `apps/scripts` |
| NEW domain script in `packages/db/src/scripts/` or `packages/api/src/scripts/` | The OLD co-location pattern — now the migration source, not a destination for new work |
| `createRequire()` to reach package `node_modules` | Fragile — breaks on hoist layout changes; a code smell, not a model. A real workspace package (`apps/scripts`) removes the need |
| `dotenvx run` WITHOUT `--overload` | Shell/`~/.env` vars shadow the repo `.env` — reintroduces the storefront-za5z wrong-DB class. `--overload` is mandatory (parity with the old dotenv-cli `-o`) |
| `dotenv.config()` hand-loaded in script body | Use the package `with-env` prefix — uniform across all scripts, no per-script drift |
| Bare `process.env.X` read for a new env var | New env vars MUST enter via a t3-env `env.ts` schema — see § Env Validation (@t3-oss/env) |
| Inventing a new env var name for `POSTGRES_URL` | Already standard — see § Raw SQL Connection |

## Canonical Example — full walkthrough

`apps/scripts/src/mock-app.ts` — a `@{ws}/db`-importing seeder. Wired inside `@{ws}/scripts`
as `"mock-app": "pnpm with-env tsx src/mock-app.ts"`, delegated from the repo root as
`"scripts:mock-app": "pnpm --filter @{ws}/scripts mock-app --"`, and invoked as `pnpm scripts:mock-app`.
Domain scripts (seeders, backfills, RBAC actions, provisioning) all live under
`apps/scripts/src/` with one file per script and shared helpers in `src/lib/`. Before creating
a new script, check the directory and the README inventory table — extension usually beats creation.

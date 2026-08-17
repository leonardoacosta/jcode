
# @{ws}/scripts

Centralized home for this repo's **domain scripts** — seeders, backfills, mocks, and ops actions
that import workspace internals (`@{ws}/db`, `@{ws}/api`, …). Placement doctrine:
`t3-code-patterns` § Script Placement.

> Root `<repo>/scripts/` stays reserved for pure infra with **zero** workspace imports. e2e suite
> infrastructure stays in `packages/e2e/scripts/`. drizzle migration output stays in
> `packages/db/drizzle/`.

## Adding a script

1. Create `src/<name>.ts` (extend a sibling first — check the inventory table below).
2. Add a package key: `"<name>": "pnpm with-env tsx src/<name>.ts"`.
3. Add a root delegation key in `<repo>/package.json`:
   `"scripts:<name>": "pnpm --filter @{ws}/scripts <name> --"`.
4. Add a turbo task **only if** another task depends on it or its output is cacheable
   (e.g. a `seed:e2e` that e2e tasks depend on). One-shot ops actions skip turbo.
5. Row it in the inventory table below.

Invoke from the repo root: `pnpm scripts:<name>`.

## Env

Scripts load the repo root `.env` via the package `with-env` sibling
(`dotenvx run --overload --quiet -f ../../.env --`). Use `with-test-env` for the `.env.test`
pairing. New env vars enter via the t3-env `env.ts` schema — never a bare `process.env.X` read.

## Inventory

| Script | Purpose | Env needs |
|---|---|---|
| `seed-example` | Placeholder — delete once real scripts land | `POSTGRES_URL` |

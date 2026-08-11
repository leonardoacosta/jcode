# Migration-on-Deploy Anti-Pattern — Full Reference

> Read this before wiring `drizzle-kit migrate` (or any schema migration) into a build/deploy step.
> Router: `../SKILL.md`.

Codified from tc's pattern (2026-05-17 fleet audit). Reference evidence: `tc/vercel.json:64` (`DATABASE_SYNC_ENABLED=true`) + `tc/packages/db/scripts/sync-production-db.ts:342-348` (`IGNORE_DB_SYNC_ERRORS=true` escape hatch).

**Rule:** Never run schema migrations on every deploy with silent error suppression. Use one-shot migration jobs that fail fast and gate the deploy.

## The trap

It looks convenient to wire `drizzle-kit migrate` into the Vercel build step (`vercel.json` → `buildCommand`). For new projects with empty migration history this seems to work. But it compounds in ways that bite later:

| Concern | Why it bites |
|---|---|
| Vercel build timeouts | A long-running ALTER TABLE on a large prod table can exceed Vercel's 45-minute build limit; the deploy fails AFTER the migration ran partially, leaving schema in a half-state |
| Silent failure flag | `IGNORE_DB_SYNC_ERRORS=true` (as in tc) lets failed migrations ship the deploy anyway — schema drifts from code, runtime errors start cascading hours later |
| No rollback path | A "deploy" rollback in Vercel reverts the code but NOT the migration. Now the old code runs against the new schema. |
| Build concurrency | Two concurrent builds (Vercel preview + main) racing the same migration produces lock contention or duplicate migrations |
| No human gate | Schema changes are often the most operationally risky kind — bypassing them via "it'll auto-apply" removes the human review opportunity |

## Correct pattern

```
Pre-deploy step (CI):
  1. Run `pnpm db:migrate --dry-run` to print pending migrations
  2. Require human approval if migration list is non-empty AND prod env
  3. On approval, run `pnpm db:migrate` against prod DB
  4. Verify migration succeeded via post-migration query
  5. ONLY THEN trigger Vercel deploy
  6. After the deploy promotes, verify with `vercel inspect <deployment-url>` —
     confirms the promoted build actually matches the migrated schema before
     calling the deploy done
```

A one-shot job (GitHub Actions workflow_dispatch, or a `pnpm migrate:prod` script the operator runs) is the right shape. The deploy assumes the migration already ran successfully.

**Banned config:** `IGNORE_DB_SYNC_ERRORS=true` in production env. The flag exists to silence development noise; using it in prod converts loud failures into silent corruption.

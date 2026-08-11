
# Database — Deep Dives

Full examples backing `SKILL.md` § Database. Load this when you need the worked example, not just
the rule.

## Raw SQL Column Names — full before/after

Drizzle uses camelCase in TypeScript but generates **snake_case** columns in Postgres. When writing
raw SQL (psql, migrations, ad-hoc queries), ALWAYS use snake_case:

```sql
-- ✅ Correct (Postgres column names)
SELECT * FROM booth_purchase bp
JOIN event e ON bp.event_id = e.id
WHERE bp.event_series_id = '...'

-- ❌ Wrong (Drizzle TypeScript field names)
SELECT * FROM booth_purchase bp
JOIN event e ON bp."eventId" = e.id
WHERE bp."eventSeriesId" = '...'
```

`eventId` → `event_id`, `eventSeriesId` → `event_series_id`, `createdAt` → `created_at`, etc. When
unsure, check the schema: `grep 'text("\|varchar("' packages/db/src/schemas/`.

## Raw SQL Connection — full command

```bash
# ✅ Correct — load .env (--overload so repo .env beats shell/~/.env) and use POSTGRES_URL
dotenvx run --overload --quiet -f .env -- bash -c 'psql "$POSTGRES_URL" -c "SELECT 1"'

# ❌ Wrong — DATABASE_URL does not exist
dotenvx run --overload --quiet -f .env -- bash -c 'psql "$DATABASE_URL" -c "SELECT 1"'
```

`DATABASE_URL` silently resolves to empty, causing psql to attempt a local socket connection and
fail. All T3 projects use `POSTGRES_URL` (Neon convention, set in `.env`).

## Migrations — deploy-side detail

`db:push` / `drizzle-kit push` is state-based live-diff — it mutates the DB without writing the
`drizzle.__drizzle_migrations` journal, can silently drop/alter columns, and collides with the
deploy's `db:migrate` replay ("already exists" drift, the a prior migration incident incident). Engineers never apply
schema directly to a live DB during a task; to test a migration locally, run `db:migrate` against a
throwaway/local DB — never `db:push` against shared/prod. Output dir: `packages/db/drizzle/`.
Enforced by `templates/pre-commit-block-db-push.sh`. Deploy-side replay pattern + the
`IGNORE_DB_SYNC_ERRORS` foot-gun: `deploy-and-env` § Migration-on-Deploy.

---
name: drizzle-best-practices
description: Expert Drizzle ORM patterns for T3 Turbo + PostgreSQL (Neon) monorepos. Covers schema design, query patterns, migration safety, and common gotchas that aren't obvious from the docs.
source: ~/.agents/skills@2026-07-13
user-invocable: false
paths: ["packages/db/**", "**/*.sql"]
---


# Drizzle Best Practices

Non-obvious patterns for Drizzle ORM with PostgreSQL in a T3 Turbo monorepo (`packages/db/src/schemas/`).

---

## Primary Key Strategy

| Type | Use when | Drizzle |
|------|----------|---------|
| `serial` / `bigserial` | Internal tables, no external exposure, high insert volume | `serial('id').primaryKey()` |
| `uuid` | Cross-service IDs, externally visible, need uniqueness across DBs | `uuid('id').defaultRandom().primaryKey()` |
| `cuid2` | User-facing slugs, URL-safe, sortable-ish, no UUID collisions | `text('id').$defaultFn(() => createId()).primaryKey()` |

**cuid2 gotcha**: `.$defaultFn()` is a TypeScript-level default only. It does NOT set a DB-level `DEFAULT`. If you insert via raw SQL (migrations, seeds), you must supply the value.

---

## Relations vs Foreign Keys — They Are Not the Same

```ts
// FK = DB constraint (enforced by Postgres)
userId: uuid('user_id').references(() => users.id, { onDelete: 'cascade' })

// Relations = TypeScript join config (no DB constraint, no enforcement)
export const postsRelations = relations(posts, ({ one }) => ({
  author: one(users, { fields: [posts.userId], references: [users.id] }),
}))
```

**Rule**: Always define both. FK enforces integrity. `relations()` enables `db.query.posts.findMany({ with: { author: true } })`. Omitting `relations()` means `with:` silently returns nothing — no error, just missing data.

---

## Branded Types on Columns

```ts
// Without branding — accepts any string
userId: text('user_id').notNull()

// With branding — TypeScript rejects raw strings
userId: text('user_id').$type<UserId>().notNull()
```

Use `.$type<>()` for IDs, slugs, or any column where you want compile-time type narrowing. The branded type only exists at the TypeScript layer; the DB still stores a plain string.

---

## `notNull()` Does Not Add a DB Default

```ts
// ❌ Misleading — TypeScript says non-nullable but DB has no DEFAULT
status: text('status').notNull()

// ✅ Non-nullable with DB default
status: text('status').notNull().default('active')

// ✅ Non-nullable, caller must always provide
status: text('status').notNull()  // fine if you always insert it
```

**InsertModel reflects this**: `InferInsertModel<typeof posts>` will require `status` because there's no `.default()`. Drizzle's type system matches the DB accurately here.

---

## Insert vs Select Types

```ts
type PostSelect = InferSelectModel<typeof posts>  // all columns present, non-null where defined
type PostInsert = InferInsertModel<typeof posts>  // optional where default exists, required otherwise
```

Columns with `.default()` or `.$defaultFn()` become optional in `PostInsert`. Columns without any default are required. This is intentional — use `PostInsert` for insert payloads, never `PostSelect`.

---

## Query Patterns

### `findFirst` returns `undefined`, not `null`

```ts
const user = await db.query.users.findFirst({ where: eq(users.id, id) })
// user is User | undefined — guard with if (!user) not if (user === null)
```

### `with:` is explicit — no lazy loading

```ts
// ❌ Won't work — Drizzle has no lazy loading
const post = await db.query.posts.findFirst({ where: ... })
post.author  // undefined

// ✅ Explicit eager load
const post = await db.query.posts.findFirst({
  where: ...,
  with: { author: true },
})
```

### N+1 prevention vs perf tradeoff

`with:` on a large relation (e.g., fetching 1000 posts each with 50 comments) generates a single JOIN but returns a massive result set. For large cardinality:
- Use `limit` on the nested relation: `with: { comments: { limit: 5 } }`
- Or fetch the list separately and join in application code

### `.prepare()` for hot paths

```ts
const getUserById = db.query.users
  .findFirst({ where: (u, { eq }) => eq(u.id, sql.placeholder('id')) })
  .prepare('get_user_by_id')
const user = await getUserById.execute({ id: userId })  // skips query parsing on repeat calls
```

### Raw SQL escape hatch

```ts
import { sql } from 'drizzle-orm'
const result = await db.execute(sql`SELECT COUNT(*) FILTER (WHERE active = true) FROM users`)
```

Use `db.execute(sql\`...\`)` for queries only. Never for DDL (use migration files).

---

## Migration Safety

> **Migration-based only — NEVER `db:push` / `drizzle-kit push`.** Schema changes ALWAYS go:
> edit `schema.ts` → `pnpm drizzle-kit generate` (ordered, reviewable `.sql`) → commit the
> migration → the **deploy** applies it via `db:migrate`. `db:push` is a state-based live-diff:
> it skips the `drizzle.__drizzle_migrations` journal, can silently drop/alter columns to
> converge, and collides with the deploy's `db:migrate` file replay → "already exists" drift.
> Test migrations against a throwaway/local DB with `db:migrate`, never `db:push` on shared/prod.

### NEVER rename a column directly

Drizzle generates `DROP COLUMN` + `ADD COLUMN`, not `ALTER TABLE ... RENAME COLUMN`. Data is destroyed.

**Safe rename (multi-deploy)**:
1. Add new column (nullable): `pnpm drizzle-kit generate`
2. Deploy + backfill: `UPDATE table SET new_col = old_col WHERE new_col IS NULL`
3. Update all code to use new column name
4. Deploy code change
5. Drop old column: `pnpm drizzle-kit generate`

### Zero-downtime column additions

| Change | Safe? | Reason |
|--------|-------|--------|
| Add nullable column | Yes | Existing rows get NULL, no constraint violation |
| Add NOT NULL without default | No | Existing rows fail constraint immediately |
| Add NOT NULL with default | Yes | Existing rows get the default value |
| Drop column | No | Old code still references it until deploy |

For NOT NULL without default: add nullable → backfill → add NOT NULL constraint in a separate migration.

### Data migrations

```bash
pnpm drizzle-kit generate --custom --name=backfill-user-slugs
# Edit the generated empty .sql file with your UPDATE/INSERT statements
```

Use `--custom` for data migrations. Never write raw `.sql` files by hand in the drizzle output directory.

---

## Common Gotchas

### `db.query.table` vs `db.select().from(table)`

```ts
// Uses relations config — requires relations() to be defined
db.query.users.findMany({ with: { posts: true } })

// Does NOT use relations — works without them, returns flat rows
db.select().from(users).leftJoin(posts, eq(users.id, posts.userId))
```

They're separate query builders. `db.query.*` is the relational API; `db.select()` is the SQL builder API.

### `ctx.db.query` causes TypeScript recursion

```ts
// ❌ Never — causes TS error: Type instantiation is excessively deep
const users = await ctx.db.query.users.findMany(...)

// ✅ Always import db directly
import { db } from '@workspace/db/client'
const users = await db.query.users.findMany(...)
```

### camelCase ≠ DB column name

Drizzle maps TypeScript camelCase to snake_case in Postgres automatically. Raw SQL must use snake_case (`created_at`, `user_id`), never quoted camelCase (`"createdAt"`, `"userId"`).

### Connection string

Use `POSTGRES_URL`, not `DATABASE_URL`. The latter does not exist in Doppler (Neon convention). Wrong var = silent empty string = psql tries local socket.

---

## NEVER Do These

- `db.execute(sql\`DROP TABLE...\`)` — use migration files, never runtime DDL
- Rename a column in one migration step — multi-step or data is lost
- Assume `.with()` is free — profile joins on high-cardinality relations
- Use `InferSelectModel` for insert payloads — use `InferInsertModel`
- Define `relations()` without the corresponding FK (or vice versa) — keep them paired

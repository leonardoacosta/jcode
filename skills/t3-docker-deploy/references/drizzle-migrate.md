# Drizzle migration discipline (t3-docker)

> The deploy pipeline applies schema changes with `drizzle-kit migrate`, never
> `drizzle-kit push`. This file explains the workflow and the one foot-gun that
> bites every project that mixes the two.

## The workflow: generate locally, migrate on deploy

Schema is code. A change is a reviewable, committed SQL migration — not a live
mutation. The loop:

1. **Edit the schema** in `packages/db/src/schema/*.ts`.
2. **Generate the migration**: `pnpm -F @<scope>/db generate` (wraps
   `drizzle-kit generate`). This diffs the schema against the last snapshot in
   `packages/db/drizzle/meta/` and writes a numbered SQL file
   (`drizzle/0007_<name>.sql`) plus an updated snapshot + `_journal.json`.
3. **Review + commit** the generated `.sql` and the `meta/` changes alongside the
   schema change. The migration is now a durable artifact in git.
4. **Deploy** with `git push` to main. The `pre-push.sh` pipeline (Phase 3) runs
   `pnpm drizzle-kit migrate` against the homelab Postgres, which applies only the
   migrations not yet recorded in the database's `__drizzle_migrations` journal.

Never hand-write SQL migrations — let `generate` produce them so the snapshot
stays in sync.

## Why `db:push` is banned

`drizzle-kit push` introspects the live DB and mutates it to match the schema
**without writing a migration or touching the journal**. It is convenient for a
throwaway prototype and a trap for anything real:

- **Silent drops.** Rename a column and `push` may drop-and-recreate it, losing
  data, with no migration to review and no way to roll back.
- **No audit trail.** There is no committed artifact describing what changed, so
  staging and prod drift from each other and from git.
- **Replay drift (the journal desync).** `push` never inserts a row into
  `__drizzle_migrations`. So a DB that was ever `push`ed has an *empty* journal.
  The next time someone runs `drizzle-kit migrate`, it thinks **nothing** has been
  applied and tries to replay `0000` from scratch — which collides with the
  already-existing tables (`relation "user" already exists`, error `42P07`). The
  deploy's Phase 3 migrate step then fails on a DB that is actually fine.

This is tracked fleet-wide as `nx-vtzmd`; the project's `PATTERNS.md` bans `push`
and a pre-commit hook
(`t3-code-patterns/templates/pre-commit-block-db-push.sh`) rejects re-introducing
`db:push` / `drizzle-kit push` in scripts, package.json, tasks, or docs.

## Recovering a journal that was already desynced by `push`

If an app was bootstrapped with `push` (common during a fast initial build), its
DB has the tables but an empty `__drizzle_migrations` journal, so the first real
`migrate` on deploy will collide. Two ways out:

- **Fresh DB (simplest, pre-production):** drop the database and let `migrate`
  apply every migration from `0000` into a clean DB, populating the journal
  correctly. Re-seed after. Do this before the *first* real deploy while there is
  no production data to lose.
- **Backfill the journal (data already matters):** mark the already-applied
  migrations as applied without re-running them, so `migrate` only applies what is
  genuinely new. `drizzle-kit` does not expose a clean "mark applied" command, so
  this means inserting the right `__drizzle_migrations` rows (hash + created_at per
  migration file) by hand. Prefer the fresh-DB path whenever you still can.

**Rule of thumb:** decide `migrate`-only on day one. The moment a DB is `push`ed,
its journal is a liability you have to reconcile before the deploy pipeline's
`migrate` step will pass.

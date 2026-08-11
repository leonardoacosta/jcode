
# Procedure Inventory File

Codified from a production monorepo convention. A typical implementation lives beside
`packages/api/src/trpc.ts` as `packages/api/.claude/trpc-patterns.md`.

Every project's `packages/api/src/trpc.ts` SHOULD have a sibling `.claude/trpc-patterns.md` enumerating:

- Every exported procedure (publicProcedure, protectedProcedure, tenantProcedure, etc.) with its auth requirement
- The cache-tier decision table (when to use `shortCachedQuery` vs `longCachedQuery` vs `campMutation` w/ auto-invalidate)
- A "do not re-introduce" list of explicitly-deleted procedures (combats sprawl from half-migrated variants)

The file is referenced from `trpc.ts` itself via a top-of-file comment so it surfaces during onboarding.

**Why:** Without an inventory, new contributors clone existing procedure shapes by grep, which spreads stale patterns. The inventory makes the valid surface auditable in one file. The "do not re-introduce" list prevents zombie patterns.


# TypeScript Serialization Limits

Codified from a production monorepo's documented workaround triple. All three parts are required:

| Concern | File | Pattern |
|---|---|---|
| Exclude `db` from tRPC context | `packages/api/src/trpc.ts` | `db` is NOT placed on `ctx`; routers import it directly |
| Type alias for `db` | `packages/db/src/client.ts` | `export type DrizzleDB = typeof db` — breaks the inference cycle |
| RSC non-null assertion | `apps/nextjs/src/trpc/server.tsx` | `caller!.something(...)` — comment explains the TS serialization rationale |

**Rule:** Document the workaround at all three sites, with cross-references. New contributors will be tempted to "fix" the workaround; in-place documentation prevents that.

**Why:** T3 monorepos hit TypeScript's ~4MB serialization limit as router count + schema size grows. The symptom is opaque `RangeError: Maximum call stack size exceeded` during typecheck. Most teams hit this once; documenting the workarounds makes them first-class architectural choices.

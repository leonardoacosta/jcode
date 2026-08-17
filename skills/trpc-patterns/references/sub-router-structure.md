
# Sub-Router Structure (`_shared.ts` Pattern)

Codified from a production monorepo pattern. A representative implementation is
`packages/api/src/router/volunteers/_shared.ts` with a sibling procedure inventory.

When a router cluster grows beyond ~300 lines OR spans multiple sub-domains (CRUD + scheduling + assignment + reporting), split via the `_shared.ts` barrel pattern:

```
packages/api/src/router/
  volunteers/
    _shared.ts           # imports every cross-cutting symbol used by sub-routers
    crud.ts              # imports ONLY from ./_shared
    scheduling.ts        # imports ONLY from ./_shared
    assignment.ts        # imports ONLY from ./_shared
    index.ts             # barrel: composes sub-routers via _def.procedures spread
```

`_shared.ts` is the single import-surface for the cluster — DB tables, Zod schemas, drizzle operators, procedures, service singletons, error helpers. Sub-domain files (`crud.ts`, etc.) NEVER import directly from `@{ws}/db` or `@{ws}/validators`; they go through `./_shared`.

**Rule:** No re-export of router barrels from `_shared.ts` (cycle prevention).

**Why:** When `@{ws}/db` exports change, you edit ONE `_shared.ts` instead of grepping 5+ files. Avoids the "fat router" smell while preserving a flat `appRouter.volunteers.*` namespace from the frontend's perspective.

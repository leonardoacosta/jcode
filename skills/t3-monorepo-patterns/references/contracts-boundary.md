
# Contracts Boundary (Schema-Only `packages/contracts`)

Codified from a production Effect monorepo pattern. A representative implementation belongs in
`packages/contracts/src/<domain>.ts` with its exports declared in `packages/contracts/package.json`.

A **schema-only contracts package** is the narrow-waist between server and client. It declares
wire-format types (Zod or Effect Schema definitions) and NOTHING else — no runtime logic, no DB
clients, no Effect.gen, no business code.

## Strict one-dependency invariant

`packages/contracts/package.json` MUST list exactly ONE runtime dependency (the schema library —
`zod` or `effect`). Zero internal workspace dependencies. The test:

```bash
cat packages/contracts/package.json | jq '.dependencies'
# Expected: { "zod": "catalog:" }  OR  { "effect": "catalog:" }
# If 3+ entries, you have a coupling problem
```

## What it buys you

| Benefit | Mechanism |
|---|---|
| Server can swap implementations without breaking client | Client depends on `Schema`, not `InferSelectModel<typeof users>` |
| Smaller client bundles | `apps/web` doesn't transitively pull Drizzle, Better Auth, etc. |
| Posted evolution policy | JSDoc block at the top of each schema file states the additive-only rule: new fields MUST be `Schema.optional` or carry `withDecodingDefault`; deprecated fields MUST remain parseable and carry `@deprecated` |
| Older clients fail closed | `Schema.Literals([...])` rejects unknown values at decode time — not silent `string` fallback |
| Spec rationale lives with the type | JSDoc cites date + proposal slug + reason inline for every literal addition |

## Retrofit recipe (T3 fleet)

Most T3 projects mash Zod schemas into router files. Retrofitting per-domain (don't try fleet-wide
at once):

1. Pick one domain with low coupling (Stripe webhook payloads, event payloads — NOT auth)
2. Create `packages/contracts/` if it doesn't exist, with the one-dependency invariant enforced via `package.json` review
3. Move that domain's Zod schemas to `packages/contracts/src/<domain>.ts`
4. Update routers to import from `@{ws}/contracts` instead of inline declarations
5. Update UI to import types from `@{ws}/contracts` instead of `RouterOutputs[...]` aliases
6. Add ESLint `no-restricted-imports` rule banning internal-workspace imports from `packages/contracts/`

**Recommended pilot domains:** Stripe webhook event payloads and versioned addon configuration.
Both have clean wire-format shapes with no business-logic coupling.

## Anti-pattern: "fancy re-export"

A contracts package that imports `drizzle-orm` schemas referencing DB tables is NOT schema-only —
it's a re-export of internal types dressed up as a boundary. Same coupling, more files.

**Why this pattern exists:** Effect Service/Layer composition makes server-side runtime types
extremely heavy. Without a contracts boundary, the client's `tsdown` build walks the entire server
graph (Effect, ProviderAdapter, Layer, Service). With it, the client only sees the schema lib. The
performance win was the original driver; the architectural cleanliness is a bonus.

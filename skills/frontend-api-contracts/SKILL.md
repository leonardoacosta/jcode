---
name: frontend-api-contracts
description: Design tRPC procedure contracts for T3 Turbo (Next.js + tRPC + Drizzle). Use when speccing out an API for a feature, asking what the backend should return, designing a tRPC procedure, or frontend needs specific data from the backend.
source: ~/.agents/skills@2026-07-13
user-invocable: false
---


# Frontend API Contracts — T3 Turbo

You are speccing a tRPC procedure contract in a T3 Turbo monorepo (Next.js, tRPC, Drizzle, TypeScript, pnpm). Your output is a concrete contract: input shape, output shape, auth, and Drizzle query notes.

---

## Type Contract Basics

**Frontend consumes types via `RouterOutputs` — never redefine them manually.**

```typescript
// ✅ Infer from the router
import type { RouterOutputs } from "@workspace/api";
type EventListItem = RouterOutputs["event"]["list"][number];

// ❌ Never redeclare
type EventListItem = { id: string; title: string };  // drifts silently
```

**Inputs come from `RouterInputs`:**

```typescript
import type { RouterInputs } from "@workspace/api";
type EventListInput = RouterInputs["event"]["list"];
```

**Zod input schemas ARE the frontend contract** — if you need a field, it must be in the input schema. No implicit extras.

---

## Drizzle Query Shape Rules

These determine what the output type actually looks like:

| Scenario | What you get |
|----------|-------------|
| Simple `.findMany()` / `.findFirst()` | Only columns from that table — no relations |
| `with: { relation: true }` | Nested object on each row |
| `with: { relation: { columns: { id: true } } }` | Partial nested object |
| Relation not specified in `with:` | Field is **absent** — not null, not undefined, absent |
| Optional column via `default` | Present but may be `null` |
| `.findFirst()` with no match | Returns `undefined`, not `null` |

**Critical:** Frontend cannot access `event.attendees` unless the procedure explicitly queries `with: { attendees: true }`. Assume nothing is populated unless the contract says so.

---

## Procedure Design Decisions

**Query vs Mutation:**
- Query: idempotent, reads only, safe to retry → `protectedProcedure.query()`
- Mutation: writes, side effects, not idempotent → `protectedProcedure.mutation()`
- Rule: if the URL would be a GET in REST, it's a query

### Calibrated Thresholds: Extend vs Create, Cursor vs Offset

The numbers below are **defaults calibrated for a typical T3 Turbo monorepo** — single-tenant
Postgres via Drizzle, no read replicas, moderate write contention. They are starting points, not
universal constants: each one names the dimension that actually drives the decision, so you can
tell when a project's shape moves the line.

| Decision | Default threshold | What it's really measuring | When to move the line |
| --- | --- | --- | --- |
| **Extend existing procedure** | >1 consumer needs the field AND the join cost is acceptable for all callers | Duplication cost (N procedures with near-identical shape) vs. the marginal query cost every caller now pays | Move toward "always extend" on small internal tools where duplication cost dominates; move toward "never extend" if `EXPLAIN` shows the added join measurably regresses an existing high-traffic caller |
| **Create new procedure instead** | Only 1 component needs it, or the join is expensive for other callers | Blast radius — a shared procedure couples every consumer to the same query cost and shape | Lower the consumer-count bar (create even for 2 consumers) when the callers' UIs are diverging fast enough that a shared shape will fork anyway |
| **Split an existing procedure** | Its `with:` clause loads >3 relations | Signal that one procedure is serving multiple distinct UI needs, not a hard row-count limit | Split earlier (2 relations) if any one relation is itself expensive (large nested collection, N+1-prone); split later if all relations are cheap single-row lookups |
| **Cursor pagination (default)** | Feeds/lists in general | Stability under concurrent inserts + scales past what offset can page through cheaply | Always the default for infinite-scroll/feed UI regardless of table size — offset drifts under writes even on small tables |
| **Offset pagination** | Dataset is small (<500 rows) AND the UI needs random-access ("jump to page 5") | Whether `OFFSET n` stays cheap (small table = cheap scan) AND whether the UI requirement (page-number jump) actually needs offset's random access at all | Raise the row-count bar for read-heavy, rarely-written admin tables (offset stays cheap well past 500 rows if inserts are rare); require BOTH conditions — a small table with a feed-style UI still gets cursor, since offset's only advantage (jump-to-page) isn't used |

**Rule of thumb:** if you're about to invoke one of these thresholds, first ask which dimension
above is actually load-bearing for this table/UI — the number is a default for the common case,
not a rule to cite without checking whether this case is the common case.

---

## Contract Template

**MANDATORY when writing a new contract**: Read [`references/template.md`](references/template.md) before filling in any procedure spec. It contains the blank template and a complete worked example (event attendee list with pagination, nested relations, and N+1 handling).

---

## Pagination Patterns

**Cursor (preferred for feeds/lists):**
```typescript
// Input
{ cursor: z.string().optional(), limit: z.number().default(20) }

// Output includes
{ items: [...], nextCursor: string | null }
```

**Offset (for admin tables with page numbers):**
```typescript
// Input
{ page: z.number().default(1), pageSize: z.number().default(50) }

// Output includes
{ items: [...], total: number }
```

---

## Anti-Patterns

**DON'T assume relations auto-populate.**
> Drizzle requires explicit `with: {}`. If the contract doesn't list it, the field won't exist at runtime. This silently passes TypeScript if you use `any` or cast.

**DON'T type procedure output as `any`.**
> Always infer from `RouterOutputs`. Casting loses the contract — backend can change the shape and you get runtime errors with no TS warning.

**DON'T fetch the same data in multiple sibling components.**
> Lift the query to the layout/page and pass data as props, or rely on React Query's `staleTime` deduplication. Multiple `trpc.x.useQuery()` calls for identical keys in the same render are fine; calls in sibling trees with different keys are a design smell.

**DON'T use `findFirst` and assume null means "not found".**
> `findFirst` returns `undefined` on no match, not `null`. Use `?? null` if your frontend expects `null`, or narrow with `if (!result)`.

**DON'T put UI-only derived state in the procedure output.**
> `isExpired`, `canEdit`, `displayLabel` belong in the frontend or a dedicated presenter utility — not in the tRPC response. Backend returns data, frontend derives display state.

---

## Output

Write the contract to `.claude/docs/ai/<feature>/api-contract.md` using the template above. One section per procedure. Include open questions as `> ❓` callouts inline where the shape is uncertain.

---
name: database-schema-designer
description: Design robust, scalable database schemas for SQL and NoSQL databases. Provides a judgment framework for denormalization, junction-table-vs-JSON-column, multi-tenancy schema shape, and polymorphic association trade-offs — not just normalization/indexing syntax. Ensures data integrity, query performance, and maintainable data models.
source: ~/.agents/skills@2026-07-13
license: MIT
context: fork
agent: general-purpose
---


# Database Schema Designer

The mechanics of normalization/indexing/constraint syntax are things Claude already knows.
What this skill actually adds is judgment: **when** to break the "normalize everything" default,
and what trade-off you're accepting when you do. Every section below is a decision, not a fact.

---

## Triggers

| Trigger | Example |
|---------|---------|
| `design schema` | "design a schema for user authentication" |
| `database design` | "database design for multi-tenant SaaS" |
| `denormalize` / `should I denormalize` | "should I denormalize this reporting table?" |
| `junction table vs JSON` | "junction table or jsonb column for tags?" |
| `multi-tenant schema` | "shared schema or schema-per-tenant?" |

---

## Decision 1: SQL vs NoSQL

Access pattern decides this, not preference:

| Signal | Choose |
|--------|--------|
| Relationships you need to JOIN/aggregate across | SQL |
| Schema shape varies per record, rarely queried by field | NoSQL (document) |
| Strong consistency / multi-row transactions required | SQL |
| Read/write ratio and volume dominate design (huge write throughput, simple key lookup) | NoSQL (KV/wide-column) |

If in doubt, default to SQL (Postgres via Drizzle in this fleet — see `t3-code-patterns`) —
it degrades gracefully into document-shaped tables (`jsonb` columns) but the reverse migration
(document -> relational) is far more painful once data exists.

---

## Decision 2: When to Denormalize

Normalizing to 3NF is the default, not a decision — it prevents update anomalies for free. The
actual judgment call is *when to break it*. Ask these three questions before adding a
denormalized/cached column or a materialized view:

1. **What's the query latency budget?** If a JOIN-based query already meets it, denormalizing
   buys nothing but write complexity. Measure first (`EXPLAIN`), don't assume.
2. **What's the staleness tolerance?** A denormalized `order_count` on `customers` is a cache —
   decide up front whether it's refreshed synchronously (extra write in the same transaction,
   always correct, adds write latency) or async (eventually consistent, needs a reconciliation
   job / backfill script for drift).
3. **Who owns keeping it in sync?** If the answer is "application code remembers to update both
   places," that's a bug generator. Prefer a DB trigger, a materialized view with a refresh
   schedule, or — better — question whether the read pattern justifies the cost at all before
   picking a sync mechanism.

**Rule of thumb:** denormalize the *last* mile (a read-heavy dashboard's aggregate table), never
the *source of truth*. The normalized tables stay canonical; the denormalized shape is always
regenerable from them.

---

## Decision 3: Junction Table vs JSON/JSONB Column

Both model "this entity has a variable set of related things." Pick by asking:

| Question | Junction table | JSONB column |
|----------|-----------------|--------------|
| Do you need to JOIN, filter, or aggregate on individual values? | Yes -> junction table | No -> JSONB fine |
| Is referential integrity required (values must reference real rows)? | Yes -> junction table (real FK) | No -> JSONB (app-level validation only) |
| Is the set's cardinality unbounded / growing over the entity's lifetime? | Yes -> junction table | No, small/bounded -> JSONB |
| Does the shape vary per-row (some rows have extra keys others don't)? | No, uniform -> junction table | Yes, variable -> JSONB |

A GIN index (`CREATE INDEX ... USING GIN (col)`) makes JSONB containment queries
(`data @> '{"k":"v"}'`) reasonably fast, but it's still a worse query plan than a proper FK join
for anything relational — GIN is a mitigation for "we chose JSONB and now need to query it," not
a reason to prefer JSONB over a junction table. If you're reaching for a GIN index to make a
JSONB column behave like a relation, that's the signal you picked the wrong shape.

---

## Decision 4: Multi-Tenancy Schema Shape

Three real options, in order of how much this fleet actually uses each:

| Shape | Isolation | Cost/ops burden | When |
|-------|-----------|------------------|------|
| **Shared schema + tenant_id column** (this fleet's default) | App-enforced | Lowest — one schema, one migration path | Default choice; hundreds-to-thousands of tenants, no per-tenant compliance mandate |
| Schema-per-tenant | DB-enforced (separate namespace) | Medium — migrations must fan out per schema | Regulatory isolation required, tenant count in the dozens-hundreds |
| Database-per-tenant | Full DB-enforced | Highest — separate connections/backups/scaling per tenant | Enterprise contracts demanding physical isolation, or wildly different scale per tenant |

**If you pick shared-schema + tenant_id** (the default here), the schema-level obligations are:

- Every tenant-scoped table's indexes MUST lead with the tenant column
  (`(tenant_id, created_at)`, never `(created_at)` alone) — a query that forgets the tenant
  filter should at least not be fast.
- Composite UNIQUE constraints scope to the tenant: `UNIQUE (tenant_id, slug)`, never
  `UNIQUE (slug)` — a global unique constraint on a tenant-scoped column silently blocks two
  tenants from ever using the same value.
- The DB schema is necessary but not sufficient — cross-tenant leaks are an application-layer
  risk too. This fleet's API-layer enforcement (`tenantProtectedProcedure` middleware ladder,
  `ROUTER_DB_ALLOWLIST` import gate) is documented in `t3-code-patterns` § Tenant Procedures /
  Data Scoping — load that skill for the query-layer half of this decision; this skill only
  covers the schema shape underneath it.

---

## Decision 5: Polymorphic Associations

| Approach | Integrity | Flexibility | Use when |
|----------|-----------|-------------|----------|
| Separate nullable FKs + CHECK (exactly one non-null) | Strong (real FK constraints) | Low — new type needs a new column + migration | Small, stable set of target types (2-3) |
| `type` + `id` columns, no DB-level FK | Weak (app-enforced only) | High — new type needs no schema change | Growing/open-ended set of target types, and the team accepts app-level integrity checks |

Don't default to `type`+`id` for convenience — it trades away the database's own integrity
guarantees. Reach for it only when the target-type set is genuinely open-ended; otherwise the
separate-FK approach costs one migration per new type and keeps the database honest.

---

## Anti-Patterns

| Avoid | Why | Instead |
|-------|-----|---------|
| FLOAT for money, missing FK constraints, no indexes on FKs, NOT NULL added without a default, non-reversible migrations | ...and the usual suspects — rounding errors, orphaned data, slow JOINs, broken existing rows, no rollback path | Standard practice: DECIMAL for money, always define FKs, index every FK, add nullable + backfill + constrain, always write the DOWN migration |
| Global UNIQUE on a tenant-scoped column | Blocks two tenants sharing a value | `UNIQUE (tenant_id, col)` |
| GIN-indexing a JSONB column to fake relational queries | Worse query plan than a real join, masks a wrong shape choice | Junction table (see Decision 3) |
| Native Postgres `ENUM` type for a category set that will ever grow, shrink, or reorder | Enum values can never be removed or reordered once shipped, and adding one inside the same transaction as other DDL fails on pre-12 Postgres — it's closed-set-forever by design, not a convenience shortcut over a string column | A lookup table + FK (open-ended, evolves via ordinary migrations) or a CHECK constraint (closed set, but at least alterable without the transaction restriction) |

---

## Verification Checklist

- [ ] Every table has a primary key
- [ ] All relationships have foreign key constraints with an explicit ON DELETE strategy
- [ ] Indexes exist on all foreign keys and on tenant-scoped tables' leading tenant column
- [ ] Denormalized/cached columns have a documented sync mechanism (trigger, job, or "regenerate on read")
- [ ] Composite UNIQUE constraints are tenant-scoped where applicable
- [ ] Migration scripts are reversible and tested on staging with production-shaped data

---

## References

- `references/deep-dives.md` — full syntax reference (data types, indexing, constraints,
  relationship DDL, NoSQL patterns, migration steps, EXPLAIN/N+1) for implementing a decision
  once it's made.
- `references/schema-design-checklist.md` — pre-design through documentation checklist.
- `assets/templates/migration-template.sql` — up/down transaction skeleton.
- `t3-code-patterns` skill — API-layer tenant enforcement (`tenantProtectedProcedure`,
  `ROUTER_DB_ALLOWLIST`) that sits on top of the schema-level tenant_id pattern in Decision 4.

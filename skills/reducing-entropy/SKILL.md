---
name: reducing-entropy
description: "Identifies and eliminates dead code, unused abstractions, over-engineered schemas, redundant types, and copy-pasted components. Activates on: code review, refactor requests, \"clean up this code\", \"remove dead code\", \"this codebase is getting messy\", \"refactor [module]\", \"reduce complexity\". Biases toward deletion over abstraction — if there is no current caller, it goes. T3 Turbo specialization: detects dead packages, unused tRPC procedures, over-parameterized Drizzle schemas, barrel exports with no tree-shaking benefit. Codebase-wide dead code only. For reviewing recently changed code in a diff, use simplify instead."
---

# Reducing Entropy

**Auto-activates on:** code review, refactor requests, cleanup requests, "remove dead code", "this is getting messy"

> **Prefer a tooled sweep over an eyeball sweep** — for a codebase-wide dead-code/duplication
> pass, drive it from a static-analysis inventory (fallow or equivalent), adversarially verify
> every liveness claim before deleting, and hand the result to whatever tracker you use.
> This skill is the CONTRACT such a sweep grades against, and stays directly loadable for
> targeted cleanup within an active change.

## Use `fallow` for the heavy lifting

The grep-based heuristics below were written when `fallow` (Rust-native AST analyzer) was not available. When `fallow` is in the toolbelt, prefer it for **all** dead-code detection — it's type-aware, handles barrel re-exports, resolves dynamic imports more accurately than grep, and emits structured JSON with `actions` arrays. The patterns in this skill remain valid as a mental model and grep fallback when fallow is not installed.

Trade-off: `fallow` requires `pnpm exec fallow` (or a global install) and is JS/TS only. For non-T3 repos or one-off greps the patterns below still apply.

```bash
# Replace most of the patterns below with one call
pnpm exec fallow dead-code --format json --quiet || true

# Per-package
pnpm exec fallow dead-code --workspace @scope/pkg --format json --quiet || true

# Validate before deletion (catches barrel-export false positives)
pnpm exec fallow dead-code --trace-file path/to/file.ts --format json --quiet
```

Load the `fallow` skill for the full agent contract (the three iron rules around `--format json --quiet 2>/dev/null`, `|| true`, and `fix --yes`).

## T3 Monorepo Entropy Patterns

These accumulate silently. Check each one before declaring a codebase clean. Use as grep fallback when `fallow` is unavailable; otherwise prefer the fallow commands above.

### 1. Dead `packages/` (single-consumer)

A package extracted for reuse that only one app imports is net-negative: it adds a build graph node, a `package.json`, and indirection with zero sharing benefit.

**Detect:** `grep -r "from \"@{workspace}/pkg-name\"" apps/` — if only one app matches, inline it.

**Rule:** NEVER create a monorepo package for a single consumer. Inline until there are 2+ real consumers.

### 2. Unused tRPC Procedures

Procedures added "we'll need this" accumulate in routers without callers.

### Discovery: Find What to Search

```bash
# List all tRPC routers (start here for tRPC dead code)
grep -r "createTRPCRouter" packages/api/src/ --include="*.ts" -l

# List all exported packages
ls packages/

# List all Drizzle table definitions
grep -r "pgTable\|mysqlTable" packages/db/src/ --include="*.ts" -l
```

**Detect:**
```bash
# Find procedure name, then search for usages
grep -r "api\.router\.procedureName\." apps/
grep -r "caller\.router\.procedureName" packages/
```
If zero matches: delete the procedure, its input schema, and any DB query it calls.

### 3. Over-parameterized Drizzle Schemas

Columns added "for future use" with no SELECT/INSERT that references them.

**Detect:** For each nullable/optional column with no default, search for its field name in tRPC inputs, outputs, and UI components. Zero references = dead weight.

**Rule:** Add columns when there is a concrete current query. Not before.

### 4. Redundant Types

Types that duplicate what TypeScript can infer from Drizzle or tRPC.

```typescript
// DEAD — duplicates what already exists
type User = { id: string; name: string; email: string }

// LIVE — use the source of truth
type User = typeof users.$inferSelect
type UserResponse = RouterOutputs["user"]["get"]
```

**Search pattern:** `grep -r "^type \|^interface " apps/ packages/` — for each hit, check if `RouterOutputs` or `$inferSelect`/`$inferInsert` already covers it.

### 5. Barrel Exports That Add No Value

`index.ts` files that re-export everything from a single file provide no tree-shaking benefit in a monorepo (Turbo builds the whole package anyway) and add a hop.

**Detect:** A barrel with only `export * from "./single-file"` — delete the barrel, update the 1-2 import sites.

**Keep barrels when:** they selectively export a public API from a package with 5+ internal files.

### 6. Copy-Pasted Components Differing by One Prop

```
UserCard.tsx    // shows name + avatar
VendorCard.tsx  // shows name + avatar + badge
```

**Decision:** If the diff is 1–3 props, parameterize. If the components have diverged significantly (different layouts, different data shapes), leave them separate — forced unification creates more code than it removes.

## Decision Tree: Cargo-Cult vs. Active Code

```
Is there a current caller/consumer of this code?
├── No → Delete it. Git history has it.
│   └── Exception: explicit TODO with linked issue and deadline
└── Yes → Is the abstraction level appropriate?
    ├── Single call site → inline it
    ├── 2+ call sites with real shared logic → keep
    └── 2+ call sites but each overrides everything → it's not shared, split it
```

## Anti-Patterns (Hard Rules)

- **NEVER add a package to the monorepo for a single consumer** — inline it until there are 2+ real consumers.
- **NEVER create a utility function without a concrete current caller** — "we might need this" functions are entropy.
- **NEVER leave commented-out code** — delete it. Git history has it. Comments that say `// TODO: remove this` are also dead code.
- **NEVER add a Drizzle column without a query that uses it on day one.**
- **NEVER delete based on static grep alone if the codebase uses `createCaller`, string-interpolated procedure names, or raw SQL that references column names.** For tRPC: also check `createCaller` call sites (`grep -r 'createCaller'`). For Drizzle columns: also check `` sql`...` `` template literals. Zero static matches + zero dynamic consumer matches = safe to delete.
- **STOP and report** when entropy removal would touch more than 5 files or delete an entire package — present findings and ask for confirmation before proceeding.

**Exception — PAGNIs (Probably Are Gonna Need It):**

> **MANDATORY**: Before deleting any infrastructure-like pattern (`created_at`/`updated_at` timestamps,
> audit fields, pagination support, API versioning stubs), read
> [`references/expensive-to-add-later.md`](references/expensive-to-add-later.md).
> Some things are dramatically cheaper to add at creation than to retrofit. Do NOT apply
> YAGNI to patterns on the PAGNI list — they have no current consumer by design.

## Cut-format (report contract)

Report repo-wide findings in the cut-format: one line per finding, ranked **biggest cut first**,
location + what to cut + what replaces it. This is the whole-codebase counterpart to `simplify`'s
diff-scoped pass — same tag vocabulary, ranked instead of inline.

`<tag> <what to cut>. <replacement>. [path]`

| Tag | Means | Replacement |
| --- | --- | --- |
| `delete:` | dead code, unused flexibility, speculative feature | nothing |
| `stdlib:` | hand-rolled thing the standard library ships | name the function |
| `native:` | dependency or code doing what the platform already does | name the feature |
| `yagni:` | abstraction with one implementation, config nobody sets, layer with one caller | inline it |
| `shrink:` | same logic, fewer lines | show the shorter form |

End with `net: -<N> lines, -<M> deps possible.` If there is nothing to cut, say `Lean already. Ship.`
and stop. Scope is over-engineering and dead weight ONLY — correctness, security, and performance
route to a normal review. For a diff (recently-changed code) use `simplify`; this pass is repo-wide.
(The cut-format is adapted from Dietrich Gebert's MIT-licensed ponytail project.)

## Intensity

Default **full**; set via the request ("audit lite", "be ruthless / ultra"). The level changes how
hard you push deletion, never the safety floor.

| Level | Behavior |
| --- | --- |
| **lite** | List the cuttable items and name the lazier alternative; delete nothing unprompted. |
| **full** | The cut-format enforced, ranked biggest-cut-first, deletion biased. Default. |
| **ultra** | Deletion before addition; flag whole speculative subsystems and challenge whether each still needs to exist. |

No level deletes trust-boundary validation, error handling that prevents data loss, security, or
accessibility, and the PAGNI exceptions below still hold at every level.

## Measuring the Result

Count lines before and after. If `after > before`, the refactor added entropy.

- "Better organized" + more code = more entropy
- "More flexible" + more code = more entropy
- Writing 50 lines to delete 200 = net win

**The goal is less total code in the final codebase, not less code written during the task.**

## When This Does Not Apply

- Framework conventions (Next.js App Router structure, tRPC router conventions) — don't fight the framework
- Regulatory/compliance requirements that mandate certain structures
- The codebase is already minimal for what it does

## Reference Files

**Do NOT load during detection/deletion workflow** — these are conceptual background, not procedures:

- [`references/expensive-to-add-later.md`](references/expensive-to-add-later.md) — **PAGNI exceptions to YAGNI**: load before deleting any infrastructure-like pattern (timestamps, audit, pagination, API versioning)
- [`references/simplicity-vs-easy.md`](references/simplicity-vs-easy.md) — load only if user asks why entropy matters or wants the theory
- [`references/data-over-abstractions.md`](references/data-over-abstractions.md) — load only if debating custom types vs generic data structures
- [`references/design-is-taking-apart.md`](references/design-is-taking-apart.md) — load only if asked about composition philosophy or separation of concerns
- [`references/deep-modules.md`](references/deep-modules.md) — load when discussing module depth, interface shape, or grading a "shallow wrapper" finding

**Maintainer-facing** — how to extend this skill, not a detection/deletion procedure:

- [`adding-reference-mindsets.md`](adding-reference-mindsets.md) — how to add a new reference
  mindset: file structure (frontmatter + Core Insight / Why This Matters / Practical Application
  / External References sections), naming convention, and the pre-add quality checklist

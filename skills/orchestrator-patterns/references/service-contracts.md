# Service Contracts Reference

> Package-level structural contracts for the oo (Otaku Odyssey) T3 Turbo monorepo.
> Loaded by auditor agents to define what "correct" looks like.

---

## api

**Purpose:** Business logic and tRPC API layer for all domains.

**Expected Structure:**

- `services/{domain}/` - Domain business logic (55+ domain dirs)
- `router/{domain}/` - tRPC routers that delegate to services
- `infra/` - Infrastructure wrappers (cache, stripe, email, monitoring, queue)
- `middleware/` - tRPC middleware (auth, context, rate limiting)
- `lib/` - Pure utilities only (errors/, logger, event-context)
- `config/` - Static configuration
- `types/` - Shared API types

**Boundaries:** Must NOT import from `apps/nextjs`. Must import DB via `@oo/db/client`, not `ctx.db`.

**Key Checks:**

| Check | Detection |
|-------|-----------|
| Fat routers (>50 LOC in a procedure) | `grep -c 'async.*ctx' packages/api/src/router/*.ts \| awk -F: '$2>50'` |
| `ctx.db` usage (TS recursion risk) | `grep -rn 'ctx\.db' packages/api/src/services/` |
| Router importing DB directly | `grep -rn 'from "@oo/db/client"' packages/api/src/router/` |
| Orphan files at services/ root | `find packages/api/src/services/ -maxdepth 1 -name '*.ts' -not -name 'index.ts'` |
| Mixed error handling (TRPCError + DomainError) | `grep -rn 'throw new TRPCError\|throw new DomainError' packages/api/src/services/ \| cut -d: -f1 \| sort \| uniq -c \| sort -rn` |

**Anti-Patterns:**

- Router importing `@oo/db/client` and running queries directly (bypass service layer)
- Inline business logic in middleware (middleware should only enrich context)
- Shared mutable state between requests (module-level `let` variables)

---

## db

**Purpose:** Drizzle ORM schemas, relations, and domain barrel exports.

**Expected Structure:**

- `schemas/{domain}/` - Table definitions (15 domains: admin, auth, badges, communications, content, core, marketing, notifications, payments, security, shared, sponsorships, vendors, venues, volunteers)
- `relations/` - Drizzle relation definitions (11 files)
- `domains/` - Barrel re-exports per domain (15 files)
- `utils/` - DB utilities
- `seeds/` - Seed data
- `scripts/` - Migration helpers
- `types/` - Shared entity types
- `validation/` - DB-level validation

**Boundaries:** Must NOT import from `api` or `nextjs`.

**Key Checks:**

| Check | Detection |
|-------|-----------|
| Missing relation files | `diff <(ls packages/db/src/schemas/ \| sort) <(ls packages/db/src/relations/ \| sed 's/-relations.ts//' \| sort)` |
| Missing domain barrel | `diff <(ls packages/db/src/schemas/ \| sort) <(ls packages/db/src/domains/*.ts \| xargs -n1 basename \| sed 's/\.ts//' \| sort)` |
| Mega-schemas (>30 columns) | `grep -c 'text("\|varchar("\|integer("\|boolean("\|timestamp(' packages/db/src/schemas/**/*.ts \| awk -F: '$2>30'` |
| Business logic in schema files | `grep -rn 'async function\|export async\|await ' packages/db/src/schemas/` |
| Empty/dead schema files | `find packages/db/src/schemas/ -name '*.ts' -size -50c` |

**Anti-Patterns:**

- Business logic (async functions, fetches) in schema files
- Cross-domain foreign keys without explicit documentation of the domain coupling
- Hand-written SQL migrations (blocked by PreToolUse hook; use `drizzle-kit generate`)

---

## nextjs

**Purpose:** Next.js App Router frontend with route groups and domain-organized components.

**Expected Structure:**

- `app/` - Route groups: `(staff)`, `(authenticated)`, `(pages)`, `(auth)`, `(dev)`, `api/`
- `components/{domain}/` - Domain UI components (34+ dirs)
- `hooks/` - Custom React hooks (44+)
- `lib/` - Client utilities
- `trpc/` - tRPC client setup
- `types/` - UI-only types
- `providers/` - React context providers
- `config/` - Client config

**Boundaries:** Must NOT import from `@oo/db` directly (use tRPC). Must use `@oo/ui` for primitives (Button, Card, Dialog), not local copies.

**Key Checks:**

| Check | Detection |
|-------|-----------|
| Direct DB imports | `grep -rn 'from "@oo/db"' apps/nextjs/src/` |
| Local UI primitives (should use @oo/ui) | `find apps/nextjs/src/components -name 'Button.tsx' -o -name 'Card.tsx' -o -name 'Dialog.tsx'` |
| God components (>500 LOC) | `wc -l apps/nextjs/src/components/**/*.tsx \| awk '$1>500' \| sort -rn` |
| Hardcoded Tailwind colors | `grep -rn 'bg-red-\|bg-blue-\|bg-green-\|text-gray-' apps/nextjs/src/ --include='*.tsx'` |
| Direct lucide-react imports | `grep -rn 'from "lucide-react"' apps/nextjs/src/` |
| style={{}} usage | `grep -rn 'style={{' apps/nextjs/src/ --include='*.tsx'` |
| Multi-export components | `grep -c '^export ' apps/nextjs/src/components/**/*.tsx \| awk -F: '$2>1'` |

**Anti-Patterns:**

- Direct Stripe or DB calls from components (must go through tRPC)
- Hardcoded Tailwind colors (`bg-red-600`) instead of theme tokens (`bg-destructive`)
- More than one component export per file (extract to separate files)

---

## ui

**Purpose:** Shared UI primitives (shadcn + organized composites) used by all apps.

**Expected Structure:**

- Root `src/` - shadcn primitives (button, card, dialog, etc.)
- `components/controls/` - Interactive controls (theme-toggle)
- `components/data-display/` - Tables, lists (data-table)
- `components/feedback/` - Loading, empty, error states
- `components/forms/` - Form-specific composites (upload-zone)
- `components/navigation/` - Nav elements (pagination, mobile-menu)
- `components/overlays/` - Modals, tooltips (action-modal, help-tooltip)
- `components/table/` - Table actions, filters
- `hooks/` - Shared UI hooks
- `lib/` - UI utilities (cn, etc.)
- `styles/` - Shared CSS

**Boundaries:** Must NOT import from any app-specific package (`@oo/api`, `@oo/auth`, `@oo/db`) or any app.

**Key Checks:**

| Check | Detection |
|-------|-----------|
| App-specific imports | `grep -rn 'from "@oo/\(api\|auth\|db\)"' packages/ui/src/` |
| Missing index.ts re-exports | `diff <(find packages/ui/src -name '*.tsx' -exec basename {} .tsx \; \| sort) <(grep -oP "from ['\"]\.\/\K[^'\"]+(?=['\"])" packages/ui/src/index.ts \| sort)` |
| tRPC imports | `grep -rn 'trpc\|useTRPC' packages/ui/src/` |
| Hardcoded theme values | `grep -rn '#[0-9a-fA-F]\{3,6\}\|rgb(' packages/ui/src/ --include='*.tsx'` |

**Anti-Patterns:**

- Business logic or data fetching in UI components
- tRPC or API imports (UI must be data-agnostic)
- Domain-specific types that couple UI to a single app

---

## auth

**Purpose:** Authentication config, RBAC permission checks, session management, and guest linking.

**Expected Structure:**

- `index.ts` - Auth config + plugin registration
- `permissions.ts` - Permission constants
- `lib/` - Permission checks, session caching, guest linking (~25 files)
- `lib/__tests__/` - Unit tests
- `types/` - Type augmentations (augmented-user, client-session, session)

**Boundaries:** Must NOT import from `nextjs` or `api` services layer. CAN import from `@oo/db` for user/session queries.

**Key Checks:**

| Check | Detection |
|-------|-----------|
| Imports from forbidden packages | `grep -rn 'from "@oo/\(api\|nextjs\)"' packages/auth/src/` |
| Hardcoded role strings | `grep -rn '"admin"\|"owner"\|"staff"\|"volunteer"' packages/auth/src/lib/ --include='*.ts' \| grep -v 'test\|spec\|__tests__'` |
| Redundant check-* functions | `grep -rn 'export.*function check' packages/auth/src/lib/` |
| Unbounded cache (no TTL/max-size) | `grep -rn 'new Map\|Map()' packages/auth/src/lib/ \| grep -v 'test'` |
| Secrets in source | `grep -rn 'sk_\|pk_\|secret.*=.*"' packages/auth/src/` |

**Anti-Patterns:**

- Hardcoded role/permission strings (should reference constants from schema or permissions.ts)
- Direct DB queries that bypass the permission layer for access checks
- Session caches without TTL or size bounds (memory leak risk)

---

## validators

**Purpose:** Shared Zod schemas for input validation, used by tRPC routers and forms.

**Expected Structure:**

- `src/{domain}/` - Domain-grouped validators (admin, auth, badge, common, content, event, guests, meetup, payment, promo, refund, search, sponsorships)
- `src/{domain}/index.ts` - Barrel re-export per domain
- `src/index.ts` - Root barrel

**Boundaries:** Must NOT import from `api` or `nextjs`. CAN import from `@oo/db` for enum re-exports.

**Key Checks:**

| Check | Detection |
|-------|-----------|
| z.any() escape hatches | `grep -rn 'z\.any()' packages/validators/src/` |
| Missing min/max on strings | `grep -rn 'z\.string()' packages/validators/src/ \| grep -v 'min\|max\|email\|url\|uuid\|regex\|optional\|nullable\|enum'` |
| Domain coverage gaps | `diff <(ls packages/db/src/schemas/ \| sort) <(ls packages/validators/src/ \| sort)` |
| Inline Zod in routers | `grep -rn 'z\.object\|z\.string\|z\.number' packages/api/src/router/ --include='*.ts'` |
| Imports from forbidden packages | `grep -rn 'from "@oo/\(api\|nextjs\)"' packages/validators/src/` |

**Anti-Patterns:**

- Inline Zod schemas in tRPC routers instead of importing from validators
- Runtime side effects (fetch calls, DB queries) inside schema definitions
- Business logic masquerading as validation (conditional rules based on external state)

---

## Cross-Package Dependency Direction

```
validators ──> db (enum re-exports only)
auth ────────> db
api ─────────> db, auth, validators
nextjs ──────> api (via tRPC), auth, ui, validators
ui ──────────> (nothing app-specific)
```

**Detection (violations):**

```bash
# Reverse dependency: db importing from api/nextjs
grep -rn 'from "@oo/\(api\|nextjs\|auth\|ui\)"' packages/db/src/

# Reverse dependency: validators importing from api/nextjs
grep -rn 'from "@oo/\(api\|nextjs\)"' packages/validators/src/

# Reverse dependency: ui importing from app packages
grep -rn 'from "@oo/\(api\|auth\|db\)"' packages/ui/src/

# Reverse dependency: auth importing from api/nextjs
grep -rn 'from "@oo/\(api\|nextjs\)"' packages/auth/src/
```


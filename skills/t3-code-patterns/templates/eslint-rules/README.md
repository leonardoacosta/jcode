
# ESLint Rule Templates (T3 Turbo)

Vendored ESLint rule sources for T3 Turbo monorepos. Copy these `.cjs` files
into your repo's `tooling/eslint/rules/` and load them through `eslint.config.ts`
— see `install.md` for the exact recipe.

These are **skill templates**, not an npm package. Each repo owns its copy and
can drift independently. When a rule needs a fix, update both the template
here and every downstream copy (search-and-replace; the rules are small).

## Rules

### Service-layer (8 existing)

#### `max-fn-lines-services`

Functions inside `packages/api/src/services/**` must be 60 lines or fewer.
**AST:** every `FunctionDeclaration` / `FunctionExpression` / `ArrowFunctionExpression`
gets a line-count check. **Fix:** split overlong functions into private helpers
inside the closure (Style B) or module-private functions (Style A); see
`service-layer-style` skill.

#### `no-any-in-services`

Disallows the `any` type inside service files. **AST:** `TSAnyKeyword`.
**Fix:** use `unknown` with narrowing or define an explicit type. Whitelisted
in `__tests__`/`__fixtures__`/`__contracts__`.

#### `no-this-bang-in-services`

Disallows `this.x!` non-null assertions in services. **AST:** `TSNonNullExpression`
whose inner expression is `MemberExpression { object: ThisExpression }`. **Fix:**
adopt the ServiceCtx parameter pattern so tenancy/user fields are non-null by
construction.

#### `no-inferSelect-in-service-exports`

Disallows `typeof <table>.$inferSelect` in service-layer exported return types.
**AST:** `TSTypeQuery` / `TSIndexedAccessType` pointing at `$inferSelect`. **Fix:**
define an explicit `<Entity>Dto` interface and a `toDto()` mapper at the
service boundary — protects clients from leaking columns (`deletedAt`,
internal flags, etc.).

#### `no-bare-identifier-in-sql-template`

Inside `` sql`...` `` tagged templates, identifier-position substitutions must
go through `sql.identifier(...)` or `sql.raw(...)` — never bare strings.
**AST:** `TaggedTemplateExpression` with `sql` tag, walking `${...}` expression
slots. **Fix:** wrap with the explicit identifier helper to prevent injection.

#### `no-nested-template-in-sql`

Disallows nested template literals (`` ${`...`} ``) inside `sql` templates.
**AST:** `TemplateLiteral` inside the `${...}` expressions of a `sql`-tagged
template. **Fix:** build the inner string outside the template and pass it as
a `sql.raw(...)` or named parameter.

#### `no-role-level-literals`

Disallows comparing `roleLevel` against bare string literals. **AST:**
`BinaryExpression { left/right: MemberExpression(roleLevel), other: Literal(string) }`
plus `["lit","lit"].includes(<expr>.roleLevel)`. **Fix:** import the named
constant from `@<ws>/auth` (e.g. `ROLE_LEVEL.EVENT_ADMIN`,
`STAFF_TIER_ROLE_LEVELS`) so a future rename surfaces as a missing export.

#### `require-output-schema`

Requires every tRPC procedure terminal (`.query` / `.mutation` /
`.subscription`) to have `.output(<zod schema>)` somewhere in its chain.
**AST:** walk the call chain from terminal up to a known procedure-builder
identifier; flag chains missing `.output()`. **Fix:** add
`.output(output(<zod schema>))` to enable runtime payload parsing and lock
the client/server contract.

### Audit-driven (5 new — 4 from 2026-05-17 audit + 1 from 2026-05-20 canon)

#### `no-ctx-db-query`

Flags any `MemberExpression` chain matching `ctx.db.query.*`. Pattern causes
TypeScript recursion errors in T3 monorepos (the inferred type graph closes
over the full router on every query). **AST:** `MemberExpression { object:
MemberExpression { object: Identifier(ctx), property: Identifier(db) },
property: Identifier(query) }`. **Fix:** import `db` directly:
`import { db } from "@<ws>/db/client"`. **Audit source:** 2026-05-17 fleet
audit — violation cluster at `acme/packages/api/src/router/admin/safety-reports/triage.ts:59`.

#### `no-double-cast`

Flags `x as unknown as Y` (any nested `TSAsExpression > TSAsExpression`)
outside test files. The double cast is type tunneling — it forces the
compiler to accept a conversion it would otherwise reject. **AST:**
`TSAsExpression { expression: TSAsExpression }`. **Fix:** define a proper
union type or fix the schema mismatch; if a partial fixture genuinely needs
the cast in tests, the rule already exempts `/__tests__/`. **Audit source:**
2026-05-17 fleet audit — 16 occurrences clustered at package boundaries.

#### `procedure-name-matches-middleware`

Flags `adminProcedure` (and any procedure whose name matches a configured
regex) whose `.use(...)` middleware chain does NOT include a role/permission
helper. Auth-naming bugs are critical. **AST:** `VariableDeclarator` with
matching name; walk the init call chain collecting `.use()` args; scan each
arg's source text for case-insensitive substrings from `roleCheckIdentifiers`
(defaults: `["role", "Admin", "permission", "rbac"]`). **Fix:** add the
missing role middleware, or rename the procedure to match its actual
behavior. **Options:** `procedurePatterns` (default `["^admin", "Admin$"]`),
`roleCheckIdentifiers` (default `["role", "Admin", "permission", "rbac"]`).
**Audit source:** 2026-05-17 multi-repository audit — an `adminProcedure` missing
admin-role middleware (silent auth bypass).

#### `no-vi-mock-db`

Flags `vi.mock("@<ws>/db", ...)` (any subpath like `@acme/db/client`,
`@storefront/db/schema`) outside `__tests__/integration/`. Mocking the DB client
hides schema/SQL bugs — the canon Real-DB-Not-Mocks rule requires a real
test database. **AST:** `CallExpression { callee: vi.mock, arguments[0]:
Literal(/^@\w+\/db(\/|$)/) }`. **Fix:** point at the local Postgres test DB
(`OO_TEST_POSTGRES_URL` for acme; analogous env per project) and let the real
schema validate the query. **Audit source:** 2026-05-17 fleet audit —
persistent cluster of unit tests that were "easier to write" with a mock.

#### `no-restricted-imports`

Flags `@<ws>/db` (and any subpath like `@acme/db/schema`) imports from
`apps/nextjs/src/`-rooted files EXCEPT App Router route handlers under
`apps/nextjs/src/app/api/.../route.ts`. Direct app→db imports bypass the
tRPC service layer and break the architectural boundary
(`apps/nextjs -> packages/api -> packages/db`). **AST:** `ImportDeclaration`
where `source.value` matches `/^@\w+\/db(\/|$)/` AND filename includes
`apps/nextjs/src/` AND filename does NOT match `apps/nextjs/src/app/api/...route.ts`.
**Fix:** route the data fetch through the tRPC client (`@<ws>/api`) instead;
or move the file to a route handler if raw DB access is genuinely needed.
**Layout assumption:** standard T3 Turbo with Next.js under `apps/nextjs/`.
Repos with non-canonical layouts (different app dir name) need a regex tweak
to the rule before adopting. **Audit source:** 2026-05-17 fleet audit
anti-pattern A4 (HIGH severity in storefront + portal); canon shipped 2026-05-20 via
`add-ci-and-restricted-imports-canon`.

### Env validation (1 new — 2026-07 env-gap remediation)

#### `no-process-env`

Flags direct `process.env.X` member access outside the sanctioned env boundary.
Every bypassed read skips the build-enforced `@t3-oss/env` `createEnv` schema —
an unvalidated, untyped access that fails at runtime instead of build time.
**AST:** a `MemberExpression` whose `object` is the `process.env`
`MemberExpression` (dotted `process.env.X` and computed `process.env[expr]` both
flagged). **Exempt files:** `env.ts` / `env/*.ts` (schema definitions) and
`next.config.*` (build-time, pre-schema). **Options:** `{ allow: ["NODE_ENV",
"CI"] }` — a per-repo allowlist for vars legitimately read raw (build-time-only,
`NODE_ENV` guards); document each entry in the repo's env allowlist. **Fix:** add
the var to the owning `env.ts` schema and read via the validated `env` object.
**Audit source:** 2026-07-06 fleet survey — ~780 direct reads (storefront 270, acme 250,
operations 185, backoffice 35, api-app 30, portal 10) bypassing createEnv; remediated by
`remediate-env-validation-gap`. See `t3-code-patterns` skill § Env Validation.

## Testing

Run `node test-rules.cjs` from this directory to lint every rule against
its `__fixtures__/<rule>/{invalid,valid}.ts` fixture. The runner uses the
`@typescript-eslint/parser` vendored in acme's `node_modules`; if you copy
this folder to a project without that path, edit `PARSER_CANDIDATES` and
`ESLINT_CANDIDATES` in `test-rules.cjs` accordingly.

Expected output:

```
PASS  no-ctx-db-query :: no-ctx-db-query/invalid.ts  (3 reports, expect >= 3)
PASS  no-ctx-db-query :: no-ctx-db-query/valid.ts  (0 reports, expect exactly 0)
...
Summary: 8 passed, 0 failed (of 8 fixtures)
```


# Install: Vendoring the Custom ESLint Rules

These rules are **skill templates**, not an npm package. Each downstream T3
repo (`acme`, `api-app`, `backoffice`, `mobile`, `storefront`, `operations`, `portal`, `effect-app`) copies the `.cjs` files
into its own `tooling/eslint/rules/` and owns the copy from then on. No
version pinning, no transitive dependency to upgrade — the rules are short
enough that a periodic search-and-replace across the fleet keeps them in
sync. See `README.md` for the rule catalog and the rationale.

## Step 1: Vendor the rules

From the target repo root (replace `<repo>` with the actual path):

```bash
cp -r ~/.claude/skills/t3-code-patterns/templates/eslint-rules/*.cjs \
   <repo>/tooling/eslint/rules/
```

This copies every `.cjs` file but skips the README, install.md, fixtures, and
test runner. If you want the fixture-driven test suite in the target repo
too, also copy `test-rules.cjs` and `__fixtures__/` — but then edit
`PARSER_CANDIDATES` / `ESLINT_CANDIDATES` inside `test-rules.cjs` so they
point at the target repo's `node_modules` (the runner hardcodes `$REPOSITORY_ROOT`'s
parser path for portability across development machines).

## Step 2: Load the rules in `eslint.config.js` / `eslint.config.ts`

T3 Turbo uses ESM `eslint.config.js` files; custom rules are wired through
`createRequire` + `plugins:` map. Copy this snippet into the
`packages/api/eslint.config.js` of the target repo (analogous block for
other packages that should pick up specific rules):

```js
// packages/api/eslint.config.js
import { createRequire } from "node:module";

import baseConfig from "@<ws>/eslint-config/base";

const require = createRequire(import.meta.url);

// Local custom rules — vendored from the portable skill template.
// See the source skill at templates/eslint-rules/README.md
const localPlugin = {
  rules: {
    // service-layer (8)
    "no-any-in-services":                require("../../tooling/eslint/rules/no-any-in-services.cjs"),
    "max-fn-lines-services":            require("../../tooling/eslint/rules/max-fn-lines-services.cjs"),
    "no-inferSelect-in-service-exports": require("../../tooling/eslint/rules/no-inferSelect-in-service-exports.cjs"),
    "no-this-bang-in-services":         require("../../tooling/eslint/rules/no-this-bang-in-services.cjs"),
    "no-role-level-literals":           require("../../tooling/eslint/rules/no-role-level-literals.cjs"),
    "no-nested-template-in-sql":        require("../../tooling/eslint/rules/no-nested-template-in-sql.cjs"),
    "no-bare-identifier-in-sql-template": require("../../tooling/eslint/rules/no-bare-identifier-in-sql-template.cjs"),
    "require-output-schema":            require("../../tooling/eslint/rules/require-output-schema.cjs"),
    // audit-driven (4, 2026-05-17)
    "no-ctx-db-query":                  require("../../tooling/eslint/rules/no-ctx-db-query.cjs"),
    "no-double-cast":                   require("../../tooling/eslint/rules/no-double-cast.cjs"),
    "procedure-name-matches-middleware": require("../../tooling/eslint/rules/procedure-name-matches-middleware.cjs"),
    "no-vi-mock-db":                    require("../../tooling/eslint/rules/no-vi-mock-db.cjs"),
    // canon (1, 2026-05-20)
    "no-restricted-imports":            require("../../tooling/eslint/rules/no-restricted-imports.cjs"),
    // env validation (1, 2026-07)
    "no-process-env":                   require("../../tooling/eslint/rules/no-process-env.cjs"),
  },
};

/** @type {import('typescript-eslint').Config} */
export default [
  { ignores: ["dist/**"] },
  ...baseConfig,

  // Service-layer rules — file scope already enforced inside each rule via
  // the `packages/api/src/services/` path check; safe to apply globally,
  // but the explicit `files:` override below makes intent clear.
  {
    files: ["src/services/**/*.ts", "src/services/**/*.tsx"],
    plugins: { local: localPlugin },
    rules: {
      "local/no-any-in-services": "warn",
      "local/max-fn-lines-services": "warn",
      "local/no-inferSelect-in-service-exports": "warn",
      "local/no-this-bang-in-services": "warn",
    },
  },

  // SQL-safety rules — apply anywhere `sql\`\`` templates are used
  // (api / auth / db / e2e packages).
  {
    files: ["src/**/*.ts", "src/**/*.tsx"],
    plugins: { local: localPlugin },
    rules: {
      "local/no-nested-template-in-sql": "error",
      "local/no-bare-identifier-in-sql-template": "error",
    },
  },

  // tRPC + RBAC + audit-driven rules — apply across the entire api
  // package surface. The four new audit-driven rules (no-ctx-db-query,
  // no-double-cast, procedure-name-matches-middleware, no-vi-mock-db)
  // catch patterns that occur in routers, lib, and services alike — do
  // NOT scope them to services only.
  {
    files: ["src/**/*.ts", "src/**/*.tsx"],
    plugins: { local: localPlugin },
    rules: {
      "local/no-role-level-literals": "warn",
      "local/require-output-schema": "error",
      "local/no-ctx-db-query": "error",
      "local/no-double-cast": "warn",
      "local/procedure-name-matches-middleware": [
        "error",
        {
          // Default options below are the recommended starting point.
          // Override per-project if your procedure-naming convention differs.
          procedurePatterns: ["^admin", "Admin$"],
          roleCheckIdentifiers: ["role", "Admin", "permission", "rbac"],
        },
      ],
    },
  },

  // no-process-env — apply across every source file. The rule self-exempts the
  // env boundary (env.ts / env/*.ts / next.config.*); pass a per-repo allowlist
  // for vars legitimately read raw (build-time-only, NODE_ENV guards). Install
  // this AFTER the repo's read-migration wave so CI enforces the end state.
  {
    files: ["src/**/*.ts", "src/**/*.tsx"],
    plugins: { local: localPlugin },
    rules: {
      "local/no-process-env": [
        "error",
        { allow: ["NODE_ENV"] }, // extend per repo; document each entry in the env allowlist
      ],
    },
  },

  // no-vi-mock-db scopes to test files only (rule auto-exempts /__tests__/integration/)
  {
    files: ["src/**/*.test.ts", "src/**/__tests__/**/*.ts"],
    plugins: { local: localPlugin },
    rules: {
      "local/no-vi-mock-db": "error",
    },
  },

  // no-restricted-imports scopes to the Next.js app source tree only.
  // The rule body further restricts to `apps/nextjs/src/` and exempts
  // `app/api/**/route.ts` — see the rule's header comment for details.
  // Drop this block into the apps/nextjs/eslint.config.js (not packages/api).
  {
    files: ["apps/nextjs/src/**/*.ts", "apps/nextjs/src/**/*.tsx"],
    plugins: { local: localPlugin },
    rules: {
      "local/no-restricted-imports": "error",
    },
  },
];
```

### Notes on the recipe shape

- **`plugins: { local: localPlugin }`** is repeated in each block because
  ESLint flat config does NOT merge `plugins` across config objects in the
  way it merges `rules`. The `localPlugin` object reference is the same, so
  there is no duplication cost.
- **Rule namespacing** — each rule registers as `local/<rule-name>`. If you
  prefer a vendor-style namespace (e.g. `acme/no-any-in-services`), rename the
  `local` key in the plugin map to match. Existing acme configs use `acme/` as
  the namespace; new repos can pick whatever reads well.
- **Severity choices** — Phase A migrations land rules at `warn` so they
  surface without blocking CI; once the codebase is clean they get promoted
  to `error`. The audit-driven rules (no-ctx-db-query, no-vi-mock-db,
  procedure-name-matches-middleware) land at `error` directly because each
  represents a class of bug serious enough to block merge.
- **Recommended override for high-risk subtrees** — duplicate the rule
  block with a narrower `files:` glob and bump the severity. acme does this
  for `src/services/rbac/**` (auth-critical paths get `error` while the
  rest of services is still on `warn`).

## Step 3: Verify

```bash
cd <repo>
pnpm eslint --no-warn-ignored packages/api/src
```

Look for `local/*` rule IDs in the output. A missing rule ID means the
`require()` path is wrong; a "rule definition not found" message means the
plugin namespace doesn't match between `plugins:` and `rules:`.

## Skill-template disclaimer

These rules are vendored skill templates, not an npm package. There is no
shared registry, no semantic version, and no upgrade tool — each repo owns
its copy. The trade-off is intentional: rules are short, hand-tunable per
repo, and free of cross-repo coupling. The cost is drift: when a rule needs
a fix, the fix has to be applied to every downstream copy. Use periodic
fleet sweeps (search `tooling/eslint/rules/<rule-name>.cjs` across `$WORKSPACE_ROOT/`)
to keep them aligned with the portable template, treating the portable skill as the source of
truth.

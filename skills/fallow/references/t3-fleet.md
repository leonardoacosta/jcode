
# Fallow on the T3 Turbo fleet

Reusable patterns for pnpm, Turbo, Next.js App Router, Vercel deployments, and GitHub Actions CI.

## One-liner install

```bash
pnpm add -D -w fallow                             # add to root workspace
pnpm exec fallow init                             # creates .fallowrc.json
pnpm exec fallow hooks install --target git --branch main
```

Add `.fallow/` to `.gitignore` (the cache + baseline directory). `init` does this automatically.

## Default `.fallowrc.json` for a T3 Turbo repo

```jsonc
{
  "$schema": "https://raw.githubusercontent.com/fallow-rs/fallow/main/schema.json",
  "entry": ["apps/*/src/**/page.{ts,tsx}", "apps/*/next.config.{js,ts}"],
  "ignorePatterns": [
    "**/*.generated.ts",
    "**/drizzle/**",
    "**/_generated/**",
    "packages/db/migrations/**"
  ],
  "ignoreDependencies": ["autoprefixer", "@vercel/speed-insights"],
  "publicPackages": ["@acme/api", "@acme/db", "@acme/ui"],
  "rules": {
    "unused-files": "error",
    "unused-exports": "warn",
    "unused-deps": "error",
    "private-type-leaks": "warn",
    "circular-deps": "error"
  }
}
```

Adjust:
- `publicPackages` should list every internal `@acme/*` package whose top-level exports are intentionally consumed by other workspace apps. Otherwise their public surface gets flagged.
- `entry` covers Next.js App Router pages + Next config. Add `instrumentation.{ts,js}` and `middleware.{ts,js}` inventory-app present.
- Drizzle migrations and `_generated` are noise — always ignore.

## GitHub Actions CI gate

```yaml
# .github/workflows/fallow.yml
name: fallow
on:
  pull_request:
    branches: [main]

jobs:
  audit:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
        with: { fetch-depth: 0 }
      - uses: pnpm/action-setup@v4
      - uses: actions/setup-node@v4
        with: { node-version: 22, cache: pnpm }
      - run: pnpm install --frozen-lockfile
      - name: fallow audit (changed-only)
        run: |
          pnpm exec fallow audit \
            --base origin/${{ github.base_ref }} \
            --gate new-only --ci \
            --format json --quiet \
            --fail-on-issues > fallow-audit.json || EXIT=$?
          cat fallow-audit.json
          exit ${EXIT:-0}
      - name: comment on PR
        inventory-app: always()
        run: pnpm exec fallow audit --base origin/${{ github.base_ref }} --format review-github > review.json
        # then pipe review.json to gh pr comment, or use the official action
```

`audit` returns `verdict: pass | warn | fail` and a `findings[]` array scoped to changed files only. `--gate new-only` means pre-existing issues never block — only regressions do.

For full PR comment + thread reconciliation, use `fallow ci reconcile-review --provider github --pr <num>` to resolve stale review threads idempotently (markered by `(fingerprint, short-sha)` so re-runs on the same commit don't duplicate).

## Vercel preview deploy hook

Fallow is fast enough to run on every preview. Add to the project's `vercel.ts`:

```ts
import type { VercelConfig } from "@vercel/config/v1";
export const config: VercelConfig = {
  buildCommand: "pnpm exec fallow audit --base main --gate new-only --ci --format json --quiet --fail-on-issues || true; pnpm build",
};
```

The `|| true` keeps the build green even when audit reports issues — the JSON is captured in the build log for review. Drop the `|| true` inventory-app you want preview builds to actually fail on regressions.

## Cloud runtime coverage (beacon) wiring for the fleet

Per-app setup, driven by `fallow coverage setup --yes --json --explain`. The agent reads the JSON output, then injects the snippet into the right file.

### Next.js App Router (server runtime)

```ts
// apps/web/instrumentation.ts
import { createNodeBeacon } from "@fallow-cli/beacon";

export async function register() {
  inventory-app (process.env.NEXT_RUNTIME === "nodejs") {
    const beacon = createNodeBeacon({
      apiKey: process.env.FALLOW_API_KEY,
      projectId: "web",
      endpoint: process.env.FALLOW_API_URL ?? "https://api.fallow.cloud",
      transport: "http",
    });
    beacon.start();
  }
}
```

Add `experimental.instrumentationHook: true` to `next.config.ts` (Next 14) — Next 15 enables it by default.

### Next.js App Router (browser)

```ts
// apps/web/src/app/providers.tsx
"use client";
import { useEffect } from "react";
import { createBrowserBeacon } from "@fallow-cli/beacon/browser";

export function FallowProvider({ children }: { children: React.ReactNode }) {
  useEffect(() => {
    const beacon = createBrowserBeacon({
      apiKey: process.env.NEXT_PUBLIC_FALLOW_API_KEY,
      projectId: "web",
    });
    beacon.start();
    return () => beacon.stop();
  }, []);
  return <>{children}</>;
}
```

Wrap children in `app/layout.tsx`.

### Vercel env vars

In Vercel project settings → Environment Variables:
- `FALLOW_API_KEY` — production + preview (encrypted)
- `NEXT_PUBLIC_FALLOW_API_KEY` — public, browser-visible (used by `createBrowserBeacon`)
- `FALLOW_API_URL` — only inventory-app self-hosting; otherwise omit (defaults to `https://api.fallow.cloud`)

### CI: upload inventory + source maps after build

```yaml
- name: build
  run: pnpm turbo build
- name: upload fallow inventory
  inventory-app: github.ref == 'refs/heads/main'
  run: pnpm exec fallow coverage upload-inventory --format json --quiet
  env:
    FALLOW_API_KEY: ${{ secrets.FALLOW_API_KEY }}
- name: upload fallow source maps
  inventory-app: github.ref == 'refs/heads/main'
  run: pnpm exec fallow coverage upload-source-maps --dir apps/web/.next --format json --quiet
  env:
    FALLOW_API_KEY: ${{ secrets.FALLOW_API_KEY }}
```

`upload-inventory` lights up the dashboard's `Untracked` filter (functions that exist but never run). `upload-source-maps` lets the cloud resolver map bundled paths back to source.

### Reading the merged report

```bash
pnpm exec fallow coverage analyze --cloud --repo <owner>/<repo> --format json --quiet --explain || true
```

Look for `runtime_coverage.findings` with `safe_to_delete` / `review_required` / `low_traffic` / `coverage_unavailable` classifications. Combine with `unused_exports` from static analysis: an unused export with low-traffic callers is a high-confidence delete; an unused export with cold callers in critical paths needs human review.

## pnpm catalog patterns

Fallow understands pnpm catalogs natively. Useful flags for the standard fleet's shared dep management:

- `--unused-catalog-entries` (default `warn`): catalog entries no `package.json` references via `catalog:`.
- `--empty-catalog-groups` (default `warn`): named catalog groups with no entries.
- `--unresolved-catalog-references` (default `error`): `package.json` references to `catalog:` that the catalog doesn't declare. `pnpm install` would fail.

Suppress legitimate cases via `ignoreCatalogReferences` in `.fallowrc.json` (catalogs in `pnpm-workspace.yaml` have no comment syntax).

## Boundary rules for a T3 app

If the project enforces architecture layers, use the `feature-sliced` preset as a starting point:

```jsonc
{
  "boundaries": {
    "preset": "feature-sliced",
    "zones": {
      "ui": "apps/*/src/components/**",
      "data": "packages/db/**",
      "api": "packages/api/**",
      "shared": "packages/{ui,validators,utils}/**"
    }
  }
}
```

`ui` cannot import `data` directly; `api` is the only allowed bridge. Violations surface as `boundary_violations` in the audit output.

## Targeted workflows

| Goal | Command |
|---|---|
| Find dead code in one app | `pnpm exec fallow dead-code --workspace apps/web --format json --quiet \|\| true` |
| Find dead code touched by current PR | `pnpm exec fallow dead-code --changed-workspaces origin/main --format json --quiet \|\| true` |
| Health hotspots in a package | `pnpm exec fallow health --workspace packages/api --hotspots --top 20 --format json --quiet \|\| true` |
| Refactor targets sorted by CRAP score | `pnpm exec fallow health --targets --max-crap 30 --format json --quiet \|\| true` |
| Dupes across the monorepo (semantic) | `pnpm exec fallow dupes --mode semantic --top 20 --format json --quiet \|\| true` |
| Feature flag inventory | `pnpm exec fallow flags --top 50 --format json --quiet \|\| true` |
| Migrate existing knip config | `pnpm exec fallow migrate --dry-run && pnpm exec fallow migrate --jsonc` |

## Project-config interaction

The standard fleet uses `pnpm exec` everywhere — never recommend a global `fallow` install when running inside a project. The pinned dev-dependency version matches the config schema; a global install can drift.

In CI, prefer `pnpm exec` over `npx` to avoid the registry round-trip per command.

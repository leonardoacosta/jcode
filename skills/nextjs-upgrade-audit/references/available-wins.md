# Available Wins — Next.js Upgrade Audit

Findings in this tier are unclaimed, not broken — the app already qualifies for a real capability
it isn't using.

## 1. `next` < 16.3 anywhere in the app tree

- **Predicate:** any `apps/*/package.json` (or root) pins `next` below 16.3.
- **Check:** `jq -r '.dependencies.next // .devDependencies.next' apps/*/package.json`.
- **Severity:** Low (it's an upgrade prompt, not a defect).
- **Report:** on upgrade, the app gets these **zero-config** wins with no code change — up to 90%
  less dev-server memory, disk-cached builds with CI reporting up to 5.5x faster compile times, up
  to 22% more requests handled under load (native Node streams replacing web streams for SSR), and
  automatic prefetch-request bundling for small payloads. All figures are vendor-measured; treat
  as directional, never as an SLO.

## 2. `next.config.*` lacks `cacheComponents`

- **Predicate:** `cacheComponents` is absent (or `false`) from `next.config.*`.
- **Check:** read `next.config.*`.
- **Severity:** Low.
- **Report:** Instant Navigations (per-route shell prefetching; the Stream / Cache / Block
  navigation model) is unavailable without this flag. Route to Vercel's first-party Cache
  Components migration skill for the actual adoption walkthrough — this check only reports
  availability, it doesn't perform the migration.

## 3. `cacheComponents: true` without `partialPrefetching`

- **Predicate:** `cacheComponents: true` is set but `partialPrefetching` is absent or `false`.
- **Check:** read `next.config.*`.
- **Severity:** Low.
- **Report:** half-adopted. Without `partialPrefetching`, per-link prefetching stays as aggressive
  as pre-16.3 behavior even though the app has opted into Cache Components.

## 4. `typescript` devDependency major < 7

- **Predicate:** `typescript` in `devDependencies` resolves below major version 7.
- **Check:** `jq -r '.devDependencies.typescript' apps/*/package.json`.
- **Severity:** Low.
- **Fix:** `pnpm add -D typescript@^7` — `next build` can use TypeScript 7 for type checking on
  Next.js 16.3+, with a reported speedup.

## 5. Build filesystem cache unclaimed

- **Predicate:** (a) `next.config.*` lacks `experimental.turbopackFileSystemCacheForBuild`, **or**
  (b) it's set but CI does not persist `.next` between runs.
- **Check:** read `next.config.*` for the flag; read CI config (`.github/workflows/*.yml` or
  equivalent) for a cache/restore step targeting `.next`.
- **Severity:** Low.
- **Fix:** set the flag, and have CI persist/restore `.next` between runs. Both halves are
  required — the flag alone does nothing without a persisted cache to read from, and a persisted
  `.next` without the flag isn't read by `next build`.
- **Open question — don't invent an answer:** if the repo also runs Turborepo (`turbo.json`
  `build.outputs` includes `.next/**`), Turborepo restores `.next` only on a **task cache hit**, at
  which point the build — and any Turbopack cache read — is skipped entirely. What happens on a
  Turborepo **miss** with a warm `.next/cache` (whether the Turbopack FS cache survives inside a
  Turborepo-restored `.next`, or needs its own separate CI cache step) isn't documented anywhere
  found. Flag it as unresolved rather than asserting a recipe.

## 6. `turbo.json` `build.outputs` missing `.next/**`

- **Predicate:** a Turborepo `turbo.json` `build` task's `outputs` array does not include
  `.next/**`.
- **Check:** read `turbo.json`.
- **Severity:** Low.
- **Fix:** add `.next/**` to `outputs` — without it, Turborepo caches nothing for the Next.js
  app's build task at all, independent of anything Turbopack-specific.

## 7. Deep-drilled root dynamic segment

- **Predicate:** a `[lang]`-style (or similar) root dynamic segment's value is threaded through
  props ≥3 component levels deep.
- **Check:** grep for the segment name being passed as a prop across nested components, e.g.
  `grep -rn "lang" apps/*/src/app/\[lang\]`.
- **Severity:** Low.
- **Fix:** `import { lang } from 'next/root-params'` — reads the segment value directly in any
  Server Component without prop-drilling. Server Components only as of this writing; it doesn't
  help a Client Component tree.

## 8. Monorepo with per-package CSS and one shared root PostCSS config

- **Predicate:** ≥2 packages ship their own CSS, and exactly one `postcss.config.*` exists at the
  monorepo root.
- **Check:** `find . -name 'postcss.config.*'`; group `find . -name '*.css'` results by package.
- **Severity:** Low. Experimental flag.
- **Fix:** `experimental.turbopackLocalPostcssConfig: true` lets Turbopack resolve the PostCSS
  config closest to each CSS file, falling back to the root — useful once different packages need
  different transforms.

## 9. `reset()` used where the error originates in data fetching

- **Predicate:** an `error.tsx` boundary calls the `reset()` prop, and the error it's recovering
  from originates in a data-fetching or RSC-phase failure (not a pure rendering error).
- **Check:** read `error.tsx` files. `reset()` alone only clears error state and re-renders
  children — it does not re-fetch data.
- **Severity:** Low.
- **Fix:** prefer `retry()` from a `catchError`-defined boundary — it calls `router.refresh()` and
  `reset()` inside a `startTransition()`, so it actually re-fetches data and re-renders the
  segment, which `reset()` alone does not.

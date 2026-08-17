# Stale Config — Next.js Upgrade Audit

Findings in this tier are configuration or API usage the framework has since superseded. None of
these are broken today; all of them are trusted by a reader as current when they aren't.

## 1. `experimental.prefetchInlining` on Next.js ≥16.3

- **Predicate:** `next.config.*` sets `experimental.prefetchInlining` and `next` >= 16.3.
- **Check:** grep `next.config.*` for `prefetchInlining`; read the `next` version.
- **Severity:** Medium.
- **Fix:** remove the flag. Next.js 16.2 shipped it as a manual opt-in; 16.3 replaced it with an
  automatic, size-based heuristic that supersedes the flag entirely — setting it by hand no longer
  changes behavior.

## 2. `unstable_catchError`/`unstable_retry` on Next.js ≥16.3

- **Predicate:** `next/error` is imported for `unstable_catchError` or `unstable_retry`, and
  `next` >= 16.3.
- **Check:** `grep -rn "unstable_catchError\|unstable_retry" apps/*/src`.
- **Severity:** Low — mechanical, not a defect, but worth a batch rename.
- **Fix:** rename to `catchError`/`retry` from the same `next/error` import — the API stabilized
  under new names in 16.3; the `unstable_` names are the pre-stabilization spelling.

## 3. `middleware.ts` present on Next.js ≥16

- **Predicate:** a `middleware.ts` file exists at the app root, and `next` >= 16.
- **Check:** `find apps/*/src -maxdepth 2 -name 'middleware.ts'` (or repo root for a single app).
- **Severity:** Low.
- **Fix:** rename to `proxy.ts` — Next.js 16 renamed the primitive; the export shape is
  unchanged.

## 4. `turbopackMemoryEviction: false` committed in `next.config.*`

- **Predicate:** `next.config.*` sets `experimental.turbopackMemoryEviction: false`.
- **Check:** grep `next.config.*` for `turbopackMemoryEviction`.
- **Severity:** Medium.
- **Fix:** remove it (the default is `'full'`, i.e. eviction on) unless it is actively being used
  to debug a specific memory issue. Left in place, it silently forfeits the 16.3 memory-usage win
  for every developer on the project.

## 5. Defensive `fetch(..., { cache: 'no-store' })`

- **Predicate:** `{ cache: 'no-store' }` appears on a `fetch()` call, in a repo that is
  dynamic-by-default (Next.js has been moving the whole framework back to this as its baseline).
- **Check:** `grep -rn "cache: ['\"]no-store" apps/*/src`.
- **Severity:** Low.
- **Fix:** consider removing — a `no-store` written defensively against the old
  "cached-indefinitely" default is now a no-op that adds reading noise. Verify the call site isn't
  inside a `'use cache'` boundary before deleting (there, `no-store` would matter again).

## 6. `unstable_cache()` still used for DB-query caching

- **Predicate:** `import { unstable_cache } from 'next/cache'` present anywhere.
- **Check:** `grep -rn "unstable_cache" apps/*/src packages/*/src`.
- **Severity:** Low.
- **Fix:** migrate to `'use cache'` — the newer primitive is more explicit and composable, and is
  what the current caching/navigation model is built around. Not urgent; `unstable_cache` still
  functions.

## Upgrade-triggered visual diff (a heads-up, not a fix)

- **`ImageResponse` default font changed (Noto Sans → Geist Sans) across 16.2.** If the app calls
  `ImageResponse` (commonly for OG images) and crosses the 16.2 boundary, expect a visual diff in
  generated images with no code change required to reproduce it. Not a bug — flag it in the
  upgrade notes so nobody chases a "why did our OG images change" ticket blind.

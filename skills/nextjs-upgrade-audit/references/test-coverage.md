# Test Coverage — Next.js Upgrade Audit

Findings in this tier are missing regression guards, not defects in the app itself — but each one
means a real behavior change (Instant Navigations, per-route prefetching) has no automated check
protecting it.

## 1. `@next/playwright` absent while an e2e package exists

- **Predicate:** `packages/e2e` (or equivalent) exists in the repo, and `@next/playwright` is not
  a dependency anywhere.
- **Check:** `grep -l "@next/playwright" packages/e2e/package.json apps/*/package.json
  2>/dev/null` — absence is the finding.
- **Severity:** Medium.
- **Fix:** add `@next/playwright` and adopt its `instant()` helper — it asserts what's visible
  *before* the network settles, which is exactly the regression class Instant Navigations
  introduces (a `cookies()` call added to a shared header, or a `<Suspense>` boundary moved during
  a refactor, silently turns an instant navigation into a blocking one with no other test
  catching it).
- **Route to vendor:** this is a first-party test primitive — don't hand-roll an equivalent timing
  assertion.

## 2. Route with zero instant-navigation signal

- **Predicate:** a route segment contains zero `<Suspense>` boundaries, zero `'use cache'`
  directives, and no `export const instant = false`.
- **Check:** per route directory, `grep -c "Suspense\|'use cache'" page.tsx layout.tsx`;
  `grep "export const instant"`.
- **Severity:** Low — a real finding, but not urgent unless the route is high-traffic.
- **Report:** every navigation to this route blocks — there's no cached shell, no streamed
  fallback, and no `instant = false` documenting a *deliberate* choice to block. It's ambiguous
  whether this is an oversight or an unstated decision; the audit's job is surfacing it, not
  deciding it.

## 3. Aggressive per-link prefetching on a list-heavy dynamic route

- **Predicate:** a dynamic-segment route (e.g. `/[id]`) is linked from a list of ≥10 sibling items
  (a sidebar, a nav list), those `<Link>`s pass `prefetch={true}`, and `partialPrefetching` is off.
- **Check:** grep the list-rendering component for `<Link ... prefetch={true}` inside a `.map()`;
  cross-check `next.config.*` for `partialPrefetching`.
- **Severity:** Low.
- **Report:** this is the shape Next.js's own example calls out — twenty links in a sidebar
  sending twenty separate prefetch requests. With `partialPrefetching` on, per-route shells
  replace the per-link cost; without it, each link still pays its own request.

## Suppress — not findings

Two patterns can *look* like the checks above but are false positives; don't flag them.

- **`<Link transitionTypes={...}>` shared between an App Router and a Pages Router wrapper.**
  `transitionTypes` is silently ignored on Pages Router links by design — this is exactly what
  makes a shared `<Link>` component safe across both routers. Don't report it as a bug.
- **A dynamic `import()` that once existed only to dodge barrel-file tree-shaking.** As of
  Next.js 16.2, Turbopack tree-shakes destructured dynamic imports the same as static ones — the
  workaround is obsolete, but its mere presence isn't a new problem; only flag it if it's actively
  creating an unnecessary code-split boundary.

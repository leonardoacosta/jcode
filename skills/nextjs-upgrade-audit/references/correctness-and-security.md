# Correctness & Security — Next.js Upgrade Audit

Findings in this tier are defects, not opportunities: something is silently broken, silently
inert, or silently insecure. Report these first, and lead with them in any summary.

## 1. `--webpack` forfeits the entire 16.3 Turbopack line

- **Predicate:** `next` >= 16.3, and a `dev`/`build` script passes `--webpack`.
- **Check:** `grep -E '"(dev|build)":.*--webpack' apps/*/package.json` (or the root
  `package.json` in a single-app repo).
- **Severity:** High.
- **Fix:** drop `--webpack`. Turbopack has been the default bundler since Next.js 16; the flag is
  an explicit opt-out that also forfeits `import.meta.glob`, the dev/build filesystem caches, and
  memory eviction — not just the headline numbers.
- **Note:** if the flag is load-bearing for a real incompatibility, that reason belongs in a
  comment next to the flag, not left silent.

## 2. `experimental.cachedNavigations` without `cacheComponents`

- **Predicate:** `next.config.*` sets `experimental.cachedNavigations: true` but does not set
  `cacheComponents: true`.
- **Check:** read `next.config.*`. The dependency is stated by Next.js itself, not inferred here.
- **Severity:** Medium.
- **Fix:** enable `cacheComponents: true` too, or remove `cachedNavigations` — as configured today
  it is a no-op.

## 3. Shared layout/header calling `cookies()` or `headers()`

- **Predicate:** a component rendered inside a shared `layout.tsx` (or nested under one) calls
  `cookies()` or `headers()`.
- **Check:** `grep -rl "cookies()\|headers()" apps/*/src/app/**/layout.tsx` and any shared
  header/nav component; confirm the calling component sits above route content in the tree, not
  inside a single leaf page.
- **Severity:** High.
- **Fix:** move the call down to the specific segment that needs it, or wrap the header's dynamic
  slice in its own `<Suspense>` so it doesn't de-opt siblings.
- **Why it's high:** this is Next.js's own named failure mode for Instant Navigations — a
  `cookies()`/`headers()` call in a shared header silently de-opts **every descendant route** to
  request-time rendering. It reads as a one-line change; the blast radius is the whole subtree.

## 4. Nonce-based CSP on an app relying on prerendered shells

- **Predicate:** a CSP with a `nonce` directive is present (`next.config.*`,
  `proxy.ts`/`middleware.ts`, or a response-header helper) **and** `cacheComponents`/Partial
  Prefetching is enabled.
- **Check:** `grep -rn "nonce" proxy.ts middleware.ts next.config.*` cross-referenced against
  `cacheComponents: true`.
- **Severity:** High.
- **Fix:** consider `experimental.sri: { algorithm: 'sha256' }` as an alternative CSP strategy —
  Subresource Integrity verifies script hashes without forcing dynamic rendering.
- **Confidence:** this pairing is **inference**, not a stated Next.js recommendation — no source
  states SRI as *the* fix for this combination. The two underlying facts are independently
  documented (nonce-based CSP forces all-pages-dynamic; Partial Prefetching depends on prerendered
  shells); the bridge between them is reasoning. Present it as a suggestion, not a directive.

## 5. No CSP found in-repo, and no `experimental.sri`

- **Predicate:** no `Content-Security-Policy` string found anywhere in the repo, and
  `experimental.sri` is absent from `next.config.*`.
- **Check:** `grep -rn "Content-Security-Policy" .` plus a read of `next.config.*`.
- **Severity:** Low-to-Medium — report, don't escalate.
- **Confidence:** **Low, by construction.** A miss here is not proof of absence — CSP is commonly
  set at the edge/CDN (a WAF, a platform firewall) where it is invisible to a repo grep. Phrase
  the finding as "no CSP configured **in this repo**," never as "this app has no CSP."

## 6. `turbopack.ignoreIssue` suppressing the app's own diagnostics

- **Predicate:** `turbopack.ignoreIssue` matches a path under the app's own source tree (not
  `node_modules`, not a `vendor/` or generated-code directory).
- **Check:** read `next.config.*` for `turbopack: { ignoreIssue: [...] }`; check each matcher's
  `path` against the app's own `src/`/`app/` tree.
- **Severity:** Medium.
- **Fix:** narrow the matcher to the third-party/generated path it was meant for, or remove it.
  `ignoreIssue` exists for vendored or generated code producing noise — using it against
  first-party source silences real warnings instead.

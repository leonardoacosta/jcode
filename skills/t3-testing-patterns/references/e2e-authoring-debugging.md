
# E2E Authoring and Debugging

Read this reference while writing specs, choosing waits, debugging a hang, adding lint gates,
piloting visual regression, or maintaining audit journeys.

## Base Fixture Imports

Import Playwright values from the application fixture:

```typescript
import { test, expect } from "@fixtures/base";
import type { BrowserContext, Page } from "@playwright/test";
```

Raw `test` or `expect` value imports from `@playwright/test` bypass headers, target identity,
personas, mutation tracking, or other application fixtures while still compiling. Type-only
imports are allowed. Enforce the value-import boundary with ESLint plus a lightweight CI scan.

For public/logged-out tests, opt out explicitly:

```typescript
test.use({ storageState: undefined });
```

## Deterministic Wait Hierarchy

Prefer the signal produced by the action and use these starting budgets:

| Wait | Use | Default maximum |
|---|---|---:|
| `expect(locator).toBeVisible()` / web-first locator assertion | Element state after an action | 5s |
| `page.waitForLoadState("domcontentloaded")` | Initial document render | 8s |
| `page.waitForSelector("[data-hydrated]")` | Explicit hydration boundary | 8s |
| `page.waitForResponse(predicate)` | Exact API response caused by the action | 10s |
| Submit -> redirect -> rendered state | Bounded multi-step workflow | 20s |

Anything above 20 seconds requires investigation before a budget increase. `waitForTimeout`
requires a nearby `@wait-exception` with a concrete reason and is reserved for behavior with no
observable condition. Avoid `Promise.race` against arbitrary timers: higher concurrency turns
latent timing assumptions into flakes.

Wrap multi-action workflows in `test.step` so traces and reports identify the failing phase:

```typescript
await test.step("submit approved change", async () => {
  await page.getByRole("button", { name: "Submit" }).click();
  await expect(page.getByText("Change accepted")).toBeVisible();
});
```

## Stuck-Spec Debugging

Use the validated root wrapper and escalate in order:

1. `pnpm test:e2e -- path/to/example.spec.ts --headed`;
2. the same selected spec with `--debug`;
3. a temporary `page.pause()` at the suspected transition;
4. an ephemeral screenshot outside the repository root;
5. the retained CI trace and sanitized network/console evidence.

Do not bypass target validation. A headed/debug run uses the fixture's authenticated persona by
default; set anonymous storage explicitly when investigating a public/auth boundary. Remove pauses
and temporary screenshots before completion.

## Runnable Non-ESLint Gate

Place a script like this at `packages/e2e/scripts/lint-gates.mjs` and run it from the package's
`lint:gates` key. It deliberately uses only Node built-ins:

```javascript
import { readdirSync, readFileSync, statSync } from "node:fs";
import { join, relative } from "node:path";

const root = new URL("../", import.meta.url).pathname;
const tests = join(root, "tests");
const files = [];
const walk = (dir) => {
  for (const name of readdirSync(dir)) {
    const path = join(dir, name);
    if (statSync(path).isDirectory()) walk(path);
    else files.push(path);
  }
};
walk(tests);

const failures = [];
for (const file of files) {
  const rel = relative(root, file).replaceAll("\\", "/");
  const source = readFileSync(file, "utf8");
  if (/\.(?:test\.ts|test\.js|spec\.js)$/.test(file))
    failures.push(`${rel}: authored tests must use .spec.ts`);
  if (/import\s*\{[^}]*\b(?:test|expect)\b[^}]*\}\s*from\s*["']@playwright\/test["']/.test(source))
    failures.push(`${rel}: import test/expect from @fixtures/base`);
  const waits = source.match(/waitForTimeout\s*\(/g)?.length ?? 0;
  const exceptions = source.match(/@wait-exception:\s*\S.+/g)?.length ?? 0;
  if (waits > exceptions)
    failures.push(`${rel}: ${waits} fixed waits but ${exceptions} justified exceptions`);
  if (/\btest\.skip\s*\(/.test(source) && !/@quarantine-id:\s*\S+/.test(source))
    failures.push(`${rel}: skip requires structured quarantine identity`);
}
if (failures.length) {
  console.error(failures.sort().join("\n"));
  process.exit(1);
}
```

Run from the root through the package script, for example
`pnpm --filter <e2e-package-name> lint:gates`. Validate the optional collection-exclusions
manifest in the collection gate because it also compares paths against authored/collected sets;
comments are not substitutes for structured quarantine or exclusion metadata.

## Visual Regression Pilot

`toHaveScreenshot` is opt-in, not a fleet default. Pilot a small stable tagged subset with human-
approved committed baselines, pinned runner OS/browser versions, failure-diff artifact upload, and
an intentional baseline-update workflow. Evidence screenshots are not golden baselines. Do not
expand the pilot until curation and update review are proven.

## Audit-Journey Contract

Engineer-generated audit journeys are permanent Playwright regression specs, not chat artifacts.
Use this concrete contract:

| Surface | Contract |
|---|---|
| Project | `audit-journeys`; no default storage state, each spec selects exact persona or anonymous state |
| Tag | `@audit-journey`, registered in the repository tag gate |
| Location | `packages/e2e/tests/audit-journeys/<journey-slug>.spec.ts` |
| Viewports | Desktop `1280x800` and mobile `375x667` |
| States | Loading, data, reachable error and empty states, plus responsive variants |
| Screenshots | Ignored ephemeral evidence directory under `packages/e2e/screenshots/audit-journey-eval/<persona>/<journey>/` |
| Accessibility | Axe sidecar JSON per captured state with documented known-issue filtering |
| Performance | Sidecar JSON per viewport using the repository's accepted navigation/render metrics |
| Network | One sanitized first-party response-duration JSON file; no headers, bodies, query secrets, or cookies |

Specs import `test`/`expect` from `@fixtures/base` and may reuse public screenshot and persona-path
helpers; they do not import private persona implementation helpers. Contract/schema changes go
through the journey generator, while manual edits are limited to focused bug fixes. Every journey
follows ordinary target, mutation ownership, cleanup, quarantine, and artifact-secret rules.

Audit evidence must never contain uploaded storage-state files. Before upload, scan/redact artifacts
as required by the topology reference; if safe publication cannot be proven, fail the evidence gate
and retain nothing rather than uploading potentially sensitive output.

A visual-tagged subset may join the visual pilot, but ordinary audit screenshots remain analyst
evidence and are not silently converted into baselines.

## Authoring Review

Before accepting a spec, verify base-fixture imports, explicit persona/anonymous state, observable
waits, named steps for long journeys, deterministic server state, exact mutable-fixture teardown,
and safe artifacts. Load the ownership reference for any mutation and the topology reference for
any collection, retry, artifact, or parallelism change.

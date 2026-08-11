
# E2E Topology and Evidence

Read this reference when configuring projects, collection, browsers, shards, reporting, quarantine,
retry classification, or worker budgets.

## Completeness Collection

Collection mode exposes one project named exactly `chromium` with no `grep`, `grepInvert`, path
filter, setup dependency, compatibility project, `webServer`, or other execution graph. It is valid
only under explicit `--list` (or equivalent proven no-test-body operation); reject collection
authority during test execution. This project owns full-suite completeness. Normalize repository-
relative POSIX paths and compare deduplicated sets:

1. authored `*.spec.ts` files in scope, minus validated exclusions; and
2. exact-project records from `playwright test --list --project=chromium`.

Fail with sorted missing and unexpected paths. Browser compatibility projects are additive and do
not divide or duplicate the completeness contract.

The sole optional exclusion source is `packages/e2e/e2e-collection-exclusions.json`. Absence means
an empty exclusion set. If present, require an array of unique `{ path, kind, reason }`: `path` is
an existing repository-relative non-glob POSIX path, `kind` is only `setup` or `helper`, and
`reason` is non-empty. Reject unknown fields, stale paths, globs, spec exclusions, config filters,
and every other hidden exclusion source.

## Projects, Shards, and Browser Closure

Use native `--shard=N/M` for canonical Chromium distribution. Do not model shards as Playwright
projects. Add browser compatibility through tagged projects such as `critical-firefox`,
`critical-webkit`, and `critical-mobile`; they may select `@critical` and exclude structured
quarantine without changing canonical collection.

Install the dependency closure for every selected project. If setup uses Chromium, a Firefox-only
job still installs Chromium plus Firefox. WebKit is an expected project on every platform by
default; do not infer omission from the host OS. Omission requires an explicit unsupported-platform
capability validated before config load, a visible expected-project inventory entry, and a retained
omission evidence record. Missing validation or evidence fails rather than silently dropping WebKit.

Keep these controls distinct:

- **workers**: concurrent test workers inside one Playwright invocation;
- **shards**: deterministic partitions of a suite across invocations;
- **workflow matrix concurrency**: concurrent CI jobs/lanes.

Evidence for one does not justify increasing another. Isolated shard targets do not prove multiple
workers inside a shard can safely mutate.

## Lane Evidence and Inventory

Each expected project/shard emits a blob report and machine-readable JSON or JUnit output. Under
`always()`, inventory expected lane IDs and artifact names (including validated capability
omissions), fail on missing/duplicate/malformed artifacts, merge blobs, upload safe raw/merged
evidence, and retain cleanup/provider ledgers. A Playwright pass without its expected artifact is
incomplete.

### Artifact secret policy

Never upload Playwright storage-state files; they contain reusable cookies/tokens. Before any
upload, scan every report, trace, screenshot metadata, log, sidecar, URL, and merged artifact.
Remove or irreversibly redact secrets, cookies, authorization headers, API/provider keys,
credentials, sensitive query parameters, and sensitive request/response headers and bodies. Do not
rely on reporter defaults. If content cannot be scanned or safely redacted, fail closed and do not
upload it. Restrict artifact access to the minimum authorized audience, apply the shortest useful
retention, prevent public links, and record scanner/redaction results in the artifact inventory.
Sanitization failure makes the run incomplete even when tests pass.

Use one CI retry. After execution, a classifier reads machine-readable results and emits records
for every failed-first/passed-retry test, including stable test ID, project, shard, retry, error
class, and artifact links. Feed these records to history and triage. Retry-pass policy may either
fail or warn according to an explicit repository decision, but the run is always flaky and never
clean benchmark evidence. Report generation alone is not flake classification.

## Quarantine

Quarantine entries carry stable test identity, owner, reason, issue, creation date, review date,
and expiry. CI rejects missing ownership, expired entries, unknown tests, and net-new skip debt
above the ratcheted budget. Bare `test.skip()` is not quarantine metadata.

A mandatory scheduled or manual lane executes quarantined tests and publishes normal evidence so
regressions remain visible. Main-lane exclusion never means tests disappear permanently.

## Worker Benchmarks

A benchmark record includes commit, target class and opaque target identity, immutable build ID,
full-suite/unsharded status, worker count, retries/flakes, duration, failures, cleanup status,
provider escaped-record status, and relevant capacity/error telemetry. Compare only records with
the same target class and materially equivalent lane shape.

Increasing workers requires two adjacent clean full-suite unsharded runs at the proposed count.
Clean means no initial failure, retry pass, missing artifact, quarantine regression, cleanup leak,
or provider escape. A sharded run, partial/tagged run, different target class, or single lucky pass
does not qualify. Missing or malformed evidence selects the conservative one-worker fallback.

Before attributing concurrency flakes to capacity:

1. compare database connections with configured limits;
2. inspect deployment error rate/status codes over the run window;
3. diff stable test IDs against the lower-worker baseline; then
4. investigate fixed-timeout races, shared personas, and ownership gaps.

Auth 429s on non-production deployments additionally require the Better Auth scoping check in
`t3-code-patterns/references/better-auth-rate-limiting.md`.

## Clean-Run Gate

A run may be called clean only when all expected lanes or validated capability omissions, reports,
classification, cleanup, escaped provider checks, secret scans, and quarantine gates complete
successfully and no test needed its retry. Preserve only safe partial evidence when the gate fails;
do not collapse incomplete into green.

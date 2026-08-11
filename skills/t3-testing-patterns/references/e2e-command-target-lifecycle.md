
# E2E Commands, Targets, and Lifecycle

Read this reference when selecting an E2E command or target, wiring environment loading, or
authoring an isolated CI lane.

## Public Command Contract

Every in-scope repository exposes package-script keys named exactly `test:e2e` and
`test:e2e:local` in its root `package.json`:

- `pnpm test:e2e -- <selectors>`: canonical manual deployed behavior.
- `pnpm test:e2e:local -- <selectors>`: explicit local behavior.

These are exact public key names, not suggestions for aliases with different names. Their values
may delegate through repository-specific internals. The behavioral chain is root script -> Turbo
task -> E2E package script -> one package-owned `dotenvx run --overload --quiet` boundary ->
validated runner -> Playwright. Internal Turbo and package task names are judged by behavior.

The package boundary owns env files because it knows their relative paths. Never nest dotenv,
`process.loadEnvFile`, custom parsers, or another dotenvx invocation in config, fixtures, or helper
modules. Arguments must survive every wrapper, including file selectors, `--project`, and
`--shard`. Consume each wrapper's own separator so pnpm's literal `--` does not arrive as an
unknown Playwright argument. Test this with selectors before accepting a wrapper.

Normal instructions use root commands. Direct `playwright test` is diagnostic-only and must
reproduce the same env and target validation first.

## Target Classes and Fail-Closed Validation

| Mode | Allowed target | Allowed operation | Required evidence |
|---|---|---|---|
| `canonical` | Deployed HTTPS, non-production only | Test execution | Exact URL plus trusted deployment/run/database/service identities and production denial |
| `local` | Loopback only | Test execution | Exact local listener plus non-production database/service identities and production denial |
| `ci` | Loopback only | Test execution | Full run/lane, listener, deployment, database, and service identities; preflight and post-preparation production denial |
| `collection` | Valid loopback, or HTTPS if non-loopback | Explicit `--list` or equivalent no-test-body collection only | Capability-bound `list` operation; exact Chromium completeness project only |

Reject every other mode/target/operation combination. Every non-loopback target requires HTTPS.

Validation runs before Playwright collection or config import can trigger work. Missing, empty,
malformed, non-HTTP(S), credential-bearing, query-bearing, fragment-bearing, or mode-incompatible
`E2E_BASE_URL` fails. Authentication/bypass credentials travel only in protected headers or
cookies, never userinfo, query, or fragment; redact them from logs, traces, and reports. `BASE_URL`
is guidance-only during migration: if it is set while `E2E_BASE_URL` is absent, explain the rename
and fail. It is never a fallback. Never supply a config-level localhost default.

Every mutation-capable mode must deny production through trusted read-only identity sources before
any provisioning, migration, seed, setup, provider, or test mutation, then revalidate the same
identities after preparation and before application/test mutation. Validate a trusted deployment-
environment assertion, deployment ID, opaque database identity, and opaque external-service
identity against an allowlisted non-production policy. DNS shape alone is not
evidence: canonical deployed means a proven non-production deployment, not merely any non-loopback
host. Local and isolated CI modes likewise reject production databases/services even when the app
listener is loopback. Collection-only mode must enforce that no server mutation can occur.

Safe pre-run diagnostics include target class, sanitized origin classification, non-sensitive
deployment ID, run/lane/project/shard, and opaque database/service identity fingerprints. Never log
URLs containing credentials, passwords, tokens, cookies, provider keys, raw connection strings, or
reversible identity material. Reject `localhost` (including a trailing dot), `127.0.0.0/8`, `::1`,
IPv4-mapped loopback, and equivalent forms for canonical deployed runs. The validated wrapper is
authoritative; template classification is defense in depth. For isolated CI, loopback is
insufficient evidence by itself: validate identities propagated by provisioning and app startup.

## Private Validated-Runner Capability

A naked URL is never authority to run. After read-only validation, the wrapper creates a
short-lived capability that binds exact mode, operation (`execute` or `list`), canonical URL,
target class, trusted non-production deployment identity, run/lane identity, database identity,
service identity, issue/expiry time, and validated browser omissions. The capability uses a cryptographically random 32-byte token passed
separately and compared in constant time. Store it in a current-user-owned regular non-symlink file
with no group/other permissions, inside a private run-owned directory; never commit or upload it.
The template validates file ownership/type/mode, token, expiry, all bindings, and explicit
`nonProduction: true` before loading projects. Missing, stale, mismatched, or insecure capability fails even when `E2E_BASE_URL` is valid.
Collection mode additionally proves the actual runner invocation contains `--list` (or an equivalent
validated no-test-body operation); a capability field alone cannot convert test execution into
collection.

## Immutable CI Lifecycle

A `ci` lane owns its dependencies and executes finite phases in this order:

1. **Before any provisioning, migration, or seed mutation**, use trusted read-only control-plane
   sources to resolve the deployment assertion and the intended/reserved database and service
   identities. Deny production and reject unknown/ambiguous identity. Pre-authorize only a
   run-owned resource namespace.
2. Mint the private validated-runner capability binding that authorization and planned identities.
3. Provision only the pre-authorized isolated database/service resources under the run/lane identity.
4. Apply migrations and seed a deterministic baseline plus the small eager persona set.
5. After preparation, re-resolve the deployment, run, database, and service identities from trusted
   sources; require them to be unchanged and still non-production. Rotate/reissue the capability if
   preparation produces final immutable identity values. Stop before application/test mutation on
   any drift.
6. Validate schema version and persona catalog, then build the application exactly once.
7. Start that immutable build on its run-owned listener; do not reuse an existing listener or dev
   server. Poll bounded readiness and require the server to attest the same capability-bound target.
8. Run the serial setup project once to produce import-safe state.
9. Execute the selected project/shard with `--no-deps`, so each shard does not repeat setup.
10. Under `if: always()`, perform bounded cleanup, session revocation, escaped-record checks,
    process termination, capability deletion, artifact inventory, secret scanning/redaction, safe
    merge, and restricted upload.

The immutable build/start rule applies to isolated CI. A developer's explicit local command may
reuse a deliberately started local server, but must still verify listener and target identity.

## Collection Isolation

Collection mode exposes exactly one unfiltered project named `chromium`. It has no setup dependency,
compatibility projects, `webServer`, or execution dependency graph. Invoke Playwright with `--list`
(or a wrapper operation independently proven not to execute test bodies). Reject collection mode
without that operation and reject test execution under collection authority. Authored-versus-
collected equality consumes only this isolated surface.

## Timeouts and Termination

Set finite phase and job timeouts. The job timeout must reserve headroom after test timeout for
cleanup, report merge, and upload. Cleanup itself is bounded and reports partial failure. Record
process IDs or ownership tokens when starting services; terminate only run-owned processes. Never
use broad `pkill`, kill by generic port, or listener reuse that can destroy another lane or a
developer process.

Cleanup failure, production-denial/target-identity mismatch, missing expected evidence, unsafe
artifacts, or an unowned process makes the lane incomplete even if Playwright tests passed.

## Prohibited Shortcuts

- Silent localhost/default URL fallback.
- Treating `BASE_URL` as runtime input.
- Nested env loaders, secret-bearing diagnostics, or uploading storage state, capability files, or
  unsanitized artifacts.
- Accepting a naked base URL, client-asserted run/owner headers, or an insecure/stale capability.
- Treating any non-loopback host as sufficient proof of a safe non-production target, or allowing
  non-loopback HTTP.
- Using local mode for a deployed target, canonical mode for loopback/HTTP, CI mode for non-loopback,
  or collection authority for test execution.
- Building separately per shard after provisioning one supposedly immutable target.
- Letting each shard rerun setup dependencies instead of setup-once plus `--no-deps`.
- Starting with listener reuse or stopping processes not proven to belong to the run.
- Infinite readiness, test, or cleanup waits.

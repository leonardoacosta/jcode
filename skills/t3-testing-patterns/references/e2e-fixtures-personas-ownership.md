
# E2E Fixtures, Personas, Ownership, and Provider Seams

Read this reference when designing auth state, mutable fixtures, cleanup, parallel safety, or
deterministic provider behavior.

## Persona Catalog

Maintain one typed catalog as the source of truth for each persona's role intent, credential
lookup key, storage-state path, and eager/lazy classification. Setup validates the catalog and
creates only import-safe readiness state plus a small eager set used broadly. Specialized personas
remain lazy to avoid login bursts and shared-session races.

A test uses an isolated browser context with the exact persona's storage path. Storage state lives
under a gitignored run-owned directory created mode `0700`; each state file is mode `0600`, written
to a same-directory private temporary file and atomically renamed. Bind state to the run/capability,
set a bounded lifetime no longer than the run, and never reuse it across runs. On completion or
failure, revoke server sessions first, delete state/capability files, and remove the run directory.
A test may perform at most one bounded recovery login for that same persona, atomically replace only
that persona's state, and retry readiness. It must never fall back to another persona, a default
authenticated page, or a more privileged session. Missing/invalid state fails safely.

Public and auth-boundary tests declare anonymous state explicitly, for example
`test.use({ storageState: undefined })`. Do not rely on the project's default persona. A headed or
debug session inherits fixture storage unless explicitly overridden, so verify the active persona
before drawing conclusions.

## Fixture Classes

| Fixture | Scope | Contract |
|---|---|---|
| Import-safe readiness | setup project | Schema/health checks and auth-state files only |
| Shared baseline | run/lane | Deterministically seeded, immutable/read-only during tests |
| Mutable test data | test/retry | Created for one owner, registered immediately, deleted/restored exactly |
| Persona context | test | Isolated context with exact storage state; no privilege substitution |

Server-prefetched and React Server Component data must come from deterministic seeded server state.
Browser `page.route` cannot establish server-side state and may bypass the behavior under test.
Use request interception only when browser transport behavior itself is the subject, such as an
offline banner or malformed response.

## Mutation Ownership

A mutation owner is narrower than a run ID. Include run, project, shard or lane, worker, stable test
ID, and retry. The application derives run/owner identity server-side from a short-lived,
authenticated, non-production-only runner capability and trusted request/session context. Reject
client-asserted run/owner headers, body fields, query values, or unsigned cookies; they are not
authority. Include only a correlation handle from the client when needed, then resolve and verify it
server-side. Give each create/restore operation an exact operation ID under that owner. Immediately
after a create succeeds, register its exact database/provider IDs before continuing. If capability,
identity, or provenance is foreign/unknown, fail closed before mutation and retain safe evidence.

**Authorize before update/delete.** Before issuing the side effect, the mutation boundary must prove
that the exact target ID was created by the same narrow owner, or acquire an explicit bounded
reversible lease containing target ID, prior-state snapshot/version, owner, expiry, and restore
operation. Reject foreign, shared-baseline, production, expired-lease, unowned, or ambiguous targets
before the application/provider mutation occurs. A post-hoc receipt cannot authorize a side effect
that already touched someone else's record.

Cleanup deletes children before parents and restores prior values in dependency-safe order. It is
idempotent, bounded, and limited to exact registered IDs; broad prefix/time-window deletion is not
proof of ownership. Retain a cleanup ledger containing attempted operation, exact IDs, outcome,
and remaining leaks. Run escaped-record checks after cleanup and fail on owned survivors or
unexplained records carrying the run identity.

### Automatic attribution

Instrument the application mutation boundary so fixtures do not depend on every author remembering
to register manually:

1. Classify the procedure/route operation as create, update, delete, restore, or read before mock or
   response provenance is considered.
2. Attach the current narrow owner to server mutation receipts.
3. Exempt only an exact synthetic response proven not to reach the server.
4. For creates, require returned exact identity. Before updates/deletes, authorize an owner-created
   exact ID or acquire the reversible lease described above; only then perform the side effect.
5. Attach prior state/version and restore operation to update/delete receipts, and reject lease or
   ownership drift before mutation.
6. Fail the test when a real mutation has missing, foreign, production, or ambiguous provenance.

A mocked response is not automatically non-mutating: classify the procedure first. Conversely, an
exact browser-only synthetic response with proof of no server hit need not enter the cleanup ledger.

## Serial Default and Earned Concurrency

Mutation-capable lanes start with `fullyParallel: false` and one worker. Isolation by database or
shard does not prove tests within that lane are safe. Increase workers only after all mutation
adapters emit narrow ownership, personas use isolated contexts without shared live-login races,
cleanup/escaped checks are clean, and two adjacent comparable full-suite benchmark runs pass at
the same target class. Read-only lanes may opt into parallelism explicitly when their immutability
assumption is enforced.

## Deterministic Provider Adapters

Payments, email, blob writes, and outbound webhooks pass through application-owned interfaces. In
explicit non-production E2E mode, the composition root selects a deterministic adapter with:

- server-derived run/owner identity from the authenticated short-lived non-production capability,
  never a client-asserted header/body/query value;
- a run-scoped store;
- realistic idempotency keys, allowed state transitions, errors, and response shapes;
- safe exact receipts containing operation, outcome, provider/application IDs, owner, and retry;
- exact reset and post-reset escaped-record verification;
- rejection of missing/foreign run identity; and
- no route to construct or fall back to a production client.

The browser still drives routing, authorization, validation, domain services, database transaction,
response handling, and UI. Only the final irreversible provider operation is substituted. Denial
and authorization regressions never target a shared live provider account.

Production and deterministic adapters share behavioral contract suites. Separately report a small
real-provider integration lane using approved non-production credentials to prove credential
handling, SDK mapping, idempotency, and response parsing. Deterministic adapter success is E2E
business-flow evidence, never live-provider integration evidence.

## Acceptance Checklist

- Every persona maps to one role and one run-owned private storage path; state is atomic, bounded,
  gitignored, revoked, and deleted, with no privilege fallback.
- Every create has exact identity; every update/delete is pre-authorized by same-owner provenance
  or an explicit bounded reversible lease before the side effect.
- Child-before-parent cleanup and escaped-record results survive as evidence.
- Unknown mutation provenance fails rather than becoming a warning.
- Provider receipts and reset checks are run-scoped, and production fallback is impossible.
- Parallelism remains disabled until ownership and same-target evidence prove it safe.

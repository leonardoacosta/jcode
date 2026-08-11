
# Better Auth Rate-Limit Policy

Read this reference when preview/development sign-ins flake, return 429, or appear to hit a
concurrency ceiling.

## Keep Reachable Environments Protected

Better Auth's default production signal derives from `NODE_ENV === "production"`, while hosted
preview/development deployments commonly also use production Node mode. The correction is not to
disable the limiter everywhere outside production. Any preview/dev endpoint reachable by other
users or the public internet keeps rate limiting enabled with an explicit environment-appropriate
policy: strict production policy in production, and an abuse-resistant preview/development policy
sized for legitimate interactive use.

```typescript
export const authRateLimitPolicy = deploymentEnvironment === "production"
  ? productionRateLimitPolicy
  : previewRateLimitPolicy; // still enabled; explicit bounded windows and limits
```

Do not use `enabled: VERCEL_ENV === "production"`: that globally removes protection from reachable
preview/dev deployments. Do not weaken production thresholds while correcting environment scope.

## Isolated E2E Capacity or Bypass

When authenticated E2E login bursts exceed the normal preview policy, prefer cached run-owned
storage state. If live login remains necessary, choose one narrowly controlled option:

1. provision isolated-E2E rate-limit capacity/store under the run identity; or
2. issue a short-lived authenticated bypass/capacity grant bound server-side to the validated
   non-production runner capability, deployment, run, persona, route set, and expiry.

The server verifies the capability and derives identity itself. Never trust a client-asserted bypass,
run, or owner header; never accept bypass credentials in a base-URL query/fragment; send protected
credentials through a redacted header or secure cookie. Reject production targets before grant
issuance and at request use. Revoke the grant during cleanup. No shared static bypass secret, broad
IP allowlist, or production fallback is allowed.

## Config Lint Boundary

Repositories that ban direct environment reads keep deployment-policy selection in a narrow typed
config module allowed by the env-access rule. Unit-test policy selection without importing full
auth/database initialization. Do not broadly allowlist the auth package or bypass lint inline.

## Concurrency Triage

When auth flakes appear after increasing E2E concurrency:

1. Confirm status codes and rate-limit evidence rather than widening Playwright timeouts.
2. Resolve deployment class from the validated deployment identity, not `NODE_ENV` alone.
3. Verify production keeps strict policy and reachable preview/dev keeps its explicit bounded policy.
4. Check for redundant live logins where a run-owned persona storage state already exists.
5. If needed, verify isolated capacity or bypass is authenticated, short-lived, non-production-only,
   persona/route scoped, server-derived, and revoked after the run.
6. Rerun at the conservative worker count and apply the benchmark gate in
   `t3-testing-patterns/references/e2e-topology-evidence.md`.

Removing a misscoped limiter does not prove parallel safety; persona isolation, mutation ownership,
and comparable benchmark evidence remain mandatory.

## Provenance

This guidance generalizes a fleet incident where a hosted non-production deployment inherited a
production-derived sign-in rule and concurrent E2E logins surfaced as a false capacity ceiling.

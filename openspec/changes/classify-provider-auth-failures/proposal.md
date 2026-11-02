---
execution:
  version: 1
  depends_on: []
---
Roadmap owner: [P06: Provider/auth/usage-budget reliability](../../../docs/plans/PRODUCT_ROADMAP.md#p06-providerauthusage-budget-reliability)

## Why

Failure handling is split across ad hoc string matching: Anthropic matches substrings (`crates/jcode-provider-anthropic-runtime/src/error_classification.rs:7-40`), `classify_failover_error_message` returns only `None|RetryNextProvider|RetryAndMarkUnavailable` (`crates/jcode-provider-core/src/failover.rs:36-72`), and OpenAI handles catalog 401/403 with a one-off force-refresh (`crates/jcode-provider-openai-runtime/src/openai_provider_impl.rs:7-11,963-983`). No shared category distinguishes expired-refreshable, revoked, quota-exhausted and model-unavailable, so the user message and recovery action differ per provider. `fallback-on-transient-model-errors` covers transient errors only and explicitly never switches on auth.

## What Changes

- Add a shared `ProviderFailureKind` in `jcode-provider-core`: `AuthExpired`, `AuthRevoked`, `QuotaExhausted`, `RateLimited`, `ModelUnavailable`, `Transient`, `Permanent`.
- Map Anthropic and OpenAI runtime errors to it; `AuthExpired` triggers one refresh then retry; `AuthRevoked` stops with the exact `jcode login <provider>` remedy; never silently switch provider or identity.
- Refresh cancelled by the user leaves stored credentials unchanged.

## Non-scope

- Model fallback chains (`fallback-on-transient-model-errors`).
- Providers beyond Anthropic and OpenAI in this slice.

## Capabilities

### New Capabilities

- `provider-failure-recovery`

## Impact

`jcode-provider-core/src/failover.rs`, Anthropic/OpenAI runtime error paths, transcript error rendering.

## Resolved defaults

1. Revoked vs expired detection when the provider returns a bare 401. Options: (a) attempt refresh, classify revoked if refresh fails; (b) treat all 401 as revoked. Selected default: (a), it matches current OpenAI behavior and avoids forcing needless re-login.

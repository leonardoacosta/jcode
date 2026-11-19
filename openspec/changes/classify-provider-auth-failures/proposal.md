---
execution:
  version: 1
  depends_on: []
---
Roadmap owner: [P06: Provider/auth/usage-budget reliability](../../../docs/plans/PRODUCT_ROADMAP.md#p06-providerauthusage-budget-reliability)

## Why

Failure handling is split across ad hoc string matching: Anthropic matches substrings (`crates/jcode-provider-anthropic-runtime/src/error_classification.rs:7-40`), `classify_failover_error_message` returns only `None|RetryNextProvider|RetryAndMarkUnavailable` (`crates/jcode-provider-core/src/failover.rs:36-72`), and OpenAI handles catalog 401/403 with a one-off force-refresh (`crates/jcode-provider-openai-runtime/src/openai_provider_impl.rs:7-11,963-983`). No shared category distinguishes expired-refreshable, revoked, quota-exhausted and model-unavailable, so the user message and recovery action differ per provider. Automatic model fallback is being retired during upstream v1.0.1 adoption; this change owns the shared auth/quota taxonomy and recovery safeguards, not a model fallback consumer.

## What Changes

- Add a shared `ProviderFailureKind` in `jcode-provider-core`: `AuthExpired`, `AuthRevoked`, `QuotaExhausted`, `RateLimited`, `ModelUnavailable`, `Transient`, `Permanent`.
- Map Anthropic and OpenAI runtime errors to it; `AuthExpired` triggers one refresh then retry; `AuthRevoked` stops with the exact supported `jcode login --provider <provider>` remedy; never silently switch provider or identity.
- Cancellation before the refresh exchange starts leaves credentials unchanged. Once rotation may have been accepted, finish bounded safe persistence and the in-memory credential commit. Keep the user operation cancelled and start no subsequent inference transport.

## Non-scope

- Automatic model-fallback chain selection is being retired by `adopt-upstream-v1-0-1`; do not retain or add a fallback-chain consumer here. This change owns shared provider failure classification and auth/quota recovery safeguards only. A failed refresh transport is not proof of revoked credentials, and quota/billing exhaustion is not retryable rate limiting. Selected-route and gateway no-substitution constraints remain authoritative.
- Providers beyond Anthropic and OpenAI in this slice.

## Capabilities

### New Capabilities

- `provider-failure-recovery`

## Impact

`jcode-provider-core/src/failover.rs`, Anthropic/OpenAI runtime error paths, transcript error rendering.

## Resolved defaults

- On 2026-11-10, the user explicitly superseded historical decision `2B`: before exchange, cancellation changes no credentials; once rotation may be accepted, bounded safe persistence plus memory commit must finish, with the turn still cancelled and no later inference transport. Historical `2B` and preparation evidence remain in `tasks.md`, not active blockers on the revised boundary.

1. Revoked vs expired detection when the provider returns a bare 401. Options: (a) attempt refresh, classify revoked if refresh fails; (b) treat all 401 as revoked. Selected default: (a), attempt one refresh on the same route and identity. Infer revocation only from definitive rejection such as `invalid_grant`, never from a timeout or temporary transport failure.

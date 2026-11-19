# provider-failure-recovery Specification

## Purpose
TBD - created by archiving change classify-provider-auth-failures. Update Purpose after archive.

## Requirements

### Requirement: Shared provider failure classification

Provider errors SHALL map to one `ProviderFailureKind` with a defined recovery action. Automatic model fallback SHALL consume this shared taxonomy; confirmed quota/billing exhaustion is not transient merely because HTTP status is 429. Existing route/account no-substitution constraints remain authoritative.

#### Scenario: Expired token

- **WHEN** a request fails with 401 and refresh succeeds
- **THEN** the request SHALL be retried once with the refreshed token on the same provider and identity

#### Scenario: Revoked token

- **WHEN** refresh is definitively rejected as revoked or invalid credentials
- **THEN** the turn SHALL stop with an `AuthRevoked` remedy using supported provider-specific login syntax and SHALL NOT switch provider
- **AND** Anthropic SHALL name `jcode login --provider claude` and OpenAI SHALL name `jcode login --provider openai`, without silently selecting another account or API-key credential

#### Scenario: Refresh transport fails
- **WHEN** refresh fails because of timeout, network or temporary service failure
- **THEN** the failure retains its transport/uncertain category, credentials remain unchanged and revocation is not inferred

#### Scenario: Quota exhausted

- **WHEN** the provider reports quota or billing exhaustion
- **THEN** the error SHALL be `QuotaExhausted` and SHALL NOT be retried as transient

### Requirement: Cancellation preserves safe refresh state

The runtime SHALL keep cancellation separate from credential rotation. This boundary supersedes the strict unchanged-credentials decision `2B` recorded in the historical execution notes, as explicitly authorized on 2026-11-10.

#### Scenario: Cancelled before refresh exchange

- **WHEN** the user cancels before the refresh exchange starts
- **THEN** no exchange SHALL start and stored credentials SHALL remain unchanged
- **AND** no subsequent inference transport SHALL start

#### Scenario: Cancelled after rotation may be accepted

- **WHEN** the user cancels after a refresh exchange may have accepted a rotating token
- **THEN** bounded safe persistence and the in-memory credential commit SHALL finish
- **AND** the user operation SHALL remain cancelled and no subsequent inference transport SHALL start

#### Scenario: Authentication still fails after refresh

- **WHEN** a request is rejected again after its one auth-triggered refresh
- **THEN** the turn SHALL stop without another auth-triggered refresh or provider, route, or identity substitution

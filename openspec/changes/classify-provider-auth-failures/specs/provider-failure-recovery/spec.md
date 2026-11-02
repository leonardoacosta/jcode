## ADDED Requirements

### Requirement: Shared provider failure classification

Provider errors SHALL map to one `ProviderFailureKind` with a defined recovery action.

#### Scenario: Expired token

- **WHEN** a request fails with 401 and refresh succeeds
- **THEN** the request SHALL be retried once with the refreshed token on the same provider and identity

#### Scenario: Revoked token

- **WHEN** refresh also fails
- **THEN** the turn SHALL stop with an `AuthRevoked` message naming `jcode login <provider>` and SHALL NOT switch provider

#### Scenario: Quota exhausted

- **WHEN** the provider reports quota or billing exhaustion
- **THEN** the error SHALL be `QuotaExhausted` and SHALL NOT be retried as transient

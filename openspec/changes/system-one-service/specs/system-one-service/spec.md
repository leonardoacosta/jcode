## ADDED Requirements

### Requirement: Modular service crate

The system SHALL provide `crates/jcode-system-one` as a self-contained crate with no dependency on jcode-app-core, jcode-tui, or jcode-protocol. The crate SHALL expose a `SystemOneService` trait with a single async evaluate method and a `SystemOneResponse` type.

#### Scenario: Crate independence
- **WHEN** a crate depends on jcode-system-one
- **THEN** it does not transitively depend on app-core, TUI, or protocol types

#### Scenario: Trait contract
- **WHEN** a mock implementation returns a valid SystemOneResponse
- **THEN** callers receive the same type whether the implementation is live, cached, or mocked

### Requirement: Provider resolution

The crate SHALL resolve the provider, endpoint, credential, and effective model from environment variables, preferring OpenRouter when its credential is available, with TypeSafe as compatibility fallback. The crate SHALL NOT accept credentials from configuration files, arguments, or serialized state.

#### Scenario: OpenRouter default
- **WHEN** OPENROUTER_API_KEY is set and no provider is explicitly selected
- **THEN** the resolver returns the OpenRouter Decisions endpoint and ~typesafe/jev-latest as the default model

#### Scenario: TypeSafe fallback
- **WHEN** OPENROUTER_API_KEY is unavailable and TYPESAFE_API_KEY is set
- **THEN** the resolver returns the TypeSafe System One endpoint and jev-latest

#### Scenario: Missing credentials
- **WHEN** neither credential is available
- **THEN** the resolver returns a sanitized error identifying the missing variable without exposing any key value

### Requirement: Typed response validation

The live service implementation SHALL validate that every requested question ID appears in the response answers map. Missing answers SHALL produce a typed error. Responses SHALL return identical shapes on cache hits and misses.

#### Scenario: Valid response
- **WHEN** the API returns answers for all requested question IDs
- **THEN** the service returns SystemOneResponse with the answers, model, and optional usage

#### Scenario: Missing answer
- **WHEN** the API returns a response missing one or more requested question IDs
- **THEN** the service returns an error naming the missing question ID

### Requirement: Evaluate tool migration preserves behavior

The Evaluate tool SHALL consume the SystemOne service trait without changing its tool schema, cache keys, model defaults, retry policy, or public tool output contract.

#### Scenario: Cache hit/miss parity
- **WHEN** identical state and questions produce a cache hit and a cache miss
- **THEN** both paths return the same response shape visible to the agent

#### Scenario: Existing tests pass
- **WHEN** existing evaluate tool tests run after migration
- **THEN** all tests pass with unchanged assertions

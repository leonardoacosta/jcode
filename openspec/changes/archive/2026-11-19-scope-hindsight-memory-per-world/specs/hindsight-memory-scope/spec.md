## ADDED Requirements

### Requirement: Hindsight access is bound to a verified world

The native Hindsight adapter SHALL resolve the active world through Jcode's verified workspace policy before every operation. Each configured world SHALL have a distinct bank route and a separate credential reference. Bank identifiers SHALL be treated as routing data, not as authorization. The adapter MUST NOT infer access from a bank name or use one world's credential for another world.

#### Scenario: Known world uses its configured bank and credential
- **WHEN** a memory operation runs in a verified world with a complete Hindsight configuration
- **THEN** the request uses only that world's configured endpoint, bank and credential reference
- **AND** the request does not expose the credential value to model context or logs

#### Scenario: Unknown world has no configured route
- **WHEN** a memory operation runs with unknown or ambiguous world identity
- **THEN** the adapter denies the operation without contacting Hindsight
- **AND** it does not use a default, shared or previously selected bank

#### Scenario: World changes while recall is pending
- **WHEN** a recall completes after the verified world or session generation changes
- **THEN** the result is discarded and is not injected into the new context

#### Scenario: Cross-world recall is requested
- **WHEN** a request asks for memories owned by a different world
- **THEN** the adapter denies the request before transport
- **AND** it does not broaden the current bank query or try another credential

### Requirement: Hindsight recall is bounded and attributable evidence

The adapter SHALL bound query size, result count or token budget, request deadline, and queued work using configured or documented limits. Recalled content SHALL be represented as untrusted evidence and retain source, time, bank and provenance through the existing memory attribution path. It MUST NOT be presented as policy or instructions.

#### Scenario: Recall returns evidence
- **WHEN** Hindsight returns a memory eligible for the bounded current-world recall
- **THEN** Jcode records its source, time, selected bank and stable source identifier as provenance
- **AND** any model-visible content is labeled untrusted evidence and stays within the configured result budget

#### Scenario: Recall exceeds a bound
- **WHEN** a query, result set or pending queue exceeds its configured bound
- **THEN** Jcode rejects, truncates or declines the excess according to the documented policy
- **AND** reports that outcome without silently relaxing the bound

#### Scenario: Hindsight request times out or fails
- **WHEN** the configured endpoint fails, times out, or returns an unusable response
- **THEN** the operation reports an unavailable or failed state
- **AND** Jcode does not silently switch provider, endpoint, bank or local memory source

### Requirement: Automatic retention requires opt-in and uses stable operation identity

Automatic retention to Hindsight SHALL be disabled by default and remain disabled until the user explicitly opts in for the applicable world and data scope. Each retain operation SHALL use a Jcode-generated idempotency key derived from stable source and turn identifiers. The adapter MUST NOT claim server-side exactly-once behavior unless the configured API contract verifies it.

#### Scenario: Retention is not enabled
- **WHEN** a turn completes and automatic Hindsight retention is not explicitly enabled
- **THEN** Jcode sends no retain request

#### Scenario: Retain operation is retried
- **WHEN** a retain request is retried for the same stable source and turn identifiers
- **THEN** Jcode reuses the same operation key
- **AND** the user-visible result does not claim that the server deduplicated the write unless verified by the API contract

#### Scenario: Stable identifiers are unavailable
- **WHEN** a retain operation has no stable source and turn identifiers
- **THEN** Jcode does not retry it as though it were idempotent
- **AND** reports the duplicate-risk limitation

### Requirement: Forget semantics match the remote API

Jcode SHALL describe remote forget as soft invalidation that excludes a memory from future recall when that is the API behavior. It MUST NOT represent soft invalidation as hard erasure or claim that backups or other derived data were erased.

#### Scenario: User forgets a remote memory
- **WHEN** a user requests forget for an attributable Hindsight memory
- **THEN** Jcode requests the supported soft invalidation for that memory
- **AND** reports that the item is excluded from future recall, not guaranteed to be erased

#### Scenario: Forget operation is unavailable or fails
- **WHEN** the configured API cannot invalidate the selected memory or the request fails
- **THEN** Jcode reports the failure and leaves the memory state visibly unresolved
- **AND** does not claim successful forgetting

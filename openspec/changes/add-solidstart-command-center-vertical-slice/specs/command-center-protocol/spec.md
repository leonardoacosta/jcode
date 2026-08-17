## ADDED Requirements

### Requirement: Versioned authoritative snapshots
The Jcode daemon SHALL provide versioned command-center snapshots whose server-owned fields are derived from authoritative Jcode domain state and normalized Orca observations rather than browser persistence.

#### Scenario: Load an initiative route
- **WHEN** an authenticated client requests an initiative snapshot
- **THEN** the daemon returns the protocol version, snapshot revision, initiative identity and state, linked schedule references, linked Jcode run references, normalized Orca references, available actions, and data freshness metadata

#### Scenario: Requested initiative does not exist
- **WHEN** a client requests an unknown or inaccessible initiative ID
- **THEN** the daemon returns a typed not-found or forbidden result without exposing unrelated initiative metadata

### Requirement: Explicit authority boundaries
The protocol SHALL distinguish Jcode-owned, Orca-owned, and client-owned fields and SHALL reject commands that attempt to mutate state outside Jcode's authority.

#### Scenario: Project identity is projected
- **WHEN** an initiative is linked to an executable project
- **THEN** the snapshot uses the canonical Orca project ID and does not mint a competing executable project identity

#### Scenario: Browser attempts a direct Orca mutation
- **WHEN** a client submits a command that writes an Orca-owned worker, terminal, worktree, gate, or orchestration field directly
- **THEN** Jcode rejects the command and requires the corresponding policy-checked Jcode runtime command

### Requirement: Idempotent typed commands
Every state-changing command SHALL use a typed payload and client-generated idempotency key, and the daemon SHALL return the resulting authoritative entity state or a typed failure.

#### Scenario: Duplicate command is retried
- **WHEN** the daemon receives the same authenticated command and idempotency key more than once
- **THEN** it returns the original recorded result without applying the mutation again

#### Scenario: Command is pending
- **WHEN** a command has been accepted but its downstream runtime action has not completed
- **THEN** the response and subsequent events identify the command as pending rather than presenting the requested outcome as complete

#### Scenario: Command fails validation
- **WHEN** a payload references an invalid transition, unavailable capability, stale revision, or inaccessible entity
- **THEN** the daemon rejects it with a typed actionable error and leaves authoritative state unchanged

### Requirement: Ordered authorization-scoped resumable events
The daemon SHALL publish versioned event envelopes with an authorization-scoped stream ID, monotonically ordered stream-local sequence values, timestamps, source identity, entity references, and typed payloads. Authorization filtering SHALL occur before an event enters a browser-replayable stream.

#### Scenario: Client resumes within replay retention
- **WHEN** a connected client supplies its last accepted sequence and all later events remain available
- **THEN** the daemon replays each missing event exactly once in sequence order before streaming new events

#### Scenario: Client attempts to reuse a cursor outside its stream scope
- **WHEN** a browser session presents a stream ID or sequence created for another session, authorization scope, initiative, or route subscription
- **THEN** the daemon rejects the cursor, emits no unrelated event metadata, and requires a newly authorized snapshot and stream

#### Scenario: Authorization changes while connected
- **WHEN** the client's authorization or subscribed entity scope changes
- **THEN** the daemon closes or invalidates the prior stream and establishes a new authorized snapshot and stream ID rather than preserving the old cursor domain

#### Scenario: Replay gap cannot be satisfied
- **WHEN** a client sequence is outside retention or an event gap cannot be repaired
- **THEN** the daemon instructs the client to fetch a fresh snapshot and does not imply that the existing projection is current

#### Scenario: Unknown event type is received
- **WHEN** a newer daemon emits an event type not understood by the client
- **THEN** the client preserves sequence progress, marks the affected projection for reconciliation when required, and does not crash or reinterpret the event as another type

### Requirement: Deterministic Rust and TypeScript contracts
Command-center DTOs SHALL have one stable Rust source definition and deterministically generated TypeScript representations suitable for SolidStart clients.

#### Scenario: Generated client is stale
- **WHEN** Rust command-center schemas change without regenerating the TypeScript client
- **THEN** the contract verification command fails with a diff identifying the stale generated output

#### Scenario: Internal persistence model changes
- **WHEN** an internal Jcode or Orca persistence struct changes without changing the public command-center contract
- **THEN** generated client output remains unchanged

### Requirement: Browser authentication and request protection
The daemon SHALL require a short-lived command-center browser session for protected snapshots, commands, and event streams, and SHALL protect state-changing requests against cross-origin request forgery.

#### Scenario: Trusted local bootstrap succeeds
- **WHEN** a local trusted Jcode client requests a browser launch
- **THEN** Jcode issues a short-lived, scope-limited bootstrap that establishes a browser session without placing provider credentials in the URL or browser storage

#### Scenario: State-changing request lacks CSRF proof
- **WHEN** an otherwise authenticated browser sends a mutation without valid same-origin and CSRF proof
- **THEN** the daemon rejects the mutation and emits no domain event

#### Scenario: Browser session expires
- **WHEN** a browser session expires while a page is open
- **THEN** reads and commands fail with a typed reauthentication requirement and the UI does not silently continue as authoritative

### Requirement: Safe network exposure
The command-center HTTP listener SHALL bind to loopback by default and SHALL require explicit authenticated configuration to bind beyond loopback.

#### Scenario: Default daemon startup
- **WHEN** the command center is enabled without an explicit remote-listener configuration
- **THEN** the HTTP listener accepts connections only from loopback

#### Scenario: Non-loopback binding is requested
- **WHEN** an operator enables a non-loopback address
- **THEN** startup requires an authenticated transport configuration and origin allowlist and reports the exposed address prominently

#### Scenario: Remote browser uses an SSH bridge
- **WHEN** the Mac reaches the homelab command center through an authenticated SSH tunnel
- **THEN** provider credentials and domain execution remain on the homelab and browser access is scoped to the command-center session

### Requirement: Normalized Orca runtime projection
Jcode SHALL normalize linked Orca lifecycle data into versioned read models while preserving original Orca identifiers and observation timestamps.

#### Scenario: Orca runtime is healthy
- **WHEN** a linked Orca project and orchestration run are reachable
- **THEN** the snapshot and events expose the run, workers, terminals, gates, and relevant lifecycle status using their canonical Orca IDs

#### Scenario: Orca becomes unavailable
- **WHEN** Orca cannot be reached after previously supplying runtime state
- **THEN** Jcode retains durable initiative state, marks runtime observations stale or unavailable, includes the last observation time, and disables actions whose safe completion requires Orca

### Requirement: Closed runtime command capability set
The first command-center slice SHALL expose only `start_initiative_run`, `retry_linked_run`, and `cancel_linked_run` as Orca-mediated runtime commands, and SHALL validate authorization, entity revision, server capability, canonical Orca references, and command-specific preconditions before adapter invocation.

For the Orca `1.4.176` compatibility profile, an Orca Run SHALL be treated as a grouping namespace and an Orca Dispatch SHALL be treated as one executable attempt. Jcode SHALL durably preserve the Run, Task, Dispatch, placement, correlation, idempotency, receipt, and recovery identities separately and SHALL NOT infer a whole-Run terminal outcome from one Dispatch.

#### Scenario: Start a linked run
- **WHEN** an authorized initiative with a canonical Orca project reference has no conflicting active start and the server advertises start capability
- **THEN** Jcode persists one correlated operation per idempotency key, composes one Orca Run, Task, and supervised worker Dispatch with explicit placement, and reports acceptance only with the Jcode attempt plus Orca Run, Task, Dispatch, placement, and `ready` receipt identities
- **AND** a partial, failed, or outcome-unknown composition remains pending or failed with its observed effects and recovery obligations instead of being replayed blindly

#### Scenario: Retry a linked run
- **WHEN** an authorized failed run is retryable and the retry capability is available
- **THEN** Jcode targets the exact prior Orca Dispatch with `worker-start --retry-of`, reconstructs placement explicitly, creates at most one replacement attempt per idempotency key, and reports the new Jcode attempt and distinct Orca Dispatch identity

#### Scenario: Cancel a linked run
- **WHEN** an authorized nonterminal linked run advertises cancel capability
- **THEN** Jcode targets the exact active Orca Dispatch with stop or abandon semantics, keeps the command pending while termination is uncertain, and reports completion only after terminal worker evidence
- **AND** any retained Run, Task, worktree, terminal, or unrelated process remains an explicit cleanup or recovery obligation

#### Scenario: Compatibility profile cannot be verified
- **WHEN** the selected Orca runtime version, command registry, or JSON response shape is unknown or does not match the pinned compatibility profile
- **THEN** Jcode advertises no runtime mutation capability, invokes no lifecycle mutation, and leaves durable state unchanged

#### Scenario: Unsupported runtime command is submitted
- **WHEN** a browser submits approval, handoff, schedule mutation, direct gate, worker, terminal, worktree, arbitrary Orca, stale, unauthorized, or capability-mismatched runtime behavior
- **THEN** Jcode rejects the command before adapter invocation and leaves Jcode and Orca runtime state unchanged

#### Scenario: Orca is unavailable for a runtime command
- **WHEN** start, retry, or cancel is requested while Orca is unavailable
- **THEN** Jcode returns a typed unavailable result, does not claim success, and does not create or advance an active runtime attempt

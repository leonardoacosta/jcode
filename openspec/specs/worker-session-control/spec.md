# worker-session-control Specification

## Purpose
Give authorized owners reliable control of every session hosted by one daemon, without mistaking membership removal, message delivery or task reports for stopped execution.

## Requirements

### Requirement: Daemon-scoped identity and preserved authorization
The daemon SHALL resolve exact session IDs independently of current swarm membership for control inspection and stop. It SHALL retain owner ancestry through membership removal, attachment changes and restart, and authorize each operation against that ancestry or the existing explicit operator override. It SHALL NOT grant control because a caller can discover or message a session. Friendly-name and prefix aliases SHALL remain scoped to the caller's swarm and reject ambiguity.

#### Scenario: Owned worker leaves original membership
- **WHEN** an owner requests status or stop using its worker's exact session ID after the worker leaves or changes swarm membership
- **THEN** the daemon SHALL resolve the same session, apply retained ownership authorization and return its control state rather than an unknown-swarm error

#### Scenario: Unrelated session is discoverable
- **WHEN** an unrelated caller discovers a session and requests control without existing override authority
- **THEN** the operation SHALL be denied without mutating the target or exposing its prompts or tool bodies

#### Scenario: Alias collision across swarms
- **WHEN** multiple sessions share an alias or an alias only exists outside the caller's swarm
- **THEN** alias resolution SHALL NOT select an out-of-swarm target and SHALL request an exact ID when appropriate

### Requirement: Durable stop admission barrier
An accepted stop SHALL durably revoke the active execution generation and every already-accepted pending generation before acknowledging acceptance. The daemon SHALL serialize stop and new-run admission for that session. A revoked generation SHALL NOT admit further turns, queued wakes, automatic lightweight registration, recovery continuation or native mutating tools, even after membership removal. A persistence failure SHALL return an error, retain a local admission barrier and SHALL NOT report durable stop acceptance.

#### Scenario: Stop races with wake
- **WHEN** stop and wake race for the same generation
- **THEN** their order SHALL be deterministic, stop SHALL revoke any admitted predecessor, and no pre-stop queued work SHALL run as a post-stop generation

#### Scenario: Late lightweight request
- **WHEN** a stopped generation makes a later swarm request using a valid persisted session stub
- **THEN** it SHALL NOT be recreated as a runnable ownerless member

#### Scenario: Stop persistence fails
- **WHEN** the daemon cannot persist stop intent
- **THEN** it SHALL report persistence failure, keep local admission blocked and SHALL NOT return terminal or durable-acceptance success

### Requirement: Observable execution settlement
Stop results and status SHALL distinguish `stop_requested`, `draining`, `stopped` and `unresolved`, separately from task completion. Terminal `stopped` SHALL require that all admitted turns and tracked owned tool work have settled. A deadline or unknown membership SHALL NOT imply settlement. Repeated stop requests SHALL return the same or a later control state without duplicating execution. Status SHALL expose the session ID, execution generation, stop request identity, outstanding work category and observation freshness without private content.

#### Scenario: Idle worker stop
- **WHEN** a worker has no admitted turn or outstanding owned tool work and its stop intent persists
- **THEN** stop SHALL return `stopped` with that generation and settlement evidence

#### Scenario: Model or native tool in flight
- **WHEN** a worker is executing a provider request or native mutation when stop is accepted
- **THEN** it SHALL enter `draining`, cancellation SHALL reach the admitted execution, and terminal success SHALL wait for tracked execution and mutations to settle

#### Scenario: Deadline expires
- **WHEN** settlement cannot be established within the stop deadline
- **THEN** the response SHALL be `unresolved` with outstanding work and a safe inspection path, while admission remains revoked and ownership transfer remains forbidden

#### Scenario: Repeated stop after settlement
- **WHEN** a client retries stop for the settled generation
- **THEN** it SHALL receive the settled result without creating new work or losing the durable barrier

### Requirement: Explicit safe generation transition
Each accepted new work run SHALL carry a monotonically increasing session-local execution generation. Authorized resume or reassignment after stop SHALL create a new generation only when the predecessor is settled. Stale commands, reports, notifications and tools SHALL NOT regain execution or write authority by observing the newer generation. Concurrent resume requests SHALL NOT create duplicate admitted runs.

#### Scenario: Resume after confirmed stop
- **WHEN** an authorized owner explicitly resumes a settled stopped session
- **THEN** exactly one new generation SHALL be admitted and old-generation work SHALL remain revoked

#### Scenario: Resume during unresolved work
- **WHEN** resume, replace or ownership transfer is requested while predecessor settlement is unresolved
- **THEN** it SHALL be rejected with the unresolved work reason, not silently release ownership or start a competitor

#### Scenario: Two clients resume concurrently
- **WHEN** two authorized clients issue conflicting new-run requests for one predecessor
- **THEN** at most one new generation SHALL be admitted and both clients SHALL receive the deterministic resulting generation or conflict

### Requirement: External effect boundaries remain explicit
Cancellation SHALL track tool ownership and known child/background work without claiming rollback of completed effects or universal control of arbitrary shell, MCP or remote work. A revoked generation SHALL be refused new native mutations independently of file-claim membership. Work that can still mutate but cannot be proved settled SHALL keep the generation unresolved and prevent automatic ownership transfer. The daemon SHALL NOT kill unrelated processes or broaden permissions to manufacture settlement.

#### Scenario: Revoked worker edits after membership removal
- **WHEN** the revoked execution invokes a native edit, write or patch after its member record is removed
- **THEN** the mutation SHALL be refused before changing file bytes rather than bypass protection as a non-worker

#### Scenario: Filesystem operation continues after caller cancellation
- **WHEN** an owned filesystem operation outlives its cancelled async caller
- **THEN** it SHALL remain accounted as outstanding until it settles, and `stopped` SHALL NOT be reported early

#### Scenario: Detached or remote work cannot be verified
- **WHEN** cancellation leaves a shell child, background operation or remote tool effect with uncertain liveness
- **THEN** the stop SHALL remain unresolved with the specific limitation and SHALL NOT claim the workspace is safe for reassignment

### Requirement: Protocol compatibility cannot synthesize stop success
Existing clients SHALL remain able to read legacy session/report data. Clients lacking structured stop-state support SHALL receive explicit pending/error diagnostics for unsettled work, not a legacy unconditional success. Legacy requests without a generation SHALL be bound to the current generation when accepted, while legacy report history SHALL NOT be silently assigned to future work.

#### Scenario: Older client stops a busy worker
- **WHEN** an older client cannot represent a draining stop
- **THEN** the server SHALL return an explicit nonterminal diagnostic and SHALL NOT return `Stopped agent` until settlement

#### Scenario: Stale legacy request after stop
- **WHEN** a legacy request arrives after stop without explicit new-work intent
- **THEN** it SHALL NOT create a fresh execution generation or clear the durable stop barrier

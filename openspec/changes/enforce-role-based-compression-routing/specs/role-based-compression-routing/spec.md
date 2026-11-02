## ADDED Requirements

### Requirement: Durable role selects an explicit input policy
The system SHALL select conservative input processing for root conversations, memory extraction, and unknown chat roles. It SHALL select optimized-trial processing only for identified bounded coding workers and exact processing for final independent review and Jev decisions. Model names and prompt text SHALL NOT grant optimized or review identity.

#### Scenario: Unknown worker role remains conservative
- **WHEN** a session has a parent but no recognized bounded coding role
- **THEN** its gateway input policy is conservative, not optimized

#### Scenario: Coding worker and final review differ
- **WHEN** an identified bounded coding worker and an independent final reviewer use the same model
- **THEN** the worker requests optimized-trial processing and the reviewer requests exact processing

#### Scenario: Memory extraction overrides its caller
- **WHEN** an optimized worker starts memory extraction
- **THEN** the memory request uses conservative processing without changing the worker's subsequent requests

### Requirement: Gateway request policy is isolated and scoped
The system SHALL request equivalent profiles on the explicitly configured Pi5 and local Omni gateways. Conservative SHALL mean session-dedup followed by lite. Optimized-trial SHALL mean RTK aggressive followed by Caveman ultra with protected-content rules. Exact SHALL disable input compression and final review SHALL retain full findings, risks, and evidence rather than force ultra-terse output styles. Request policy SHALL NOT leak between forks, concurrent tasks, or direct providers.

#### Scenario: Concurrent forks retain their own policy
- **WHEN** conservative, optimized, and exact forks issue requests concurrently
- **THEN** captured chat and Responses requests carry the correct per-fork policy without shared-state contamination

#### Scenario: Direct provider is an explicit exception
- **WHEN** an explicitly selected direct provider handles a request
- **THEN** gateway-specific headers are not sent and gateway compression is reported unavailable rather than applied

### Requirement: Gateway fallback is conservative and routing remains explicit
Both gateways SHALL use a conservative fallback for unbound chat. Pi5 SHALL remain primary, local Omni SHALL remain a manually selected alternate, and gateway or provider failures SHALL NOT silently substitute another route. Effective pipeline selection SHALL be verified independently of combo-card labels.

#### Scenario: Missing binding has a safe fallback
- **WHEN** a chat route lacks a named profile binding
- **THEN** both gateways apply the documented conservative fallback and expose the selected plan

#### Scenario: Primary route fails
- **WHEN** Pi5 cannot serve a request
- **THEN** Jcode reports that route's failure without automatically switching to local Omni or direct Azure

#### Scenario: Stored mode differs from resolver state
- **WHEN** a routing combo stores a compression mode that differs from effective profile settings
- **THEN** tests and receipts identify the actual selected plan and do not treat the stored label as enforcement evidence

### Requirement: Real TypeSafe Jev verification uses the production client
Acceptance SHALL include a bounded real TypeSafe request through Jcode's production Jev client using synthetic nonsecret input. It SHALL verify explicit provider selection, model and endpoint, schema parsing, and the expected decision without fallback. Credentials SHALL remain inside the existing resolver and SHALL NOT appear in logs, commands, reports, or commits.

#### Scenario: TypeSafe returns the expected decision
- **WHEN** the explicit TypeSafe route evaluates the synthetic verification question
- **THEN** the real response parses into the expected decision and the evidence records a passing live check

#### Scenario: Credentials or service access fail
- **WHEN** the explicit TypeSafe route has missing credentials or an authentication, quota, transport, or schema failure
- **THEN** acceptance remains blocked with a sanitized cause and no other provider substitutes for TypeSafe

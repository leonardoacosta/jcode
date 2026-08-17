## ADDED Requirements

### Requirement: Mac browser fleet discovery
The Mac broker SHALL continuously discover connected supported Chrome and Edge browsers and SHALL expose stable, generation-tagged browser, window, and tab references with policy-filtered metadata and explicit capabilities.

#### Scenario: Discover ordinary browser tabs
- **WHEN** the approved extension and native host connect from an ordinary Chrome or Edge session
- **THEN** the broker SHALL list its supported windows and tabs with stable opaque references
- **AND** it SHALL report browser kind, active state, connection health, target generation, and supported action capabilities.

#### Scenario: A supported browser is absent
- **WHEN** Chrome or Edge is not installed, not running, or lacks a connected extension
- **THEN** fleet status SHALL report the corresponding truthful absent or disconnected state
- **AND** the other browser SHALL remain usable.

#### Scenario: Redact inventory metadata
- **WHEN** a tab URL contains userinfo, query parameters, fragments, or another configured sensitive component
- **THEN** inventory results SHALL omit those components
- **AND** policy MAY hide the title, path, or full origin without making the target appear controllable beyond its reported capabilities.

### Requirement: Explicit remote fleet routing
Jcode SHALL expose an explicit Mac fleet browser route and SHALL NOT silently migrate local browser workflows to or from remote Mac targets.

#### Scenario: List remote Mac targets
- **WHEN** a browser action explicitly selects the Mac fleet route and requests status or target listing
- **THEN** Jcode SHALL query the forwarded broker socket
- **AND** result metadata SHALL identify the Mac fleet backend and the broker protocol version.

#### Scenario: Preserve local Chrome profile semantics
- **WHEN** a request uses `browser: "chrome"` with or without a profile
- **THEN** Jcode SHALL continue using homelab-local agent-browser behavior
- **AND** it SHALL NOT route the action to a Mac browser.

#### Scenario: Reject a stale target reference
- **WHEN** an action supplies a browser, window, or tab reference from an earlier target generation
- **THEN** the broker SHALL reject the action as stale
- **AND** Jcode SHALL require fresh fleet discovery before retrying.

### Requirement: Private authenticated SSH transport
The fleet protocol SHALL operate over a mode-restricted Unix socket forwarded through the existing persistent SSH connection and SHALL NOT require a public TCP listener.

#### Scenario: Establish a fleet connection
- **WHEN** the Mac broker is running and the reverse stream-local SSH forward is active
- **THEN** homelab Jcode SHALL authenticate to the broker, negotiate a supported protocol version, and obtain fleet status through the forwarded Unix socket.

#### Scenario: Authentication or protocol negotiation fails
- **WHEN** the peer secret is missing or invalid, the socket permissions are unsafe, or no protocol version overlaps
- **THEN** both sides SHALL fail closed with bounded actionable diagnostics
- **AND** no browser inventory or steering action SHALL be accepted.

#### Scenario: Transport reconnects
- **WHEN** SSH or the broker socket disconnects and later recovers
- **THEN** read-only inventory MAY retry after bounded exponential backoff
- **AND** mutations SHALL NOT be automatically replayed.

### Requirement: Mac-owned confirmation policy
The Mac broker SHALL be the final authority for every browser operation and SHALL allow topology inventory by default while requiring Mac-side confirmation for state-changing actions not covered by a valid lease.

#### Scenario: Perform read-only inventory
- **WHEN** Jcode requests fleet health, browser listing, window listing, or policy-filtered tab listing
- **THEN** the broker SHALL answer without mutation approval
- **AND** it SHALL not grant permission to inspect page content or perform later mutations.

#### Scenario: Request a mutation without a lease
- **WHEN** Jcode requests navigation, click, typing, form fill, upload, download, tab creation, tab closure, or another state-changing action without a matching lease
- **THEN** the broker SHALL present a Mac-local approval request
- **AND** it SHALL execute only after explicit approval before the request deadline.

#### Scenario: Agent attempts to change policy
- **WHEN** the homelab requests policy edits, self-approval, lease issuance, emergency-stop release, or another Mac-authority operation
- **THEN** the broker SHALL reject the request
- **AND** the denial SHALL not be overridable through the fleet protocol.

### Requirement: Scoped expiring autonomy leases
The Mac broker SHALL support locally issued capability leases scoped by target, origin, action set, and expiration, with a default maximum shortcut duration of 15 minutes.

#### Scenario: Use an approved lease
- **WHEN** a mutation matches a live Mac-issued lease's browser, profile, tab, origin, action set, target generation, and expiration
- **THEN** the broker SHALL execute without another prompt
- **AND** it SHALL record a secret-safe local audit event.

#### Scenario: Revoke a lease
- **WHEN** the lease expires, the broker restarts, policy reloads, target generation changes, or the emergency stop is activated
- **THEN** the broker SHALL revoke the lease immediately
- **AND** later matching mutations SHALL require fresh approval.

### Requirement: Immutable hard-deny boundaries
The broker SHALL refuse steering in hard-denied contexts regardless of approval prompts or autonomy leases.

#### Scenario: Target a hard-denied context
- **WHEN** an action targets incognito, password-manager surfaces, browser settings, extension management, privileged browser URLs, payment or banking confirmation, account-security changes, or authentication and recovery settings
- **THEN** the broker SHALL deny the action without offering remote override
- **AND** it SHALL return only bounded category-level diagnostics.

#### Scenario: Emergency stop is active
- **WHEN** the Mac user activates the fleet emergency stop
- **THEN** the broker SHALL revoke every lease, reject all steering mutations, and preserve only configured health/status inspection
- **AND** the homelab SHALL NOT be able to release the stop.

### Requirement: Capability-faithful hybrid control
The broker SHALL use extension/native-host control for ordinary tabs and CDP only for explicitly managed instances, and SHALL advertise and enforce the capability set of each target.

#### Scenario: Ordinary tab lacks a requested capability
- **WHEN** Jcode requests an operation that the extension-backed target cannot faithfully execute
- **THEN** the broker SHALL return an actionable unsupported-capability error
- **AND** it SHALL not silently approximate the operation or relaunch the daily profile under CDP.

#### Scenario: Managed CDP target supports richer inspection
- **WHEN** an explicitly managed browser endpoint reports a supported CDP capability
- **THEN** the broker MAY expose that capability for the target
- **AND** Mac approval and hard-deny policy SHALL still apply.

### Requirement: Safe Mac lifecycle and setup
Jcode SHALL provide idempotent setup, status, and removal flows for the Mac broker, launch agent, native hosts, extension installation state, policy defaults, and SSH forwarding guidance.

#### Scenario: Install or refresh Mac fleet support
- **WHEN** the user invokes explicit Mac fleet setup
- **THEN** Jcode SHALL install or refresh only Jcode-owned broker, launch-agent, native-host, and policy artifacts
- **AND** it SHALL report browser extension and SSH-forwarding steps that still require human approval.

#### Scenario: Remove Mac fleet support
- **WHEN** the user invokes explicit removal
- **THEN** Jcode SHALL unload the launch agent and remove Jcode-owned artifacts
- **AND** it SHALL not delete or modify browser profiles, browser credentials, or unrelated SSH configuration.

### Requirement: Browser fleet verification
The repository SHALL include deterministic and real-environment verification for discovery, routing, policy, transport, lifecycle, Chrome, and Edge behavior.

#### Scenario: Run deterministic fleet tests
- **WHEN** fleet-focused tests run with fake extension, CDP, approval, and transport peers
- **THEN** they SHALL cover protocol authentication, versioning, payload limits, metadata redaction, capability routing, stale generations, approvals, leases, hard denies, emergency stop, reconnect, non-replayed mutations, setup, and removal.

#### Scenario: Run Mac acceptance workflow
- **WHEN** opt-in fleet acceptance runs on a Mac with the broker, SSH forward, and at least one supported browser installed
- **THEN** Jcode's public browser interface SHALL discover real targets, inspect allowed content, require approval for a mutation, execute an approved mutation, exercise a temporary lease, reject a hard-denied target, survive a broker or SSH reconnect, and leave no test lease active
- **AND** the workflow SHALL exercise Chrome and Edge independently when each is installed.

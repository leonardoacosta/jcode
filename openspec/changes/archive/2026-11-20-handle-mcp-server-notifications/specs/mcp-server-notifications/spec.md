## ADDED Requirements

### Requirement: Negotiate notification support by MCP protocol version

The client MUST negotiate only notification methods supported by the selected MCP protocol version and server capabilities. For protocol version 2025-11-25, it MUST use `resources/subscribe` only when the server advertises `resources.subscribe`, and MUST honor the applicable `tools.listChanged`, `prompts.listChanged`, and `resources.listChanged` server capabilities. For protocol version 2026-07-28, it MUST use `subscriptions/listen` with explicit filters and MUST NOT send the removed `resources/subscribe` method. It MUST correlate current-version acknowledgments and notifications by `io.modelcontextprotocol/subscriptionId` and honor the acknowledged filter subset.

#### Scenario: Legacy resource subscription is supported

- **WHEN** the selected protocol version is 2025-11-25 and the server advertises `resources.subscribe`
- **THEN** the client MAY subscribe to configured resource URIs with `resources/subscribe`
- **AND** it accepts matching `notifications/resources/updated` notifications
- **AND** it does not assume that a server without the capability supports subscriptions

#### Scenario: Current subscription stream is negotiated

- **WHEN** the selected protocol version is 2026-07-28
- **THEN** the client opens `subscriptions/listen` with only configured `toolsListChanged`, `promptsListChanged`, `resourcesListChanged`, and resource URI filters
- **AND** it waits for `notifications/subscriptions/acknowledged` before accepting events for that subscription
- **AND** it ignores or reports notification types omitted from the acknowledged filter

#### Scenario: Unsupported protocol feature

- **WHEN** the server does not support a requested filter or selected protocol version
- **THEN** the client leaves that filter inactive without falling back to an unnegotiated notification method
- **AND** normal MCP tool, prompt, and resource operations remain available

### Requirement: Dispatch supported server notifications

The MCP client MUST dispatch recognized notifications to the owning server manager and MUST NOT discard them as non-response JSON-RPC messages. A resource update MUST invalidate only the matching subscribed URI. A tools, prompts, or resources list-changed notification MUST refresh the corresponding catalog from the server. Each notification is a hint to re-read authoritative server state; its payload MUST NOT be treated as agent instructions or authorization.

#### Scenario: Resource update invalidates content

- **WHEN** a subscribed resource notification names a configured URI
- **THEN** the manager re-reads that URI from the same MCP server and auth context
- **AND** it exposes the newly read content only through the existing resource policy
- **AND** malformed or failed reads leave the prior known state intact and report the refresh failure

#### Scenario: Catalog changed

- **WHEN** `notifications/tools/list_changed`, `notifications/prompts/list_changed`, or `notifications/resources/list_changed` is received and allowed for that server
- **THEN** the corresponding catalog is refreshed from that server
- **AND** a failed refresh preserves the last known catalog and surfaces a per-server error

#### Scenario: Unknown, malformed, or stale notification

- **WHEN** a notification has an unknown method, invalid parameters, a mismatched subscription ID, or an unconfigured resource URI
- **THEN** the client does not wake an agent or alter catalog/resource state
- **AND** it records a bounded diagnostic without logging resource contents

### Requirement: Reconcile after disconnect

The client MUST re-establish negotiated subscriptions after a connection reconnects and then re-read relevant authoritative resource and catalog state. It MUST tolerate missing, repeated, and reordered notifications without assuming event replay or exactly-once delivery.

#### Scenario: Reconnect restores subscriptions and state

- **WHEN** a subscribed server disconnects and later reconnects
- **THEN** the client negotiates and opens the subscriptions supported by that connection again
- **AND** it re-reads the subscribed resources and opted-in catalogs
- **AND** missed or duplicate notifications do not produce duplicate agent work

#### Scenario: Server does not support subscriptions after reconnect

- **WHEN** a reconnect returns capabilities that omit a previously supported feature
- **THEN** the client does not recreate that subscription
- **AND** it refreshes available catalogs through ordinary list requests
- **AND** it reports that resource update listening is unavailable

### Requirement: Bound notification processing

The client MUST deduplicate equivalent pending invalidations, coalesce repeated notifications for the same server and target, and enforce configurable per-server rate and queue limits. Overload MUST NOT grow an unbounded queue or starve interactive requests. A per-server allowlist MUST control which notification classes and resource URIs may refresh catalogs/resources and separately which may request an agent wake.

#### Scenario: Notification burst is coalesced

- **WHEN** a server sends repeated notifications for one resource URI or catalog before refresh completes
- **THEN** the manager performs at most one pending refresh for each affected target
- **AND** it records a bounded coalescing or rate-limit outcome

#### Scenario: Server exceeds its event budget

- **WHEN** notification input exceeds the configured per-server queue or rate limit
- **THEN** the client drops or coalesces excess invalidation hints according to a documented deterministic policy
- **AND** it keeps memory and refresh work bounded without delaying interactive requests

#### Scenario: Notification is not allowlisted for wake

- **WHEN** a valid notification is not on that server's agent-wake allowlist
- **THEN** the client may perform an independently allowed catalog/resource refresh
- **AND** it MUST NOT route an agent wake for the event

### Requirement: Route explicitly allowed agent wakes

The client MUST route an allowlisted wake only to its configured session target or through the existing ambient schedule/queue mechanism. The wake MUST identify the configured MCP server and notification class, and MUST NOT forward notification payload or resource content as trusted instructions. If the target is unavailable or ambient execution is disabled, the client MUST report or queue the permitted wake according to existing queue policy without bypassing authorization or session admission.

#### Scenario: Session-targeted wake

- **WHEN** an allowlisted event maps to a configured session
- **THEN** the event is routed through the existing session wake/admission path
- **AND** an absent, busy, or revoked target is handled by that path without creating a parallel agent turn

#### Scenario: Ambient-targeted wake

- **WHEN** an allowlisted event maps to ambient work
- **THEN** the client uses the existing schedule/ambient queue and trigger path
- **AND** repeated events for the same pending server target are coalesced
- **AND** ambient policy remains authoritative over whether work runs

## ADDED Requirements

### Requirement: Gateway endpoints own compression selection
Jcode SHALL use the upstream-compatible HTTP compression transport and send the selected combo/model while allowing the gateway's predefined compression policy to apply. Jcode SHALL NOT select OmniRoute compression from session roles, retain superseded Jev-specific compression/cache override controls, or automatically cross provider/gateway routes. Final review and swarm outputs SHALL retain complete findings, risks, and evidence. Local transport success SHALL NOT count as external gateway applied-policy acceptance.

#### Scenario: Root and worker use endpoint defaults
- **WHEN** root, coding-worker, or reviewer sessions issue requests
- **THEN** Jcode leaves compression selection to the selected endpoint and preserves its combo/model ID

#### Scenario: Legacy client selector is present
- **WHEN** named-provider configuration still contains an OmniRoute compression selector
- **THEN** it does not override gateway defaults or produce a requested-policy notice

### Requirement: Upstream compression transport preserves request identity and report semantics
The upstream compression transport SHALL preserve selected route/model identity across supported HTTP request paths. Jcode SHALL NOT inject role-selected compression or forced cache-bypass controls into gateway or direct-provider requests. It SHALL preserve complete swarm reports and report-completion semantics. Gateway-specific pipeline configuration and applied-policy evidence remain owned by the external gateway and require separate acceptance.

#### Scenario: Concurrent requests retain selection and report completeness
- **WHEN** concurrent sessions issue requests through the upstream compression transport
- **THEN** each request retains its selected route/model and report output is not truncated or treated as complete before its completion event

#### Scenario: Direct provider remains unaffected
- **WHEN** an explicitly selected direct provider handles a request
- **THEN** gateway-specific compression headers and requested-policy notices are absent

### Requirement: Gateway routing remains explicit
Pi5 SHALL remain primary and local Omni SHALL remain a manually selected alternate unless separately authorized. Gateway or provider failures SHALL NOT silently substitute another route. External gateway effective pipeline selection SHALL be verified independently of combo-card labels and is not satisfied by this change's local transport tests.

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

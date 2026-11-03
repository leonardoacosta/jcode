## ADDED Requirements

### Requirement: Gateway endpoints own compression selection
Jcode SHALL send the selected combo/model and allow the gateway's predefined compression policy to apply. Jcode SHALL NOT select OmniRoute compression from session roles, send compression override or forced cache-bypass headers, or emit requested-policy notices. Legacy named-provider compression header configuration SHALL NOT enable client selection. Final review SHALL retain full findings, risks, and evidence rather than force ultra-terse output styles.

#### Scenario: Root and worker use endpoint defaults
- **WHEN** root, coding-worker, or reviewer sessions issue requests
- **THEN** Jcode leaves compression selection to the selected endpoint and preserves its combo/model ID

#### Scenario: Legacy client selector is present
- **WHEN** named-provider configuration still contains an OmniRoute compression selector
- **THEN** it does not override gateway defaults or produce a requested-policy notice

### Requirement: Gateway defaults apply across compatible transports
The same endpoint-owned policy SHALL apply to Chat Completions and Responses requests, including concurrent forks and model changes. Jcode SHALL NOT inject gateway-specific compression controls into direct-provider requests. Conservative SHALL mean session-dedup followed by lite. Optimized-trial SHALL mean RTK aggressive followed by Caveman ultra with protected-content rules. Exact SHALL disable input compression. These pipeline choices belong in gateway configuration, not Jcode role headers.

#### Scenario: Concurrent forks select different endpoints
- **WHEN** root, worker, and reviewer forks issue concurrent Chat or Responses requests with different selected models
- **THEN** each request retains its model ID without compression or forced cache-bypass headers

#### Scenario: Direct provider remains unaffected
- **WHEN** an explicitly selected direct provider handles a request
- **THEN** gateway-specific compression headers and requested-policy notices are absent

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

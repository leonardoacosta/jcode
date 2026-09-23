## ADDED Requirements

### Requirement: F1 Explicit backend compatibility
webfetch SHALL default to direct and use hosted Firecrawl only when explicitly selected and configured.

#### Scenario: Legacy call
- **WHEN** backend is omitted
- **THEN** current direct formats work without Firecrawl, Node or credentials

#### Scenario: Unsupported format or missing setup
- **WHEN** Firecrawl html/text is requested or CLI/auth/config is absent
- **THEN** a clear bounded error occurs without auto-install or hidden fallback

### Requirement: F2 Hosted egress boundary
Firecrawl requests SHALL pass public HTTPS, exact-host allowlist and DNS validation and SHALL exclude credentials, profiles, actions and private content.

#### Scenario: Public allowlisted URL
- **WHEN** all resolved addresses are public and destination is allowed
- **THEN** the literal-argv CLI request proceeds and exposed final URL is validated

#### Scenario: Unsafe URL
- **WHEN** URL credentials, local/IP destination, disallowed host, mixed DNS or disallowed final URL is encountered
- **THEN** the request is refused or returned content is withheld without leaking secrets

### Requirement: F3 Result semantics
The fetch result SHALL expose backend, target status, final URL, truncation, requested freshness and reported cache/credit information without inventing metadata.

#### Scenario: Rendered page
- **WHEN** Firecrawl supplies JS-populated content
- **THEN** returned content includes that text and remains untrusted page data

#### Scenario: Target failure or missing metadata
- **WHEN** API/CLI succeeds but target status is 404, or cache status is omitted
- **THEN** 404 is a tool failure and unknown cache state is not described as fresh or a hit

### Requirement: F4 Bounded lifecycle
Hosted execution SHALL cap downloaded output at 5 MiB, apply the existing display-output limit and timeout ceiling, and terminate its child on cancellation.

#### Scenario: Large result
- **WHEN** CLI emits an oversized payload or long extracted text
- **THEN** bounded resource use and explicit error or text truncation result

#### Scenario: Failure or cancellation
- **WHEN** CLI hangs, returns malformed JSON, rate limits, fails authentication or is cancelled
- **THEN** sanitized actionable errors return, no child remains, and no automatic paid retry occurs

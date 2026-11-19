## ADDED Requirements

### Requirement: Response-observed gateway route
Compatible-provider requests SHALL record the serving route reported by the response that served them, and cache-miss classification SHALL compare the baseline request's serving route with the current request's serving route.

#### Scenario: Connection switch is attributed
- **WHEN** two consecutive append-only requests return different `X-OmniRoute-Selected-Connection-Id` values and the second request reads below the miss threshold
- **THEN** the miss reason is `account_switch`, or `upstream_switch` when `X-OmniRoute-Provider` differs, not `low_read`, and the detail shows only short opaque fingerprints

#### Scenario: Same route keeps low_read
- **WHEN** both requests report the same provider and connection and the read is low
- **THEN** the miss stays `low_read` with prefix-unchanged evidence

#### Scenario: Streaming responses on OmniRoute 3.8.54
- **WHEN** the gateway is OmniRoute 3.8.54 and the request streams
- **THEN** neither header is present, so classification follows the headers-absent behavior; attribution applies to non-streaming responses until the gateway emits them on streams

#### Scenario: Headers absent
- **WHEN** a provider response carries none of these headers
- **THEN** classification matches current behavior and no route switch is inferred from missing values

### Requirement: Truthful TTL timing
Miss details SHALL describe idle time relative to the TTL accurately.

#### Scenario: Idle beyond estimated TTL
- **WHEN** idle time is at least an estimated TTL
- **THEN** the detail says idle exceeded the estimated TTL, the reason is not `expired`, and the text does not say "within"

#### Scenario: Idle under TTL
- **WHEN** idle time is below the TTL
- **THEN** the detail says the request was within the TTL

## ADDED Requirements

### Requirement: Probability entries must be numeric
The poke observer response parser SHALL reject a probability distribution containing any value that cannot be decoded as a JSON number, returning the existing malformed-response abstention. It SHALL NOT coerce strings, nulls, booleans, arrays, or objects to zero.

#### Scenario: Malformed zero-weight entry
- **WHEN** a response has numeric entries totaling one and an additional nonnumeric entry
- **THEN** the parser returns `Some(Err(AbstainReason::MalformedResponse))`

#### Scenario: Valid numeric zero
- **WHEN** an otherwise valid response contains a numeric zero probability
- **THEN** the parser accepts it subject to existing distribution and threshold checks

### Requirement: Existing observer validation remains intact
The parser SHALL preserve existing finite and range validation, sum tolerance, selected-choice consistency, and configured probability and margin thresholds.

#### Scenario: Invalid numeric distribution
- **WHEN** a numeric probability is outside the permitted zero-to-one range or the total exceeds the existing sum tolerance
- **THEN** the parser returns the existing malformed-response abstention

#### Scenario: Uncertain valid distribution
- **WHEN** a numeric distribution passes structural validation but fails a configured confidence threshold
- **THEN** the parser returns the existing uncertainty or insufficient-margin abstention

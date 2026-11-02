## ADDED Requirements

### Requirement: Report detected terminal capabilities and active fallbacks
The system SHALL report detected terminal capabilities and active fallbacks.

#### Scenario: Success path
- **WHEN** the user runs `jcode debug terminal` **THEN** output SHALL list each capability with its detected value and active fallback

#### Scenario: Failure path
- **WHEN** a capability is forced off via environment **THEN** the report SHALL show the fallback and rendering tests SHALL pass without that capability

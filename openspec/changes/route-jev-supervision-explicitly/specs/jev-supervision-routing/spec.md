## ADDED Requirements

### Requirement: Explicit supervision route
The system SHALL construct a Jev supervision client only from an explicitly configured provider and model, and SHALL NOT fall back to Memory, Browser or Voice routes or credentials.

#### Scenario: Configured route resolves
- **WHEN** `[jev.supervision]` sets a valid provider and model
- **THEN** a supervision client SHALL be created with purpose `supervision` and capability `supervision_jev`

#### Scenario: Missing or auto provider refused
- **WHEN** supervision config is absent, empty, or uses provider `auto`
- **THEN** resolution SHALL return `NotConfigured` with a reason, SHALL NOT panic, and SHALL NOT use another purpose's route

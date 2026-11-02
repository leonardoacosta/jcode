## ADDED Requirements

### Requirement: Honour NO_COLOR in the TUI and provide a keyboard path for every mouse-only action
The system SHALL honour NO_COLOR in the TUI and provide a keyboard path for every mouse-only action.

#### Scenario: Success path
- **WHEN** `NO_COLOR` is set **THEN** TUI frames SHALL contain no color attributes and SHALL distinguish selected, error and status rows by text or glyph

#### Scenario: Failure path
- **WHEN** a mouse-only action has no keyboard equivalent **THEN** the parity test SHALL fail naming the action

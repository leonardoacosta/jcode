## ADDED Requirements

### Requirement: Plan turns deny mutating tools
The system SHALL reject mutating native tool calls during an active `/plan` turn with an error naming plan mode, and SHALL allow read-only tools.

#### Scenario: Read during plan succeeds
- **WHEN** a `/plan` turn calls `read` or `agentgrep`
- **THEN** the call SHALL execute normally

#### Scenario: Edit during plan is denied
- **WHEN** a `/plan` turn calls `edit`, `write` or `apply_patch`
- **THEN** the call SHALL fail with a plan-mode denial and no file SHALL change

#### Scenario: Restriction ends after the plan
- **WHEN** the plan turn completes or is cancelled and the user sends an ordinary turn
- **THEN** mutating tools SHALL be available again

#### Scenario: Server cannot enforce the guard
- **WHEN** the connected server does not acknowledge the plan-only flag
- **THEN** the client SHALL show that plan restrictions are unenforced

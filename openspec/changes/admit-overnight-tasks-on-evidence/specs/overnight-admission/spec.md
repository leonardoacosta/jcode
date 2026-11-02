## ADDED Requirements

### Requirement: Evidence-gated coordinator turns

The overnight supervisor SHALL start a coordinator turn only after recording an `admitted` decision citing an eligible task or new signal.

#### Scenario: Eligible work admitted

- **WHEN** a background task completes during a run
- **THEN** the supervisor SHALL record `admitted` citing that completion and start one turn

#### Scenario: No eligible work

- **WHEN** the previous turn reported no remaining eligible work and no new signal arrived
- **THEN** the supervisor SHALL record `waiting` and SHALL NOT start another turn before a new signal or the wake boundary

### Requirement: Auth-blocked preflight

The run SHALL NOT start coordinator turns when preflight finds no usable provider auth.

#### Scenario: All auth expired

- **WHEN** every configured provider reports missing or expired auth at preflight
- **THEN** the run SHALL record `blocked` with the provider reasons and end without a coordinator turn

#### Scenario: One provider usable

- **WHEN** at least one provider authenticates
- **THEN** the run SHALL proceed to admission normally

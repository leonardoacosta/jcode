## ADDED Requirements

### Requirement: Resolved worker model shown
The system SHALL show each worker's resolved model and its source (`explicit`, `config`, `env`, `inherit`, `slot`) in spawn output and `swarm list`.

#### Scenario: Config default applied
- **WHEN** a worker is spawned without `model` and `agents.swarm_model` is set
- **THEN** spawn output SHALL show that model with source `config`

#### Scenario: Explicit model
- **WHEN** a worker is spawned with `model`
- **THEN** `swarm list` SHALL show that model with source `explicit`

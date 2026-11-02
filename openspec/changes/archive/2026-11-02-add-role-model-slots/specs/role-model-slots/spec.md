## ADDED Requirements

### Requirement: Role slot resolution
When an agent is spawned with a role and no explicit model, the system SHALL use the configured model for that role, falling back to `swarm_model` and then coordinator inheritance.

#### Scenario: Slot used
- **WHEN** `agents.roles.research` is set and a worker is spawned with role `research` and no model
- **THEN** the worker uses the research slot model

#### Scenario: Explicit model wins
- **WHEN** a worker is spawned with role `edit` and an explicit `model`
- **THEN** the explicit model is used

### Requirement: Slot source reported
The system SHALL report the resolved model source as `slot:<role>` when a role slot determined the model.

#### Scenario: Source shown
- **WHEN** a worker's model came from the `edit` slot
- **THEN** its resolved-model report shows source `slot:edit`

### Requirement: Research slot is read-only
Workers spawned with role `research` SHALL NOT have access to file-editing tools.

#### Scenario: Edit denied
- **WHEN** a research worker attempts to call an edit tool
- **THEN** the tool is unavailable

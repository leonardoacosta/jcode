## ADDED Requirements

### Requirement: Chain models served by the route

At start, the run SHALL drop chain models that the selected route does not serve and record each dropped model in preflight. If the coordinator chain or the worker chain is left empty, the run SHALL NOT start and SHALL report `blocked` with the unserved models. A route with no model list SHALL keep every chain model.

#### Scenario: Unserved model skipped
- **WHEN** the route does not serve `claude-opus-5.5`
- **THEN** preflight records it as dropped and the coordinator starts on `gpt-6-sol`

#### Scenario: Empty chain blocks
- **WHEN** the route serves no worker chain model
- **THEN** the run does not start and the reason lists the unserved worker models

#### Scenario: Custom profile route auth
- **WHEN** the session uses a custom `[providers.<name>]` profile whose key is present
- **THEN** preflight treats its auth as usable without consulting the runtime provider name

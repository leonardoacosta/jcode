# Command Center Web Delta

## ADDED Requirements

### Requirement: The web client's absence of local persistence is enforced

The Command Center web application SHALL hold no durable client-side state, and the repository SHALL carry a check that fails when a browser storage API is introduced.

#### Scenario: The check runs on a clean tree

- **WHEN** the no-frontend-persistence check runs against `apps/command-center/src`
- **THEN** it SHALL find no reference to `localStorage`, `sessionStorage`, or `indexedDB`
- **AND** it SHALL exit with status `0`

#### Scenario: The check catches a regression

- **WHEN** a file under `apps/command-center/src` introduces a browser storage call
- **THEN** the check SHALL exit non-zero
- **AND** its output SHALL name the file and line

#### Scenario: Client-owned state remains reversible interface state

- **WHEN** documentation enumerates what the web client owns
- **THEN** it SHALL list only layout, focus, filters, selections, scroll position, and transient drafts
- **AND** it SHALL state that all durable state is read from and written through the daemon

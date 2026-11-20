# World issue board

## ADDED Requirements

### Requirement: Resolve tracker board bindings by verified world

Jcode MUST resolve an issue board only from the active repository's verified canonical path and verified world identity. A binding MUST identify its backend and stable team identifier, and MAY narrow queries to a stable project identifier. Display names MUST NOT select a team or project. Personal and Priceless worlds MAY use Linear. Brown worlds MUST use Azure DevOps (ADO) and MUST NOT query Linear, including the BrownandBrown team visible through a shared Linear connection. Unknown, ambiguous, or unverified world identity MUST produce no board and MUST NOT fall back to user-global, last-used, or another world's binding.

#### Scenario: Personal or Priceless world uses its explicit Linear binding
- **WHEN** the active repository resolves to a verified Personal or Priceless world with an explicit Linear team binding
- **THEN** Jcode queries only that configured team and optional configured project
- **AND** it does not infer another scope from teams or projects visible to the authenticated connection

#### Scenario: Brown world refuses a Linear binding
- **WHEN** the active repository resolves to Brown and its configured board backend is Linear
- **THEN** Jcode refuses the board before issuing a Linear request
- **AND** it reports that Brown boards require ADO

#### Scenario: Unknown world has no board
- **WHEN** the active repository has no verified world identity, an ambiguous identity, or no matching explicit binding
- **THEN** Jcode shows no board with a clear unavailable explanation
- **AND** it does not reuse a previous world's binding or query any tracker

#### Scenario: Path aliases resolve to the verified repository
- **WHEN** the active path uses a symlink or worktree path
- **THEN** Jcode resolves it using the verified canonical repository/world policy before selecting a binding
- **AND** a path-prefix resemblance alone does not grant world membership

### Requirement: Query a scoped issue board through an allowlisted adapter

Jcode MUST expose a read surface limited to the configured backend's workflow-state and issue query operations, with explicit team and optional project identifiers. It MUST NOT pass through an MCP server's full tool inventory. Cached results MUST be isolated by verified world, backend/auth identity, team and project identifiers. A provider failure MUST NOT trigger a different backend or binding as fallback.

#### Scenario: Linear connection can see more than its binding
- **WHEN** the authenticated Linear connection can see multiple teams, including BrownandBrown
- **THEN** a Personal or Priceless board query returns only the explicitly bound team and project scope
- **AND** no Brown issue is included because the connection can see it

#### Scenario: ADO adapter is missing or unauthorized
- **WHEN** an authorized Brown binding exists but ADO is unavailable, unsupported, or unauthorized
- **THEN** the board reports the relevant failure state
- **AND** Jcode does not query Linear or show cached data from another binding

#### Scenario: Cache is reused only for the same binding
- **WHEN** a query cache entry exists for another world, team, project or backend/auth identity
- **THEN** Jcode does not return that entry for the current binding

### Requirement: Present a usable Kanban view

The TUI MUST display workflow states as columns using the selected team's provider-defined workflow state order, and place issues according to their current state. It MUST support keyboard navigation between columns and issues and Escape to return to the prior view. Compact UI chrome MUST fit terminal display-cell width or truncate with an ellipsis without wrapping. The view MUST distinguish loading, empty, provider error and unauthorized states.

#### Scenario: Board has issues in ordered workflow states
- **WHEN** a scoped query returns workflow states and issues
- **THEN** the TUI renders one column per returned state in provider order and places each issue under its current state
- **AND** keyboard navigation can move among columns and issues

#### Scenario: Board has no issues
- **WHEN** a successful scoped query returns no issues
- **THEN** the TUI shows an empty state distinct from loading or failure

#### Scenario: Query is loading or fails
- **WHEN** the board query is pending, fails, or returns an authorization failure
- **THEN** the TUI shows loading, provider error, or unauthorized state respectively
- **AND** it does not present stale data as current success

#### Scenario: Narrow terminal or long text
- **WHEN** the terminal is too narrow for board chrome or an issue title exceeds its available cells
- **THEN** each compact chrome row remains a single visual row and overlong text is truncated by display-cell width

#### Scenario: Escape leaves the board
- **WHEN** the user presses Escape from the board
- **THEN** Jcode returns to the prior view without changing tracker data

### Requirement: Constrain issue status changes

Jcode MUST allowlist only the Linear backend's issue-state update operation for board writes; ADO bindings have no write operation in v1. Every proposed state change MUST require explicit user confirmation for that individual change. The write MUST include the expected prior state and MUST be refused if the live state no longer matches. A retry MUST use a stable idempotency identity for the same confirmed operation and MUST recheck the expected prior state before retrying. Before each write, Jcode MUST revalidate the active repository's canonical path and verified world against the binding. V1 MUST NOT write labels, assignees or comments.

#### Scenario: User confirms one status change
- **WHEN** the user selects a new workflow state for an issue and confirms that change
- **THEN** Jcode calls only the configured backend's issue-state update operation with issue ID, requested state, expected prior state and stable operation identity
- **AND** no other issue fields are changed

#### Scenario: User declines or has not confirmed
- **WHEN** the user declines a proposed state change or dismisses its confirmation
- **THEN** Jcode performs no tracker write

#### Scenario: Prior state changed remotely
- **WHEN** the issue's live state differs from the expected prior state before an update or retry
- **THEN** Jcode refuses the write and refreshes or reports the conflict
- **AND** it does not silently overwrite the newer state

#### Scenario: Retry after uncertain response
- **WHEN** a confirmed update receives an uncertain transport result and is retried
- **THEN** Jcode uses the same operation identity and expected prior state
- **AND** the backend's idempotency contract prevents duplicate application, or Jcode stops and reports that safe retry cannot be guaranteed

#### Scenario: Repository world changes before write
- **WHEN** the active repository path or verified world no longer matches the confirmed binding before dispatch
- **THEN** Jcode refuses the write without contacting the tracker

#### Scenario: Non-state mutation is requested
- **WHEN** a board action would change a label, assignee or comment
- **THEN** Jcode refuses the operation because it is outside the v1 write allowlist

### Requirement: Keep an append-only local status-change audit

For every confirmed status-change attempt, Jcode MUST append a local audit record before dispatch to a per-user append-only file under the Jcode data directory created with owner-only permissions. The board MUST NOT rotate, truncate or delete audit records. The record MUST identify the verified world, backend, team, optional project, issue, expected prior state, requested state, confirmation, operation identity, timestamp and eventual outcome when known. Existing records MUST NOT be edited or removed by the board. Audit failures MUST prevent dispatch. Audit data MUST NOT include credentials or issue content beyond identifiers and states needed to explain the change.

#### Scenario: Audit record is written before dispatch
- **WHEN** a user confirms a status change
- **THEN** Jcode appends its intent record before sending the update
- **AND** if appending fails, Jcode makes no tracker request

#### Scenario: Outcome becomes known
- **WHEN** the tracker response or a later safe reconciliation establishes the operation outcome
- **THEN** Jcode appends an outcome record linked to the original operation without rewriting the intent record

#### Scenario: Audit history is append-only
- **WHEN** a later status change or retry is recorded
- **THEN** Jcode appends a new event and preserves all prior audit entries

#### Scenario: World changes during a pending operation
- **WHEN** an operation is pending and the user switches to a different or unknown world
- **THEN** the pending result remains attributed to its original world and operation identity
- **AND** it cannot initiate or authorize a write in the new world

### Requirement: Label board scope and keep ADO read-only in v1

A board binding MUST include a stable team identifier and MAY include a project identifier. A board without a project MUST be labeled as team-wide. Brown/ADO boards MUST be read-only in this change: Jcode MUST NOT offer or dispatch status changes for an ADO binding.

#### Scenario: Team-wide board is labeled
- **WHEN** a Personal or Priceless binding names a team and no project
- **THEN** the board shows that team's issues and labels the scope as team-wide
- **AND** Jcode does not pick a project from visible results

#### Scenario: ADO status change is refused
- **WHEN** a user attempts to move an issue on a Brown/ADO board
- **THEN** Jcode refuses before confirmation or dispatch and reports that ADO boards are read-only in this version
- **AND** no audit intent record or provider call is made

## ADDED Requirements

### Requirement: Compact header rows

The TUI header MUST NOT render the product/status row, the model row, the account router heading or its rows, the MCP summary row, the skills row, or the working-directory row. It MUST render the server and client on one row as `🗼 {server version} · 💻 {client version}`, without the session name or session emoji. On that row it MUST show `srv↑`/`cli↑` when updates are available. Every header row MUST fit its terminal-cell width or truncate with an ellipsis.

#### Scenario: Remote client header
- **WHEN** a remote client connects to a server with matching versions
- **THEN** the header shows one row `🗼 v1.0.287 · 💻 v1.0.287`
- **AND** it shows no `client:`, `server:`, model, account router, mcp, skills, or directory rows

#### Scenario: Version mismatch
- **WHEN** the server and client versions differ
- **THEN** both version labels use the mismatch highlight style

#### Scenario: Narrow terminal
- **WHEN** the width is smaller than a row's text
- **THEN** the row truncates with `…` and does not wrap

### Requirement: Workspace row

When the working directory matches a `[[workspaces.rules]]` entry, the header MUST show `ws: {world} · {repo}:` followed by `key:value` pairs from that rule (`tracker`, `branch`, and the identity flag when present). When no rule matches, it MUST NOT show the row. The row MUST NOT run git on the render thread.

#### Scenario: Matched personal repo
- **WHEN** the origin matches rule `personal` with `tracker = "linear"`
- **THEN** the row reads `ws: personal · jcode: tracker:linear branch:main`

#### Scenario: Unmatched repo
- **WHEN** no rule matches the origin
- **THEN** the header shows no `ws:` row

### Requirement: Linear board row

When the verified world has a Linear board binding, the header MUST show `linear: {org} · {team} · {board}: N {State} | …` with counts per workflow state from the bound scope, using the board's state names in board order. It MUST omit states of type `completed` and `canceled`. Names and counts MUST come from the allowlisted board provider, off the render thread, and be cached per scope. When no binding exists, or the world is Brown/ADO or unverified, it MUST NOT show the row or query Linear. A provider failure MUST show `linear: unavailable` and MUST NOT show counts from a different scope.

#### Scenario: Bound personal board
- **WHEN** the board has states Backlog (4), Todo (2), In Progress (1), Done (9), Canceled (3)
- **THEN** the row ends `4 Backlog | 2 Todo | 1 In Progress`

#### Scenario: No binding
- **WHEN** the matched rule has no `[workspaces.rules.board]` with backend `linear`
- **THEN** no `linear:` row appears and no Linear request is made

#### Scenario: Provider failure
- **WHEN** the Linear query fails before any data has loaded for this scope
- **THEN** the row reads `linear: unavailable`

### Requirement: OpenSpec row

When `openspec/` exists in the working directory, the header MUST show `openspec: N not ready | N ready | N active`. An active change has at least one checked task and is not complete. A ready change has zero checked tasks and passes `openspec validate`. A not-ready change has zero checked tasks and fails validation or has no `tasks.md`. The counts MUST come from one background `openspec validate --changes --json` call plus `openspec list --json`. The result is cached and refreshed only when `openspec/changes/` changes on disk.

#### Scenario: Repo with changes
- **WHEN** there are 20 active changes, 5 valid unstarted changes, and 2 invalid unstarted changes
- **THEN** the row reads `openspec: 2 not ready | 5 ready | 20 active`

#### Scenario: No openspec dir
- **WHEN** the working directory has no `openspec/`
- **THEN** no `openspec:` row appears and no CLI is spawned

#### Scenario: CLI missing
- **WHEN** `openspec` is not on `PATH` or exits nonzero
- **THEN** no `openspec:` row appears and the header still renders

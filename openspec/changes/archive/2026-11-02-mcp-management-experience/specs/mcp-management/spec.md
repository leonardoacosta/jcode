## ADDED Requirements

### Requirement: MCP status command
The TUI SHALL provide `/mcp` showing each configured server's name, config scope, state, tool count, and last failure class with stderr tail.

#### Scenario: Degraded server visible
- **WHEN** a server is degraded and the user runs `/mcp`
- **THEN** the row shows `degraded`, the failure class, and the stderr tail

#### Scenario: No servers configured
- **WHEN** no MCP config exists
- **THEN** `/mcp` shows an empty state naming the config paths checked

### Requirement: Info widget MCP row
The info widget SHALL show connected/total MCP servers and degraded count on one row that truncates with an ellipsis instead of wrapping.

#### Scenario: Narrow terminal
- **WHEN** the widget is 12 cells wide
- **THEN** the MCP row occupies one line and fits within 12 display cells

### Requirement: Direct tool exposure
A server with `expose: "direct"`, or a tool listed in `mcp.pin_tools`, SHALL be registered as a model tool named `mcp__<server>__<tool>`; others remain search-only.

#### Scenario: Pinned tool callable directly
- **WHEN** `mcp.pin_tools` contains `aperture/query` and the server is connected
- **THEN** the model tool list includes `mcp__aperture__query`

#### Scenario: Pinned tool on failed server
- **WHEN** the pinned tool's server is degraded
- **THEN** the tool is omitted and no spawn is attempted

### Requirement: Workspace scoping and lazy start
A server with a `workspaces` list SHALL start only for sessions whose cwd matches a listed glob; a server with `lazy: true` SHALL connect on first search or call.

#### Scenario: Out-of-scope workspace
- **WHEN** a session starts in `~/dev/brown/x` and the server lists only `~/dev/personal/**`
- **THEN** the server is not spawned and `/mcp` shows it as `out of scope`

#### Scenario: Lazy server first use
- **WHEN** a lazy server's tool is called for the first time
- **THEN** it connects, then the call proceeds; on failure the classified reason is returned

### Requirement: Config import
`jcode mcp import --from claude|codex` SHALL print the servers it would add without writing, and write them only with `--apply`, skipping names already present.

#### Scenario: Dry run
- **WHEN** the user runs import without `--apply`
- **THEN** no file changes and the planned additions are listed

#### Scenario: Name conflict
- **WHEN** an imported server name already exists
- **THEN** it is skipped and reported, existing config unchanged

### Requirement: In-TUI OAuth login
`/mcp login <server>` SHALL run the existing OAuth flow from the TUI and report success or failure inline.

#### Scenario: Login cancelled
- **WHEN** the browser flow times out
- **THEN** the TUI shows a timeout error and stored credentials are unchanged

# capability-discovery Specification

## Purpose
Let users inspect native and connected MCP tool capabilities without invocation, lazy-server startup or client-local substitution for remote session data.

## Requirements

### Requirement: Users can list native and MCP tools
The TUI SHALL provide a read-only listing of native tools and connected MCP tools with kind, name, owning MCP server and description.

#### Scenario: Mixed listing
- **WHEN** the user runs `/tools` with native tools and a connected MCP server
- **THEN** rows SHALL show kind `tool` for native tools and kind `mcp` with the server name for MCP tools

#### Scenario: Query filter with no match
- **WHEN** the user runs `/tools zzz` and nothing matches
- **THEN** the view SHALL show an empty-result notice and SHALL NOT invoke any tool

#### Scenario: Same-name collision
- **WHEN** an MCP tool shares a native tool's name
- **THEN** the native tool SHALL be listed as the effective tool and the MCP tool SHALL remain listed under its server namespace

### Requirement: Remote and lazy safety
Listing SHALL NOT start lazy MCP servers, and remote sessions SHALL degrade explicitly on older servers.

#### Scenario: Remote server supports listing
- **WHEN** a remote client runs `/tools`
- **THEN** it SHALL show the server's tools, not the client's

#### Scenario: Remote server lacks listing
- **WHEN** the remote server does not support the request
- **THEN** the client SHALL show "tool list unavailable from this server"

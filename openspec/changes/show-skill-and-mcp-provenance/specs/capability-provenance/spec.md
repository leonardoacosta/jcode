## ADDED Requirements

### Requirement: Skills record and display provenance
Each loaded skill SHALL record its discovery root, resolved path, symlink flag and shadowed roots, and the TUI SHALL display them before invocation.

#### Scenario: Project skill overrides global
- **WHEN** a project skill and a global skill share a name
- **THEN** `/skill info <name>` SHALL show root `project` and list the shadowed global root

#### Scenario: Symlinked skill
- **WHEN** a skill directory is a symlink
- **THEN** the detail view SHALL show the symlink flag and resolved target

#### Scenario: Unknown skill
- **WHEN** `/skill info` names a skill that is not loaded
- **THEN** it SHALL report not found and SHALL NOT activate anything

### Requirement: MCP tool provenance degrades explicitly
MCP tool details SHALL show the owning server and config scope when known.

#### Scenario: Scope known
- **WHEN** the MCP manager exposes the server's config scope
- **THEN** the detail SHALL show server name and scope

#### Scenario: Scope unavailable or legacy remote
- **WHEN** scope data is missing or the remote peer is legacy
- **THEN** the detail SHALL show "unknown" and SHALL NOT guess

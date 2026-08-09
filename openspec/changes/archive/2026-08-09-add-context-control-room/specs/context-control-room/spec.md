## ADDED Requirements

### Requirement: Control Room overlay toggle
Jcode SHALL provide a temporary TUI Control Room overlay that opens and closes with `Alt+O` without replacing the side panel or session picker.

#### Scenario: Open the overlay from normal input
- **WHEN** the TUI is connected and no higher-priority overlay owns input
- **AND** the user presses `Alt+O`
- **THEN** Jcode SHALL open a centered Context Control Room overlay
- **AND** the current input draft SHALL remain unchanged.

#### Scenario: Close the overlay
- **WHEN** the Control Room overlay is open
- **AND** the user presses `Esc` or `Alt+O`
- **THEN** Jcode SHALL close the overlay
- **AND** focus SHALL return to the previously active input surface.

#### Scenario: Existing overlay has priority
- **WHEN** a higher-priority overlay such as session picker, login picker, account picker, or usage overlay is open
- **AND** the user presses `Alt+O`
- **THEN** Jcode SHALL preserve the active overlay's input ownership
- **AND** it SHALL NOT leak the key into the prompt draft.

### Requirement: Context hierarchy display
The Control Room overlay SHALL display the active context hierarchy from organization through execution pane with provenance labels.

#### Scenario: Render complete context
- **WHEN** persisted semantic context and execution-substrate metadata are available
- **THEN** the overlay SHALL show organization, project, workspace/worktree, initiative, task/run, Jcode session, and Herdr pane
- **AND** each row SHALL include a provenance label and stable identifier where available.

#### Scenario: Context is partially inferred
- **WHEN** a project, workspace, initiative, task/run, or Herdr field is derived from cwd, path hash, current client metadata, or runtime environment rather than durable semantic storage
- **THEN** the overlay SHALL label that field as inferred or current-client/Herdr provenance
- **AND** it SHALL NOT present inferred data as persisted authority.

#### Scenario: Context is unavailable
- **WHEN** a context field cannot be determined
- **THEN** the overlay SHALL show an explicit unavailable state for that row
- **AND** it SHALL keep the rest of the overlay usable.

### Requirement: Jcode authoritative context snapshot
Jcode SHALL assemble a context snapshot that separates durable semantic identity from local execution-substrate identity.

#### Scenario: Build snapshot from persisted identity
- **WHEN** persisted organization, project, workspace, initiative, task/run, or session identity exists
- **THEN** the context snapshot SHALL prefer those persisted IDs over path-derived or Herdr-derived labels.

#### Scenario: Build snapshot from current session metadata
- **WHEN** persisted semantic identity is missing but the current session has working directory, title, model/provider, resume group, or client subscription metadata
- **THEN** the context snapshot SHALL include those values with current-client or inferred provenance.

#### Scenario: Herdr metadata is available
- **WHEN** trusted local Herdr metadata is available for the current pane/session
- **THEN** the context snapshot SHALL include it under execution substrate
- **AND** it SHALL NOT treat Herdr pane identity as organization, project, or initiative authority.

#### Scenario: Herdr is unavailable
- **WHEN** Herdr is absent, disconnected, times out, or returns incomplete metadata
- **THEN** the context snapshot SHALL include a degraded Herdr unavailable reason
- **AND** snapshot assembly SHALL NOT fail.

### Requirement: Control Room navigation and safe actions
The overlay SHALL support read-mostly keyboard navigation and safe actions without destructive context administration.

#### Scenario: Navigate overlay rows
- **WHEN** the overlay is open and the user presses supported movement keys
- **THEN** the selected row or section SHALL change visibly
- **AND** scrolling SHALL keep the selection visible on small terminals.

#### Scenario: Copy selected context value
- **WHEN** the overlay is open and the user invokes the copy action on a row with a copyable value
- **THEN** Jcode SHALL copy that row's stable ID or display value through the existing clipboard path
- **AND** it SHALL show non-blocking feedback.

#### Scenario: Focus existing execution surface
- **WHEN** the overlay is open and the selected row refers to a focusable existing Jcode session or Herdr pane
- **THEN** Jcode MAY invoke the existing safe focus/resume path
- **AND** it SHALL NOT spawn new work or create a new pane as an implicit side effect.

#### Scenario: Unsupported action
- **WHEN** a selected row has no safe copy/focus behavior
- **THEN** Jcode SHALL show an unsupported or unavailable message
- **AND** it SHALL leave the context unchanged.

### Requirement: Context persistence across reload and reconnect
Jcode SHALL persist durable context identifiers across server reload and client reconnect without trusting transient `$HOME` or shell cwd as authoritative project identity.

#### Scenario: Reconnect reports home directory
- **WHEN** a session with persisted project/workspace identity reconnects from a client whose cwd is `$HOME`
- **THEN** Jcode SHALL preserve the persisted project/workspace identity
- **AND** it SHALL label the reconnect cwd separately if shown.

#### Scenario: Server reload restores session
- **WHEN** a session is restored after a Jcode server reload
- **THEN** the Control Room snapshot SHALL include the same persisted semantic IDs as before the reload
- **AND** execution-substrate fields SHALL refresh or degrade independently.

#### Scenario: Different workspace is intentionally selected
- **WHEN** an explicit supported workflow changes the session's workspace/project context
- **THEN** Jcode SHALL update the persisted context through that workflow
- **AND** the overlay SHALL show the new context with persisted provenance.

### Requirement: Context documentation and evidence
The implementation SHALL maintain separate documentation for context architecture and implementation evidence.

#### Scenario: Architecture documentation exists
- **WHEN** the feature is complete
- **THEN** `docs/CONTEXT_ARCHITECTURE.md` SHALL document the context hierarchy, identity ownership, persistence boundary, and Herdr/Jcode authority split.

#### Scenario: Evidence documentation exists
- **WHEN** the feature is complete
- **THEN** `docs/CONTEXT_CONTROL_ROOM_EVIDENCE.md` SHALL document validation coverage, degraded Herdr behavior, keybinding checks, and known limitations.

### Requirement: Regression protection
The Control Room implementation SHALL preserve existing session, overlay, and keybinding behavior.

#### Scenario: Existing overlay tests run
- **WHEN** focused TUI overlay and keybinding tests run
- **THEN** existing session picker, account/login picker, usage overlay, prompt input, and copy/scroll behaviors SHALL continue to pass.

#### Scenario: Existing session tests run
- **WHEN** focused app-core session subscribe/resume/reload tests run
- **THEN** existing session restoration and cwd preservation behavior SHALL continue to pass.

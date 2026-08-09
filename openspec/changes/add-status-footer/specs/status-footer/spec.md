## ADDED Requirements

### Requirement: Persistent status footer row

Jcode SHALL render a persistent one-row status footer as the bottom-most row of the chat column chrome when `display.footer.style` is `segments`, and SHALL reserve zero height for it when the style is `off`.

#### Scenario: Footer visible by default

- **WHEN** the TUI is connected and `display.footer.style` is `segments` (the default)
- **THEN** the footer SHALL occupy exactly one row at the bottom of the chat column
- **AND** it SHALL render below the input, overscroll, and idle-animation rows.

#### Scenario: Footer disabled restores prior layout

- **WHEN** `display.footer.style` is `off`
- **THEN** the layout SHALL reserve zero rows for the footer
- **AND** the rendered frame SHALL be identical to the pre-change layout.

#### Scenario: Packed and scrolling layouts

- **WHEN** transcript content fits within the available height (packed layout) or exceeds it (scrolling layout)
- **THEN** the footer SHALL remain the bottom-most row in both layouts
- **AND** it SHALL not shift the transcript, input, or overlay layout beyond its own reserved row.

#### Scenario: Overlay interactions

- **WHEN** a full-screen overlay such as session picker, help, changelog, or control room is open
- **THEN** existing overlay draw behavior SHALL take precedence over the footer row
- **AND** the footer SHALL re-render correctly when the overlay closes.

### Requirement: Footer segments

The footer SHALL present session-scoped segments assembled read-only from existing cached state: working directory, execution-mode marker, git branch and status, optional session name, model with provider, reasoning effort, context usage, token counts, and session cost.

#### Scenario: Full render with all data available

- **WHEN** all segment data is available and the width permits
- **THEN** the left zone SHALL show the working directory, execution-mode marker, and git branch with status indicators
- **AND** the right zone SHALL show the model label with provider, reasoning effort, context usage, token counts, and session cost.

#### Scenario: Missing git data

- **WHEN** the session working directory is not inside a git repository
- **THEN** the footer SHALL omit the git segment entirely
- **AND** the remaining segments SHALL keep their positions and separators stable.

#### Scenario: Missing cost data

- **WHEN** the session has no accrued or priced cost
- **THEN** the footer SHALL omit the cost segment
- **AND** it SHALL NOT render a zero-valued placeholder.

#### Scenario: Stale context data

- **WHEN** context state is being updated and no authoritative snapshot is available
- **THEN** the footer SHALL render the last known context value with a stale marker
- **AND** it SHALL NOT present the stale value as authoritative.

#### Scenario: Unnamed session

- **WHEN** the session has no explicit name
- **THEN** the footer SHALL omit the session-name segment
- **AND** it SHALL leave no separator or placeholder artifacts.

#### Scenario: Execution mode marker

- **WHEN** the session runs locally, remotely, or in hybrid mode
- **THEN** the footer SHALL display the corresponding execution-mode marker adjacent to the working directory
- **AND** a remote session SHALL display the session's remote working directory rather than the client's local directory.

### Requirement: Width degradation

The footer SHALL degrade at narrow widths by dropping segments in a fixed documented priority order and SHALL never wrap to a second row.

#### Scenario: Segments drop by priority

- **WHEN** the composed row exceeds the available width
- **THEN** segments SHALL drop in the documented order: session name, cost, tokens, effort, upstream extras, directory depth, git ahead/behind counts
- **AND** the resulting row SHALL fit on one line.

#### Scenario: Extreme narrow width

- **WHEN** the width cannot fit the full remaining segments after dropping
- **THEN** branch and context labels SHALL truncate with the established smart-truncation behavior
- **AND** directory and model SHALL be the last segments truncated.

#### Scenario: Deterministic resize behavior

- **WHEN** the terminal is resized across widths
- **THEN** the footer SHALL render the documented segment subset at each width
- **AND** identical state at an identical width SHALL produce byte-identical rows across repeated renders.

### Requirement: Capability degradation

The footer SHALL degrade cleanly on terminals without Nerd Font or color support.

#### Scenario: ASCII icon mode

- **WHEN** icon mode resolves to ASCII by configuration or terminal detection
- **THEN** the footer SHALL use documented ASCII markers in place of Nerd Font glyphs
- **AND** the row layout and segment order SHALL be unchanged.

#### Scenario: No-color terminal

- **WHEN** the terminal does not support color
- **THEN** the footer SHALL render unstyled text
- **AND** context threshold indication SHALL degrade to a documented textual marker.

### Requirement: Read-only render state

The footer SHALL assemble from cached snapshots only and SHALL NOT mutate agent state, spawn work, or probe the filesystem or git on the render path.

#### Scenario: No per-frame probing

- **WHEN** frames render repeatedly with the footer enabled
- **THEN** git facts SHALL come only from the TTL-cached git info path
- **AND** no subprocess or filesystem access SHALL occur on the render path.

#### Scenario: Session isolation

- **WHEN** multiple sessions are active, including remote client subscriptions
- **THEN** each client SHALL render only its own connected session's footer data
- **AND** no cross-session data SHALL appear.

#### Scenario: No secrets or raw content

- **WHEN** the footer renders
- **THEN** it SHALL contain only numeric aggregates and display-processed metadata
- **AND** it SHALL NOT include credentials, API keys, or raw prompt content.

### Requirement: Footer is not copyable chrome

The footer SHALL be excluded from copy-selection snapshots and mouse hit targets.

#### Scenario: Copy selection excludes the footer

- **WHEN** the user performs mouse copy selection over the screen
- **THEN** the footer row SHALL contribute no selectable rows
- **AND** existing transcript and input copy behavior SHALL be unchanged.

### Requirement: Configuration schema

Jcode SHALL accept an additive `display.footer` configuration section with documented defaults and SHALL ignore unknown keys.

#### Scenario: Defaults preserve compatibility

- **WHEN** an existing configuration file without a footer section is loaded
- **THEN** the footer SHALL resolve to its documented defaults
- **AND** all other configuration SHALL parse unchanged.

#### Scenario: Per-segment visibility

- **WHEN** a segment visibility toggle is disabled in configuration
- **THEN** the footer SHALL omit that segment at all widths
- **AND** remaining segments SHALL keep stable positions and separators.

#### Scenario: Color configuration

- **WHEN** footer color keys are set in `display.colors`
- **THEN** the footer SHALL resolve those colors
- **AND** unset keys SHALL fall back to documented theme tokens.

# composer-frame Specification

## Purpose
TBD - created by archiving change add-composer-frame. Update Purpose after archive.
## Requirements
### Requirement: Composer accent rail

Jcode SHALL render an accent rail on the left of every composer row when `display.composer.style` is `rail`, colored by the active composer mode, and SHALL reserve no rail column when the style is `flat`.

#### Scenario: Rail renders in every composer mode

- **WHEN** the composer is in normal, shell, queued/processing, or skill mode
- **THEN** every composer row (input rows and composer-owned hint rows) SHALL begin with a rail glyph
- **AND** the rail color SHALL follow the existing mode color for that mode.

#### Scenario: Flat style restores prior composer

- **WHEN** `display.composer.style` is `flat`
- **THEN** the composer SHALL reserve no rail column
- **AND** the rendered composer SHALL be byte-identical to the pre-change layout.

#### Scenario: ASCII icon mode

- **WHEN** icon mode resolves to ASCII
- **THEN** the rail SHALL render as `|`
- **AND** row layout and mode coloring SHALL be unchanged.

#### Scenario: No-color terminal

- **WHEN** the terminal does not support color
- **THEN** the rail SHALL render unstyled
- **AND** the composer SHALL remain otherwise unchanged.

### Requirement: Composer metadata row

When `display.composer.metadata` is true, Jcode SHALL render one metadata row at the bottom of the composer showing model, provider, and reasoning effort.

#### Scenario: Full metadata render

- **WHEN** model, provider, and effort are known
- **THEN** the metadata row SHALL render `model · provider · effort` right-aligned with muted styling.

#### Scenario: Effort omitted when off

- **WHEN** reasoning effort is off or unset
- **THEN** the metadata row SHALL omit the effort segment
- **AND** it SHALL leave no separator artifacts.

#### Scenario: Model unavailable keeps height stable

- **WHEN** the model label is unavailable, such as before authentication completes
- **THEN** the composer SHALL still reserve the metadata row
- **AND** the row SHALL render empty muted content so composer height does not shift.

#### Scenario: Metadata during processing

- **WHEN** a turn is processing, prompts are queued, or overscroll is active
- **THEN** the metadata row SHALL remain rendered
- **AND** it SHALL not depend on the right fact stack's stand-down rules.

#### Scenario: Metadata disabled

- **WHEN** `display.composer.metadata` is false
- **THEN** the composer SHALL reserve zero rows for metadata
- **AND** the composer height SHALL equal its text rows plus existing reservations.

### Requirement: Width degradation

The composer frame SHALL degrade at narrow widths in a fixed documented order and SHALL never increase the composer's reserved height beyond the rail plus one metadata row.

#### Scenario: Metadata drops before rail

- **WHEN** the composer width is too narrow for the full metadata row
- **THEN** the metadata row SHALL drop the effort segment, then the provider segment, then truncate the model label
- **AND** the rail SHALL remain on every composer row.

#### Scenario: Deterministic rendering

- **WHEN** identical state renders repeatedly at an identical width
- **THEN** the composer frame SHALL produce byte-identical output.

### Requirement: Copy safety

The rail and metadata row SHALL be excluded from copy-selection snapshots and SHALL NOT alter typed-text selection.

#### Scenario: Rail is never copied

- **WHEN** the user performs mouse copy selection over composer rows
- **THEN** the rail column and metadata row SHALL contribute no copied text
- **AND** selected typed text SHALL be byte-identical to the pre-change behavior.

### Requirement: Frame scope

The composer frame SHALL apply only to the composer chunk and SHALL NOT change the rendering or layout ownership of the queued-messages row, inline UI rows, command-suggestions overlay, or send-mode indicator.

#### Scenario: Suggestions overlay stability

- **WHEN** the command-suggestions overlay is active during streamed output
- **THEN** composer rows SHALL NOT shift relative to pre-change positions
- **AND** the overlay SHALL continue to render as a later pass.

### Requirement: Configuration schema

Jcode SHALL accept an additive `display.composer` configuration section with documented defaults and SHALL ignore unknown keys.

#### Scenario: Defaults preserve compatibility

- **WHEN** an existing configuration file without a composer section is loaded
- **THEN** the composer frame SHALL resolve to its documented defaults
- **AND** all other configuration SHALL parse unchanged.

#### Scenario: Color configuration

- **WHEN** composer frame color keys are set in `display.colors`
- **THEN** the rail and metadata SHALL resolve those colors
- **AND** unset keys SHALL fall back to documented theme tokens.


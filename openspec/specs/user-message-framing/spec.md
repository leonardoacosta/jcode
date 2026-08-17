# user-message-framing Specification

## Purpose
TBD - created by archiving change add-user-message-framing. Update Purpose after archive.
## Requirements
### Requirement: User-message frame styles

Jcode SHALL decorate transcript user prompt rows with one of five frame styles: `framed`, `framed-copy-friendly`, `compact`, `labeled`, or `off`, selected by `display.user_messages.style` with `framed` as the default.

#### Scenario: Framed style

- **WHEN** the style is `framed`
- **THEN** each user prompt SHALL be surrounded by full-width top and bottom border rows
- **AND** every prompt row SHALL begin with an accent rail
- **AND** the existing prompt number, `›` glyph, and background band SHALL render inside the frame unchanged.

#### Scenario: Framed copy-friendly style

- **WHEN** the style is `framed-copy-friendly`
- **THEN** border rows and the background band SHALL render as in `framed`
- **AND** prompt rows SHALL have a one-cell leading gutter with no rail glyphs.

#### Scenario: Compact style

- **WHEN** the style is `compact`
- **THEN** prompt rows SHALL begin with an accent rail
- **AND** no border rows SHALL be added, so transcript height is unchanged from the pre-change layout.

#### Scenario: Labeled style

- **WHEN** the style is `labeled`
- **THEN** each user prompt SHALL render inside a rounded box
- **AND** the top border SHALL contain the fixed label `User`.

#### Scenario: Off style restores prior rendering

- **WHEN** the style is `off`
- **THEN** user prompt rows SHALL render byte-identically to the pre-change layout with no rail, gutter, or border decoration.

### Requirement: Frame derivation from prepared rows

Frames SHALL be derived from the prepared-line pipeline's user prompt row anchors so that frames always span exactly the prompt's wrapped rows at the current width.

#### Scenario: Multi-line prompt framing

- **WHEN** a user prompt wraps across multiple rows
- **THEN** rail or gutter decoration SHALL appear on every wrapped row
- **AND** border rows SHALL appear once before the first and once after the last wrapped row.

#### Scenario: Re-wrap on resize

- **WHEN** the terminal width changes and prepared rows re-wrap
- **THEN** frames SHALL re-derive from the new anchors
- **AND** identical state at an identical width SHALL produce byte-identical rows across repeated renders.

#### Scenario: Style switch re-renders deterministically

- **WHEN** the frame style changes at runtime through configuration reload
- **THEN** user rows SHALL re-render through the existing prepared-cache path
- **AND** identical inputs SHALL produce identical rows.

### Requirement: Width and capability degradation

Frames SHALL degrade cleanly at narrow widths and on terminals without Nerd Font or color support.

#### Scenario: Narrow width

- **WHEN** the chat column is narrow
- **THEN** border rows SHALL render at the available width
- **AND** frame decoration SHALL NOT cause prompt text to wrap beyond its own wrapped rows.

#### Scenario: ASCII icon mode

- **WHEN** icon mode resolves to ASCII
- **THEN** borders SHALL draw with `-`, `|`, and `+` glyphs and the label as plain text
- **AND** layout SHALL be unchanged.

#### Scenario: No-color terminal

- **WHEN** the terminal does not support color
- **THEN** frames SHALL render unstyled
- **AND** the user background band behavior SHALL be unchanged.

### Requirement: Copy safety

Frame decoration SHALL be excluded from copy-selection snapshots and SHALL NOT alter prompt-text selection.

#### Scenario: Decoration is never copied

- **WHEN** the user performs mouse copy selection over framed user prompt rows
- **THEN** border rows, rail and gutter columns, and the `User` label SHALL contribute no copied text
- **AND** selected prompt text SHALL be byte-identical to the pre-change behavior.

### Requirement: Streaming and scroll stability

Framing SHALL NOT change transcript scroll behavior or add per-frame work during streamed output.

#### Scenario: Streaming bottom anchoring

- **WHEN** assistant output streams after a framed user prompt
- **THEN** bottom anchoring and auto-scroll SHALL behave identically to pre-change behavior
- **AND** frame decoration SHALL NOT be recomputed per frame for static user rows.

#### Scenario: Prompt numbering and preview

- **WHEN** prompt preview or navigation references prompt numbers
- **THEN** numbering SHALL render inside the frame unchanged
- **AND** prompt-preview references SHALL resolve to the same prompts as before the change.

### Requirement: Configuration schema

Jcode SHALL accept an additive `display.user_messages` configuration section with documented defaults and SHALL ignore unknown keys.

#### Scenario: Defaults preserve compatibility

- **WHEN** an existing configuration file without a user-messages section is loaded
- **THEN** the frame style SHALL resolve to `framed`
- **AND** all other configuration SHALL parse unchanged.

#### Scenario: Color configuration

- **WHEN** user-message frame color keys are set in `display.colors`
- **THEN** borders, rails, and labels SHALL resolve those colors
- **AND** unset keys SHALL fall back to documented theme tokens.


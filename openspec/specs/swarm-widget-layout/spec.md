# swarm-widget-layout Specification

## Purpose
TBD - created by archiving change swarm-widget-cell-width. Update Purpose after archive.

## Requirements

### Requirement: Display-cell truncation
`truncate_smart` SHALL measure input and output in Unicode display cells and SHALL return text whose display width does not exceed the requested limit, ending with `...` when truncated.

#### Scenario: Wide characters truncated by cells
- **WHEN** a 10-character CJK name (20 cells) is truncated to 12 cells
- **THEN** the result is at most 12 cells wide and ends with `...`

#### Scenario: Fitting text unchanged
- **WHEN** text of 8 cells is truncated to 12 cells
- **THEN** the text is returned unchanged with no ellipsis

### Requirement: Member rows fit widget width
Each swarm member row SHALL fit within the widget inner width, including role prefix and status icon.

#### Scenario: Long member name in narrow widget
- **WHEN** a coordinator member with a long name and detail renders in a 20-cell inner width
- **THEN** the full row (prefix, icon, text) is at most 20 cells and ends with `...`

#### Scenario: Short member row
- **WHEN** a member row's content fits the width
- **THEN** it renders without truncation

### Requirement: Clipped swarm page rows show ellipsis
Swarm page lines clipped to width SHALL end with `…` and SHALL NOT exceed the width.

#### Scenario: Hint row narrower than content
- **WHEN** the swarm page renders at width 30
- **THEN** the hint row is at most 30 cells and ends with `…`

#### Scenario: Line already fits
- **WHEN** a line is narrower than the width
- **THEN** it is unchanged and has no ellipsis

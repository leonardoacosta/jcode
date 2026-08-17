## ADDED Requirements

### Requirement: Consolidated status facts
The TUI SHALL render model, provider, access or effort, and context usage as one primary status-line group, and SHALL avoid rendering duplicate copies of those same facts in the default detail widgets.

#### Scenario: Full data is available
- **WHEN** model, provider, effort, and context usage are available
- **THEN** the primary status line renders one grouped segment containing those facts
- **AND** the detail surface does not render a second default copy of the same status facts

#### Scenario: Partial data is available
- **WHEN** one or more facts are unavailable
- **THEN** the group omits unavailable facts without placeholder text or layout corruption

### Requirement: Independent status controls
The TUI SHALL expose independent clickable status segments for the grouped model/context details, KV cache details, and provider-limit details.

#### Scenario: Toggle model/context details
- **WHEN** the user clicks the model-context segment
- **THEN** the TUI toggles the model and context detail surface as one group
- **AND** it does not change KV cache, provider-limit, todo, or memory visibility

#### Scenario: Toggle KV cache details
- **WHEN** the user clicks the KV cache segment
- **THEN** only the KV cache detail widget visibility toggles

#### Scenario: Toggle provider limits
- **WHEN** the user clicks the provider-limit segment
- **THEN** only the provider-limit detail widget visibility toggles

#### Scenario: Repeated clicks
- **WHEN** the user clicks the same segment repeatedly
- **THEN** its corresponding visibility alternates predictably without affecting unrelated widgets

### Requirement: Preserve todos and memories
Hiding any model/context, KV cache, or provider-limit detail surface SHALL preserve independently enabled todo and memory surfaces.

#### Scenario: Secondary panels are hidden
- **WHEN** model/context, KV cache, and provider-limit widgets are hidden
- **AND** todo and memory widgets are enabled
- **THEN** todo and memory widgets remain visible

### Requirement: Width-safe status rendering
The TUI SHALL degrade status segments at narrow widths without overlap, wrapping, or interaction regions that extend outside the status row.

#### Scenario: Narrow terminal
- **WHEN** the terminal width is insufficient for all status segments
- **THEN** secondary segment content is abbreviated or omitted in a deterministic priority order
- **AND** the model-context group remains the highest-priority group when data exists

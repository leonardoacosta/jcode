# skill-selector-metadata Specification

## Purpose
TBD - created by archiving change add-typed-skill-selector-metadata. Update Purpose after archive.

## Requirements

### Requirement: Typed selector entries show real descriptions
The slash-command selector SHALL represent each candidate with a kind and description, and skill rows SHALL show the skill's frontmatter description instead of a fixed placeholder.

#### Scenario: Local skill shows its description
- **WHEN** a loaded skill `foo` with description "Does foo" matches the typed prefix
- **THEN** its row SHALL show kind `skill` and text "Does foo"

#### Scenario: Remote legacy peer sends names only
- **WHEN** a remote server sends skill names without descriptions
- **THEN** the row SHALL show description "unknown" and SHALL NOT invent text

### Requirement: Rows never wrap and native precedence holds
Selector rows SHALL fit their cell width or truncate with an ellipsis, and a native command SHALL win over a skill with the same name.

#### Scenario: Narrow terminal
- **WHEN** the selector renders at widths 0, 1, 20, 40 and 80 cells
- **THEN** each candidate SHALL occupy exactly one visual row with no wrapping

#### Scenario: Name collision
- **WHEN** a skill is named the same as a native command
- **THEN** only the native command row SHALL appear

#### Scenario: Control text in description
- **WHEN** a skill description contains control or escape characters
- **THEN** they SHALL be stripped before rendering

## ADDED Requirements

### Requirement: Explicit semantic artifact contract
The system SHALL allow a tool to mark its output as an explicit rendered artifact with a recognized `markdown`, `message`, or `code` kind, an optional title, and an optional code language without inferring the kind from tool name or content.

#### Scenario: Tool emits a recognized artifact
- **WHEN** a tool completes with a recognized rendered-artifact descriptor
- **THEN** the descriptor is associated with that exact tool result and the output string remains the artifact body

#### Scenario: Ordinary tool emits no descriptor
- **WHEN** a tool completes without rendered-artifact metadata
- **THEN** the system uses the existing generic tool-output behavior unchanged

### Requirement: Artifact metadata survives transcript lifecycle
The system SHALL preserve recognized artifact metadata through live streaming, session persistence, reconnect, history rendering, and replay so the same tool result retains the same artifact kind.

#### Scenario: Live artifact is restored
- **WHEN** a rendered artifact is saved and the session is later reconnected or replayed
- **THEN** its restored display uses the same artifact kind, title, language, and body as the live display

#### Scenario: Legacy session has no metadata
- **WHEN** a stored tool result predates rendered-artifact metadata
- **THEN** it deserializes successfully and renders through the existing generic tool path

### Requirement: Distinct artifact card hierarchy
The TUI SHALL render recognized Markdown, Message, and Code artifacts as visually distinct cards that are distinguishable from reasoning and generic tool actions.

#### Scenario: Markdown artifact renders
- **WHEN** a tool result is marked as a Markdown artifact
- **THEN** the TUI renders a document-blue rounded card with a Markdown identity and preserves Markdown structures inside it

#### Scenario: Message artifact renders
- **WHEN** a tool result is marked as a Message artifact
- **THEN** the TUI renders a warm neutral rounded card with a Message identity and readable formatted prose inside it

#### Scenario: Code artifact renders
- **WHEN** a tool result is marked as a Code artifact
- **THEN** the TUI renders a terminal-green rounded card with a Code identity, includes the language when provided, and syntax-highlights through the existing code renderer

#### Scenario: Artifact renders at narrow width or in ASCII mode
- **WHEN** any recognized artifact card renders in a narrow viewport or ASCII capability mode
- **THEN** its borders, title, and body remain width-bounded and readable using supported fallback glyphs

### Requirement: Artifact copy semantics exclude chrome
The system SHALL preserve the original artifact body as selectable or copyable content while excluding card borders, titles, labels, and gutters from copied text.

#### Scenario: Code artifact is copied
- **WHEN** a user copies a Code artifact through the code-block copy target
- **THEN** the copied content equals the exact source body and retains the optional language metadata

#### Scenario: Markdown or Message artifact is selected
- **WHEN** a user selects text from a Markdown or Message artifact
- **THEN** the semantic copy mapping excludes all card chrome

### Requirement: Safe fallback for unsupported metadata
The system SHALL retain visible tool output when artifact metadata is absent, malformed, or unsupported.

#### Scenario: Unsupported artifact metadata reaches the TUI
- **WHEN** a tool result contains an artifact kind the current client does not recognize
- **THEN** the full output renders through the generic tool path without transcript loss or failure

### Requirement: Non-artifact rendering remains unchanged
The system SHALL NOT apply artifact-card styling to ordinary assistant messages, reasoning, user messages, plans, system cards, or untagged tool actions.

#### Scenario: Mixed transcript renders
- **WHEN** a transcript contains reasoning, generic tool actions, ordinary assistant Markdown, and explicit rendered artifacts
- **THEN** only the explicitly tagged artifacts use the new Markdown, Message, or Code card UIs

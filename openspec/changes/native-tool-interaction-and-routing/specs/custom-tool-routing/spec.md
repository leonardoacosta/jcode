## ADDED Requirements

### Requirement: P1 Complete gated inventory
Tool routing guidance SHALL cover every registered custom tool through a documented family or individual rule and omit unavailable capabilities.

#### Scenario: Enabled registry
- **WHEN** normal, restricted, ambient and selfdev inventories are assembled
- **THEN** each tool has a coverage entry and exposed guidance matches session policy

#### Scenario: Platform difference
- **WHEN** macos_computer_use is not registered on Linux
- **THEN** prompt does not instruct the model to call it; macOS guidance describes point coordinates, AX preference and discover/permissions

### Requirement: P2 Effective prompt composition
Tool guidance SHALL reach both bundled and custom base prompts without overwriting user files or bypassing stricter policies.

#### Scenario: Global replacement
- **WHEN** global or project system-prompt.md replaces the default
- **THEN** capability guidance remains present once with deterministic ordering

#### Scenario: Safety or unavailable service
- **WHEN** skills require a specific browser workflow or hosted tools are disabled
- **THEN** guidance preserves those constraints and does not suggest bypassing them

### Requirement: P3 Browser contract parity
Advertised browser actions SHALL match safe executable behavior, including scoped jev_select when enabled.

#### Scenario: Scoped selection
- **WHEN** an authorized Jev selection targets a specific tab/window/frame and valid element
- **THEN** snapshot and action use that same target, schema exposes the supported action and result reports what happened

#### Scenario: Invalid selection
- **WHEN** target is stale/unknown, operation unsupported, value absent or Jev unavailable
- **THEN** no guessed mutation occurs; return an actionable error and preserve manual browser actions

### Requirement: P4 Task routing and consent
Guidance SHALL distinguish human choices, machine evaluation, page retrieval, live browser interaction and desktop control without turning probabilistic output into authorization.

#### Scenario: Task selection
- **WHEN** agent faces a blocking preference, typed evaluation, public document or live UI task
- **THEN** it routes respectively to question, evaluate, fetch or available browser/computer capability

#### Scenario: Mutating action
- **WHEN** a selected action would send, purchase, delete or otherwise require consent
- **THEN** tool guidance requires existing authorization and never treats evaluate or a default answer as consent

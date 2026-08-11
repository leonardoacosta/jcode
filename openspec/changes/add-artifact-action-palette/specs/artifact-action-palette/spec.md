## ADDED Requirements

### Requirement: Contextual artifact action palette
Jcode SHALL provide a configurable action palette for the currently focused rendered artifact or rendered URL, with a default `Alt+Ctrl+A` binding and actions for Brief aloud, Open on Mac, Remote preview, Send to iPhone, and Copy target.

#### Scenario: Focused artifact opens the palette
- **WHEN** the user invokes the palette binding while a rendered artifact is focused
- **THEN** Jcode opens the palette with a stable typed snapshot of that artifact and shows supported actions

#### Scenario: Focused rendered URL opens the palette
- **WHEN** the user invokes the palette binding while a rendered URL is focused
- **THEN** Jcode opens the palette for the resolved URL without requiring the user to copy or re-enter it

#### Scenario: No actionable target is focused
- **WHEN** the user invokes the palette binding without a resolvable artifact or URL target
- **THEN** Jcode does not guess a target and shows a concise unavailable notice

#### Scenario: Source changes while the palette is open
- **WHEN** transcript updates, resize, or navigation invalidate the captured source before action execution
- **THEN** Jcode fails closed or uses the unchanged captured semantic target and never silently retargets another artifact or URL

### Requirement: Persisted Decision Brief artifact
Jcode SHALL represent an options-and-recommendation document as an explicit `DecisionBrief` rendered artifact whose Markdown body, title, and identity survive live display, persistence, reconnect, and replay.

#### Scenario: Decision Brief renders
- **WHEN** a tool produces a recognized Decision Brief artifact
- **THEN** the TUI renders a visually distinct Decision Brief card using Markdown semantics and excludes card chrome from copied body text

#### Scenario: Decision Brief is restored
- **WHEN** a session containing a Decision Brief is saved and later reconnected or replayed
- **THEN** the restored card retains the same title, Markdown body, and Decision Brief identity

#### Scenario: Older client sees an unknown kind
- **WHEN** a client cannot recognize the Decision Brief kind
- **THEN** the full body remains visible through the existing safe generic fallback

### Requirement: Paired written and spoken brief representations
Jcode SHALL compose a compact written Decision Brief and separate natural spoken prose from the same focused artifact and decision context.

#### Scenario: Brief pair is generated
- **WHEN** the user selects Brief aloud for a supported target
- **THEN** Jcode produces a persisted Markdown Decision Brief and separate 60-150-word spoken prose covering outcome, why it matters, decision points, and next step

#### Scenario: Spoken register is validated
- **WHEN** Jcode prepares spoken prose for Herald
- **THEN** the text contains natural sentences, effects rather than implementation narration, and no Markdown, code, file paths, identifiers, or unrequested measurements

#### Scenario: Briefing remains explicit-only
- **WHEN** an artifact renders, a tool completes, a hook fires, or a session ends without the user selecting Brief aloud
- **THEN** Jcode MUST NOT invoke Herald briefing delivery

### Requirement: Herald briefing integration
Jcode SHALL send spoken briefing prose through Herald's existing explicit brief path without implementing a parallel speech, synthesis, playback, retry, or history pipeline.

#### Scenario: Herald accepts a briefing
- **WHEN** the user selects Brief aloud and Herald accepts the request
- **THEN** Jcode reports the briefing as accepted and treats Herald history as the authoritative eventual delivery outcome

#### Scenario: Herald is unavailable
- **WHEN** the brief entry point cannot be resolved or launched
- **THEN** Jcode preserves the written Decision Brief, reports speech as unavailable, and does not fail the turn or session

#### Scenario: Request outcome is ambiguous
- **WHEN** a Herald invocation may have reached the service but returns an error or timeout
- **THEN** Jcode MUST NOT retry through another speech path that could speak the briefing twice

### Requirement: Explicit opener destinations
Jcode SHALL execute explicit palette actions through installed `mopen`, `ropen`, and `iopen` commands while preserving ordinary click behavior outside the palette.

#### Scenario: Open on Mac
- **WHEN** the user selects Open on Mac for a supported target and `mopen` is available
- **THEN** Jcode invokes `mopen` with the resolved target as one argument and reports the launch result

#### Scenario: Remote preview
- **WHEN** the user selects Remote preview for a supported target and `ropen` is available
- **THEN** Jcode invokes `ropen` with the resolved target as one argument and reports the launch result

#### Scenario: Send to iPhone
- **WHEN** the user selects Send to iPhone for a supported target and `iopen` is available
- **THEN** Jcode invokes `iopen` with the resolved target as one argument and reports the launch result

#### Scenario: Optional helper is missing
- **WHEN** an opener is not installed or executable
- **THEN** only that action is disabled or fails softly with a concise notice

#### Scenario: Unsafe target resembles an option
- **WHEN** a target is unresolved, empty, unsupported, or begins with an option prefix the helper cannot safely disambiguate
- **THEN** Jcode rejects the action without spawning the helper

#### Scenario: Ordinary click remains unchanged
- **WHEN** the user clicks a rendered link without opening the palette
- **THEN** Jcode retains existing repository-Markdown side-panel and detached-open/copy behavior

### Requirement: Palette accessibility and compatibility
The palette SHALL remain keyboard-operable, narrow-width safe, discoverable through hotkey feedback, and compatible with sessions and tools that emit no new metadata.

#### Scenario: Keyboard operation
- **WHEN** the palette is open
- **THEN** the user can inspect, select, execute, or cancel every action without a mouse

#### Scenario: Narrow terminal
- **WHEN** the palette renders in a narrow terminal or ASCII glyph mode
- **THEN** labels remain readable and width-bounded without hiding availability or failure reasons

#### Scenario: Binding conflict or disabled binding
- **WHEN** the configured binding conflicts or is disabled
- **THEN** Jcode reports the state through existing keybinding feedback and does not steal unrelated input

## ADDED Requirements

### Requirement: Integrated question pane
The TUI SHALL render pending questions in place of the composer while preserving transcript, status, draft and cursor, with distinct focus/selection, indented descriptions and existing theme styles.

#### Scenario: Answer and restore draft
- **WHEN** a pending batch appears over a nonempty draft and the user reviews and submits answers
- **THEN** conversation context remains visible at normal sizes, submission is atomic, and the original draft and cursor return unchanged

#### Scenario: Narrow and long content
- **WHEN** question text, Unicode labels or descriptions exceed available space or the terminal resizes
- **THEN** the pane wraps and scrolls, preserves access to all choices, pins progress/controls when space permits, and never draws outside the viewport

### Requirement: Keyboard and focus continuity
The pane SHALL retain single/multi-select, Other editing, back, review and cancel behavior and allow explicit focus transfer to transcript scrolling without altering answers.

#### Scenario: Edit and cancel
- **WHEN** the user changes answers, edits Other, navigates backward and cancels
- **THEN** editing keys affect only the focused surface and cancellation restores the composer without fabricating an answer

#### Scenario: Tiny terminal
- **WHEN** the viewport cannot fit a heading, option and footer
- **THEN** a bounded compact hint is shown and restoring size recovers the full state

# TUI Link Opening

## Purpose

Define user-visible behavior for deliberately opening supported Markdown links from the interactive TUI, including local files addressed by `file:///` URLs.

## ADDED Requirements

### Requirement: Deliberate user clicks open rendered Markdown links

The interactive TUI MUST allow a user to open a supported Markdown link by clicking its rendered link label. The clicked label MUST resolve to the link destination authored in the Markdown, not to display text or a nearby URL. This behavior MUST work when the label is wrapped across terminal lines or appears in a scrolled message viewport.

#### Scenario: Click an HTTP or HTTPS link
- **WHEN** the user clicks the rendered label of a valid HTTP or HTTPS Markdown link
- **THEN** the TUI requests that the operating system open the link using the configured system opener

#### Scenario: Click a local file URL
- **WHEN** the user clicks the rendered label of a valid `file:///` Markdown link whose local target exists
- **THEN** the TUI opens the local file through the system opener and leaves the choice of application to the desktop's file association

#### Scenario: File type is selected by the desktop
- **WHEN** the clicked file URL points to a type such as HTML, Markdown, PNG, SVG, or JSON
- **THEN** the TUI passes the file target to the system opener without requiring an application-level extension allowlist

### Requirement: Unsafe or unsupported destinations are not launched

The TUI MUST reject unsupported schemes and malformed link targets rather than launching them. A `file:` URL MUST resolve to a local filesystem path; remote file authorities or invalid file URLs MUST NOT be treated as local files.

#### Scenario: Unsupported scheme
- **WHEN** a user clicks a Markdown link whose scheme is not supported by the TUI system-opening policy
- **THEN** the TUI does not launch an application for that target and reports that the target cannot be opened

#### Scenario: Invalid or missing local file
- **WHEN** a user clicks an invalid `file:` URL or a file URL whose local target does not exist
- **THEN** the TUI does not invoke the system opener and reports the resolution error

### Requirement: Explicit link opening preserves non-user-initiated suppression

Opening a link from a deliberate user click MUST NOT disable suppression for agent-initiated open-tool requests, automatic opens, or test-harness execution. Explicit `NO_BROWSER` and `JCODE_NO_BROWSER` opt-outs MUST remain effective for TUI link clicks.

#### Scenario: Agent open request remains suppressed in harness
- **WHEN** an agent invokes the generic open tool while running in a test harness
- **THEN** the request remains suppressed and no external application is launched

#### Scenario: Test execution remains side-effect free
- **WHEN** a test exercises link opening behavior without an explicit fake opener
- **THEN** it does not launch a real desktop application

#### Scenario: Explicit no-browser opt-out
- **WHEN** `NO_BROWSER` or `JCODE_NO_BROWSER` disables opening
- **THEN** a user click does not launch an external application and the TUI reports the suppressed action

### Requirement: Link opening does not break ordinary mouse behavior

Clicks outside a rendered link MUST retain their existing TUI behavior, including selection and navigation. A click on a supported link MUST not be mistaken for a selection drag or an input-pane click.

#### Scenario: Click ordinary message text
- **WHEN** the user clicks rendered message text that is not part of a link
- **THEN** the current message mouse behavior is preserved

#### Scenario: Link-open failure
- **WHEN** the system opener cannot be started or reports failure
- **THEN** the TUI presents an actionable error without terminating the session

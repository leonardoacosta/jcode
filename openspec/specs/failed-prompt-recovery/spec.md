# failed-prompt-recovery Specification

## Purpose
Keep user-authored prompts recoverable after interactive submission failures, with complete copyable text beside the error and no loss of newer drafts or existing privacy protections.

## Requirements

### Requirement: Lossless recovery across submission failures

For ordinary conversational input in local and daemon-connected TUI modes, Jcode MUST retain the complete logical user-authored prompt before destructive submission preparation. Recovery MUST include expanded pasted text and typed slash-invocation text, preserving Unicode, whitespace and newlines. Validation, preparation, dispatch, transport and provider failures MUST NOT leave only an error or an unusable paste placeholder. Existing input limits MUST remain enforced without discarding the rejected draft.

#### Scenario: Unknown or unavailable skill rejects a multiline prompt

- **GIVEN** a slash invocation with multiline pasted text names an unknown or unavailable skill
- **WHEN** submission rejects the invocation before a user turn is recorded
- **THEN** the complete invocation and pasted text remain recoverable beside the error
- **AND** no provider request is sent for that rejected submission

#### Scenario: Preparation or dispatch fails after paste expansion

- **GIVEN** a draft contains paste placeholders whose backing text is expanded during preparation
- **WHEN** preparation or local or remote dispatch fails
- **THEN** recovery contains the expanded original text even if transient paste storage was cleared
- **AND** recovery does not depend on persistent prompt history

#### Scenario: An input limit rejects the draft

- **WHEN** an existing input-size limit rejects a submitted draft
- **THEN** the input is not dispatched and its complete text remains available for copy and editing
- **AND** no recovery behavior bypasses the input limit

### Requirement: Full prompt block with lossless copying

A user-facing failure of an ordinary conversational submission MUST expose the associated prompt in a clearly labeled **Failed prompt** block beside the error. The block MUST make the entire prompt inspectable and provide a keyboard-accessible way to copy its authoritative text, not merely its visible viewport. Copying MUST exclude the error, label, presentation fences and UI wrapping. A truncated preview, status notice or history entry alone MUST NOT satisfy this requirement.

#### Scenario: Large input extends beyond the viewport or history limit

- **GIVEN** a failed prompt exceeds the viewport and the persistent-history recording limit
- **WHEN** the user inspects and copies its failed-prompt block
- **THEN** all prompt text is accessible and the clipboard payload exactly equals the original logical prompt
- **AND** it contains no UI ellipsis, synthetic line wraps or unresolved paste placeholders

#### Scenario: Clipboard copying fails

- **WHEN** the clipboard backend reports failure while copying a failed prompt
- **THEN** Jcode reports the copy failure without discarding the recovery block or editable draft
- **AND** the full text remains inspectable for another copy attempt or manual selection

### Requirement: Non-destructive editing of failed input

Jcode MUST restore a failed prompt into an empty composer and MUST NOT overwrite, append to or otherwise alter a newer draft. When a newer draft exists, the failed prompt MUST remain independently copyable and available for editing through a deliberate user action. Recovery MUST NOT silently remove still-available attachment state or imply that unavailable attachments will be resent.

#### Scenario: The composer is empty when a turn fails

- **WHEN** a conversational turn fails and the composer is empty
- **THEN** the complete failed prompt becomes editable in the composer
- **AND** the failed-prompt block remains available beside the error

#### Scenario: A newer draft exists when an earlier turn fails

- **GIVEN** the user has typed a new draft while an earlier submission is in flight
- **WHEN** that earlier submission fails
- **THEN** the new draft remains unchanged
- **AND** the earlier prompt remains available in its own recovery block without destructive replacement of the new draft

### Requirement: Submission identity and deliberate resubmission

Recovery MUST remain associated with the failed submission rather than whichever draft is currently visible. Repeated notifications for one failed attempt MUST NOT duplicate its recovery record or user turn. Showing, copying or editing recovery MUST NOT itself send another request or change existing automatic-retry policy. The error MUST distinguish a known pre-dispatch rejection from an uncertain transport outcome that could already have caused server-side work.

#### Scenario: Duplicate or delayed error events arrive

- **GIVEN** a failed submission has a recovery block and another draft or submission is newer
- **WHEN** a duplicate or delayed error for the failed attempt arrives
- **THEN** it refers to the original failed prompt, not the newer input
- **AND** no duplicate recovery record, successful user turn or extra provider submission is created by recovery

#### Scenario: A connection drops after dispatch

- **WHEN** the connection fails after a prompt could have reached the server
- **THEN** Jcode preserves and exposes that prompt while marking the delivery outcome uncertain
- **AND** it does not claim that no work occurred or automatically replay the prompt because recovery was displayed

### Requirement: Safe rendering and privacy boundaries

Failed-prompt recovery MUST treat user text as inert content, including embedded Markdown fences and terminal-control sequences. It MUST NOT inject skill instructions, system context or attachment bytes that were not part of the user's text. Inputs handled by existing login, account or SSH secret-interception flows MUST NOT enter a failed-prompt block, ordinary transcript, clipboard payload, history or additional logs through this feature. Existing history exclusions MUST remain intact; recovery MUST NOT introduce a persistent prompt journal.

#### Scenario: Prompt text contains rendering delimiters or terminal escapes

- **GIVEN** a failed prompt contains nested backtick fences, Markdown-looking commands, Unicode and terminal-control text
- **WHEN** Jcode renders and copies the recovery block
- **THEN** no embedded content executes or escapes the inert prompt presentation
- **AND** copying returns the original logical prompt without added rendering delimiters

#### Scenario: A secret-intercept input fails

- **GIVEN** a designated login, account or SSH input-interception flow is active
- **WHEN** its input is rejected or that flow errors
- **THEN** the entered value is absent from conversational recovery, transcript, copy targets and additional persistence
- **AND** the existing secret-flow recovery behavior remains responsible for that input

#### Scenario: Slash and oversized prompts remain excluded from persistent history

- **WHEN** an ordinary conversational submission excluded by existing history policy fails
- **THEN** it remains available through in-session failed-prompt recovery
- **AND** recovery does not add it to persistent history or create another durable prompt store

### Requirement: Successful submission behavior remains unchanged

Successful conversational and valid skill submissions MUST retain their existing dispatch, attachment and transcript behavior. Recovery MUST NOT introduce extra user turns, extra dispatches or failure blocks for successful submissions.

#### Scenario: Valid skill invocation succeeds with an image

- **WHEN** a valid skill invocation with trailing prompt text and an image is accepted and completes successfully
- **THEN** its user turn and attachment are submitted exactly once through the existing path
- **AND** no failed-prompt block appears for that successful attempt

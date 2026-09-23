## ADDED Requirements

### Requirement: Q1 Structured answer contract
The tool SHALL validate bounded questions and accept only server-validated human answers correlated by stable ids.

#### Scenario: Valid answer
- **WHEN** a user selects options and supplies Other text
- **THEN** the result preserves option ids and free text under each question id

#### Scenario: Invalid input
- **WHEN** ids collide, payload bounds are exceeded or a single-select answer has multiple options
- **THEN** the request is rejected without beginning or completing a question

### Requirement: Q2 TUI interaction
The TUI SHALL display question text, descriptions and progress and support keyboard single/multi-selection, Other editing, review, submit and Esc cancellation without modifying the chat draft.

#### Scenario: Answer a batch
- **WHEN** the user navigates four questions with keyboard and submits
- **THEN** one answered result returns all answers and restores the prior draft

#### Scenario: Cancel or small terminal
- **WHEN** the user cancels or uses a narrow terminal
- **THEN** no answer is fabricated; cancellation resolves once and wrapped content remains navigable

### Requirement: Q3 Session lifecycle
Pending questions SHALL belong to the session and tool call, remain available on live reconnect, and resolve at most once.

#### Scenario: Reconnect
- **WHEN** a client detaches and reconnects to the same live session
- **THEN** the question is replayed and its valid response resumes the original call

#### Scenario: Race or termination
- **WHEN** duplicate/wrong-session responses, concurrent question calls, turn cancel, session clear or daemon restart occur
- **THEN** duplicates do not resume twice, foreign responses fail, concurrent calls do not overwrite and terminated waits produce an explicit terminal outcome

### Requirement: Q4 Capability and provider boundaries
The question tool SHALL be available only for interactive root sessions with a capable client and SHALL maintain balanced tool history across native and direct provider execution.

#### Scenario: Unsupported caller
- **WHEN** an old client, worker or headless/direct caller invokes questions
- **THEN** the tool is hidden when possible and returns unavailable immediately if called

#### Scenario: Provider resume
- **WHEN** a human answers through either supported execution path
- **THEN** the next model step receives exactly one matching tool result

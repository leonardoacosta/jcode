## ADDED Requirements

### Requirement: One bounded pre-chat classification
Jcode SHALL classify eligible new user submissions into exactly one registered command candidate or chat. Explicit existing commands SHALL retain their current handling. Automatic messages, tool results and collected content MUST NOT become user-command authorization. Classification SHALL have bounded inputs/outputs and a configured deadline; the default proposed deadline is 300 ms. The router MUST NOT generate plans or arbitrary executables.

#### Scenario: Clear supported request
- **WHEN** a user requests one registered action with an unambiguous permitted target and classification meets the calibrated threshold
- **THEN** Jcode validates the command candidate without invoking the full chat model

#### Scenario: Ambiguous or multi-step request
- **WHEN** the request has multiple plausible targets, uncertain intent, or requires multiple actions
- **THEN** exactly the original submission falls through to chat and no command is dispatched

#### Scenario: Explicit command or untrusted content
- **WHEN** an existing explicit command is submitted or an automatic/tool message contains command-like text
- **THEN** existing explicit-command handling remains unchanged and the automatic/tool message is not routed as an authorized user command

### Requirement: Typed catalog-bound decisions
Classifier results SHALL use a versioned closed schema bound to a snapshot of registered action IDs and schemas. Unknown action IDs, fields, invalid arguments, arbitrary tool/executable names, multiple actions and unsupported versions MUST abstain before execution. Confidence SHALL select intent only, not grant permissions.

#### Scenario: Classifier invents a tool call
- **WHEN** a classifier returns an unknown action or shell/tool name outside the provided catalog
- **THEN** Jcode rejects the candidate and routes the original message to chat with no tool side effect

#### Scenario: Malformed result
- **WHEN** classification times out, fails, returns oversized output or violates its result schema
- **THEN** Jcode falls through to chat without retrying classification or dispatching a command

### Requirement: Trusted context and current target resolution
The router SHALL resolve relative references only from bounded trusted current-session context. It MUST revalidate the session/workspace, artifact identity/revision, catalog binding and supported surface immediately before dispatch. It MUST NOT read arbitrary files or transmit full history merely to resolve a target.

#### Scenario: Selected artifact is unique
- **WHEN** the user says “open the selected file” and exactly one current artifact supports the requested registered surface
- **THEN** its trusted identity resolves the command target

#### Scenario: Selection changes during classification
- **WHEN** the selection, workspace, artifact revision or catalog binding changes before dispatch
- **THEN** no stale-target action executes and the message falls through to chat

#### Scenario: Source file has no compatible sidebar surface
- **WHEN** the user requests source-code viewing but the current registered panel supports only Markdown/PDF
- **THEN** the router does not execute an incompatible panel action or claim success and instead falls through to chat

### Requirement: Safe fallback and privacy
Pre-dispatch abstention SHALL deliver the original user submission to chat once. After dispatch begins, errors/timeouts MUST NOT automatically replay the command through chat. Router telemetry SHALL exclude raw user content, tool arguments, file contents, credentials and MCP results.

#### Scenario: Executed tool times out
- **WHEN** tool dispatch has started and times out with uncertain side-effect completion
- **THEN** Jcode exposes that execution state and requires explicit continuation instead of sending the original request to chat as a fresh action

#### Scenario: Classification telemetry is collected
- **WHEN** the router records a decision
- **THEN** telemetry contains only bounded action/disposition/version/latency/error metadata, not request or tool payload content

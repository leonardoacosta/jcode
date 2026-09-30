## ADDED Requirements

### Requirement: Explicit native and MCP command catalog
Jcode SHALL register a bounded semantic-action catalog containing closed argument schemas, trusted target constraints, risk/authorization policy, cancellation/result handling and stable execution bindings. Catalog admission SHALL include interface, read-only and bounded local actions. MCP discovery and tool annotations alone MUST NOT authorize fast-path execution.

#### Scenario: Admitted native action
- **WHEN** a clear request selects a supported native action whose target and policy requirements pass
- **THEN** Jcode executes that registered binding rather than a classifier-generated tool or shell command

#### Scenario: MCP tool is connected but not admitted
- **WHEN** a connected MCP tool lacks an explicit reviewed command binding or risk classification
- **THEN** it is not directly executable by the command classifier and the request goes to chat

#### Scenario: MCP alias collides with a denied tool
- **WHEN** an MCP binding is exposed through an alias that collides with an existing denied tool identity
- **THEN** the alias cannot bypass the current deny/availability policy

### Requirement: Authorization-preserving execution
All routed actions SHALL preserve existing tool/session/workspace permission checks and confirmation policy. Classifier confidence MUST NOT bypass those checks. External/destructive actions MUST NOT become automatic defaults. When necessary confirmation cannot be presented, Jcode SHALL abstain before execution.

#### Scenario: Clear but restricted command
- **WHEN** the classifier recognizes a command that requires confirmation
- **THEN** the existing confirmation flow runs before any side effect, or the original submission falls through to chat if confirmation is unavailable

#### Scenario: Disabled action or tool
- **WHEN** a binding or underlying tool is disabled between classification and dispatch
- **THEN** Jcode executes nothing and follows the pre-dispatch fallback path

### Requirement: Bounded local execution recipes
Local mutations and test actions SHALL select registered executable/argument recipes with bounded workspace targets and execution limits. The classifier MUST NOT synthesize arbitrary shell text or permit target/scope escalation.

#### Scenario: Named test action
- **WHEN** a user requests an admitted named repository test target
- **THEN** Jcode runs its registered direct-argv recipe within the bound workspace and existing authorization limits

#### Scenario: Formatter request includes injection or escaping path
- **WHEN** formatter arguments contain a shell fragment, unknown recipe or path outside the permitted workspace
- **THEN** the candidate is rejected before any process or file change

### Requirement: Once-only dispatch and visible receipts
A routed action SHALL have a session/message-scoped invocation identity, execution claim and visible history receipt. Repeated delivery or reconnect of the same originating message MUST NOT execute the action again. A distinct repeated user message remains a separate request. Failures and cancellation SHALL remain visible and use existing tool semantics.

#### Scenario: Reconnect replays the submission
- **WHEN** the same message is delivered again after execution began
- **THEN** Jcode resumes or shows its existing receipt rather than dispatching a duplicate action

#### Scenario: User deliberately repeats a command
- **WHEN** a new distinct message requests the same action
- **THEN** it is evaluated as a new request under current target and policy checks

#### Scenario: Action fails or is canceled
- **WHEN** the dispatched tool fails or the user cancels it
- **THEN** the interface shows the action, target and final known status and does not silently relaunch it through chat

### Requirement: Feature-gated compatibility
The fast path SHALL be initially disabled and have a user-accessible enable/disable setting. Disabled routing SHALL preserve current explicit-command and chat behavior. Enabling SHALL require passing routing evaluation and dispatcher parity tests.

#### Scenario: Router disabled
- **WHEN** fast command routing is disabled
- **THEN** ordinary submissions follow the current chat path without classifier calls and explicit commands behave as before

#### Scenario: Rollback with prior executions
- **WHEN** the fast path is disabled after previous routed actions
- **THEN** existing receipts remain inspectable and no old action is replayed

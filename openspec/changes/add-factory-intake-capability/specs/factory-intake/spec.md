## ADDED Requirements

### Requirement: Provider-neutral intake envelope

Intake SHALL represent every inbound message as a provider-neutral envelope. Transport-specific identifiers SHALL NOT appear in the envelope, the intake store schema, or any interface consumed by the factory lifecycle.

#### Scenario: A message arrives from any transport

- **WHEN** an adapter delivers an inbound message
- **THEN** intake produces an envelope carrying sender identity, conversation identity, content, attachments, arrival time, and the originating adapter name
- **AND** every identifier in the envelope is assigned by intake, not by the transport
- **AND** the raw transport payload is retained separately for audit and backfill.

#### Scenario: A second transport is added

- **WHEN** a new adapter is introduced
- **THEN** no change is required to the envelope, the intake store schema, or the promotion rules.

### Requirement: Content-derived deduplication

Intake SHALL derive deduplication keys from message content and identity. Deduplication SHALL NOT depend on sequence numbers or delivery counters assigned by a transport.

#### Scenario: A transport replays a delivery

- **WHEN** the same message is delivered more than once
- **THEN** intake records it once and marks subsequent deliveries as duplicates

#### Scenario: A transport resets its sequence numbering

- **WHEN** a transport assigns non-monotonic or reused sequence identifiers after a period of inactivity
- **THEN** deduplication remains correct, because no key derives from those identifiers.

### Requirement: Durable record before interpretation

Intake SHALL persist every inbound message durably before interpreting it, classifying it, or acting on it.

#### Scenario: Interpretation fails

- **WHEN** intake cannot classify a message, or classification raises an error
- **THEN** the message remains recorded and inspectable
- **AND** the failure is recorded against it rather than discarding it.

#### Scenario: An operator inspects history

- **WHEN** an operator reviews intake history
- **THEN** every message ever received is present, including duplicates, throttled messages, unrecognized messages, and messages that never became work.

### Requirement: Credential scrubbing at ingress

Intake SHALL redact credential-shaped strings before writing any record. No other content SHALL be altered.

#### Scenario: A credential is pasted into a message

- **WHEN** inbound content matches a credential-shaped pattern
- **THEN** the matched span is replaced with a redaction marker before the record is written
- **AND** the redaction is recorded as an event the operator can see
- **AND** no unredacted copy is written to any store or log.

#### Scenario: Ordinary content arrives

- **WHEN** content contains no credential-shaped strings
- **THEN** it is stored verbatim.

### Requirement: Promotion is explicit and audited

Intake SHALL NOT create tracked work directly. Converting a recorded message into tracked work SHALL be an explicit transition that records who approved it and when.

#### Scenario: A message requests new work

- **WHEN** a recorded message is interpreted as a request for work
- **THEN** intake produces a proposal in an awaiting-approval state
- **AND** no initiative, task, or issue is created until the proposal is approved

#### Scenario: A proposal is approved

- **WHEN** an approver approves a proposal
- **THEN** tracked work is created and linked to the originating record
- **AND** the approving identity, the approval time, and the approval channel are recorded.

#### Scenario: A message is read-only

- **WHEN** a recorded message is interpreted as a request for research or status only
- **THEN** intake may answer without approval
- **AND** the response performs no mutation of repository, initiative, or configuration state.

### Requirement: Intake authority is bounded

The intake store SHALL be the authority for inbound message history only. Conversation history SHALL NOT be treated as authoritative for task state, approval state, or work state.

#### Scenario: Conversation history is unavailable

- **WHEN** conversation history is deleted, edited, or unreachable
- **THEN** intake records, approvals, and tracked work remain complete and correct.

#### Scenario: An approval is issued

- **WHEN** an approval is recorded
- **THEN** its authority derives from the recorded approver identity
- **AND** not from the location, thread, or conversation in which it was expressed.

### Requirement: Execution admission control

Intake SHALL bound the rate at which recorded messages cause execution. Admission control SHALL NOT reduce what is recorded.

#### Scenario: Messages arrive faster than the configured execution budget

- **WHEN** inbound volume exceeds the execution budget
- **THEN** every message is still recorded in full
- **AND** execution is deferred or declined for messages beyond the budget
- **AND** the deferral is recorded against each affected message.

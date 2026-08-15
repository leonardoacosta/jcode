## ADDED Requirements

### Requirement: Telegram update mapping

The adapter SHALL map Telegram `Update` objects into provider-neutral intake envelopes. Telegram field names SHALL NOT propagate beyond the adapter.

#### Scenario: A supported update arrives

- **WHEN** an `Update` containing a message is received
- **THEN** the adapter maps sender, chat, content, and attachments into envelope fields
- **AND** retains the full raw `Update` for audit
- **AND** emits no Telegram-specific identifier into the envelope.

#### Scenario: An unsupported update type arrives

- **WHEN** an `Update` carries a variant the adapter does not handle
- **THEN** the raw payload is still recorded
- **AND** the update is marked unhandled rather than dropped
- **AND** the unhandled variant name is recorded so coverage gaps are visible.

### Requirement: Delivery identifiers are not trusted for deduplication

The adapter SHALL NOT present `update_id` as a deduplication key to intake.

#### Scenario: Telegram randomizes update_id after inactivity

- **WHEN** Telegram assigns a randomized `update_id` sequence following a period of bot inactivity
- **THEN** deduplication behavior is unaffected, because intake derives keys from content and identity.

### Requirement: Group activation requires explicit address

In group chats, the adapter SHALL only forward messages that explicitly address the bot.

#### Scenario: A group message mentions the bot

- **WHEN** a group message mentions the bot or replies to one of its messages
- **THEN** the adapter forwards it to intake

#### Scenario: A group message does not mention the bot

- **WHEN** a group message neither mentions the bot nor replies to it
- **THEN** the adapter does not forward it
- **AND** no intake record is created.

#### Scenario: A direct message arrives

- **WHEN** a message arrives in a direct conversation
- **THEN** it is forwarded without requiring an explicit mention.

### Requirement: Sender authorization

The adapter SHALL forward messages only from senders mapped to a known operator identity.

#### Scenario: An unmapped sender messages the bot

- **WHEN** a sender with no mapped operator identity sends a message
- **THEN** the message is recorded as unauthorized
- **AND** it is not promoted, executed, or answered with any repository content.

#### Scenario: A mapped sender messages the bot

- **WHEN** a sender mapped to an operator identity sends a message
- **THEN** the message is forwarded to intake carrying that operator identity.

### Requirement: Outbound delivery and redaction notice

The adapter SHALL deliver intake responses back to the originating conversation, and SHALL surface redaction events to the operator.

#### Scenario: Intake produces a response

- **WHEN** intake produces a response for a recorded message
- **THEN** the adapter delivers it to the conversation the message came from

#### Scenario: A credential was redacted at ingress

- **WHEN** intake redacted credential-shaped content from a message
- **THEN** the adapter notifies the operator in that conversation that redaction occurred
- **AND** the notification does not restate the redacted value.

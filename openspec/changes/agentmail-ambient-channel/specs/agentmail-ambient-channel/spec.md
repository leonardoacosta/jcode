## Purpose

This capability lets Jcode users optionally send Ambient notifications through an AgentMail inbox and route valid replies back as directives without replacing existing notification channels.

## ADDED Requirements

### Requirement: Optional AgentMail notification delivery
When AgentMail is configured and enabled, Jcode SHALL send eligible Ambient notifications using the selected AgentMail inbox. When AgentMail is not enabled or configured, existing notification behavior SHALL remain unchanged.

#### Scenario: AgentMail notification enabled
- **WHEN** an Ambient notification is dispatched and AgentMail delivery is enabled with valid configuration
- **THEN** Jcode sends the notification through the configured AgentMail inbox

#### Scenario: AgentMail notification disabled
- **WHEN** AgentMail delivery is disabled or lacks required configuration
- **THEN** Jcode does not attempt AgentMail delivery and continues other enabled notification channels

#### Scenario: AgentMail service unavailable
- **WHEN** an AgentMail send request fails
- **THEN** Jcode records a sanitized delivery failure and other notification channels continue

### Requirement: Secure AgentMail credential configuration
Jcode SHALL load AgentMail credentials from secret-safe configuration and SHALL NOT expose credential values in user-visible status, logs, or ordinary config display output.

#### Scenario: Credential configured
- **WHEN** the user configures an AgentMail API key through the supported secret source
- **THEN** AgentMail operations can authenticate without writing the secret into project files or normal status output

#### Scenario: Credential unavailable
- **WHEN** AgentMail is enabled but no credential is available
- **THEN** Jcode reports a non-secret setup error and does not attempt an unauthenticated request

### Requirement: Authenticated reply directives
When reply handling is enabled, Jcode SHALL accept an inbound AgentMail message as an Ambient directive only when it belongs to the configured inbox, is from an authorized sender, and can be associated with an eligible Jcode notification or configured directive policy.

#### Scenario: Authorized reply to Ambient notification
- **WHEN** a message arrives in the configured inbox from an authorized sender and references a recognized Ambient notification
- **THEN** Jcode records its text once as a pending directive for Ambient

#### Scenario: Unauthorized sender or unrelated message
- **WHEN** a message arrives from an unapproved sender, for another inbox, or without the required notification association
- **THEN** Jcode ignores it as a directive and records only a sanitized rejection reason

#### Scenario: Duplicate event delivery
- **WHEN** AgentMail delivers the same inbound event more than once
- **THEN** Jcode processes it at most once using the provider event or message identifier

#### Scenario: Reply handling disabled
- **WHEN** AgentMail reply handling is disabled
- **THEN** inbound messages do not create Ambient directives

### Requirement: Private outbound event connection
Jcode SHALL receive AgentMail reply events using an outbound-initiated connection or another private transport that does not require exposing a public inbound listener by default.

#### Scenario: Private event connection
- **WHEN** reply handling is enabled on a host without a publicly reachable webhook URL
- **THEN** Jcode can receive eligible reply events over an outbound connection

#### Scenario: Connection interruption
- **WHEN** the event connection is interrupted
- **THEN** Jcode reconnects with bounded backoff and resumes listening without duplicating already processed directives

### Requirement: Existing notification channels remain independent
Adding AgentMail SHALL NOT change existing SMTP/IMAP, ntfy, Telegram, or Discord delivery or reply behavior.

#### Scenario: Existing channels remain configured
- **WHEN** AgentMail is enabled alongside an existing notification channel
- **THEN** each enabled channel retains its existing configuration and behavior

#### Scenario: AgentMail not configured
- **WHEN** a user runs Jcode without AgentMail configuration
- **THEN** startup and Ambient cycles do not require AgentMail dependencies or credentials

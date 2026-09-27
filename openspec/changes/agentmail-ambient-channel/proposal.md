## Why

Jcode's Ambient notifications currently use SMTP for email and IMAP polling for replies, while AgentMail offers an agent-oriented inbox API and persistent event WebSockets that need no public inbound endpoint. Supporting it as an optional channel would simplify agent email setup while preserving existing notification choices and keeping reply directives authenticated and scoped.

## What Changes

- Add optional AgentMail outbound email notifications for Ambient cycles.
- Add opt-in inbound reply handling that turns eligible replies into Ambient directives, with sender/inbox validation, deduplication, and safe handling of unrelated or untrusted messages.
- Store API credentials outside the project TOML and expose only non-secret connection status.
- Preserve current SMTP/IMAP, ntfy, Telegram, and Discord behavior; AgentMail remains disabled unless configured.
- Prefer AgentMail's outbound WebSocket event stream for replies so Jcode needs no public webhook listener.

## Capabilities

### New Capabilities

- `agentmail-ambient-channel`: Optional AgentMail delivery of Ambient notifications and authenticated reply-to-directive handling.

### Modified Capabilities

None.

## Impact

The change affects Ambient notification dispatch and lifecycle management, safety configuration/secret storage, inbound directive validation, and tests. It adds an outbound connection to AgentMail's API/WebSocket service. Existing channel adapters and their configuration remain unchanged. AgentMail inbox creation, account signup, and external email sending require a separate user-directed setup and are not part of automatic enablement.

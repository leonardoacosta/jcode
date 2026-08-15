# Telegram and Slack Inbox Triage for Jcode

## Scope

Research and implementation blueprint for receiving Telegram and Slack messages, triaging them, and routing approved work through Jcode ambient mode.

## Current repository evidence

- `ambient/queue.json` currently contains 2 scheduled items.
- No Telegram or Slack inbox entries are present.
- No tracked Telegram, Slack, webhook, or inbox adapter implementation exists.
- End-to-end provider acceptance is therefore blocked until adapters and test credentials are added.

## Provider configuration

### Telegram

1. Create a bot with BotFather.
2. Store `TELEGRAM_BOT_TOKEN` in a secret manager.
3. Expose an HTTPS webhook endpoint.
4. Configure `setWebhook` with a random `secret_token`.
5. Verify `X-Telegram-Bot-Api-Secret-Token` on every request.
6. Begin with one allowlisted private chat. Require commands or replies in groups.

Suggested environment:

```text
TELEGRAM_BOT_TOKEN=secret
TELEGRAM_WEBHOOK_SECRET=secret
TELEGRAM_ALLOWED_CHAT_IDS=chat-id-1
TELEGRAM_ALLOWED_USER_IDS=user-id-1
```

### Slack

1. Create a Slack app with a bot user.
2. Enable Event Subscriptions.
3. Prefer Socket Mode for a local or firewalled Jcode installation.
4. Generate an app-level token with `connections:write`.
5. Install using OAuth v2 and store workspace installation tokens securely.
6. Start with `app_mention` and `message.im`; add channel history only for explicitly approved channels.
7. Acknowledge every Socket Mode envelope or HTTP event promptly.

Suggested environment:

```text
SLACK_BOT_TOKEN=<xoxb-token>
SLACK_APP_TOKEN=<xapp-token>
SLACK_SIGNING_SECRET=<signing-secret>
SLACK_CLIENT_ID=<client-id>
SLACK_CLIENT_SECRET=<client-secret>
SLACK_ALLOWED_WORKSPACE_IDS=T123
SLACK_ALLOWED_CHANNEL_IDS=C123
```

## Normalized inbox envelope

```json
{
  "source": "telegram|slack",
  "source_message_id": "provider-specific-id",
  "conversation_id": "provider-conversation-id",
  "thread_id": "optional-thread-id",
  "sender_id": "provider-user-id",
  "text": "message text",
  "attachments": [],
  "received_at": "RFC3339",
  "dedupe_key": "stable-provider-event-key"
}
```

## Triage lifecycle

```text
received -> verified -> triaged -> queued -> running -> completed
                                      |-> waiting_for_approval
                                      |-> failed
                                      |-> ignored
```

Triage classes:

- `command`
- `task`
- `question`
- `incident`
- `approval`
- `notification`
- `needs_clarification`
- `noise`

Mutating actions require explicit approval. Read-only diagnostics, plans, status, and explanations may be automated.

## Acceptance matrix

| Boundary | Expected behavior | Current evidence | Status |
|---|---|---|---|
| Telegram Bot API | Invalid token rejected | Real request returned HTTP 404 | Verified |
| Slack Web API | Invalid token rejected | Real request returned `invalid_auth` | Verified |
| Official docs | Platform setup references reachable | Telegram and three Slack docs returned HTTP 200 | Verified |
| Ambient queue | Existing queue remains readable | 2 items parsed successfully | Verified |
| Telegram inbound event | Message normalized and queued | No adapter or credentials | Blocked |
| Slack inbound event | Event acknowledged and queued | No adapter or credentials | Blocked |
| Approval reply | Original chat/thread updated | No adapter or credentials | Blocked |
| Deduplication/retry | Duplicate events do not create tasks | No inbox implementation | Blocked |

## Extensibility constraints

Before this research is codified into a proposal, apply the factory-intake constraints in [Inbox extensibility for the software factory pattern](inbox-factory-extensibility.md). They require the inbox to be built as a provider-neutral intent source and approval surface rather than a chat integration.

## Recommended implementation order

1. Platform-neutral inbox record and lifecycle.
2. Telegram private-chat adapter with webhook verification.
3. Slack Socket Mode adapter with event acknowledgement.
4. Shared authorization, deduplication, triage, and approval policy.
5. Thread-aware reply adapters.
6. Integration tests using provider test credentials and recorded redacted payloads.
7. Production hardening: rate limits, dead-letter queue, secret rotation, metrics, and replay tooling.

## Sources

- https://core.telegram.org/bots/api
- https://api.slack.com/apis/connections/events-api
- https://api.slack.com/apis/connections/socket
- https://api.slack.com/authentication/oauth-v2

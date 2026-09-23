## 1. Configuration and API adapter

- [x] 1.1 Add optional AgentMail enable flags, inbox identity, and secret-source configuration; verify defaults preserve the current SMTP/IMAP and other channel behavior and add config parsing tests.
- [x] 1.2 Implement an AgentMail HTTP client adapter for Ambient notification sends; verify request construction, successful responses, provider errors, and timeout handling with mocked HTTP tests.
- [ ] 1.3 Wire AgentMail into notification dispatch without blocking or suppressing other channels; verify a failed AgentMail send leaves ntfy/desktop/Discord dispatch unaffected.
- [x] 1.4 Ensure API keys are redacted from config display, logs, error messages, and status output; verify with secret-redaction tests.

## 2. Authenticated inbound replies

- [ ] 2.1 Implement an outbound AgentMail WebSocket listener with subscription confirmation and bounded reconnect; verify connection loss/recovery and shutdown behavior using a fake server.
- [ ] 2.2 Add opaque notification correlation and configured sender allowlist checks; verify accepted replies require correct inbox, authorized sender, and recognized correlation. (Partial: parser gates for inbox, sender, subject correlation, and provider IDs are tested; persisted transcript correlation acceptance is not.)
- [ ] 2.3 Persist processed event/message identifiers and route accepted reply text into the existing Ambient directive queue; verify duplicate events and process restart do not duplicate directives. (Partial: deterministic directive ID and duplicate insertion are tested against persisted storage; event redelivery/restart integration is not.)
- [ ] 2.4 Verify disabled reply mode, unrelated messages, unauthorized senders, and malformed events never enqueue directives. (Partial: no-allowlist early return, unrelated/unauthorized sender, inbox mismatch, absent correlation, and non-received event rejection are tested; malformed-frame/reply-disabled full listener behavior is not.)

## 3. Integrated behavior and operations

- [ ] 3.1 Add an integration test covering AgentMail send, event receive, authorization, directive enqueue, and Ambient consumption while existing channels remain enabled.
- [ ] 3.2 Add non-secret status/log reporting for disabled, misconfigured, connected, and degraded AgentMail states; verify messages contain no API key or email body content.
- [ ] 3.3 Document human-controlled AgentMail account/inbox verification, secret setup, opt-in send/reply enablement, and rollback; verify documented keys and commands match implementation.
- [ ] 3.4 Run focused crate tests and OpenSpec validation; verify all AgentMail scenarios have coverage and `openspec validate agentmail-ambient-channel` passes.

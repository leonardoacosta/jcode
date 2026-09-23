# AgentMail Ambient channel

AgentMail support is optional. Outbound delivery reuses `safety.email_to` as the recipient and requires `safety.agentmail_enabled`, `safety.agentmail_inbox_id`, and an API key. It sends the private Ambient summary/transcript to that configured recipient. Replies require `safety.agentmail_reply_enabled`, the same inbox/key, and an explicit `agentmail_allowed_senders` list. Jcode listens over an outbound WebSocket and accepts only replies with a known Ambient session marker (`[jcode:<session-id>]`). Configure the inbox and allowlist in `config.toml`, and provide the API key through `JCODE_AGENTMAIL_API_KEY` or the private environment used to launch Jcode. Never place a live key in project configuration.

Both features are disabled by default. Verify the inbox and authorized sender through your human-controlled AgentMail account before enabling replies. Start with outbound-only delivery. Enable replies only after reviewing the sender allowlist and correlation requirements. To roll back, set both enable flags to false and remove or revoke the API key. Existing SMTP/IMAP and other notification channels remain independent.

Jcode does not create an AgentMail account or inbox, verify ownership, or send test mail during startup. Status output reports only disabled, misconfigured, connected, or degraded state, never credentials or message body text.

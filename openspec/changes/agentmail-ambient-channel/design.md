## Context

See `proposal.md` for motivation and `specs/agentmail-ambient-channel/spec.md` for the observable contract. Jcode currently sends Ambient email through SMTP and receives replies through a separate IMAP polling loop. AgentMail supports HTTP send APIs and an outbound WebSocket event stream, so reply support does not need a new public listener. Jcode's MCP client currently supports stdio servers only, so an AgentMail MCP bridge is not a drop-in runtime integration.

## Goals / Non-Goals

**Goals:**
- Add an optional direct AgentMail adapter to the existing Ambient notification and directive lifecycle.
- Keep API credentials out of `config.toml`, logs, and status output.
- Use a private outbound event connection for replies, with bounded reconnect and deduplication.
- Preserve the independence and default-disabled state of all existing channels.

**Non-Goals:**
- Do not expose a public webhook endpoint or change Tailscale/Portless routing.
- Do not create AgentMail accounts/inboxes, verify human email ownership, or send external email during startup.
- Do not make AgentMail the general-purpose Jcode email provider or replace SMTP/IMAP.
- Do not use the AgentMail MCP server as a runtime dependency; Jcode's MCP transport support does not currently accept the HTTP MCP endpoint described by AgentMail.

## Decisions

1. **Build a first-party channel adapter over AgentMail APIs, not MCP.** The existing `NotificationService` and channel abstraction are the integration seams. Direct API access supports the daemon lifecycle and works with the existing Rust service. The official MCP bridge is useful for interactive agent actions but would introduce a subprocess/MCP dependency and is not the current fit for unattended notification dispatch.

2. **Use the AgentMail outbound WebSocket stream for inbound reply events.** AgentMail documents that this requires no public URL and is outbound-only. Webhooks are rejected for the first version because they require a reachable callback and would expand network exposure. The WebSocket client must reconnect with bounded backoff and must not block other Ambient services.

3. **Make send and reply modes independently configurable and opt-in.** API key and inbox identity are separate from enable flags. Missing or invalid configuration disables only AgentMail and surfaces a sanitized setup warning; existing channels continue. Secrets belong in the existing private credential/env mechanism, never TOML or ordinary diagnostics.

4. **Use notification correlation plus an explicit sender allowlist for replies.** Include an opaque Jcode correlation identifier in notification metadata or subject, map it to the originating Ambient cycle, and accept directives only from configured sender identities. Do not trust message body text as an authorization signal. Persist processed provider message/event IDs to make WebSocket redelivery idempotent.

5. **Feed accepted replies into the existing Ambient directive queue.** Reuse the pending-directive mechanism used by email/message channels rather than invoking an agent directly from the WebSocket callback. This preserves current priority, safety, and cycle ownership rules.

6. **Keep failures isolated and observable without leaking message contents.** Send failures and connection state use sanitized logs/status. A transient AgentMail failure must not fail Ambient cycle completion or prevent ntfy/desktop/Discord delivery.

## Risks / Trade-offs

- [AgentMail API or WebSocket protocol changes] → isolate provider-specific transport behind an adapter and test event parsing with recorded sanitized fixtures.
- [Duplicate or replayed replies] → persist message/event IDs and correlation records; test redelivery and restart cases.
- [Forged or unsolicited email becoming an instruction] → require inbox match, sender allowlist, and recognized correlation; treat all email body content as untrusted instructions subject to normal Jcode safeguards.
- [Credential loss or setup complexity] → document environment/private-secret configuration and provide explicit non-secret readiness status.
- [WebSocket outage delays replies] → bounded reconnect, maintain other notification paths, expose connected/degraded state, and avoid unbounded queues.
- [AgentMail account verification/limits] → setup is a separate user-controlled prerequisite; test adapter behavior with fakes before live account verification.

## Migration Plan

1. Ship with AgentMail send and reply disabled unless explicitly configured.
2. Configure an API key and inbox through the supported secret source; first verify outbound notifications without reply ingestion.
3. Add an authorized sender and enable reply handling only after confirming the intended inbox and authorization policy.
4. Roll back by disabling the two AgentMail flags and removing/revoking the AgentMail key. Existing SMTP/IMAP and other channels remain untouched.

## Open Questions

None that change the chosen behavior contract. Exact configuration key names and API client implementation are implementation details to settle during apply while preserving these requirements.

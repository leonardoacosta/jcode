## 1. Intake core

- [x] 1.1 Define the provider-neutral envelope, with intake-assigned identifiers and an adapter-name field.
- [x] 1.2 Implement content-and-identity dedupe key derivation; add a test asserting stability when a simulated transport reuses or randomizes its sequence identifiers.
- [x] 1.3 Implement the intake store: durable append, dedupe lookup, pending-approval query, raw payload retention.
- [x] 1.4 Implement ingress credential scrubbing with a redaction event; test that no unredacted copy reaches any store or log.
- [x] 1.5 Implement record-before-interpret ordering; test that a classification failure leaves an inspectable record.

## 2. Promotion and approval

- [x] 2.1 Implement classification into: work request, research request, status request, unrecognized.
- [x] 2.2 Implement proposal creation in awaiting-approval state; test that no tracked work is created before approval.
- [x] 2.3 Implement the approval transition recording approver identity, time, and channel.
- [x] 2.4 Implement the two read-only paths; test that neither mutates repository, initiative, or configuration state.
- [x] 2.5 Implement execution admission control; test that throttled messages are still recorded in full.

## 3. Identity

- [x] 3.1 Implement a single-entry sender allowlist in configuration. Single-operator: no mapping table.
- [x] 3.2 Record the sender identifier on unauthorized attempts, so the operator can self-configure from the first message.
- [x] 3.3 Test that non-allowlisted senders are recorded as unauthorized and answered with no repository content.

## 4. Telegram adapter

- [x] 4.1 Implement update mapping into envelopes, retaining raw payloads.
- [x] 4.2 Implement unhandled-variant recording, including the variant name.
- [x] 4.3 Implement group mention/reply gating and direct-message pass-through.
- [x] 4.4 Implement outbound delivery and redaction notices.
- [x] 4.5 Add the bot credential to the existing secret-handling path (`TELEGRAM_BOT_TOKEN` / hardened `telegram.env` via `jcode-provider-env`); verify synthetic credentials never appear in stored records, `Debug`, public errors, or provider error bodies.

## 5. Acceptance

- [x] 5.1 `openspec validate add-factory-intake-capability --strict` passes.
- [x] 5.2 `scripts/check-intake-boundary.py` reports clean for this change.
- [ ] 5.3 End-to-end: a real message from a mapped sender produces a durable record and a proposal, and no tracked work until approval.
- [ ] 5.4 End-to-end: approving from chat creates tracked work linked to the originating record.
- [x] 5.5 Neutrality conformance: a second adapter (`jcode-intake-slack`, structurally unlike Telegram) compiles and passes against a byte-identical core (md5 `e2f7e8e4c7ea6e81b3bb5fa69232cce6` before and after).
- [ ] 5.6 Delete the conversation used in 5.3 and confirm records, approvals, and tracked work remain complete.

## 6. Command Center Decision Inbox

- [x] 6.1 Project provider-neutral Telegram and Slack records from the durable SQLite intake store without exposing provider credentials or raw payloads.
- [x] 6.2 Preserve source adapter, sender identity, conversation, category, approval state, dedupe evidence, retry evidence, and retained-payload status in the read model.
- [x] 6.3 Expose the read model through an authenticated, read-only Command Center endpoint that fails closed when the store cannot be read.
- [x] 6.4 Render the Decision Inbox in SolidStart with provider provenance, category, approval state, duplicate evidence, an empty state, and a responsive single-column layout.
- [x] 6.5 Verify Telegram and Slack normalization, restart persistence, deduplication, reconnect redelivery, credential redaction, authenticated HTTP projection, frontend unit/type/build gates, deterministic Playwright content acceptance, and the isolated managed-host launcher.
- [ ] 6.6 Credential-gated production acceptance: ingest one real Telegram message and one real Slack Socket Mode message into the configured durable store, then observe both through the managed Command Center host.
  - 2026-08-17 Telegram evidence: the configured bot ingested a user-originated message into the durable SQLite store, preserving Telegram provenance and content. The authenticated managed-host endpoint returned both records, and credential-gated Playwright rendered the acceptance message in repository-local and Orca-unavailable projects. Telegram transport, persistence, API, and browser projection are accepted; Slack Socket Mode remains outstanding, so 6.6 stays open.

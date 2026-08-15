## 1. Intake core

- [ ] 1.1 Define the provider-neutral envelope, with intake-assigned identifiers and an adapter-name field.
- [ ] 1.2 Implement content-and-identity dedupe key derivation; add a test asserting stability when a simulated transport reuses or randomizes its sequence identifiers.
- [ ] 1.3 Implement the intake store: durable append, dedupe lookup, pending-approval query, raw payload retention.
- [ ] 1.4 Implement ingress credential scrubbing with a redaction event; test that no unredacted copy reaches any store or log.
- [ ] 1.5 Implement record-before-interpret ordering; test that a classification failure leaves an inspectable record.

## 2. Promotion and approval

- [ ] 2.1 Implement classification into: work request, research request, status request, unrecognized.
- [ ] 2.2 Implement proposal creation in awaiting-approval state; test that no tracked work is created before approval.
- [ ] 2.3 Implement the approval transition recording approver identity, time, and channel.
- [ ] 2.4 Implement the two read-only paths; test that neither mutates repository, initiative, or configuration state.
- [ ] 2.5 Implement execution admission control; test that throttled messages are still recorded in full.

## 3. Identity

- [ ] 3.1 Implement a single-entry sender allowlist in configuration. Single-operator: no mapping table.
- [ ] 3.2 Record the sender identifier on unauthorized attempts, so the operator can self-configure from the first message.
- [ ] 3.3 Test that non-allowlisted senders are recorded as unauthorized and answered with no repository content.

## 4. Telegram adapter

- [ ] 4.1 Implement update mapping into envelopes, retaining raw payloads.
- [ ] 4.2 Implement unhandled-variant recording, including the variant name.
- [ ] 4.3 Implement group mention/reply gating and direct-message pass-through.
- [ ] 4.4 Implement outbound delivery and redaction notices.
- [ ] 4.5 Add the bot credential to the existing secret-handling path; verify it never appears in stored records or logs.

## 5. Acceptance

- [ ] 5.1 `openspec validate add-factory-intake-capability --strict` passes.
- [ ] 5.2 `scripts/check-intake-boundary.py` reports clean for this change.
- [ ] 5.3 End-to-end: a real message from a mapped sender produces a durable record and a proposal, and no tracked work until approval.
- [ ] 5.4 End-to-end: approving from chat creates tracked work linked to the originating record.
- [ ] 5.5 Neutrality conformance: draft a second adapter spec and confirm the `factory-intake` spec requires no edit. This is the real test of the design, and it is expected to be run before the first adapter is considered done.
- [ ] 5.6 Delete the conversation used in 5.3 and confirm records, approvals, and tracked work remain complete.

## 1. Prerequisite and contract admission

- [ ] 1.1 Confirm proposal approval and completion of `unified-jev-service-config`. Its owner must resolve missing execution metadata. Run dependency validation. Do not start implementation while either approval or prerequisite readiness is missing.
- [ ] 1.2 Confirm current TUI, protocol, server-dispatch, and resolver paths. Add failing tests for the typed result contract, strict distribution validation, uncertainty boundaries, and cache hit/miss parity. Scope any service extraction to shared behavior preservation.

## 2. Assessment service (depends on 1)

- [ ] 2.1 Implement an allowlisted, redacted, UTF-8-safe 8 KiB evidence builder. Test credential sentinels, excluded raw fields, oversized required context, and evidence freshness.
- [ ] 2.2 Implement asynchronous assessment through the shared resolver with no retries, a two-second total deadline, per-session 20-request budget, revision deduplication, and typed abstention. Test both provider routes, absent credentials, malformed responses, deadline, cancellation, and hit/miss parity.
- [ ] 2.3 Add consent-generation request/result identity and a single per-session daemon assessment lease. Test two attached clients, lease revocation, disconnect, stale results, and budget preservation across toggles. Local TUI must reuse the service without remote transport.

## 3. TUI integration (depends on 2)

- [ ] 3.1 Add command parsing, help/disclosure, default-off state, and read-only status. Test invalid commands, no credential leakage, no persisted opt-in, and missing-credential status.
- [ ] 3.2 Hook eligible turn-end snapshots without awaiting or changing the existing scheduler. Add queue/todo parity assertions for every result label, failure and timeout, plus known-wait and overnight exclusions.
- [ ] 3.3 Invalidate pending assessments on new input, `/poke off`, shadow off, cancel, session switch, and disconnect. Test reconnect/reload disabled state and removal of previous result.

## 4. Public acceptance and documentation (depends on 3)

- [ ] 4.1 Add labeled fixture replay through the typed service boundary. Report label confusion and abstentions for fixtures without claiming hosted model accuracy. Document how separately consented hosted replay would measure unnecessary nudges, missed continuations/blockers, latency, and cost before any future intervention proposal.
- [ ] 4.2 Run focused config, protocol, service, command and existing poke regression tests. Record exact commands/results and requirement-to-test mapping in verification.md. Failures must be fixed before completion claims.
- [ ] 4.3 Build the TUI through selfdev and exercise the public commands in a tester using an isolated socket and the changed binary. Verify default-off sends no calls, opt-in shows provider and result, a delayed test provider cannot delay a continuation, shadow off rejects a late result, and two clients do not double-launch. Record binary identity and tester frames. Do not repoint the shared daemon merely to test.
- [ ] 4.4 Document commands, data disclosure, budget/deadline, abstention semantics, native rather than MCP transport, and opt-in hosted evaluation requirements. Run strict artifact validation and dependency checks again. Request separate approval for any future active intervention.

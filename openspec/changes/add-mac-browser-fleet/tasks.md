# Tasks

## 1. Protocol and policy foundations

- [x] 1.1 Define versioned bounded fleet protocol types for authentication, inventory, target generations, capabilities, actions, approvals, leases, errors, and audit metadata.
  - touches: new shared fleet protocol module or crate and deterministic protocol tests
  - Done when malformed, oversized, unauthenticated, unsupported-version, duplicate-ID, and stale-generation messages fail closed with secret-safe errors.
- [x] 1.2 Implement the Mac-owned policy engine with action classification, metadata redaction, immutable hard denies, emergency stop, one-action approvals, and scoped monotonic-time leases.
  - depends on: 1.1
  - Done when table-driven tests cover every read-only, mutation, hard-deny, lease-match, expiry, restart, policy-reload, target-change, and emergency-stop branch.

## 2. Mac broker and browser adapters

- [ ] 2.1 Add the Mac browser fleet broker with a mode-0600 Unix socket, authenticated handshake, bounded concurrency, request deadlines, idempotent read-only handling, non-replayed mutations, inventory generations, and local audit events.
  - depends on: 1.1, 1.2
  - Done when fake peers prove connection, disconnect, reconnect, backpressure, timeout, cancellation, duplicate request, and target churn behavior.
- [x] 2.2 Add Manifest V3 extension and native-messaging host assets shared by Chrome and Edge, with minimal permissions, explicit host grants, browser/window/tab inventory, capability reporting, and ordinary-tab actions.
  - depends on: 2.1
  - Done when extension tests and a fake native host prove Chrome/Edge identity, inventory deltas, policy-filtered metadata, disconnect cleanup, and supported/unsupported actions.
- [x] 2.3 Add explicitly managed CDP target discovery and control without relaunching or attaching to ordinary daily profiles.
  - depends on: 2.1
  - Done when fake CDP tests prove endpoint trust, capability advertisement, target generation changes, richer inspection, bounded output, and policy enforcement.
- [ ] 2.5 Complete the ordinary-profile native bridge with Chromium stdio framing, broker-socket forwarding, initial and event-driven inventory synchronization, disconnect cleanup, approved action routing, and separate Chrome/Edge extension IDs.
  - depends on: 2.1, 2.2
  - Done when native-host framing and reconnect tests pass, extension tests prove initial snapshots and bounded deltas, setup renders independent browser allowlists, and real Chrome ordinary-profile tabs appear through Jcode with a non-CDP browser reference.
- [ ] 2.4 Add the Mac-local approval and status surface, including single-action approval, scoped lease issuance, lease listing/revocation, emergency stop, connection health, and browser extension state.
  - depends on: 1.2, 2.1
  - Done when the homelab protocol cannot invoke authority-only operations and local UI tests prove every decision reaches the broker policy engine.

## 3. SSH transport and Jcode routing

- [ ] 3.1 Extend the existing Mac-to-homelab SSH setup with a reverse stream-local forward for the broker, safe socket permissions, stale-socket recovery, keepalive behavior, and status diagnostics.
  - depends on: 2.1
  - Done when a transport integration test proves no TCP listener exists, the forwarded socket authenticates, reconnect restores read-only inventory, and mutations are not replayed.
- [ ] 3.2 Add an explicit Mac fleet browser provider to Jcode with browser/window/tab references, target generations, capability-aware action mapping, normalized metadata, timeout handling, and secret-safe errors.
  - depends on: 1.1, 3.1
  - Done when provider tests cover status, listing, content inspection, every supported mutation, unsupported capabilities, stale refs, approval-required responses, denial, lease use, broker absence, and protocol mismatch.
- [x] 3.3 Preserve strict local-provider semantics and schema compatibility.
  - depends on: 3.2
  - Done when existing Firefox and `browser: "chrome"` profile tests pass unchanged, `auto` never silently selects the Mac fleet, and transformed provider schemas retain all local and remote targeting fields.

## 4. Setup, operations, and safety

- [x] 4.1 Add idempotent setup/status/removal commands for the Mac broker binary, launch-agent plist, browser-specific native-host manifests and extension IDs, extension installation state, peer secret, policy defaults, and SSH-forwarding guidance.
  - depends on: 2.1, 2.2, 3.1
  - Done when fixture-based macOS tests prove install, refresh, operator-file backup, partial setup reporting, launch-agent reload, status, removal, and preservation of browser profiles and unrelated SSH configuration.
- [ ] 4.2 Add runtime observability with bounded metadata-only logs and diagnostics for broker, extension, SSH, policy, approval, lease, and provider states.
  - depends on: 2.1, 3.2
  - Done when tests prove URLs, typed values, credentials, page content, approval details, and peer secrets never enter logs or rendered diagnostics.
- [x] 4.3 Document topology, setup, approval behavior, temporary autonomy, emergency stop, Chrome/Edge differences, capability limits, troubleshooting, rollback, and recovery.
  - touches: `docs/MAC_HOMELAB_SSH_TOPOLOGY.md`, `docs/BROWSER_PROVIDER_PROTOCOL.md`, README or dedicated fleet guide
  - depends on: 4.1
  - Done when every public command and socket location matches implemented help output and no unsupported capability is claimed.

## 5. Verification and delivery

- [ ] 5.1 Run formatting, static checks, deterministic fleet tests, existing browser-provider regression tests, extension checks, shell/plist validation, and strict OpenSpec validation.
  - depends on: 1.1 through 4.3
  - Expected: every command exits 0 with no conflict markers, unsafe permissions, secret findings, or orphaned broker/browser test processes.
- [ ] 5.2 Run opt-in real Mac Chrome acceptance through Jcode's public browser interface.
  - depends on: 5.1
  - Expected: discover Chrome, inspect allowed content, observe mutation approval, execute an approved mutation, use and revoke a temporary lease, reject a hard-denied target, reconnect SSH/broker, and leave no test lease or process active.
- [ ] 5.3 Run the same acceptance independently against Edge when installed, or record a truthful not-installed result without treating Chrome success as Edge proof.
  - depends on: 5.1
- [ ] 5.4 Deploy the verified exact commit, reload Jcode and the Mac launch agent, then repeat status, inventory, approved mutation, lease revocation, emergency stop, and reconnect checks through the installed public interfaces.
  - depends on: 5.2, 5.3
  - Done when installed version and broker build identify the containing commit and the end-to-end workflow succeeds without using source-tree-only binaries.

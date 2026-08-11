# Mac Browser Fleet Verification Report

Date: 2026-08-11
Commit under verification: `879a626`
Environment: Linux homelab source checkout plus the configured `mac` SSH host, an arm64 macOS machine with Chrome and Edge installed.

## Interpreted acceptance boundary

The implementation follows the user-approved design: ordinary Mac Chrome and Edge tabs use a Manifest V3 extension and native messaging, explicitly managed instances may use loopback CDP, homelab Jcode selects the fleet only with `browser: "mac"`, transport is a reverse SSH Unix socket, and the Mac remains the authority for approval, leases, hard denies, and emergency stop. Before approval, the only implementation choice inferred was using a dedicated provider rather than changing local `browser: "chrome"` behavior. The approved OpenSpec subsequently made that choice explicit.

## Requirement traceability

| Requirement | Concrete check and observed result | Status |
|---|---|---|
| Mac browser fleet discovery | Extension inventory/action suites passed as part of all 37 Node tests. Broker generation handling passed in `protocol_broker_cdp.rs`. No real Mac extension/native-host connection was available. | Deterministic pass, live blocked |
| Explicit remote fleet routing | `scripts/dev_cargo.sh test -p jcode-app-core browser_tests:: --no-default-features` passed 15 tests with one opt-in live smoke ignored. Tests proved explicit `mac` routing, local Chrome profile preservation, schema fields, bounded request mapping, and stale/approval error meaning. | Pass |
| Private authenticated SSH transport | The built `jcode-mac-browser-fleet` binary was started through its public CLI and contacted over its public Unix-socket protocol. Authenticated health succeeded, invalid authentication failed without revealing the secret, and `stat` reported socket mode `600`. On the real Mac, launchd ran the broker and both the Mac socket and homelab forwarded socket reported mode `600`. Jcode queried the broker through SSH before and after a launchd broker restart. | Installed boundary pass |
| Mac-owned confirmation policy | `cargo test --manifest-path crates/jcode-mac-browser-policy/Cargo.toml` passed 10 table-driven policy tests. Broker tests proved read-only health succeeds while mutation requests return `approvalRequired`. The Mac-local approval UI is not implemented. | Engine pass, UI incomplete |
| Scoped expiring autonomy leases | Policy tests cover scope matching, the 15-minute maximum, monotonic expiration, restart/policy/target invalidation, and emergency-stop revocation. No installed Mac lease workflow was available. | Engine pass, live blocked |
| Immutable hard-deny boundaries | Policy table tests cover incognito, password managers, settings, extensions, privileged URLs, payment/banking confirmation, account security, authentication/recovery, and emergency stop. Category-level errors are secret-safe. | Pass |
| Capability-faithful hybrid control | All 37 extension tests passed. `cargo test --manifest-path crates/jcode-mac-browser-fleet/Cargo.toml` passed managed-CDP trust, loopback restriction, capability advertisement, bounded output, generations, and mutation replay tests. | Pass |
| Safe Mac lifecycle and setup | `cargo test --manifest-path crates/jcode-mac-browser-setup/Cargo.toml` passed 3 lifecycle tests covering install/status/remove, mode-0600 secret, operator-file backup, manifest and SSH rendering, profile preservation, and unrelated SSH preservation. `bash -n scripts/mac-browser-fleet/setup.sh` passed. On the real arm64 Mac, all three crates tested and built, setup installed six Jcode-owned artifacts, status reported mode `384` (`0600`) and no TCP listener, and launchd reported the broker running. | Installed pass, extension approval pending |
| Browser fleet verification | Package formatting, policy/broker/setup tests, all extension tests, Jcode browser-provider tests and check, shell syntax, `git diff --check`, and `openspec validate add-mac-browser-fleet --strict` all exited 0. Through Jcode's public `browser: "mac"` interface, status reported ready, listing returned authenticated health, an unapproved navigation returned approval-required, a stale generation failed closed, and listing recovered after a real launchd broker restart. Chrome/Edge target steering remains unavailable until the unpacked extensions are approved and the local approval UI is completed. | Public provider boundary pass, target acceptance blocked |

## Public-interface observation

The real broker executable accepted this public invocation shape:

```bash
jcode-mac-browser-fleet broker \
  --socket PATH \
  --peer-secret PATH \
  --policy PATH
```

Observed protocol results:

- Correct secret and `listBrowsers`: `ok: true`, health generation `0`, connected targets `0`.
- Incorrect secret: `ok: false`, error kind `unauthenticated`, diagnostic `fleet authentication failed`.
- The rejected response contained no peer secret.
- The bound Unix socket mode was `600`.

This proves the executable, argument parser, filesystem boundary, authentication boundary, response encoding, Jcode-compatible newline JSON transport, real arm64 launchd lifecycle, Unix-socket SSH forwarding, public Jcode provider routing, approval-required behavior, stale-generation behavior, and read-only recovery after broker restart. It does not prove extension attachment, approved mutation execution, lease UX, or real Chrome/Edge target steering.

## Remaining acceptance handoff

A Mac-attached session must still:

1. Install the broker, native hosts, and Chrome/Edge extension.
2. Provision the peer secret on the homelab and activate the reverse Unix-socket forward.
3. Complete or supply the Mac-local approval, lease, revocation, and emergency-stop surface.
4. Exercise Chrome and Edge independently through Jcode's public `browser: "mac"` interface.
5. Verify approved mutation, lease use and revocation, hard denial, broker/SSH reconnect, cleanup, and no remaining lease or process.

Until those steps pass, the repository provides a verified foundation rather than a completed end-to-end Mac browser automation product.

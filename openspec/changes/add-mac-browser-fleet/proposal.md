# Add Mac Browser Fleet

## Why

Jcode runs on the homelab, so its current `agent-browser` provider can only discover and steer browsers running on that Linux host. The user needs agents to safely enumerate and operate active Chrome and Edge tabs on the Mac without exposing a network listener or giving the homelab authority to weaken Mac-side safety policy.

## What Changes

- Add a Mac-resident browser fleet broker installed as a launch agent.
- Discover supported Chrome and Edge instances and expose stable browser, window, and tab references through a private local protocol.
- Connect homelab Jcode to the broker through the existing SSH topology using a forwarded Unix socket and authenticated protocol handshake.
- Extend the normalized Jcode `browser` tool with an explicit remote Mac fleet route while preserving existing Firefox and homelab-local Chrome behavior.
- Make inventory and health inspection read-only by default and gate state-changing actions through Mac-owned approval policy.
- Support scoped, expiring autonomous leases approved on the Mac, with an emergency stop and immutable hard-deny categories.
- Use an extension plus native host for ordinary user-launched Chrome and Edge tabs, with CDP used only for managed instances or capabilities that explicitly require it.
- Add deterministic protocol, policy, routing, reconnect, and failure tests plus opt-in acceptance tests against real Mac Chrome and Edge installations.

## Capabilities

### New Capabilities

- `mac-browser-fleet`: Mac-side browser discovery, secure SSH transport, confirmation-gated steering, expiring autonomy leases, lifecycle management, and acceptance behavior for Chrome and Edge.

### Modified Capabilities

None. Existing homelab-local Chrome and Firefox requirements remain unchanged; the remote route is defined by the new fleet capability.

## Impact

- Adds a Mac broker binary or subcommand, browser extension/native-host assets, launch-agent installation, and broker protocol types.
- Extends `crates/jcode-app-core/src/tool/browser.rs` provider routing and metadata.
- Reuses the existing SSH Unix-socket topology documented in `docs/MAC_HOMELAB_SSH_TOPOLOGY.md`; no public TCP listener is introduced.
- Requires local Mac approval UI or menubar integration for mutations, leases, policy status, and emergency stop.
- Daily browser credentials remain on the Mac. The homelab receives only policy-filtered browser metadata and action results.
- base-commit: jcode@a67b5fc85da2

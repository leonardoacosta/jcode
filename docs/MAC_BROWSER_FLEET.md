# Mac Browser Fleet

## Purpose

The Mac browser fleet lets the Jcode server on the homelab discover and safely steer supported Chrome and Microsoft Edge tabs running on the Mac. The Mac remains the security authority. The homelab cannot approve its own actions, change policy, issue leases, or release the emergency stop.

## Topology

```text
Mac                                                     Homelab

Chrome extension ─┐
                  ├─ native messaging ─ Mac fleet broker.sock
Edge extension ───┘                          │
                                            │ reverse StreamLocalForward over SSH
                                            ▼
                                  ~/.jcode/browser/mac-fleet.sock
                                            │
                                            ▼
                                  Jcode browser provider
```

Both endpoints are Unix sockets. No browser control service listens on a LAN, Tailscale, or public TCP port.

## Security policy

Default behavior:

- Fleet health and policy-filtered browser, window, and tab inventory are read-only.
- Page-content inspection is treated separately from topology inventory.
- Navigation, clicking, typing, form filling, uploads, downloads, tab creation, and tab closure require Mac-local approval.
- The Mac can issue a capability lease scoped to browser, profile, tab, origin, action set, generation, and expiration.
- The default maximum convenience lease is 15 minutes.
- Broker restart, target-generation change, policy reload, expiration, or emergency stop revokes leases.

The broker always denies:

- incognito targets
- password-manager pages
- browser settings and extension management
- privileged browser URLs
- payment or banking confirmation
- account-security changes
- authentication and recovery settings

These boundaries cannot be overridden over SSH or through the Jcode browser tool.

## Build the components

From the Jcode source checkout:

```bash
cargo build --release --manifest-path crates/jcode-mac-browser-fleet/Cargo.toml
cargo build --release --manifest-path crates/jcode-mac-browser-setup/Cargo.toml
```

The extension assets live under:

```text
extensions/mac-browser-fleet/
├── manifests/chrome.json
├── manifests/edge.json
├── native-host/chrome/
├── native-host/edge/
└── src/
```

Chrome and Edge require user approval to load or install the extension. Setup reports this state rather than claiming the browsers are connected.

## Install on the Mac

Set the broker path when it differs from `/usr/local/bin/jcode-mac-browser-broker`:

```bash
export JCODE_MAC_BROWSER_FLEET_BROKER="$HOME/.local/bin/jcode-mac-browser-fleet"
export JCODE_MAC_BROWSER_FLEET_EXTENSION_ID="<32-character ID shown by chrome://extensions or edge://extensions>"
export JCODE_MAC_BROWSER_FLEET_HOMELAB_HOST="<SSH host alias used by the Mac>"
cargo run --manifest-path crates/jcode-mac-browser-setup/Cargo.toml -- install
```

Load the unpacked extension first, copy its browser-assigned ID, and then run setup. Setup rejects its fixture placeholder so it cannot silently install a native-host manifest that no real extension may use.

Setup creates only Jcode-owned artifacts:

- `~/Library/LaunchAgents/dev.jcode.mac-browser-fleet.plist`
- Chrome and Edge native-messaging host manifests
- `~/Library/Application Support/Jcode/MacBrowserFleet/peer.secret` with mode `0600`
- `~/Library/Application Support/Jcode/MacBrowserFleet/policy.toml`
- `~/.ssh/jcode-mac-browser-fleet.conf`

Operator-edited Jcode-owned files are backed up before refresh. Browser profiles and unrelated SSH configuration are never removed or rewritten.

Inspect setup:

```bash
cargo run --manifest-path crates/jcode-mac-browser-setup/Cargo.toml -- status
```

Remove Jcode-owned setup artifacts:

```bash
cargo run --manifest-path crates/jcode-mac-browser-setup/Cargo.toml -- remove
```

## SSH forwarding

Include the generated SSH fragment from the Mac's `~/.ssh/config`:

```sshconfig
Include ~/.ssh/jcode-mac-browser-fleet.conf
```

The generated host block uses:

```sshconfig
ExitOnForwardFailure yes
ServerAliveInterval 30
ServerAliveCountMax 3
StreamLocalBindUnlink yes
RemoteForward ~/.jcode/browser/mac-fleet.sock "~/Library/Application Support/Jcode/MacBrowserFleet/jcode-mac-browser-fleet.sock"
```

Reconnect the persistent SSH session after installation. Securely provision the same peer secret on the homelab at `$JCODE_HOME/browser/mac-fleet.secret`, or set `JCODE_MAC_BROWSER_FLEET_SECRET` in Jcode's environment. Do not put the secret in the SSH configuration or command history. A recovered connection may retry read-only inventory. Mutations are never automatically replayed after disconnect.

## Browser differences

Ordinary Chrome and Edge tabs connect through the extension and native host. Each target advertises the operations it can faithfully perform. Unsupported actions return an explicit capability error.

Managed CDP targets are opt-in and loopback-only. They can expose richer inspection or evaluation capabilities, but they remain subject to Mac approval and hard-deny policy. Jcode never relaunches a daily browser profile under CDP.

## Current implementation boundary

The repository contains a runnable authenticated broker, policy engine, setup lifecycle, Chrome and Edge extension/native-host assets, and an explicit `browser: "mac"` Jcode provider. The broker's public Unix-socket health path is executable today. Full ordinary-tab inventory and action delivery still requires installing and wiring the extension/native host on a real Mac. The Mac-local approval UI, lease controls, and emergency-stop UI are not yet complete, so mutations correctly remain approval-blocked rather than silently proceeding.

## Verification

Deterministic checks:

```bash
cargo test --manifest-path crates/jcode-mac-browser-policy/Cargo.toml
cargo test --manifest-path crates/jcode-mac-browser-fleet/Cargo.toml
cargo test --manifest-path crates/jcode-mac-browser-setup/Cargo.toml
for test in extensions/mac-browser-fleet/test/*.test.mjs \
            extensions/mac-browser-fleet/tests/*.test.mjs; do
  node --test "$test"
done
```

Real acceptance must be run on the Mac through the installed Jcode public browser interface. It must independently prove Chrome and Edge when each is installed, an approved mutation, a temporary lease, lease revocation, a hard denial, emergency stop, SSH or broker reconnect, and cleanup with no test lease remaining.

## Troubleshooting

- **Browser absent:** Confirm it is installed, running, and has the extension enabled.
- **Native host disconnected:** Re-run setup and verify the browser-specific native-host manifest.
- **Broker unavailable:** Check the launch agent and the local mode-0600 socket.
- **Homelab cannot connect:** Reconnect SSH and verify the reverse socket exists on the homelab.
- **Approval required:** Approve on the Mac or issue a bounded local lease.
- **Stale target:** Refresh fleet inventory. Never reuse a target reference after its generation changes.
- **Emergency stop active:** Release it locally on the Mac. The homelab cannot release it.

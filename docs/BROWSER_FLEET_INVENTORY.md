# Jcode Browser Fleet Inventory

Devices, browsers, and profiles reachable from Jcode, with the access path and
control level for each. Verified 2026-08-11 unless noted.

## Access levels

| Level | Meaning |
| --- | --- |
| `full` | Jcode may read and mutate without per-action approval. |
| `approval-gated` | Read-only by default; each mutation needs a Mac-local lease. |
| `configured` | Wiring installed, but not yet observed working end to end. |
| `unconfigured` | Known to exist, not yet enrolled. |

## homelab (Linux, this host)

| Browser | Profile | Path | Access |
| --- | --- | --- | --- |
| agent-browser Chrome | `social` | local agent bridge | `full` |
| Firefox | n/a | agent bridge not ready | `unconfigured` |

Local Chrome is the default automation surface: `browser` with `browser="chrome"`.

## macbook (macOS arm64, Tailscale `macbook`)

Reached over an SSH reverse Unix-socket forward to `~/.jcode/browser/mac-fleet.sock`
on the homelab. Secrets never leave the Mac. Two LaunchAgents keep it alive:
`dev.jcode.mac-browser-fleet` (broker) and `dev.jcode.mac-browser-fleet-tunnel`
(reverse tunnel).

### Chrome

| Source | User-data-dir | Profile | Access |
| --- | --- | --- | --- |
| `managed-chrome` | `Chrome-AgentDebug` | Default | `full` (loopback CDP on 9222) |
| `ordinary-chrome` | `Chrome` | `Profile 1` (operator's signed-in profile) | `approval-gated` |

`ordinary-chrome` is bridged by the unpacked MV3 extension
`mlgjaoahakdijgckgjpmpkafccgffpgd` talking to the native host over stdio. Tabs
appear as opaque `tab_*` / `win_*` references; raw Chrome ids stay host-only.

### Edge

Edge is installed with four profiles in the stable user-data-dir. `Local State`
display names do not match on-disk directory names, so both are recorded. The
extension ID is `fhhloicfoliebblijlggiohfkdkfljcc` and the native-host manifest
at `Microsoft Edge/NativeMessagingHosts/dev.jcode.mac_browser_fleet.json`
authorizes exactly that origin.

| Directory | Display name | Account | Last used | Access |
| --- | --- | --- | --- | --- |
| `Profile 1` | Profile 2 | BBAdminLAcosta@bbins.com (bbadmin) | 2026-08-11 | `approval-gated`, bridged as `ordinary-edge` |
| `Default` | Profile 1 | leonardo.acosta@bridgespecialty.com (o365) | 2026-08-06 | `unconfigured`, extension not loaded in this profile |
| `Profile 2` | Profile 4 | none | 2026-06-23 | not targeted |
| `Profile 4` | Profile 3 | none | 2026-08-04 | not targeted |

Beta, Dev, and Canary user-data-dirs exist but hold no profiles and are ignored.

#### Per-profile extension caveat

Chromium loads *unpacked* extensions per profile, not per user-data-dir. The
native-host manifest is shared across every profile in the user-data-dir, so it
only has to be installed once, but each profile that should be steerable must
load the unpacked extension itself. `bbadmin` is loaded; `o365` is not.

To enroll `o365`: open Edge on that profile, go to `edge://extensions`, enable
Developer mode, choose "Load unpacked", and select
`~/.jcode/mac-browser-fleet-extension-edge`.

No reinstall is needed afterwards. Chromium derives an unpacked extension's ID
from the absolute directory path, so loading the same directory in a second
profile produces the same ID (`fhhloicfoliebblijlggiohfkdkfljcc`), which the
shared native-host manifest already authorizes. Only if the directory moves does
the ID change, requiring `jcode-mac-browser-setup install` to be re-run with
`JCODE_MAC_BROWSER_FLEET_EDGE_EXTENSION_ID` set to the new value.

Edge here is unmanaged (no `ExtensionInstallForcelist` or `ExtensionSettings`
policy), so this load cannot be automated and is deliberately left to the
operator.

Each profile reports its own `profileLabel`, so leases stay scoped per profile
even though both share one user-data-dir and one manifest.

#### MV3 service-worker suspension

The bridge only runs while the extension's MV3 service worker is alive. An open
native-messaging port keeps it alive, but once the browser suspends the worker
(observed on Edge within minutes of idling) nothing restarted it, and that
browser silently disappeared from the fleet until the extension was reloaded.

The extension now registers a persistent one-minute `chrome.alarms` keepalive
(`jcode-mac-browser-fleet-keepalive`). Each tick wakes the worker and
re-establishes the native connection if it was lost, so a browser rejoins the
fleet on its own. This costs one extra permission, `alarms`, recorded with its
rationale in `config/permissions.json`.

A browser can still be missing from a single `list_tabs` call taken during the
gap between suspension and the next tick.




## Enrolling a new browser or profile

1. Load the unpacked extension from `~/.jcode/mac-browser-fleet-extension-<browser>`
   and copy its extension ID.
2. Re-run `jcode-mac-browser-setup install` with
   `JCODE_MAC_BROWSER_FLEET_EDGE_EXTENSION_ID` (or the Chrome equivalent) set.
   Manifests are written to every discovered user-data-dir automatically.
3. Confirm the profile appears through `browser list_tabs browser=mac`.

Additional user-data-dirs are auto-discovered. Override with
`JCODE_MAC_BROWSER_FLEET_CHROME_USER_DATA_DIRS` or
`JCODE_MAC_BROWSER_FLEET_EDGE_USER_DATA_DIRS` (colon-separated) when needed.

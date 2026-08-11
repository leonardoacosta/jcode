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

Edge is installed with four profiles in the stable user-data-dir. The two that
matter for work are below; `Local State` display names do not match the on-disk
directory names, so both are recorded.

| Directory | Display name | Account | Last used | Access |
| --- | --- | --- | --- | --- |
| `Default` | Profile 1 | leonardo.acosta@bridgespecialty.com (o365) | 2026-08-06 | `unconfigured` |
| `Profile 1` | Profile 2 | BBAdminLAcosta@bbins.com (bbadmin) | 2026-08-11 | `unconfigured` |
| `Profile 2` | Profile 4 | none | 2026-06-23 | not targeted |
| `Profile 4` | Profile 3 | none | 2026-08-04 | not targeted |

Edge enrollment is blocked only on loading the extension and capturing its Edge
extension ID; the manifest is staged at
`NativeMessagingHosts/dev.jcode.mac_browser_fleet.json.pending-extension-id`.
Because Edge shares one user-data-dir across profiles, a single extension
install covers both `o365` and `bbadmin`, and each profile reports its own
`profileLabel` so leases stay scoped per profile.

Beta, Dev, and Canary user-data-dirs exist but hold no profiles and are ignored.

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

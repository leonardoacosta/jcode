# The Codesign Bridge (Aqua-Session Signing)

> Load this file BEFORE attempting any signed build, `codesign` call, or device install over
> `ssh mac`. It has the root-cause diagnosis, the full headless iOS install procedure, the
> bridge internals to clone for any new GUI-session-bound task, and the two sharp-edge notes
> (uid-501/no-sudo, device-must-be-unlocked-to-launch).

## THE codesign gotcha: signing needs a GUI (Aqua) session — SOLVED for iOS

`codesign` with the team identity (`8E12…`/`DX3Y367L2A`) fails with **`errSecInternalComponent`**
inside a **background SSH session** (`managername=Background`). Root cause (nx-tceo6): NOT a locked
keychain — the login keychain is already unlocked in the permanently-logged-in console session. It
is a **session-CONTEXT** block: the team identity only signs inside the **Aqua (GUI) session**.

Both macOS *and* iOS work around this with the **gui/501 Aqua bridge**: when NOT in Aqua, the
SSH-side script `launchctl kickstart`s a GUI-scoped LaunchAgent that **re-enters the signed build
inside the Aqua session** (where signing works), writes an `OK`/`SKIP`/`FAIL` marker, and the SSH
side polls that marker.

- **macOS `Nexus.app`:** `deploy/lib/macos-swift-deploy.sh` + `dev.leonardoacosta.nexus.deploy`.
- **iOS device install (SOLVED):** `deploy/lib/ios-device-deploy.sh` +
  `deploy/launchagents/dev.leonardoacosta.nexus.ios-deploy.plist`. This is **no longer a hand-off
  to Leo** — a signed iOS device install runs fully headless over `ssh mac`.

### Headless iOS device install over SSH (the working procedure)

```bash
# 0. Discover the device UDID (device shows available(paired) when reachable):
ssh mac 'xcrun devicectl list devices'
# 1. One-time: load the GUI LaunchAgent into gui/501 (no sudo — see uid-501 note):
ssh mac '~/dev/personal/nexus/deploy/ios-deploy.sh --install'   # also run by deploy/install.sh
# 2. Build (signed, in Aqua) + install on the device, headless:
ssh mac '~/dev/personal/nexus/deploy/ios-deploy.sh --device <UDID>'
#   -> non-Aqua caller kickstarts gui/501/<label>, polls the marker, returns 0 on OK.
#      Success prints devicectl's "App installed:" block + bundleID + installationURL.
```

The bridge internals (clone these conventions for any new GUI-session-bound task): the lib detects
`launchctl managername != Aqua`, writes the target UDID to a sentinel file
(`~/Library/Application Support/Nexus/ios-deploy-device.txt`), resets the marker, then
`launchctl kickstart -k gui/501/dev.leonardoacosta.nexus.ios-deploy`. The agent wrapper
(`deploy/lib/ios-deploy-agent.sh`) runs INSIDE Aqua: `xcodegen generate`; signed
`xcodebuild build -scheme nexus-ios -destination 'generic/platform=iOS' -allowProvisioningUpdates
DEVELOPMENT_TEAM=DX3Y367L2A CODE_SIGN_STYLE=Automatic`; locates the `.app` under
`Build/Products/Debug-iphoneos/`; `xcrun devicectl device install app`; best-effort
`device process launch`; writes the marker. Log: `~/Library/Logs/nexus-ios-deploy.log`.

> **uid-501 / no-sudo note:** over `ssh mac` you are **uid 501**, the SAME uid as the console/Aqua
> user → you can `launchctl bootstrap`/`kickstart gui/501/<label>` WITHOUT sudo. There is NO
> passwordless sudo, so `launchctl asuser` is NOT an option — the **gui/501 kickstart IS the bridge**.

> **Device must be UNLOCKED to LAUNCH** (not to install). `devicectl device install` succeeds on a
> locked phone; `device process launch` returns `FBSOpenApplicationErrorDomain error 7 (Locked)`
> until the screen is unlocked. The bridge treats install as the contract and launch as best-effort.

## Device install (modern toolchain, raw)

```bash
xcrun devicectl list devices                                   # paired devices over coredevice net (no cable)
xcrun devicectl device install app --device <UDID> <Built.app> # install (works on a locked device)
xcrun devicectl device process launch --device <UDID> <bundle.id>  # requires the device be unlocked
```
`ios-deploy` is legacy and not installed on this Mac — use `devicectl`. A device shows
`available (paired)` when reachable; `unavailable` when off/asleep. The raw commands only sign in
the Aqua session — over SSH, drive them through `deploy/ios-deploy.sh` (the gui/501 bridge above).

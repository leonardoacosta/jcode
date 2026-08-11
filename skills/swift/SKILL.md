---
name: swift
description: >-
  Swift project layout and xcodebuild gates for native iOS/macOS apps. Triggers
  on .swift files, SwiftUI, xcodegen, xcodebuild, codesign, nx menubar, or watchOS.
category: Framework
level: library
engineer: ui-engineer
gate: "cd apps/swift && xcodegen generate && xcodebuild -scheme nexus-mac test -only-testing:nexus-mac-Tests -only-testing:NexusSharedTests CODE_SIGNING_ALLOWED=NO"
bundles: []
audit-rubric:
  - "Headless typecheck over SSH passes: `ssh mac 'xcrun --sdk <sdk> swiftc -typecheck ...'` exits 0 with zero diagnostics for any self-contained new module before it's wired into a target."
  - "xcodegen regenerates cleanly: `xcodegen generate` after a `project.yml`/`Sources/` change produces no diff outside `project.pbxproj` + `*/Generated/Info.plist` (regen drift only), and the regenerated pbxproj is committed."
  - "Codesign path honored: a signed build/device-install either runs inside the Aqua (GUI) session directly, or is routed through the gui/501 kickstart bridge (`deploy/lib/ios-device-deploy.sh` et al.) — never attempted raw over a background SSH session."
allowed-tools: Read, Glob, Grep, Bash
---

# Swift

Native iOS/macOS/watchOS development for XcodeGen-based projects (the **nexus / `nx`**
fleet: a menubar Mac app + iOS + watch + shared framework). Covers project conventions,
headless compile verification from Linux, signing, and the device-deploy gotchas.

## XcodeGen projects (project.yml is the source of truth)

`nexus.xcodeproj` is **generated** — do not hand-edit `project.pbxproj`. Targets + sources
live in `apps/swift/project.yml`:

```yaml
nexus-ios:
  type: application
  platform: iOS
  sources:
    - path: nexus-ios/Sources   # DIRECTORY GLOB — any .swift under here auto-includes
```

- **Adding a file = drop it under the globbed `Sources/` dir + `xcodegen generate`.** No
  `PBXFileReference`/`PBXBuildFile`/build-phase surgery. Verify: `grep -c MyFile.swift nexus.xcodeproj/project.pbxproj` (expect 4 — fileRef, buildFile, group, sources phase).
- The `.xcodeproj` IS tracked (committed-generated). After `xcodegen generate` adds a file,
  **commit the regenerated pbxproj** so teammates who open Xcode without running xcodegen get it.
- `project.pbxproj` and `*/Generated/Info.plist` are XcodeGen output — a dirty diff on them is
  regen drift, not work. Don't stash/commit-as-precious; `xcodegen generate` overwrites them.
- Targets in nx: `nexus-ios` (iphoneos), `nexus-mac` (menubar), `nexus-watch`, `NexusShared`
  (shared framework), `MarkdownUI`. HealthKit/biometrics is **iOS-only** → `nexus-ios`.

## Headless compile verification (Linux → Mac over SSH)

You cannot run `xcodebuild` on Linux, but you can verify a **self-contained** file (imports only
Foundation + a system framework, no app-type deps) against the real SDK over SSH:

```bash
scp File.swift mac:/tmp/
ssh mac 'xcrun --sdk iphoneos swiftc -typecheck -target arm64-apple-ios16.0 /tmp/File.swift; echo EXIT=$?'
# EXIT=0 + zero diagnostics = clean. Pass multiple files together to check integration.
ssh mac 'plutil -lint /tmp/Info.plist /tmp/App.entitlements'   # validate plists/entitlements
```

Keep new modules self-contained (e.g. an actor that does its own URLSession POST + endpoint
resolution via `Bundle.main.object(forInfoDictionaryKey:)`) so they type-check in isolation and
drop in cleanly.

## Signing + headless capability registration (no Developer-portal clicking)

- nx uses **Automatic** signing (`CODE_SIGN_STYLE: Automatic`, `DEVELOPMENT_TEAM: DX3Y367L2A`).
- An **App Store Connect API key** (`~/.appstoreconnect/private_keys/AuthKey_<KEYID>.p8`) lets
  tooling hit Apple's portal with NO interactive Apple-ID/2FA login:
  - `xcodebuild ... -allowProvisioningUpdates -authenticationKeyID <id> -authenticationKeyIssuerID <iss> -authenticationKeyPath <p8>` **auto-registers new capabilities** (e.g. HealthKit) on the App ID + regenerates the profile. Adding the entitlement to `*.entitlements` + this flag is usually enough — no manual portal step.
  - fastlane lanes (`sigh_ios`, `ensure_bundle_id`) drive `Spaceship::ConnectAPI` with the same .p8
    (fastlane `produce` does NOT support API-key auth; Spaceship does).
- Add a capability = add its key to `<target>/Resources/<target>.entitlements`
  (e.g. `com.apple.developer.healthkit` + `…healthkit.background-delivery`) + a usage string in
  `Info.plist` (e.g. `NSHealthShareUsageDescription`). XcodeGen carries the entitlements path.

## Signed builds need a GUI (Aqua) session — the SOLVED codesign bridge

`codesign` with the team identity (`8E12…`/`DX3Y367L2A`) fails with **`errSecInternalComponent`**
inside a background SSH session — NOT a locked keychain, a **session-CONTEXT** block: the team
identity only signs inside the **Aqua (GUI) session**. Both macOS and iOS builds work around this
with the **gui/501 Aqua bridge**: the SSH-side script `launchctl kickstart`s a GUI-scoped
LaunchAgent that re-enters the signed build inside Aqua, writes an `OK`/`SKIP`/`FAIL` marker, and
the SSH side polls it. iOS device install is fully SOLVED and headless
(`deploy/lib/ios-device-deploy.sh` — no longer a hand-off to Leo).

**MANDATORY: load `references/codesign-bridge.md` BEFORE attempting any signed build, `codesign`
call, or device install over `ssh mac`.** It has the root-cause diagnosis, the full headless iOS
install procedure, the bridge internals (clone for any new GUI-session-bound task), the
uid-501/no-sudo note, the device-must-be-unlocked-to-launch gotcha, and the raw `devicectl`
command reference.

## nx quick reference

- **Mac over SSH:** `ssh mac` (config alias; user `leonardoacosta`, key `~/.ssh/id_ed25519`),
  Xcode 26.4, repo at `/Users/leonardoacosta/dev/personal/nexus`. (`macbook-pro`/wrong-user fails.)
- **Networking:** `Network.postJSON(url:body:)` helper; endpoints resolve from an `Info.plist`
  key with a `http://homelab:<port>` fallback (see `ApnsRegistrar` — the canonical actor+endpoint
  pattern to clone for any homelab-push module).
- **Prefs:** `NexusShared.SettingsStore` (UserDefaults-backed, typed surface, shared across targets).
- **iOS app entry:** `nexus-ios/Sources/App/NexusIOSApp.swift` (`@UIApplicationDelegateAdaptor`
  `NexusAppDelegate` — bootstrap background work from `didFinishLaunchingWithOptions`).
- **Naming watch-out:** nx already uses "health" for SYSTEM metrics (CPU/mem/disk —
  `HealthSummaryScene`/`HealthCollector`). Apple biometric HealthKit work must be named distinctly
  (e.g. `HealthKit*`) to avoid the collision.

## NEVER

- **NEVER hand-edit `project.pbxproj`.** It's XcodeGen-generated from `apps/swift/project.yml` —
  any manual edit is silently overwritten by the next `xcodegen generate`. Add files via the
  globbed `Sources/` dir and regenerate instead.
- **NEVER stash, revert, or treat a `project.pbxproj`/`*/Generated/Info.plist` diff as precious
  work.** A dirty diff on those two paths after a `project.yml`/`Sources/` change is regen drift,
  not real work — `xcodegen generate` overwrites them; commit the regenerated result instead of
  fighting it.
- **NEVER attempt a signed build or raw `codesign` call over a background SSH session.** The team
  identity (`8E12…`/`DX3Y367L2A`) fails with `errSecInternalComponent` outside the Aqua (GUI)
  session — this is NOT a locked-keychain problem, it's a session-context restriction. Route
  through the gui/501 kickstart bridge (`deploy/lib/ios-device-deploy.sh` et al.); never a raw
  `ssh mac codesign ...`. Load `references/codesign-bridge.md` first for the full bridge internals.
- **NEVER run `xcodebuild` directly on Linux.** It doesn't exist there. Verify a self-contained
  module with `ssh mac 'xcrun --sdk <sdk> swiftc -typecheck ...'` instead, and keep new modules
  self-contained (Foundation + system frameworks only, no app-type deps) so they type-check in
  isolation before being wired into a target.
- **NEVER assume `macbook-pro` or a different SSH user works.** Only `ssh mac` (user
  `leonardoacosta`, key `~/.ssh/id_ed25519`) resolves to the nx dev Mac — `macbook-pro`/wrong-user
  fails.
- **NEVER name new Apple HealthKit (biometric) work with a bare `Health*` prefix.** nx already
  reserves "health" for SYSTEM metrics (`HealthSummaryScene`/`HealthCollector`) — a biometric
  module needs a distinct name (e.g. `HealthKit*`) to avoid the collision.

# Replace the native browser bridge with jev-ultrafast

## Why

Jcode currently maintains a native browser tool adapter plus Firefox-specific bridge setup, extension/host management, browser sessions, and shell-command rewriting. This duplicates browser automation infrastructure and exposes provider-specific internals. Replace it with [`browser-use/jev-ultrafast`](https://github.com/browser-use/jev-ultrafast), with labeled local browser profiles managed by the user or agent.

## What changes

- Replace the browser tool's Firefox/native bridge implementation with the supported `jev-ultrafast` integration surface.
- Remove Firefox-specific tool schema, provider metadata, setup/status flows, bundled bridge scripts/artifacts, and browser-command shell rewriting.
- Add named profile management to browser actions and user configuration: the agent can create/select profiles, inspect/list profiles, and associate browser sessions with a profile. Profile deletion remains user-managed and explicitly confirmed.
- Preserve browser automation actions through the upstream-supported capabilities rather than preserving Jcode's private bridge protocol.
- Support fresh temporary or persistent profiles, plus labeled references to existing profiles. Temporary state is session-owned; attached state remains externally owned.

## Actors

- Jcode agent: selects a profile and performs browser actions.
- Jcode user: creates, lists, selects, and deletes profiles; controls persistence and browser identity.
- `jev-ultrafast`: performs browser automation using its supported runtime.

## Scope

- Browser tool API and tool registration.
- Profile configuration/storage, lifecycle, and profile-to-session selection.
- Runtime invocation, dependency/setup diagnostics, and migration from existing browser configuration.
- Removal of Jcode-owned Firefox bridge implementation and shell special-casing.
- Tests for profile isolation, lifecycle, tool contract, migration, and failure handling.

## Non-scope

- Rewriting the upstream `jev-ultrafast` project or maintaining a fork absent a demonstrated upstream gap.
- Importing Firefox cookies, extensions, passwords, or other browser data into new profiles.
- Automatic migration of live browser state or automatic deletion of old user data.
- Changing unrelated shell command semantics, browser security policy, or consent requirements for consequential website actions.
- Desktop browser UI or a new graphical profile manager.

## Proposed behavior

1. User or agent can create named profiles; both can list and inspect them. Names are validated, unique, and safe for filesystem use.
2. Browser tool calls may specify a profile. Omitted profile uses the explicitly configured default, or returns an actionable error if no default exists. No implicit profile creation.
3. Fresh profiles start empty with either temporary or persistent storage. Distinct managed profiles use separate Chrome data directories. Attached profiles reference existing state without copying it.
4. User or agent can select/change the default profile through the supported profile interface. Selection applies to new browser sessions; active sessions remain bound to their starting profile.
5. Explicit deletion of managed persistent profiles requires user confirmation and refuses active profiles. Temporary state is automatically cleaned up after its owning session closes and browser handles are released. Attached data is never deleted by Jcode.
6. Status reports runtime availability, selected profile, and actionable setup errors without exposing secrets or profile contents.
7. Missing/incompatible upstream runtime fails clearly; no fallback to Firefox bridge or hidden native implementation.
8. Existing Firefox bridge state remains untouched during migration. Jcode stops invoking it. User may remove old files manually after verifying the new setup.

## Integration decisions

- Upstream integration form: Python package/subprocess, CLI, or another supported interface. Inspect upstream documentation and pin a supported version; do not invent a protocol.
- Profile lifecycle interface: provide agent-accessible create/list/inspect/select/default operations. Destructive deletion stays user-confirmed and unavailable to browser automation.
- Default profile policy for existing installations: require an explicit choice, rather than silently adopting or copying existing Firefox state.
- Packaging policy for installing the upstream runtime: prefer an explicit optional setup path; never install software or access the network implicitly during ordinary tool calls.

## Dependencies

- Upstream project: `https://github.com/browser-use/jev-ultrafast` (integration capabilities and license/version to verify before implementation).
- No existing Jcode browser implementation is a required compatibility layer.

## Touched capabilities

- `browser-automation`: replace provider-specific bridge behavior with upstream-backed browser automation and profile selection.
- Profile lifecycle requirements are included in `specs/browser-automation/spec.md`.

## Acceptance evidence

- Tool schema and dispatch contain no Firefox provider/browser choices or Jcode-native bridge-only actions.
- Repository no longer contains executable Firefox bridge setup/session/command rewrite code or bundled bridge artifacts; historical changelogs and archived proposals may retain context.
- A real isolated Jcode process can create/list/select two profiles, persist them across restart, run a harmless browser action in each, and demonstrate state isolation.
- Missing runtime, invalid/duplicate profile name, absent default, active-session deletion, and corrupt profile metadata produce clear errors without data loss.
- Existing bridge files and user browser data remain unchanged by upgrade/startup.
- Focused Rust tests and a real upstream-backed isolated runtime workflow pass. If upstream setup or browser runtime is unavailable, report that acceptance as blocked, not passed.

## Status

Lifecycle design approved on 2026-09-26: local Jcode-managed fresh temporary/persistent profiles and labeled attachment to existing profiles. Selecting an attached label authorizes use of its signed-in sessions without another login-consent prompt. Consequential website actions retain their normal authorization requirements. Full written revision approved by user at 2026-09-26T06:36:28Z; execution resumed. The earlier assertion that missing upstream profile APIs makes isolation impossible is withdrawn; a local launcher plus scoped upstream connection is permitted and must be tested. Cloud profiles are out of scope.

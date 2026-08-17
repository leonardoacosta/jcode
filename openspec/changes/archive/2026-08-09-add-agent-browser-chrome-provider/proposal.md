# Add Agent-Browser Chrome Provider

## Why

Jcode's first-class `browser` tool currently routes only to Firefox Agent Bridge, so browser automation is unavailable whenever the Firefox extension is missing or disconnected even when a healthy Chrome-capable automation runtime is already installed. The local environment has `agent-browser 0.27.3` and Google Chrome available, and its JSON/session command surface closely matches Jcode's normalized browser contract.

## What Changes

- Add an `AgentBrowserProvider` that exposes a trusted installed `agent-browser` CLI as the built-in `chrome` backend.
- Route explicit `browser: "chrome"` requests to isolated Chrome sessions without touching the user's daily browser profile or inheriting user/project agent-browser configuration.
- Normalize Chrome tabs, snapshots, content, interactions, waits, evaluation, uploads, screenshots, errors, and readiness into the existing Jcode browser tool contract.
- Add a string `tab_ref` input for Chrome stable tab IDs while retaining the existing integer `tab_id` contract for Firefox and provider dialect compatibility.
- Prove explicit Chrome parity before enabling a sticky, readiness-aware `auto` fallback that keeps healthy Firefox first and preserves one provider per Jcode session.
- Add deterministic fake-CLI coverage, live localhost parity coverage, session and state isolation checks, bounded subprocess/file handling, secret-safe output checks, and updated provider documentation.

## Capabilities

### New Capabilities

- `chrome-browser-provider`: First-class Chrome automation through a trusted `agent-browser`, including explicit routing, isolated sessions, normalized actions and outputs, parity-gated automatic fallback, and provider verification.

### Modified Capabilities

None. The repository has no existing checked-in browser capability specification.

## Impact

- No new Rust crate dependency is required; Jcode invokes an optional external executable discovered from an explicit absolute override or a securely validated `PATH` candidate.
- Explicit Chrome setup may run `agent-browser install` only when the trusted CLI exists but its Chrome runtime is unavailable. Jcode does not install the npm package automatically.
- Firefox remains supported and retains initial priority for `auto` while healthy.
- Chrome callers use `tab_ref` values such as `t1`; existing Firefox callers continue using integer `tab_id`.
- Chrome snapshots surface one-level iframe refs; unsupported direct frame/window targeting fails explicitly rather than being ignored.
- Chrome `provider_command` remains unsupported in this change, avoiding exposure of auth, profile, connection, install, upgrade, dashboard, or cross-session CLI controls.

- touches: `crates/jcode-app-core/src/tool/browser.rs`
- touches: `crates/jcode-app-core/src/tool/browser_tests.rs`
- touches: `crates/jcode-app-core/src/tool/browser/agent_browser.rs (new)`
- touches: `crates/jcode-app-core/src/tool/browser/agent_browser_tests.rs (new)`
- touches: `README.md`
- touches: `docs/BROWSER_PROVIDER_PROTOCOL.md`
- base-commit: jcode@34ff755aad4529f80cc0cdf0e20b45a78c0d3a0d

## Preconditions

- `agent-browser` remains an optional external runtime with a supported version range of `>=0.27.3,<0.28.0` and the required machine-readable command surface.
- Chrome or Chromium is available to the trusted CLI, or the user explicitly invokes Chrome setup to install its browser runtime.
- The existing Firefox provider and its setup flow remain independently selectable.
- Implementation tests override executable discovery with an absolute `JCODE_AGENT_BROWSER_BIN` fake and do not depend on developer-machine browser state.

## Decisions

- `browser: "chrome"` is explicit and never falls back to Firefox.
- Executable discovery canonicalizes and pins the trusted binary. Automatic mode rejects relative/current-directory, repository-local, insecurely writable, replaced, or incompatible `PATH` candidates; an absolute environment override is treated as explicit operator trust but still must be a regular executable.
- Each Jcode session uses a collision-resistant `jcode-<readable-prefix>-<stable-hash>` provider namespace, an explicit Jcode-owned neutral config and cwd, and a cleared `AGENT_BROWSER_*` environment except controlled settings such as idle timeout.
- Any successful browser action establishes or updates a sticky provider affinity for that Jcode session. Later `auto` actions reuse it; provider failure is reported instead of silently migrating tabs and refs to another backend. Per-session Chrome actions are serialized.
- Automatic readiness uses a short-lived cache invalidated by setup, provider failure, executable fingerprint change, or affinity reset. Chrome doctor is non-installing but may perform only its documented stale daemon sidecar cleanup.
- User-controlled URLs, text, form values, scripts, keys, and similar arguments travel through structured stdin batches where supported. Native command echoes and known sensitive inputs are recursively removed from provider output, metadata, and diagnostics.
- Provider subprocesses use hard timeout and byte limits with child kill/reap. Screenshots use exclusive Jcode-owned paths, exact-path validation, regular-file and PNG checks, byte/dimension limits, and unconditional cleanup.
- Chrome uses additive string `tab_ref`; Firefox retains integer `tab_id`. Unsupported ID forms return provider-specific guidance.
- Chrome `provider_command` is rejected until a separately specified safe extension surface exists.
- Building a native CDP client and attaching to the user's existing Chrome profile are rejected because they add unnecessary implementation and credential exposure risk.

## Done Means

- `browser: "chrome"` reports `backend: agent_browser` and performs every supported normalized action through the pinned trusted executable.
- Two Jcode sessions do not share tabs, cookies, local storage, session storage, active-tab state, or browser configuration, including collision-prone session IDs and hostile inherited config/env inputs.
- Explicit Firefox and Chrome never cross-fall back; parity is proven before `auto` activation; automatic selection is sticky per session and reports its selected backend and fallback reason.
- Chrome string `tab_ref` and Firefox integer `tab_id` both work across raw and transformed provider schemas without changing existing callers.
- Screenshots are validated, attached without absolute-path disclosure, bounded, and cleaned up; uploads reject invalid non-regular inputs.
- Typed values and batch command payloads are absent from user-visible provider output, metadata, and bounded provider errors.
- Missing, insecure, replaced, incompatible, or unhealthy executables; timeouts; oversized/malformed output; stale sessions/refs; unsupported targeting; and invalid files produce actionable errors.
- All new provider tests, existing browser tests, strict OpenSpec validation, and the opt-in localhost Chrome smoke test pass.

## Testing

- Run `cargo test -p jcode-app-core browser`; expected result: all shared Firefox and Chrome provider tests exit 0.
- Run the targeted fake-provider tests with `cargo test -p jcode-app-core agent_browser`; expected result: trust validation, routing, command construction, stdin redaction, affinity/serialization, cache invalidation, timeout/output limits, malformed output, file validation, and schema compatibility pass.
- Run the opt-in live test with `JCODE_AGENT_BROWSER_LIVE=1 cargo test -p jcode-app-core agent_browser_provider_live_smoke -- --ignored --nocapture`; expected result: a local test site completes open, snapshot, form interactions, tabs, one-level iframe refs, screenshot, every state-isolation assertion, and close without leaving a live test session.
- Run `openspec validate add-agent-browser-chrome-provider --strict --no-interactive`; expected result: validation exits 0 with the new capability delta accepted.

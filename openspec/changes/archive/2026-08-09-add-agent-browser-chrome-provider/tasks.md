# Tasks

## 1. Shared browser contract and trusted discovery

- [x] 1.1 Add additive opaque Chrome tab references without changing legacy Firefox tab IDs.
  - touches: `crates/jcode-app-core/src/tool/browser.rs`, `crates/jcode-app-core/src/tool/browser_tests.rs`, `crates/jcode-provider-antigravity/src/lib.rs`
  - depends on: none
  - Done when `tab_ref: string` handles Chrome IDs such as `t1`, existing Firefox `tab_id: integer` callers continue to pass, ambiguous/wrong-provider identifiers return guidance, and raw plus transformed schemas preserve both fields without relying on a string-or-integer union.

- [x] 1.2 Add the `AgentBrowserProvider` module boundary, trusted executable discovery, compatible-version/protocol checks, pinned executable fingerprinting, and Chrome-specific offline-doctor status/setup behavior.
  - touches: `crates/jcode-app-core/src/tool/browser.rs`, `crates/jcode-app-core/src/tool/browser/agent_browser.rs (new)`, `crates/jcode-app-core/src/tool/browser/agent_browser_tests.rs (new)`
  - depends on: 1.1
  - Done when explicit Chrome status distinguishes missing CLI, unsafe PATH candidate, absolute override, insecure writable/repository-local executable, replaced executable, incompatible version, missing JSON capability, unhealthy doctor output, healthy readiness, documented stale-sidecar cleanup, no-op setup, and explicit browser-runtime installation without invoking a package manager.

- [x] 1.3 Implement collision-resistant session naming, neutral config/cwd creation, environment allowlisting, and per-session Chrome action serialization.
  - touches: `crates/jcode-app-core/src/tool/browser/agent_browser.rs (new)`, `crates/jcode-app-core/src/tool/browser/agent_browser_tests.rs (new)`
  - depends on: 1.2
  - Done when tests prove `jcode-<readable-prefix>-<stable-hash>` uniqueness for punctuation, Unicode, empty, long, and normalized-collision IDs; the child environment clears inherited `AGENT_BROWSER_*`; hostile config/env cannot set profile, state, session, auto-connect, CDP, extensions, provider/engine, executable, init scripts, remote provider, or auth behavior; commands run from a Jcode-owned runtime directory with a neutral mode-0600 config; and same-session actions are serialized.

## 2. Agent-browser execution adapter

- [x] 2.1 Implement bounded direct subprocess execution with argument arrays, stdin batch support, hard timeout clamps, bounded stdin/stdout/stderr streaming, child kill/reap, JSON envelope parsing, and safe errors.
  - touches: `crates/jcode-app-core/src/tool/browser/agent_browser.rs (new)`, `crates/jcode-app-core/src/tool/browser/agent_browser_tests.rs (new)`
  - depends on: 1.3
  - Done when fake executables prove successful envelopes, provider errors, nonzero exits, timeouts, malformed/empty output, oversized stdout/stderr/stdin, bounded diagnostics, no shell interpolation, and no unbounded `.output()`-style buffering.

- [x] 2.2 Implement recursive output scrubbing for native command echoes, direct arguments, secrets, scripts, and provider diagnostics.
  - touches: `crates/jcode-app-core/src/tool/browser/agent_browser.rs (new)`, `crates/jcode-app-core/src/tool/browser/agent_browser_tests.rs (new)`
  - depends on: 2.1
  - Done when fake-provider tests prove typed text, form values, select values, scripts, secret-bearing URL components, and batch `command` arrays are absent from rendered tool output, metadata, trace summaries, timeout diagnostics, malformed-output errors, and provider error envelopes.

- [x] 2.3 Map Chrome navigation and inspection actions: list-tabs, new-tab, select-tab via `tab_ref`, active-tab, open, snapshot, get-content, and interactables, including normalized accessibility refs.
  - touches: `crates/jcode-app-core/src/tool/browser/agent_browser.rs (new)`, `crates/jcode-app-core/src/tool/browser/agent_browser_tests.rs (new)`
  - depends on: 2.2
  - Done when each normalized action produces the expected agent-browser argv/stdin and Jcode output shape in fake-provider tests.

- [x] 2.4 Map Chrome interaction actions: selector/ref/text/coordinate click, type, fill-form, select, wait, eval, scroll, upload, and press; reject unsupported frame/window targeting without discarding inputs.
  - touches: `crates/jcode-app-core/src/tool/browser/agent_browser.rs (new)`, `crates/jcode-app-core/src/tool/browser/agent_browser_tests.rs (new)`
  - depends on: 2.2
  - Done when fake-provider tests cover selector and accessibility refs, semantic text lookup, coordinate mouse batches, stdin form batches, submit behavior, waits, iframe refs, stale refs, uploads, keys, and every rejected `list_frames`, `window_id`, `frame_id`, and `all_frames` branch.

- [x] 2.5 Implement Chrome screenshot attachment and cleanup, upload file validation, and Chrome `provider_command` rejection.
  - touches: `crates/jcode-app-core/src/tool/browser/agent_browser.rs (new)`, `crates/jcode-app-core/src/tool/browser/agent_browser_tests.rs (new)`
  - depends on: 2.1
  - Done when successful PNG output from an exclusively created Jcode-owned path is validated, attached without absolute-path disclosure, and removed; missing, substituted, symlinked, non-regular, oversized, and invalid-PNG screenshots fail cleanly; upload rejects missing/symlinked/non-regular/provider-substituted files; and Chrome `provider_command` rejects all raw agent-browser CLI surfaces.

## 3. Routing, parity gate, affinity, and automatic fallback

- [x] 3.1 Wire `browser: "chrome"` to `AgentBrowserProvider` while keeping explicit Firefox requests isolated and attaching selected-provider metadata to all outputs.
  - touches: `crates/jcode-app-core/src/tool/browser.rs`, `crates/jcode-app-core/src/tool/browser_tests.rs`
  - depends on: 1.2, 2.3, 2.4, 2.5
  - Done when explicit-provider tests prove Chrome and Firefox never cross-fall back, Chrome uses the pinned trusted executable, unsupported Safari/Edge values retain actionable errors, and Chrome provider-command requests are rejected.

- [x] 3.2 Add and pass an ignored explicit-Chrome localhost parity test before enabling automatic fallback.
  - touches: `crates/jcode-app-core/src/tool/browser/agent_browser_tests.rs (new)`
  - depends on: 3.1
  - Verification recipe: run `agent-browser --json doctor --offline` and confirm `summary.fail` is `0`, then run `JCODE_AGENT_BROWSER_LIVE=1 cargo test -p jcode-app-core agent_browser_provider_live_smoke -- --ignored --nocapture`; expected result: explicit Chrome completes open, snapshot, fill, click, content read, select, tabs, same-origin and cross-origin iframe refs where supported, screenshot, cookies, local storage, session storage, tab separation, active-tab separation, collision-prone two-session isolation, and close against deterministic localhost pages, and no smoke session remains live.

- [x] 3.3 Add sticky readiness-aware `auto` selection with session affinity, a short-lived invalidated cache, healthy Firefox priority, healthy Chrome fallback only after task 3.2 passes, combined diagnostics when neither provider is ready, non-installing automatic status, and Firefox-first automatic setup semantics.
  - touches: `crates/jcode-app-core/src/tool/browser.rs`, `crates/jcode-app-core/src/tool/browser_tests.rs`, `crates/jcode-app-core/src/tool/browser/agent_browser.rs (new)`, `crates/jcode-app-core/src/tool/browser/agent_browser_tests.rs (new)`
  - depends on: 3.2
  - Done when tests cover every readiness combination, cache reuse and invalidation, executable-fingerprint invalidation, selected backend metadata, fallback reason, session-affinity reuse, affinity-provider failure without silent migration, documented stale-sidecar cleanup, no installer while either provider is ready, and explicit Chrome setup as the only Chrome runtime installation path.

## 4. Final verification and documentation

- [x] 4.1 Complete deterministic fake-agent-browser coverage for discovery trust, replacement, compatible-version checks, setup, command mapping, `tab_ref` schema compatibility, collision-resistant session names, neutral config/cwd, cleared inherited settings, action serialization, explicit and automatic routing, affinity/cache behavior, redaction, timeout, output limits, malformed output, screenshots, uploads, unsupported targeting, Chrome provider-command rejection, and forbidden profile attachment.
  - touches: `crates/jcode-app-core/src/tool/browser/agent_browser_tests.rs (new)`, `crates/jcode-app-core/src/tool/browser_tests.rs`, `crates/jcode-provider-antigravity/src/lib.rs`
  - depends on: 3.3
  - Done when the browser-focused test filter executes all deterministic Chrome and Firefox cases without requiring a real browser.

- [x] 4.2 Update the README and browser-provider protocol to list Firefox and Chrome as wired providers, explain explicit and sticky automatic routing, document optional agent-browser setup, trusted executable discovery, stable `tab_ref` IDs, iframe-ref behavior, neutral configuration, cleared environment, doctor sidecar cleanup, idle timeout, and profile-isolation guarantees.
  - touches: `README.md`, `docs/BROWSER_PROVIDER_PROTOCOL.md`
  - depends on: 3.3
  - Done when public documentation matches the implemented status/setup and routing behavior without claiming unsupported frame/window, provider-command, or daily-profile features.

- [x] 4.3 Run formatting and the deterministic browser-provider test gate.
  - touches: `crates/jcode-app-core/src/tool/browser.rs`, `crates/jcode-app-core/src/tool/browser_tests.rs`, `crates/jcode-app-core/src/tool/browser/agent_browser.rs (new)`, `crates/jcode-app-core/src/tool/browser/agent_browser_tests.rs (new)`, `crates/jcode-provider-antigravity/src/lib.rs`
  - depends on: 4.1, 4.2
  - Verification recipe: run `cargo fmt --all -- --check && cargo test -p jcode-app-core browser`; expected result: both commands exit 0 and all existing Firefox plus new Chrome deterministic tests pass.

- [x] 4.4 Rerun the real Chrome localhost parity test against the final implementation bytes.
  - touches: `crates/jcode-app-core/src/tool/browser/agent_browser_tests.rs (new)`
  - depends on: 4.3
  - Verification recipe: run `agent-browser --json doctor --offline` and confirm `summary.fail` is `0`, then run `JCODE_AGENT_BROWSER_LIVE=1 cargo test -p jcode-app-core agent_browser_provider_live_smoke -- --ignored --nocapture`; expected result: both commands exit 0, all live navigation, interaction, screenshot, isolation, and cleanup checks pass, and `agent-browser --json session list` contains no live smoke sessions.

- [x] 4.5 Run strict feature and repository validation.
  - touches: `openspec/changes/add-agent-browser-chrome-provider/proposal.md`, `openspec/changes/add-agent-browser-chrome-provider/design.md`, `openspec/changes/add-agent-browser-chrome-provider/specs/chrome-browser-provider/spec.md`, `openspec/changes/add-agent-browser-chrome-provider/tasks.md`
  - depends on: 4.3, 4.4
  - Verification recipe: run `openspec validate add-agent-browser-chrome-provider --strict --no-interactive`; expected result: command exits 0 with the proposal, design, capability delta, and task checklist accepted.

- [x] 4.6 Review and persist only the owned implementation, tests, docs, and OpenSpec paths.
  - touches: `crates/jcode-app-core/src/tool/browser.rs`, `crates/jcode-app-core/src/tool/browser_tests.rs`, `crates/jcode-app-core/src/tool/browser/agent_browser.rs (new)`, `crates/jcode-app-core/src/tool/browser/agent_browser_tests.rs (new)`, `crates/jcode-provider-antigravity/src/lib.rs`, `README.md`, `docs/BROWSER_PROVIDER_PROTOCOL.md`, `openspec/changes/add-agent-browser-chrome-provider/proposal.md`, `openspec/changes/add-agent-browser-chrome-provider/design.md`, `openspec/changes/add-agent-browser-chrome-provider/specs/chrome-browser-provider/spec.md`, `openspec/changes/add-agent-browser-chrome-provider/tasks.md`
  - depends on: 4.5
  - Done when `git diff --check` exits 0, the reviewed path set contains no unrelated changes, and the containing commit includes the exact validated bytes.

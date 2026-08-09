# Design

## Context

Jcode exposes one normalized `browser` tool but currently has one concrete provider, `FirefoxBridgeProvider`. Provider resolution routes `auto` and `firefox` to a Firefox-specific bridge binary and native extension. The schema advertises Chrome, Safari, and Edge, but those values currently return a not-wired error.

The installed `agent-browser 0.27.3` runtime provides a Chrome-first CLI with isolated named sessions and JSON output. Live probing against a deterministic localhost page confirmed:

- Commands return `{ "success": true, "data": ..., "error": null }` or a structured failure envelope.
- Tabs use stable opaque IDs such as `t1`; positional integers are rejected.
- `snapshot -i` returns an accessibility tree plus stable element refs, including one level of iframe content.
- `open`, `fill`, `click`, `get`, `select`, `tab`, `screenshot`, `batch`, and `close` cover Jcode's required core actions.
- `doctor --json --offline` checks CLI, Chrome, daemons, config, security, and a live launch. It is non-installing but automatically removes stale daemon socket/pid/version sidecars.
- Named `--session` values isolate browser instances and state.
- Stdin batch results echo command arrays and therefore require recursive output redaction.
- Agent-browser normally loads user/project config and many `AGENT_BROWSER_*` variables, so merely omitting profile flags is insufficient isolation.

The adapter must preserve the public Jcode browser contract, keep Firefox behavior intact, avoid untrusted executable discovery and daily-profile attachment, protect workflow continuity, and remain optional on machines without agent-browser.

## Goals / Non-Goals

**Goals:**

- Add a trusted first-class Chrome provider behind the existing browser tool.
- Preserve strict explicit-provider semantics and enable Firefox-first automatic fallback only after explicit Chrome parity succeeds.
- Isolate every Jcode session's browser process, configuration, state, and command ordering.
- Normalize agent-browser output, string tab refs, screenshots, errors, and provider metadata.
- Prevent user-controlled text and native command echoes from reappearing in rendered output, metadata, or bounded diagnostics.
- Bound subprocess, JSON, stdin, screenshot, and file behavior.
- Provide deterministic fake-process tests and an opt-in real Chrome smoke test.

**Non-Goals:**

- Bundle or silently install the npm package when its executable is missing.
- Attach to the user's existing Chrome profile, restore saved auth state, auto-connect to Chrome/CDP, or inherit agent-browser project/user behavior.
- Build a native CDP client in Rust.
- Guarantee Chrome support for every Firefox-specific targeting extension.
- Replace Firefox Agent Bridge or change explicit `browser: "firefox"` behavior.
- Expose agent-browser's raw `provider_command`, auth vault, profiles, remote providers, dashboard, chat, install/upgrade, recording, or cross-session lifecycle controls.
- Add Safari or Edge providers.

## Decisions

### 1. Implement a dedicated provider adapter

Add `tool/browser/agent_browser.rs` with an `AgentBrowserProvider` implementation of `BrowserProvider`. Keep shared input validation, explicit/automatic routing, provider affinity, and common output helpers in `browser.rs`. Keep Chrome-specific executable trust, neutral config, command construction, process execution, readiness, envelope parsing, redaction, and screenshot handling in the adapter module.

**Why:** The provider trait already creates the correct seam. A bounded external-process adapter is smaller and more testable than native CDP.

**Rejected alternatives:**

- Driving `/usr/bin/chromium --headless` directly lacks durable sessions, accessibility refs, interactions, and normalized JSON.
- Implementing CDP directly duplicates mature automation behavior and expands credential/lifecycle scope.

### 2. Trust and pin the executable before use

Resolve the candidate in this order:

1. `JCODE_AGENT_BROWSER_BIN`, which must be an absolute path and represents explicit operator trust.
2. `agent-browser` discovered from `PATH` for explicit Chrome and automatic use only after validation.

Validation canonicalizes the path, requires a regular executable, rejects relative/current-directory and repository-local PATH candidates, rejects group/world-writable files or unsafe writable ancestor chains, and records a fingerprint containing canonical path, platform file identity where available, size, modification time, and supported version.

The supported initial range is `>=0.27.3,<0.28.0`. A candidate outside that range reports installed-but-incompatible. The pinned fingerprint is revalidated before execution; replacement invalidates readiness cache and session affinity instead of running changed bytes.

**Why:** `PATH` lookup is code execution. Automatic fallback must not execute a repository-shadowed or replaced binary merely because Firefox is unavailable.

### 3. Diagnose and set up Chrome explicitly

Chrome status runs the pinned executable's version command, then `doctor --json --offline` with a bounded timeout. Any doctor `fail` check makes readiness false; warnings and informational checks remain diagnostics. Status discloses that doctor may remove only stale daemon sidecars. Full native doctor output is summarized so daily-profile paths and unrelated provider configuration are not exposed as browser tool metadata.

Explicit Chrome setup:

- Missing or untrusted CLI: return install/trust guidance; do not invoke a package manager.
- Compatible CLI with missing Chrome runtime: run `agent-browser install` only because the user explicitly requested Chrome setup, then rerun offline doctor.
- Healthy runtime: return status without installation.

Automatic setup returns immediately if either provider is ready. If neither is ready, it preserves existing compatibility by using Firefox setup; Chrome runtime installation requires explicit `browser: "chrome"`.

### 4. Isolate configuration, sessions, and action ordering

Derive a bounded session name `jcode-<readable-prefix>-<stable-hash>` from the complete Jcode session ID. The readable prefix is sanitized and truncated; the deterministic hash suffix prevents punctuation, Unicode, empty-prefix, and long-ID collisions.

Every Chrome command receives:

- `--session <derived-name>` and `--json`.
- `--config <runtime-dir>/agent-browser-jcode.json`, an atomically written mode-0600 neutral config.
- A neutral runtime working directory rather than the repository cwd.
- A child environment that removes all inherited `AGENT_BROWSER_*` values, then sets only Jcode-controlled values such as `AGENT_BROWSER_IDLE_TIMEOUT_MS=1800000`.

The adapter never supplies profile, state, session-name persistence, auto-connect, CDP, extension, init-script, engine/provider override, executable override, or remote-provider settings. Standard locale and explicitly allowed network proxy variables may remain.

Use a per-Jcode-session async mutex for Chrome actions so active-tab and ref-dependent commands cannot race. Maintain a bounded, expiring session-affinity map:

- Any successful non-status/setup action records the actual provider used.
- Later `auto` actions reuse that provider, preserving tabs and refs even if another provider recovers.
- A later successful explicit action may intentionally replace affinity.
- Provider failure returns an error and alternate-provider status; it does not silently migrate the workflow.
- Affinity expires with inactivity and is cleared when executable fingerprint changes.

Agent-browser's controlled idle timeout reclaims abandoned daemons. Immediate Jcode disconnect cleanup is out of scope because current reload/successor classification is broader than this provider and could close a resumable browser session.

### 5. Execute bounded subprocesses without a shell

Spawn with `tokio::process::Command` and argument arrays. Use piped stdin for structured batch requests and captured streaming readers for output. No shell command is constructed.

Initial hard limits:

- Normal action timeout: 30 seconds; user timeout clamped to 1–120 seconds.
- Explicit browser installation timeout: 10 minutes.
- Stdin JSON: 1 MiB.
- Stdout: 4 MiB.
- Stderr: 256 KiB.
- Screenshot: 25 MiB and at most 16,384 pixels on either axis.

Readers stop and fail when a limit is exceeded rather than buffering the full stream. On timeout/overflow, kill and reap the child. Provider daemons are controlled through agent-browser session/idle behavior, not by killing unrelated descendants.

### 6. Map normalized actions

| Jcode action | Agent-browser operation |
| --- | --- |
| `list_tabs` | `tab list` |
| `new_tab` | stdin batch `tab new [url]` |
| `select_tab` | `tab <tab_ref>` |
| `get_active_tab` | `tab list`, select `active: true` |
| `open` | stdin batch `open <url>`; `tab new` when `new_tab=true` |
| `snapshot` | `snapshot` and normalize accessibility refs |
| `get_content` | `get text`, `get html`, `get title`, or snapshot text by format |
| `interactables` | `snapshot -i` |
| `click` | stdin batch selector/ref click, semantic text locator, or mouse move/down/up for coordinates |
| `type` | stdin batch type/fill/keyboard plus optional Enter |
| `fill_form` | stdin `batch --bail` with fill/check/uncheck operations |
| `select` | stdin batch select |
| `wait` | stdin batch selector/ref, `--text`, or predicate wait |
| `screenshot` | `screenshot <exclusive-temp-path>` |
| `eval` | stdin batch eval |
| `scroll` | stdin batch scroll, scrollintoview, or mouse wheel |
| `upload` | stdin batch upload after local file validation |
| `press` | stdin batch optional focus then press |
| `provider_command` | unsupported for Chrome |

Use stdin batch for all operations containing user-controlled URLs, text, values, scripts, keys, selectors, or file paths where agent-browser supports it, avoiding process-list exposure. Strip native batch `command` arrays before normalization.

`list_frames` is explicitly unsupported. Chrome snapshots inline one iframe level and permit refs within supported frames. `window_id`, `frame_id`, and `all_frames` combinations that cannot be honored fail rather than being ignored.

### 7. Add string `tab_ref` without changing legacy `tab_id`

Keep `tab_id: integer` for Firefox and existing provider-dialect compatibility. Add `tab_ref: string` for opaque IDs/labels. Chrome requires `tab_ref`; Firefox continues accepting `tab_id`. Supplying both or the wrong form returns provider-specific guidance.

**Why:** Some model-provider schema transformers collapse `oneOf`, so an additive field is safer than changing `tab_id` into a union.

### 8. Normalize output and recursively redact sensitive inputs

Parse agent-browser success/error envelopes and expose normalized `data` after adding `backend: "agent_browser"` and `browser: "chrome"`. Treat `success: false`, nonzero exits, timeouts, limit violations, malformed JSON, and missing mandatory output as provider errors.

Build a sensitive-value set from typed text, form values, select values, scripts, and secret-bearing URL components. Before constructing tool output, metadata, trace summaries, or errors:

- Remove batch `command` fields recursively.
- Replace exact sensitive values in nested JSON, stdout, and stderr with `<redacted>`.
- Bound diagnostic text and avoid absolute local paths in user-visible labels.

Normalize common shapes:

- Snapshot: `{ content, refs, origin }`.
- Content: `{ text }`, `{ html }`, or `{ title, url }` with secret-bearing URL components redacted.
- Interactables: accessibility snapshot plus refs.
- Tabs: preserve `tabId` and active state, exposing the ID as `tab_ref` to Jcode callers.
- Eval: `{ result, type }` where safe.

### 9. Validate screenshot and upload files

Create screenshot paths exclusively in Jcode's runtime directory. Accept only the exact expected returned path. Open without following symlinks, require a regular file, enforce size and PNG magic/dimension limits, attach through `ToolOutput::with_labeled_image`, label it generically, and remove it on every exit path.

Uploads require a canonical regular file and no symlink. Preserve existing user-authorized path scope while rejecting nonregular, missing, or provider-substituted files.

### 10. Add sticky, parity-gated automatic selection

Before activating automatic Chrome fallback, the explicit Chrome localhost parity task must pass.

Automatic initial selection:

1. Reuse existing session affinity if present and healthy.
2. Otherwise choose ready Firefox first.
3. Otherwise choose ready trusted Chrome.
4. Otherwise return combined bounded diagnostics.

Readiness results use a short TTL and are invalidated by setup, execution failure, affinity reset, or executable fingerprint change. Status reports both providers and the provider that would be selected but does not establish affinity. Doctor's only permitted status-side maintenance is documented stale-sidecar cleanup.

### 11. Verify with fake and live providers

Fake executable tests record argv, stdin, cwd, and environment and emit controlled streams/files. They cover trust validation and replacement, version range, neutral config, cleared environment, session collisions, serialization, affinity, cache invalidation, action mappings, tab fields, errors, redaction, limits, screenshot/upload validation, unsupported targets, and Chrome provider-command rejection.

The ignored live test runs only with `JCODE_AGENT_BROWSER_LIVE=1`. It serves deterministic same-origin and cross-origin localhost pages and exercises open, snapshot, fill, click, content read, select, tabs, one-level iframe refs, stale refs after navigation, screenshot, cookies, local/session storage, tab/active-tab separation, collision-prone session IDs, and close. It asserts no live smoke sessions remain.

Existing Firefox tests and transformed-schema tests remain in the browser gate.

## Risks / Trade-offs

- **External CLI drift** → Support a narrow initial version range, pin the executable fingerprint, parse defensively, and update intentionally.
- **Untrusted PATH shadowing** → Reject relative, repository-local, insecurely writable, and replaced candidates before auto use.
- **Automatic readiness adds latency** → Cache briefly and keep session affinity sticky.
- **Automatic fallback loses workflow state** → Choose only at session start/explicit override; never silently migrate after provider failure.
- **User config or environment attaches credentials/extensions** → Use neutral config/cwd and clear all inherited `AGENT_BROWSER_*` variables.
- **Typed values leak through argv/errors** → Use stdin batches and recursive known-value redaction.
- **Oversized output/files exhaust memory** → Stream with hard limits and validate files before loading.
- **String IDs break provider schemas** → Add `tab_ref` instead of changing integer `tab_id`, and test transformed schemas.
- **Iframe behavior differs from Firefox** → Test documented one-level refs and fail unsupported targeting explicitly.
- **Browser daemons outlive Jcode** → Set provider idle timeout; avoid risky disconnect coupling in this change.

## Migration Plan

1. Land trusted executable discovery, neutral configuration, additive `tab_ref`, explicit Chrome routing, bounded adapter behavior, and fake tests.
2. Pass the explicit Chrome localhost parity gate.
3. Activate sticky Firefox-first automatic fallback and rerun deterministic tests.
4. Rerun final live parity, then update README and provider protocol documentation.
5. Roll back by removing Chrome resolution and `tab_ref` use while retaining Firefox; no user data migration is required.

## Open Questions

None. The user approved explicit Chrome routing, isolated sessions, secret-safe subprocess behavior, Firefox-first automatic fallback after parity verification, and the verification contract. Independent review findings about trust, config inheritance, workflow continuity, schema dialects, subprocess bounds, and file safety are resolved in this design.

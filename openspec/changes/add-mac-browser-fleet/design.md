# Design

## Context

Jcode's server and current Chrome provider execute on the homelab Linux machine. The existing SSH setup forwards Jcode's Unix socket to the Mac, but that direction does not make Mac browser processes visible to homelab tools. Chrome and Edge ordinary user sessions also do not expose CDP unless launched with debugging enabled, and attaching directly to daily profile directories would be unsafe.

The approved policy is read-only discovery by default, confirmation-gated mutations, optional Mac-approved autonomy leases, immutable hard denies, and a Mac-local emergency stop. The Mac must remain the final authority even if the homelab process or an agent session is compromised.

## Goals / Non-Goals

**Goals:**

- Discover active supported Chrome and Edge browsers, windows, profiles, and tabs on the Mac.
- Let homelab Jcode inspect and steer selected Mac tabs through the normalized browser tool.
- Keep all policy enforcement, approval decisions, credentials, and kill controls on the Mac.
- Reuse private SSH Unix-socket forwarding with authenticated, bounded messages and automatic reconnect.
- Support ordinary user-launched tabs through an extension/native host and richer managed-browser control through CDP.
- Preserve existing local Firefox and Chrome behavior and require explicit remote routing.

**Non-Goals:**

- Safari support.
- Reading browser password stores, cookies, authentication tokens, incognito tabs, or extension internals.
- Making arbitrary daily browser sessions debuggable by relaunching them.
- Public TCP listeners, cloud relays, or direct inbound access to the Mac.
- Allowing homelab agents to edit Mac policy, approve their own actions, or disable the emergency stop.
- Pixel-perfect parity between extension-backed ordinary tabs and CDP-managed instances.

## Decisions

### 1. Use a hybrid extension/native-host and CDP architecture

A signed or locally installed Manifest V3 extension runs in Chrome and Edge and reports ordinary tab/window inventory through a native messaging host. It executes the subset of actions browser extension APIs can faithfully support. Separately, the broker discovers explicitly managed CDP endpoints and uses them for accessibility snapshots, richer evaluation, and capabilities unavailable to extensions.

The broker returns per-target capabilities so Jcode rejects unsupported operations rather than silently approximating them.

**Rejected alternatives:** CDP-only cannot attach to ordinary already-running browsers. Extension-only cannot provide full debugging semantics. Relaunching daily profiles under CDP risks profile corruption and credential exposure.

### 2. Make the Mac broker the security boundary

The broker listens only on a mode-0600 Unix socket. It authenticates the forwarded peer with a Mac-generated secret stored in Keychain or a mode-0600 file and uses protocol version negotiation. Every request includes a unique ID, target generation, action, declared sensitivity, deadline, and bounded payload.

Read-only inventory operations are allowed by default. Mutations enter the approval engine unless covered by a valid Mac-issued lease. Hard-denied categories cannot be leased or overridden remotely: password managers, browser settings, extensions, downloads pages, payment and banking confirmation, account-security changes, authentication/recovery settings, incognito, and browser-internal privileged URLs.

### 3. Use scoped, expiring capability leases

The Mac UI can approve one action or issue a lease scoped by browser, profile, tab, origin, action set, and expiration. The default autonomy shortcut lasts 15 minutes and excludes every hard deny. Leases are held only by the broker, expire against monotonic time, are revoked on broker restart, target generation change, emergency stop, or policy reload, and are never writable from the homelab.

### 4. Add explicit remote fleet routing to Jcode

Extend the browser tool with a route such as `browser: "mac"` plus optional `browser_ref`, `window_ref`, and `tab_ref`. Existing `browser: "chrome"` continues to mean homelab-local agent-browser Chrome, including explicit profiles. `auto` does not silently move a workflow between local and remote providers.

Fleet list/status results expose browser kind, display name, profile label when available, window/tab refs, title, redacted URL, active state, connection health, capability set, policy state, and target generation. Sensitive URL components and all credentials are scrubbed.

### 5. Reuse SSH with reverse local-socket forwarding

The Mac broker owns a local socket. The existing persistent Mac-to-homelab SSH connection adds a reverse stream-local forward that makes the broker available at a mode-restricted homelab runtime path. Jcode connects only to that forwarded socket. No listener binds to LAN or Tailscale TCP interfaces.

Socket loss marks targets unavailable, fails in-flight mutations without retry, and reconnects with bounded exponential backoff. Read-only inventory may retry after reconnect. Mutations require a fresh target generation and are never automatically replayed.

### 6. Separate discovery from steering

The broker continuously maintains a bounded inventory and emits generation-tagged deltas. Discovery does not imply permission to inspect page content. Listing tabs may reveal only policy-filtered titles and origins. Snapshot/content retrieval is classified separately from topology inventory and can be configured as read-only or approval-required by Mac policy.

### 7. Package Mac lifecycle through Jcode setup

Add an explicit setup command that installs or refreshes the broker executable, launch-agent plist, native-host manifests for Chrome and Edge, extension assets/instructions, policy defaults, and SSH-forwarding guidance. Setup is idempotent and backs up replaced operator-edited files. Uninstall disables the launch agent and removes Jcode-owned artifacts without touching browser profiles.

## Risks / Trade-offs

- **Extension installation requires user/browser approval** → setup reports exact incomplete steps and fleet status remains truthful until both browsers connect.
- **Browser extension APIs differ from CDP** → advertise per-target capabilities and fail unsupported operations explicitly.
- **Tab metadata can be sensitive** → redact URL userinfo/query/fragment, allow title/origin hiding, and keep logs metadata-only.
- **SSH reconnection can duplicate actions** → never replay mutations; bind actions to request IDs and target generations.
- **A compromised extension could broaden access** → minimal permissions, explicit host grants, native-host allowlist, signed messages, and broker-side policy enforcement.
- **Approval fatigue** → scoped one-action approvals and short leases while hard denies remain immutable.
- **Edge may not be installed** → report `not_installed` without degrading Chrome operation.

## Migration Plan

1. Land protocol types, policy engine, deterministic tests, and Jcode remote provider behind an opt-in setting.
2. Land the Mac broker, extension/native host, setup command, and launch-agent lifecycle.
3. Install on the Mac, establish the reverse Unix-socket forward, and verify read-only inventory.
4. Enable confirmation-gated steering and run real Chrome and Edge acceptance tests.
5. Keep local providers as the rollback path. Disable the fleet setting and unload the Mac launch agent to roll back.

## Open Questions

None. Safe defaults are explicit: read-only inventory, confirmation-gated mutations, 15-minute maximum default lease, immutable hard denies, explicit remote routing, and no public listener.

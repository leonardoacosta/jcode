---
name: local-chrome-agent-targets
description: Use the homelab-local, SOCKS-routed Chrome targets chrome_bbadmin and chrome_o365 for explicitly requested authenticated browser work. Trigger when the user asks to drive BB-admin or O365 Chrome, Azure Portal through the named local profile, local Chrome target selection, or LAN browser access through the homelab SOCKS connection. Do not use for extracting cookies, bearer tokens, refresh tokens, or browser profile data.
---

# Local Chrome Agent Targets

Use only when the user explicitly requests a named target. The ordinary browser remains the default.

## Targets

- `chrome_bbadmin`: persistent BB-admin browser profile.
- `chrome_o365`: persistent O365 browser profile.

Each target has an isolated profile directory under `~/.local/share/jcode/` and must never reuse the default Chrome profile or the other target's profile.

## Jcode browser usage

Pass the named target on every browser action that should use it:

```json
{"action":"status","browser":"chrome_bbadmin"}
{"action":"open","browser":"chrome_bbadmin","url":"https://portal.azure.com"}
{"action":"snapshot","browser":"chrome_bbadmin"}
```

Use `chrome_o365` instead when the requested account boundary is O365. Do not pass a separate `profile` value with a named target. The target owns its profile path, CDP endpoint, lifecycle, and proxy policy.

## Security contract

- Traffic must use the configured local SOCKS endpoint. The launcher/provider fails closed when the proxy is unavailable.
- CDP must remain bound to loopback only.
- Never pass `--user-data-dir`, `--proxy-server`, remote CDP binding, or proxy-bypass overrides through the browser tool.
- Never read or export cookies, local storage, browser session databases, bearer tokens, refresh tokens, passwords, or profile files.
- Do not use the browser profile as a token vault. For Azure CLI or REST access, use the normal Azure CLI OAuth/token-cache flow and keep token output local and protected.
- Account-affecting actions such as deleting resources, changing permissions, sending mail, or modifying tenant configuration require explicit user confirmation immediately before the action.

## Operating workflow

1. Confirm the user explicitly named `chrome_bbadmin` or `chrome_o365`.
2. Check `browser` status for that target.
3. Attach or launch through the normalized browser tool. Do not invoke raw Chrome flags from a skill.
4. Navigate and inspect using normal browser actions.
5. Before an account-affecting action, summarize the exact action and obtain confirmation.
6. Report authentication state only from visible page behavior. Do not inspect hidden credentials.

## Failure handling

- If the target is unavailable, report whether the failure is proxy readiness, profile lock, CDP readiness, or browser runtime setup.
- Do not silently fall back from a named target to the ordinary browser or another account.
- If a target is already authenticated, do not claim that this authorizes CLI access. Browser authorization and CLI token-cache authorization are separate OAuth client contexts.

## Human terminal aliases

The homelab also exposes:

```bash
chrome_bbadmin
chrome_o365
```

These aliases call the shared local launcher, preserve the isolated profile and SOCKS policy, and reject security-sensitive overrides. They are operator entry points; skills should use the normalized browser interface instead.

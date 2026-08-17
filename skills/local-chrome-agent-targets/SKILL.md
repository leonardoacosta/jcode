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

Use the named target's normalized browser interface for navigation. The initial URL is not a routing control: once Chrome is launched with the target policy, all tabs and later navigations in that Chrome process inherit the same isolated profile, loopback CDP, and SOCKS proxy settings.

## Authentication and runtime state

- Visible `Sign out` controls or an authenticated Microsoft 365 shell prove that the browser profile has a visible authenticated session. They do not authorize Azure CLI or REST calls.
- A Microsoft app can be authenticated and still fail during application initialization. For example, Teams may show `CREATE_USER_CONTEXT_FAILED_GENERIC` while also showing `Sign out`. Report those as separate states: authenticated session present, application context failed.
- If status succeeds but target actions fail, check whether the owned Chrome process exited. Relaunch through the named wrapper or normalized provider. Do not attach to an unrelated Chrome process or silently fall back.
- Do not infer SOCKS routing from page appearance. Verify the live process launch policy and SOCKS readiness. Chrome does not display a general SOCKS indicator in page content.


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

- If the target is unavailable, report whether the failure is proxy readiness, profile lock, CDP readiness, browser runtime setup, or an exited owned process.
- Do not silently fall back from a named target to the ordinary browser or another account.
- If a target is already authenticated, do not claim that this authorizes CLI access. Browser authorization and CLI token-cache authorization are separate OAuth client contexts.

## Human terminal aliases

The homelab also exposes:

```bash
chrome_bbadmin
chrome_o365
```

These aliases call the shared local launcher, preserve the isolated profile and SOCKS policy, and reject security-sensitive overrides. They are operator entry points; skills should use the normalized browser interface instead.

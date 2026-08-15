# O365 Local Chrome Manual Sign-In Readiness

Status: live readiness note
Date: 2026-08-15
Scope: `chrome_o365` named local Chrome target only

## Security boundary

This readiness process never automates credentials, login forms, MFA prompts, cookies, browser local storage, profile databases, bearer tokens, refresh tokens, or password-manager contents.

Use the normalized browser target only:

```json
{"action":"status","browser":"chrome_o365"}
{"action":"open","browser":"chrome_o365","url":"https://www.office.com/"}
{"action":"snapshot","browser":"chrome_o365"}
```

Do not pass a separate `profile` value and do not launch Chrome manually with `--user-data-dir`, `--proxy-server`, remote CDP binding, or proxy-bypass overrides.

## Current inspected state

Observed on 2026-08-15 through the Jcode browser provider:

- Provider: ready through `agent-browser 0.34.0`.
- Target: `chrome_o365`.
- Launch behavior: target process is owned by the named launcher and uses the persistent O365 profile.
- The initial page is not a routing control. All later tabs and navigations in this Chrome process inherit the same SOCKS proxy policy.
- Visible authentication state: authenticated session confirmed after manual sign-in. The Teams page exposed a visible `Sign out` control.
- Visible recovery attempt: clicking Teams `Restart` did not clear `CREATE_USER_CONTEXT_FAILED_GENERIC`; the page continued to show both the error and `Sign out`.
- Browser-origin route probe: navigating this target to `https://api.ipify.org/` returned the same address as `curl --proxy socks5h://127.0.0.1:1080 https://api.ipify.org`, and differed from the direct no-proxy result. This is end-to-end evidence that the O365 Chrome process used the SOCKS route for that navigation.
- LAN hostname probe: navigating the target to `http://homelab/` reached a certificate warning for the `homelab` host (`NET::ERR_CERT_AUTHORITY_INVALID`) rather than a DNS or connection failure. This proves the hostname resolved and the target reached the local service path through the browser, but certificate validation prevented page content inspection.

## Manual sign-in readiness checklist

Before asking the human operator to sign in manually, verify these checks from visible behavior only:

1. `browser.status` for `chrome_o365` reports the provider is ready.
2. Opening `https://www.office.com/` with `browser: "chrome_o365"` succeeds without falling back to another browser target.
3. The resulting page is a Microsoft-owned HTTPS origin such as `https://m365.cloud.microsoft/`, `https://www.office.com/`, or `https://login.microsoftonline.com/`.
4. The visible page either shows a `Sign in` control or an authenticated Microsoft 365 shell. If it shows a sign-in form, stop and let the human operate it.
5. No agent step types into email, password, OTP, MFA, passkey, recovery, or consent fields.
6. No agent step reads cookies, local storage, browser profile files, credential stores, token caches, request authorization headers, or password-manager data.
7. Any account-affecting action after sign-in requires explicit user confirmation immediately before the action.

## Post-manual sign-in visible smoke check

After the human signs in, an agent may perform only visible readiness checks:

1. Reopen `https://www.office.com/` with `browser: "chrome_o365"`.
2. Confirm a visible Microsoft 365 shell, app navigation, or `Sign out` control.
3. Treat application initialization errors separately. A page may show `Sign out` and still report an application error such as `CREATE_USER_CONTEXT_FAILED_GENERIC`.
4. If status succeeds but actions fail, check whether the owned Chrome process exited and relaunch through the named target. Do not attach to an unrelated browser.
5. Verify the live process launch policy and SOCKS listener when routing evidence is needed. Do not infer routing from page appearance.
6. Record only page title, URL origin, and visible UI labels needed to prove readiness.
7. Do not record account identifiers unless the user explicitly asks and the identifier is already visible on the page.

## Failure classification

Report one of these states without attempting credential work:

- `provider-unavailable`: `browser.status` is not ready for `chrome_o365`.
- `target-launch-failed`: the named target cannot open a page.
- `auth-required`: Microsoft page loads and asks for sign-in.
- `manual-mfa-required`: Microsoft page asks for MFA, passkey, device approval, or similar human-only proof.
- `signed-in-visible`: Microsoft 365 shell or app navigation is visible after human sign-in.
- `unexpected-origin`: navigation lands on a non-Microsoft origin or an interstitial not expected for Microsoft 365.

## Latest readiness finding

`chrome_o365` is operational as a named target and the persistent profile has a visible authenticated Microsoft 365 session, confirmed by the Teams `Sign out` control. A browser-origin external-IP probe matched the SOCKS path and differed from the direct path. A browser navigation to `homelab` reached the local hostname but stopped at its untrusted-certificate warning. Teams still reports `CREATE_USER_CONTEXT_FAILED_GENERIC` after a visible `Restart` attempt, so authentication and proxy routing are ready but Teams application context is not. External VNC client authentication and rotated-key cleanup remain separate open checks.
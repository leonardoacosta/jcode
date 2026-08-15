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
- Launch behavior: reused a Chrome target owned by the named target launcher.
- URL after navigation to `https://www.office.com/`: `https://m365.cloud.microsoft/`.
- Page title: `Microsoft 365 Copilot - Sign in`.
- Visible authentication state: signed out. The page exposes a visible `Sign in` button and public Microsoft 365 marketing/app entry points.
- Credentials/login automation: not attempted.
- Hidden browser state: not inspected.

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
2. Confirm the page no longer presents the primary signed-out marketing state.
3. Confirm visible Microsoft 365 app navigation is present, such as Outlook, Word, Excel, PowerPoint, OneDrive, Teams, or Copilot app launch controls.
4. Record only page title, URL origin, and visible UI labels needed to prove readiness.
5. Do not record account identifiers unless the user explicitly asks and the identifier is already visible on the page.

## Failure classification

Report one of these states without attempting credential work:

- `provider-unavailable`: `browser.status` is not ready for `chrome_o365`.
- `target-launch-failed`: the named target cannot open a page.
- `auth-required`: Microsoft page loads and asks for sign-in.
- `manual-mfa-required`: Microsoft page asks for MFA, passkey, device approval, or similar human-only proof.
- `signed-in-visible`: Microsoft 365 shell or app navigation is visible after human sign-in.
- `unexpected-origin`: navigation lands on a non-Microsoft origin or an interstitial not expected for Microsoft 365.

## Latest readiness finding

`chrome_o365` is operational as a named target but not signed in. It is ready for a human-only manual O365 sign-in. The next agent action should stop at the visible Microsoft sign-in prompt and wait for the human operator to complete authentication outside agent automation.

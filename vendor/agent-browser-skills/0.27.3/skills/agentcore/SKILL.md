---
name: agentcore
description: Run agent-browser against AWS Bedrock AgentCore managed cloud browsers with SigV4 credentials, region and browser selection, persistent profiles, session timeouts, and Live View inspection. Use when browser automation must run on AWS infrastructure, the user asks for AgentCore or an AWS-hosted cloud browser, CI uses AWS credentials, or a managed remote browser session is required. Triggers include agentcore, AWS Bedrock AgentCore, bedrock browser, AWS cloud browser, agentcore session, AGENTCORE_REGION, AGENTCORE_PROFILE_ID, and `agent-browser -p agentcore`.
allowed-tools: Bash(agent-browser:*), Bash(npx agent-browser:*)
---

# AWS Bedrock AgentCore

Use the AgentCore provider when execution must occur in an AWS-managed browser rather than a local browser. The interaction commands remain standard agent-browser commands. Provider selection, AWS credential resolution, session lifecycle, and profile persistence are the AgentCore-specific parts.

## Decide the Session Shape First

| Need | Configuration | Important consequence |
|---|---|---|
| One ad hoc AWS browser session | Add `-p agentcore` to the opening command | Subsequent commands operate on that active agent-browser session |
| Every command in a script should use AgentCore | Set `AGENT_BROWSER_PROVIDER=agentcore` | Avoids repeating the provider flag |
| Reuse cookies and localStorage later | Set a stable `AGENTCORE_PROFILE_ID` before opening | Browser state persists across sessions |
| Isolate an untrusted or one-off task | Leave `AGENTCORE_PROFILE_ID` unset | Prevents the task from sharing a persistent profile |
| Use a non-default AWS region | Set `AGENTCORE_REGION` before opening | The session is created in that region |
| Run longer than one hour | Increase `AGENTCORE_SESSION_TIMEOUT` before opening | The default timeout is 3600 seconds |

Choose region, browser ID, profile, and timeout before `open`. These values shape the remote session being created.

## Credential Gate

AgentCore resolves credentials in this order:

1. `AWS_ACCESS_KEY_ID` and `AWS_SECRET_ACCESS_KEY`, plus `AWS_SESSION_TOKEN` when using temporary credentials.
2. AWS CLI fallback via `aws configure export-credentials`. This supports SSO, IAM roles, and named profiles selected with `AWS_PROFILE`.

Before starting a session, establish which path is intended:

```bash
# Named profile or SSO
aws sso login --profile my-profile
AWS_PROFILE=my-profile agent-browser -p agentcore open https://example.com

# Existing default credential chain
agent-browser -p agentcore open https://example.com
```

For CI, inject credentials through the CI secret mechanism. Do not place access keys in skill files, source files, shell history examples, screenshots, or logs.

## Core Workflow

```bash
# Create the remote session and navigate
agent-browser -p agentcore open https://example.com

# Interact using the normal snapshot-ref workflow
agent-browser snapshot -i
agent-browser click @e1
agent-browser screenshot page.png

# End the remote session when finished
agent-browser close
```

Take a new interactive snapshot after navigation or any substantial DOM change. Element refs such as `@e1` belong to the current snapshot and may become stale.

## Persistent Profiles

Set `AGENTCORE_PROFILE_ID` only when state should survive `close` and be available to a future session:

```bash
# First session: authenticate interactively
AGENTCORE_PROFILE_ID=my-app agent-browser -p agentcore open https://app.example.com/login
agent-browser snapshot -i
agent-browser fill @e1 "user@example.com"
agent-browser fill @e2 "$APP_PASSWORD"
agent-browser click @e3
agent-browser close

# Later session: reuse the saved browser state
AGENTCORE_PROFILE_ID=my-app agent-browser -p agentcore open https://app.example.com/dashboard
```

Use distinct profile IDs for different users, tenants, trust levels, or test environments. If persisted state is not required, omit the profile ID rather than creating a disposable shared profile.

## Provider-Wide Configuration

For CI or a multi-command script, select the provider through the environment:

```bash
export AGENT_BROWSER_PROVIDER=agentcore
export AGENTCORE_REGION=us-east-2

agent-browser open https://example.com
agent-browser snapshot -i
agent-browser click @e1
agent-browser close
```

Supported AgentCore variables:

| Variable | Purpose | Default |
|---|---|---|
| `AGENTCORE_REGION` | AWS region for the AgentCore endpoint | `us-east-1` |
| `AGENTCORE_BROWSER_ID` | Browser identifier | `aws.browser.v1` |
| `AGENTCORE_PROFILE_ID` | Persistent cookies and localStorage | none |
| `AGENTCORE_SESSION_TIMEOUT` | Session timeout in seconds | `3600` |
| `AWS_PROFILE` | AWS CLI profile used for credential resolution | `default` |

## Live View

When the session starts, AgentCore prints a Live View URL to stderr. Open that AWS Console URL when visual inspection or human supervision is useful. Treat the URL as session-scoped operational data and avoid publishing it in durable logs or reports.

## Failure Triage

| Symptom | Likely cause | Action |
|---|---|---|
| `Failed to run aws CLI` | AWS CLI is unavailable and environment credentials were not resolved | Install/configure the AWS CLI, or provide credentials through the environment |
| Error instructs `aws sso login` | Cached SSO credentials expired | Run `aws sso login --profile <name>`, then retry with the same `AWS_PROFILE` |
| Authentication fails with temporary credentials | Session token is missing or expired | Refresh all three temporary credential variables, including `AWS_SESSION_TOKEN` |
| Session ends during a long task | Configured timeout is too short | Start a new session with a larger `AGENTCORE_SESSION_TIMEOUT` |
| Expected login state is absent | Profile ID was omitted, changed, or isolated by region/account | Reopen with the intended profile configuration and authenticate if necessary |
| Element ref no longer works | The page changed after the snapshot | Run `agent-browser snapshot -i` again and use the new ref |

When diagnosing startup failures, first separate credential resolution from browser interaction. Do not debug selectors until the AgentCore session has opened successfully.

## Never

- **Never expose AWS secrets in commands, screenshots, reports, or committed files.** Use environment injection or an AWS profile.
- **Never assume `AWS_ACCESS_KEY_ID` and `AWS_SECRET_ACCESS_KEY` are sufficient for temporary credentials.** Temporary credentials also require `AWS_SESSION_TOKEN`.
- **Never reuse one persistent profile across unrelated users, tenants, or trust boundaries.** It carries cookies and localStorage across sessions.
- **Never expect a changed region, profile, browser ID, or timeout to retrofit an already-created session.** Close it and open a correctly configured session.
- **Never reuse stale snapshot refs after navigation or major page changes.** Refresh the interactive snapshot first.
- **Never leave a remote session open after the task is complete.** Call `agent-browser close` so the managed session is released cleanly.

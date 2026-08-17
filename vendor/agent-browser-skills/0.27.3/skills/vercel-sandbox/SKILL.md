---
name: vercel-sandbox
description: Run agent-browser and headless Chrome inside Vercel Sandbox microVMs from server-side Vercel applications. Use when implementing isolated browser automation, screenshots, accessibility snapshots, multi-command browser sessions, scheduled checks, or prebuilt Sandbox snapshots in Next.js, SvelteKit, Nuxt, Remix, or Astro. Triggers include Vercel Sandbox, @vercel/sandbox, agent-browser on Vercel, microVM Chrome, serverless browser automation, and AGENT_BROWSER_SNAPSHOT_ID.
allowed-tools: Bash(agent-browser:*), Bash(npx agent-browser:*)
---

# agent-browser in Vercel Sandbox

Use a Vercel Sandbox when browser automation must run from a Vercel server workload without packaging Chrome into the application deployment. The Sandbox is the isolation and process boundary. The agent-browser session persists across `runCommand` calls made in that Sandbox.

## Choose the execution path

| Situation | Path |
|---|---|
| Prototyping or one-off run | Create a `node24` Sandbox and install dependencies at runtime |
| Repeated or production run | Boot from a prebuilt Sandbox snapshot |
| One page read | `open` then `snapshot -i -c` |
| Image output | `open` then `screenshot --json`, then read the returned file |
| Multi-step interaction | Keep every agent-browser command in the same Sandbox |

Before implementing, decide:

1. Is the target URL trusted or validated? A public route that accepts arbitrary URLs creates an SSRF-capable browser service.
2. Is this repeated work? If yes, use a prebuilt Sandbox snapshot instead of installing on every request.
3. What is the cleanup boundary? Create and stop the Sandbox in the same server-side operation.
4. Does the endpoint need authentication, authorization, rate limiting, and a bounded timeout?

## Install the SDK

```bash
pnpm add @vercel/sandbox
```

## Core lifecycle

The fallback branch installs Chromium's Linux libraries, agent-browser, and its browser binary. A configured snapshot skips that setup.

```ts
import { Sandbox } from "@vercel/sandbox";

const CHROMIUM_SYSTEM_DEPS = [
  "nss", "nspr", "libxkbcommon", "atk", "at-spi2-atk", "at-spi2-core",
  "libXcomposite", "libXdamage", "libXrandr", "libXfixes", "libXcursor",
  "libXi", "libXtst", "libXScrnSaver", "libXext", "mesa-libgbm", "libdrm",
  "mesa-libGL", "mesa-libEGL", "cups-libs", "alsa-lib", "pango", "cairo",
  "gtk3", "dbus-libs",
];

function sandboxCredentials() {
  const { VERCEL_TOKEN, VERCEL_TEAM_ID, VERCEL_PROJECT_ID } = process.env;
  if (VERCEL_TOKEN && VERCEL_TEAM_ID && VERCEL_PROJECT_ID) {
    return {
      token: VERCEL_TOKEN,
      teamId: VERCEL_TEAM_ID,
      projectId: VERCEL_PROJECT_ID,
    };
  }
  return {};
}

async function withBrowser<T>(
  use: (sandbox: InstanceType<typeof Sandbox>) => Promise<T>,
): Promise<T> {
  const snapshotId = process.env.AGENT_BROWSER_SNAPSHOT_ID;
  const credentials = sandboxCredentials();
  const sandbox = snapshotId
    ? await Sandbox.create({
        ...credentials,
        source: { type: "snapshot", snapshotId },
        timeout: 120_000,
      })
    : await Sandbox.create({
        ...credentials,
        runtime: "node24",
        timeout: 120_000,
      });

  try {
    if (!snapshotId) {
      await sandbox.runCommand("sh", [
        "-c",
        `sudo dnf clean all && sudo dnf install -y --skip-broken ${CHROMIUM_SYSTEM_DEPS.join(" ")} && sudo ldconfig`,
      ]);
      await sandbox.runCommand("npm", ["install", "-g", "agent-browser"]);
      await sandbox.runCommand("npx", ["agent-browser", "install"]);
    }

    return await use(sandbox);
  } finally {
    await sandbox.stop();
  }
}
```

Keep setup inside `try` so a failed installation still reaches `sandbox.stop()`.

## Common operations

### Accessibility snapshot

Use accessibility snapshots for page understanding and element refs. Re-snapshot after navigation or major DOM changes because refs are page-state-specific.

```ts
export async function snapshotUrl(url: string) {
  return withBrowser(async (sandbox) => {
    await sandbox.runCommand("agent-browser", ["open", url]);
    const result = await sandbox.runCommand("agent-browser", [
      "snapshot", "-i", "-c",
    ]);
    return await result.stdout();
  });
}
```

### Screenshot

With `--json`, agent-browser reports the screenshot file path. Read that file before the Sandbox is stopped.

```ts
export async function screenshotUrl(url: string) {
  return withBrowser(async (sandbox) => {
    await sandbox.runCommand("agent-browser", ["open", url]);

    const result = await sandbox.runCommand("agent-browser", [
      "screenshot", "--json",
    ]);
    const output = JSON.parse(await result.stdout());
    const path = output?.data?.path;
    if (typeof path !== "string") {
      throw new Error("agent-browser did not return a screenshot path");
    }

    const encoded = await sandbox.runCommand("base64", ["-w", "0", path]);
    return (await encoded.stdout()).trim();
  });
}
```

### Multi-step interaction

Use refs from the actual snapshot. Do not hard-code an example ref without verifying the current page state.

```ts
export async function fillAndSubmit(
  url: string,
  fieldRef: string,
  value: string,
  submitRef: string,
) {
  return withBrowser(async (sandbox) => {
    await sandbox.runCommand("agent-browser", ["open", url]);
    await sandbox.runCommand("agent-browser", ["snapshot", "-i"]);
    await sandbox.runCommand("agent-browser", ["fill", fieldRef, value]);
    await sandbox.runCommand("agent-browser", ["click", submitRef]);
    await sandbox.runCommand("agent-browser", [
      "wait", "--load", "networkidle",
    ]);

    const result = await sandbox.runCommand("agent-browser", [
      "snapshot", "-i", "-c",
    ]);
    return await result.stdout();
  });
}
```

## Prebuild a Sandbox snapshot

A Vercel Sandbox snapshot is a saved microVM image. It is not the same as `agent-browser snapshot`, which prints a page accessibility tree.

```ts
import { Sandbox } from "@vercel/sandbox";

async function createBrowserSnapshot(): Promise<string> {
  const sandbox = await Sandbox.create({
    ...sandboxCredentials(),
    runtime: "node24",
    timeout: 300_000,
  });

  try {
    await sandbox.runCommand("sh", [
      "-c",
      `sudo dnf clean all && sudo dnf install -y --skip-broken ${CHROMIUM_SYSTEM_DEPS.join(" ")} && sudo ldconfig`,
    ]);
    await sandbox.runCommand("npm", ["install", "-g", "agent-browser"]);
    await sandbox.runCommand("npx", ["agent-browser", "install"]);

    const snapshot = await sandbox.snapshot();
    return snapshot.snapshotId;
  } finally {
    await sandbox.stop();
  }
}
```

Run snapshot creation as a controlled build or administrative task, then configure its returned ID:

```bash
AGENT_BROWSER_SNAPSHOT_ID=snap_xxxxxxxxxxxx
```

Rebuild the snapshot when the desired agent-browser or browser version changes. Pin the installed agent-browser version in reproducible production snapshot builders instead of silently inheriting a future global `latest` version.

## Authentication and deployment boundaries

On Vercel, the Sandbox SDK can use the deployment's OIDC environment. For local development or explicit credentials, provide the complete credential set:

```bash
VERCEL_TOKEN=<personal-access-token>
VERCEL_TEAM_ID=<team-id>
VERCEL_PROJECT_ID=<project-id>
```

Do not expose these values to client code, browser output, logs, or command arguments. Keep all Sandbox creation and automation in server-only modules, route handlers, actions, loaders, or equivalent framework server entry points.

Framework placement examples:

| Framework | Server-only location |
|---|---|
| Next.js | Route handler, server action, API route |
| SvelteKit | `+page.server.ts`, `+server.ts` |
| Nuxt | `server/api/`, `server/routes/` |
| Remix | Server-side `loader` or `action` |
| Astro | Server-rendered frontmatter or API route |

## Scheduled workflows

Vercel Cron can invoke a server route that uses `withBrowser`. Authenticate the cron route according to the application's Vercel Cron setup, and keep the browser workload within the route and Sandbox time limits.

```ts
export async function GET() {
  const snapshot = await snapshotUrl("https://example.com/pricing");
  return Response.json({ ok: true, snapshot });
}
```

```json
{
  "crons": [{ "path": "/api/cron", "schedule": "0 9 * * *" }]
}
```

## Failure triage

| Symptom | Check first |
|---|---|
| `agent-browser` not found | Runtime install completed, or snapshot contains the global package |
| Browser launch fails | Snapshot/runtime includes both Chromium libraries and `agent-browser install` output |
| Commands lose page state | Every command is running in the same Sandbox and before `sandbox.stop()` |
| Element ref fails | Re-run `snapshot -i`; navigation or DOM mutation may have invalidated the ref |
| Screenshot path is missing | Inspect the command's JSON output and fail before invoking `base64` |
| Request times out | Use a prebuilt snapshot, reduce work, and set bounded Sandbox/application timeouts |
| Works on Vercel but not locally | Supply all three explicit Vercel credential variables |

## NEVER do these

- **NEVER run Sandbox/browser code in a client bundle.** Credentials and privileged browser control belong on the server.
- **NEVER pass an unvalidated user URL to `agent-browser open`.** Restrict schemes and destinations to prevent SSRF and access to private services.
- **NEVER omit `sandbox.stop()` from a `finally` block.** Setup and browser commands can fail before normal cleanup.
- **NEVER treat accessibility refs as durable selectors.** They can change after navigation or DOM updates.
- **NEVER return a Sandbox-local file path to the caller.** Read or upload the artifact while the Sandbox still exists.
- **NEVER install dependencies on every production request when a maintained snapshot is viable.** Runtime installation consumes most of the request budget and introduces version drift.
- **NEVER make a cron or browser-automation endpoint public by accident.** Apply authentication, authorization, rate limits, and input bounds.

## Environment variables

| Variable | Required | Purpose |
|---|---|---|
| `AGENT_BROWSER_SNAPSHOT_ID` | No, recommended for repeated runs | Prebuilt Vercel Sandbox snapshot |
| `VERCEL_TOKEN` | Local/explicit auth only | Vercel access token |
| `VERCEL_TEAM_ID` | With explicit auth | Vercel team ID |
| `VERCEL_PROJECT_ID` | With explicit auth | Vercel project ID |

For the version-matched agent-browser command reference, load the bundled core skill with `agent-browser skills get core --full`. For a working repository example, inspect `examples/environments/` in the agent-browser source tree.

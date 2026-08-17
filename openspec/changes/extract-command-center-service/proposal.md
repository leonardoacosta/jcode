## Why

The Command Center frontend is currently built as static assets and hosted by the Jcode daemon. That couples UI-only deployment to the Jcode binary lifecycle. The UI needs an independently buildable and restart-persistent service while Jcode remains the sole authority for inbox, issue, initiative, command, and event state.

## What Changes

- Run the built SolidStart SPA in a dedicated loopback-only Node service.
- Proxy `/api/command-center/*` to the existing Jcode daemon so browser requests remain same-origin.
- Add one concise repository-root installer that builds, atomically installs, writes a systemd user unit, enables restart persistence, starts the service, and verifies health.
- Keep Jcode as the only domain backend. The standalone service owns no database or lifecycle state.
- Retain the daemon-hosted asset path as a compatibility fallback until a later cleanup explicitly removes it.

## Decisions

- Use a systemd user service, not a privileged system service.
- Use a small repository-owned Node HTTP server, not Caddy, Nginx, Docker, or a frontend database.
- Bind to loopback by default. Remote access remains the responsibility of an authenticated tunnel or reverse proxy.
- Use `Restart=on-failure`, `RestartSec=2`, `WantedBy=default.target`, and user lingering when available.
- Install immutable release directories under `$HOME/.local/lib/jcode-command-center/releases` and switch a `current` symlink only after build and staging succeed.

## Done Means

- Running `./install-command-center.sh` from the repository root builds and installs the app without rebuilding Jcode.
- `jcode-command-center.service` is enabled, active, restart-persistent, and serves the SPA.
- `/healthz` reports service health and proxied Command Center API calls reach Jcode.
- Re-running the installer safely upgrades the release and restarts the service.
- A failed build or health check does not replace the last installed release.

## Impact

- Adds a standalone UI deployment boundary without moving domain authority out of Jcode.
- Changes the preferred production topology documented by the existing Command Center vertical slice, while preserving its daemon-hosted fallback.
- Requires Node.js, pnpm for build time, and systemd user services on the deployment host.

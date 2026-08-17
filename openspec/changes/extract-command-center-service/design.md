## Context

`apps/command-center` is a client-only SolidStart application (`ssr: false`). The Jcode daemon must expose the Command Center API on `43118` without serving `.output/public`; the standalone UI service on `43119` preserves same-origin browser behavior without creating a second backend authority.

## Architecture

`install-command-center.sh` builds the existing app, stages the public assets plus `apps/command-center/server.mjs`, and atomically activates a release under `$HOME/.local/lib/jcode-command-center`. It writes `$HOME/.config/systemd/user/jcode-command-center.service` and an optional environment file.

The Node service:

1. Binds `JCODE_COMMAND_CENTER_UI_BIND`, default `127.0.0.1:43119`.
2. Serves `/healthz` directly.
3. Proxies `/api/command-center/*` to `JCODE_COMMAND_CENTER_API_URL`, default `http://127.0.0.1:43118`.
4. Serves immutable assets from the active release and falls back to `index.html` for client routes.
5. Proxies only `/api/command-center/*`; unrelated `/api/*` paths and malformed upstream URLs fail closed.

The Jcode daemon serves the authenticated `/api/command-center/*` routes only. It
does not read `JCODE_COMMAND_CENTER_ASSET_DIR`, serve `index.html`, or expose
legacy client-side and asset routes. This makes `43118` an API-only listener and
keeps all UI packaging under the standalone systemd service.

Jcode remains responsible for authentication, CSRF, commands, events, projections, and durable state. The proxy forwards method, body, and safe headers and preserves streaming responses. It omits `Origin` only for the bootstrap request because the existing bootstrap contract accepts the same-origin request without an Origin header; authenticated mutation requests retain the browser Origin so Jcode's allowlist and CSRF checks remain authoritative.

## Operations

The installer requires Node, pnpm, curl, and `systemctl --user`. It installs dependencies with the lockfile, runs the production build, creates a timestamped release, updates `current` atomically, writes the unit, reloads systemd, enables and restarts the service, then polls `/healthz`. If activation fails, it restores the previous symlink and restarts the previous release.

The unit uses `Restart=on-failure`, `RestartSec=2`, `StartLimitIntervalSec=60`, and `StartLimitBurst=5`. It is enabled under `default.target`. The installer attempts `loginctl enable-linger "$USER"` only when it can do so non-interactively and otherwise prints the exact follow-up command without failing an otherwise healthy install.

## Security

Both UI and upstream bind to loopback by default. The service does not terminate public TLS, store browser credentials, or add authentication. Operators must use an authenticated tunnel or reverse proxy for remote access. Environment files are mode `0600`; installed files are not writable by the service process during runtime.

## Testing

- Unit/integration test the standalone server against a temporary mock upstream for static routes, SPA fallback, health, proxy methods, bodies, streaming, error handling, traversal rejection, and unrelated API rejection.
- Test the daemon HTTP host for `/api/command-center/*` compatibility and `404` rejection of `/`, client-side routes, and legacy asset paths.
- Test the installer with isolated HOME and mocked systemctl/loginctl to verify unit content, atomic activation, idempotent upgrades, and rollback.
- Run the actual build and app test gates.
- Install to the real user service, verify enabled/active status, browser-facing health, API bootstrap behavior, forced process failure recovery, and restart persistence.

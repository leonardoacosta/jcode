# The pre-push deploy pipeline (t3-docker homelab)

> How a `git push` to `main` becomes a running, Tailscale- and internet-reachable
> container. The pipeline lives in `scripts/deploy/pre-push.sh` (generated from
> `${CLAUDE_PLUGIN_ROOT:-$HOME/.claude}/templates/workflow/t3-docker.pre-push.sh.tmpl`) and runs as a git
> `pre-push` hook. Reading order: this file → the actual `pre-push.sh` in the repo.

## Trigger and gates

- Runs on `git push` when the current branch is `main`. Other branches are a no-op.
- **Skip:** `SKIP_DEPLOY=1 git push origin main` (push code without deploying).
- The hook is `set -euo pipefail` with an `ERR` trap that prints the failed phase,
  the log path (`/tmp/<code>-deploy-*.log`), and the rollback command.

## Deploy-mode detection (where the build lands)

The homelab box (`100.73.182.4` over Tailscale, `192.168.1.100` on LAN, deploy
user `<user>`) is both a dev machine and the deploy target. The script picks a mode:

| Condition | Mode | Behavior |
| --- | --- | --- |
| Running ON the homelab (repo at `/home/<user>/dev/<code>`) | **local** | Build + deploy in place; no rsync. |
| Homelab reachable over Tailscale | **remote** | SSH + rsync to the homelab, run docker there. |
| Tailscale down, LAN reachable | **remote (LAN)** | Same, via `192.168.1.100`. |
| Neither reachable | **abort** | Exit 1 — nothing to deploy to. |

Tailscale is the primary path precisely so a push from anywhere on the tailnet
reaches the homelab without exposing SSH to the internet.

## The four phases

**Phase 1 — Local build.** `pnpm turbo typecheck --filter=<app>` then
`pnpm turbo build --filter=<app>`, then verify the Next.js standalone output exists
(`apps/*/.next/standalone`). Fails fast before anything touches the homelab.

**Phase 2 — Sync to homelab.** Skipped in local mode. Otherwise `rsync -avz
--delete` the repo to `<user>@<homelab>:/home/<user>/dev/<code>/`, excluding
`node_modules`, `.next`, `.turbo`, `dist`, `.git`, and **`.env*`** (secrets never
rsync — they live on the homelab).

**Phase 3 — Database migrations.** Bring up `<code>-postgres`, wait for
`pg_isready`, then `pnpm drizzle-kit migrate` (see
[`drizzle-migrate.md`](drizzle-migrate.md) — `migrate`, never `push`). Runs
**before** the web rebuild so the schema is ready when the new code boots. If
`POSTGRES_URL` is unset it is derived from the running container or the convention
URL `postgresql://<code>:<code>_secret@<host>:5432/<code>`.

**Phase 4 — Deploy & verify.** `docker compose build --no-cache <code>-web`,
`up -d <code>-web`, then poll `http://<host>:<prod-web-port>/api/health` up to 10×.
A failed health check exits 1 and the ERR trap prints the rollback.

## Container & network topology

From the compose (`t3-docker.compose.yml.tmpl`):

- **`<code>-web`** — Next.js standalone. Container binds its **prod web port**
  internally (`PORT=<prod>`, `HOSTNAME=0.0.0.0`); the host publishes it 1:1 (e.g.
  `6000:6000`). On the external `homelab` Traefik bridge with a static `DOCKER_IP`.
  Health: `/api/health`. We avoid `3000` (Grafana owns it on the homelab).
- **`<code>-postgres`** — `postgres:16-alpine`. Container **5432** internally (the
  one service that is NOT 1:1 — it is reached only over the private network); host
  publishes the registry's **prod postgres port** (e.g. `6001:5432`). Named volume
  `<code>-postgres` for persistence. Healthcheck `pg_isready`.
- **Traefik** routes `Host(`<domain>`)` → the web container's port (its prod port,
  e.g. `6000`), with `tls.certresolver=cloudflare` (Cloudflare DNS-01 + Let's
  Encrypt). This is what makes `https://<code>.leonardoacosta.dev` work; Tailscale
  handles the private/SSH path, Traefik handles public HTTPS.
- **Watchtower disabled** (`com.centurylinklabs.watchtower.enable=false`) — deploys
  are deliberate via the pre-push hook, never auto-pulled.

> The web container binds its prod port 1:1 (`6000:6000`); postgres is the one fixed
> `5432` override. Dev uses the 50xx block (`next dev -p 5000`, no container), prod
> uses the 60xx block. See SKILL.md § Port Allocation.

## Rollback & inspection

```bash
docker compose -f scripts/deploy/docker-compose.yml up -d --force-recreate <code>-web
docker logs -f <code>-web
docker compose -f scripts/deploy/docker-compose.yml ps
cat /tmp/<code>-deploy-*.log          # the deploy log the run wrote
```

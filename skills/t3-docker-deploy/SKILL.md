---
name: t3-docker-deploy
description: >
  Deploy playbook for T3 Turbo apps shipping to the homelab via Docker + Traefik + Tailscale (t3-docker stack, NOT
  Vercel). Triggers: deploy/redeploy a t3-docker project, git push -> pre-push.sh, homelab box (Tailscale
  100.73.182.4 / ap.leonardoacosta.dev), Traefik routing / Cloudflare TLS, scripts/deploy/ compose+Dockerfile, on-deploy
  schema (drizzle-kit migrate, never push), host ports / subdomains (*.leonardoacosta.dev), new app / service (redis,
  worker), "what port/subdomain is X on", "how is X exposed", "deploy ap to homelab", "add a redis service",
  "register a new app".
allowed-tools: Read, Glob, Grep, Bash
---

# t3-docker-deploy

The t3-docker stack runs T3 Turbo apps as Docker containers on a homelab box,
fronted by Traefik (public HTTPS via a Cloudflare cert) and reached privately over
Tailscale. Deploys are deliberate: `git push` to `main` fires a `pre-push` hook
that builds, migrates, and redeploys. This skill is the source of truth for three
things that must stay consistent across the fleet:

1. **Port allocation** — every app owns a reserved host-port block, tracked in a
   registry, so nothing collides.
2. **The deploy pipeline** — what `pre-push.sh` actually does, end to end.
3. **Migration discipline** — `db:generate` locally + `db:migrate` on deploy;
   `db:push` is banned.

## When This Skill Applies

Trigger whenever the task touches a t3-docker deploy — even implicitly. Standing up a NEW t3-docker app or adding a service (redis, a worker) MUST reserve a port block from the registry FIRST, so two apps never collide on a host port. Bare lookups count too: "what port is X on", "what subdomain is X on", "how is X exposed" all resolve here (the `*.leonardoacosta.dev` ingress map + port registry), without the user naming Docker or Traefik. This is the homelab stack, NOT Vercel — for Vercel deploys use `deploy-and-env`.

## Port allocation (do this FIRST for any new app or service)

Host ports are a shared, exhaustible resource on one box. To keep them
collision-free, every app reserves a contiguous **10-port DEV block in the 5000s**
and a mirrored **PROD block in the 6000s** (`prod = dev + 1000`). Within a block,
services take fixed offsets: `web = base+0`, `postgres = base+1`, then `+2, +3 …`
for extras (redis, workers).

**Container-internal port = the service's PROD host port (1:1 host:container, e.g.
`6000:6000`).** Do **not** use `3000` for web — Grafana owns `3000` on the homelab,
and a 1:1 mapping is less confusing than a `6000:3000` indirection anyway. The one
exception is **postgres**, which always binds `5432` internally (it is reached only
over the private `homelab` network, never published 1:1), so it maps `6001:5432`.
Dev runs `next dev -p 5000` directly on the host — no container — so the container
port only matters in prod.

```
app    idx  dev block    prod block   web(d/p)    postgres(d/p)
ap     0    5000-5009    6000-6009    5000/6000   5001/6001
<next> 1    5010-5019    6010-6019    5010/6010   5011/6011
```

The registry (`references/port-registry.json`) is the single source of truth for
**both** host-port blocks **and** the subdomain every service answers at (the
`ingress` + `infra_services` keys — see § Ingress & DNS). Use the script — do not
hand-pick ports, because the script guarantees the next block is free:

```bash
SKILL=~/.claude/skills/t3-docker-deploy
python3 $SKILL/scripts/port-registry.py list            # every app + its ports
python3 $SKILL/scripts/port-registry.py show ap         # one app
python3 $SKILL/scripts/port-registry.py next            # next free block
python3 $SKILL/scripts/port-registry.py domains         # subdomain log: every service -> subdomain, backend, ingress, status
# Reserve a block (writes the registry). Lists services in offset order:
python3 $SKILL/scripts/port-registry.py allocate tc \
    --name "Tea Club" --domain tc.leonardoacosta.dev --services web,postgres
```

Non-T3 homelab services (Homebridge, Grafana, Immich, AdGuard…) have a subdomain
but no port-block — record them by hand in the registry's `infra_services` array;
they show up in `domains`.

Once allocated, wire the ports into the app:

| Where | Dev or Prod | Value |
| --- | --- | --- |
| `apps/web` dev script `next dev -p <port>` | dev web | `5000` (for ap) |
| local dev Postgres host port + `.env` `POSTGRES_URL` | dev postgres | `5001` |
| `scripts/deploy/docker-compose.yml` web `ports: "<prod>:<prod>"` + Dockerfile `PORT`/`EXPOSE`, entrypoint, healthcheck, Traefik `server.port` | prod web (1:1) | `6000:6000` |
| compose postgres `ports: "<prod>:5432"` | prod postgres | `6001:5432` |
| `.claude/CLAUDE.md` § Port Allocation | both | the block |

> This supersedes the old ad-hoc `3100–3199` dev-port range for newly registered
> apps. Existing fleet apps can be migrated into the registry opportunistically.

## Deploying

`git push` to `main` → the `pre-push` hook runs `scripts/deploy/pre-push.sh`, a
4-phase pipeline: **local build** (turbo typecheck + build) → **rsync to homelab**
(secrets excluded; skipped when running on the box) → **DB migrate** → **docker
build + up + `/api/health` check**. Tailscale is the primary transport (LAN
fallback); Traefik serves `https://<code>.leonardoacosta.dev`.

```bash
git push origin main             # full deploy
SKIP_DEPLOY=1 git push origin main   # push code, skip deploy
./scripts/deploy/rollback.sh         # or: docker compose ... up -d --force-recreate <code>-web
```

Read [`references/deploy-pipeline.md`](references/deploy-pipeline.md) for the full
phase-by-phase contract, deploy-mode detection, the Traefik/Tailscale topology, and
rollback — **read it before editing `pre-push.sh`, the compose, or the Dockerfile.**

### Dockerfile lint gate (pre-flight)

Before pushing, catch Dockerfile mistakes locally with BuildKit's structural linter —
faster feedback than waiting for the `pre-push.sh` build phase to fail:

```bash
docker build --check .    # lints the Dockerfile without building an image
```

## Schema changes on deploy

Edit schema → `pnpm -F @<scope>/db generate` → commit the generated SQL migration →
`git push` (Phase 3 runs `drizzle-kit migrate`). **Never `drizzle-kit push`** — it
skips the journal, can silently drop columns, and desyncs `__drizzle_migrations` so
the deploy's `migrate` step later collides. If an app was bootstrapped with `push`
(common in a fast initial build), its journal is empty and the first real deploy
`migrate` will fail until you reconcile it. Full workflow + the recovery procedure:
[`references/drizzle-migrate.md`](references/drizzle-migrate.md).

## Standing up a new t3-docker app — checklist

1. **Reserve ports**: `port-registry.py allocate <code> --services web,postgres`.
2. **Generate deploy infra** from the `t3-docker.*` templates
   (`${CLAUDE_PLUGIN_ROOT:-$HOME/.claude}/templates/workflow/`): `docker-compose.yml`, `Dockerfile`,
   `entrypoint.sh`, `migrate.mjs`, `pre-push.sh` — fill `WEB_PORT`/`DB_PORT` with
   the **prod** ports, `DOMAIN`, `HOMELAB_IP`, `DOCKER_IP`.
3. **Record the ports** in `.claude/CLAUDE.md` § Port Allocation (dev + prod).
4. **Migrate, never push** — set up `db:generate` + commit migrations from day one.
5. **First deploy**: ensure the DB journal is clean (fresh DB if it was ever
   pushed), then `git push origin main` and watch the 4 phases.

## Ingress & DNS

**Preference: Traefik is the one ingress for `*.leonardoacosta.dev`.** It owns the
host `0.0.0.0:443` — a single bind that covers the LAN **and** the tailnet IP (the
`tailscale0` interface) — terminates every subdomain with a **Cloudflare DNS-01
wildcard cert** (Traefik `certResolver: cloudflare`, `CF_DNS_API_TOKEN`), and
Host-routes to each backend defined in `homelab/traefik/dynamic/routes.yml`.
Host-native daemons (no container) are routed via the docker gateway
`http://172.20.0.1:<port>`; containers via their `172.20.0.x:<port>`.

**Hard rule: `tailscaled` must NOT hold `:443`.** `tailscale serve --https=443`
binds the tailnet IP and **shadows Traefik** (EADDRINUSE / the kernel delivers
tailnet `:443` to whoever bound it). If you see Traefik with no published ports,
that's the symptom — free `:443` (move any serve handler off it) and recreate
Traefik so it publishes its declared `80/443/8080`.

**DNS — Cloudflare stays the authority** (zone owner + cert issuer; do not replace
with AdGuard/Tailscale DNS):

| Layer | Record | Notes |
| --- | --- | --- |
| Off-LAN / tailnet | Public Cloudflare `*.leonardoacosta.dev` A → host **tailnet IP** | **GREY-CLOUD / DNS-only** — Cloudflare can't proxy a `100.x` CGNAT addr. Only tailnet peers route there; the wildcard cert + tailnet ACL gate access. |
| LAN | AdGuard manifest rewrites `*.leonardoacosta.dev` → tailnet IP | `homelab/adguardhome/rewrites.yaml`; coexists with the public record. |
| Apex | `leonardoacosta.dev` → Vercel | public site; never wildcard-captured. |
| Cert | Traefik DNS-01 wildcard via the CF API token | unchanged — Cloudflare is the cert authority. |

`tailscale serve` (below) is the **fallback**, not the default — use it only for
host-native daemons or a quick `.ts.net` URL when a Traefik route isn't wired.

## Exposing a service over Tailscale without Traefik (`tailscale serve` — fallback)

Per § Ingress & DNS, Traefik is the primary ingress; reach for `tailscale serve`
only as a fallback. The constraint that makes it a *fallback* and not the default:
Traefik and `tailscaled` **cannot both own the tailnet `:443`** — `tailscaled`
binds the node's tailnet IP, so a docker-proxy `0.0.0.0:443` collides (EADDRINUSE).
So serve is the reliable choice **when Traefik's `:443` ingress is not available** —
the same mechanism Grafana currently uses on this box.

**Use `tailscale serve` when:** the service is a **host-native daemon** (no container,
no Traefik route — e.g. Homebridge on `:8581`), or you need a working HTTPS URL now and
don't require the `leonardoacosta.dev` custom domain.

**Why it's the reliable choice here:** TLS is terminated by `tailscaled` with a
tailnet-issued cert (auto-renew, no Cloudflare/DNS-01 dependency); `tailscaled` is
native systemd and boot-hardened; the serve config persists in tailscaled state across
reboots; and each `--https=<port>` is an **additive** handler on the same MagicDNS host,
so a new port can't disturb existing ones.

**The tradeoff:** the URL is the node's MagicDNS name + port
(`<host>.tailNNNN.ts.net:<port>`), **not** `<svc>.leonardoacosta.dev` — `tailscaled`
has no cert for the custom domain. If you need the custom domain, that requires the
Traefik ingress (a separate, larger fix), not `tailscale serve`.

> **Gotcha — pick the serve port carefully.** If the app binds **all interfaces**
> (`*:PORT`, e.g. Homebridge on `*:8581`), it also owns the **tailnet IP** on that
> port, so the kernel delivers tailnet traffic straight to the app's **plain-HTTP**
> listener and `tailscale serve --https=PORT` is **shadowed** (curl shows
> `TLS error: wrong version number`). Two ways out:
> - **Serve on a distinct free TLS port** (e.g. `--https=8443` → app's `:8581`).
>   Preserves the app's existing LAN/mDNS/direct access. **Use this when LAN access
>   matters.**
> - **Bind the app to `127.0.0.1` only**, then serve on the app's own port. Cleaner
>   (single TLS ingress, natural port) but **removes LAN/mDNS access** — only do this
>   if tailnet-only is acceptable.

```bash
# Host-native Homebridge UI binds *:8581 (LAN + mDNS used), so serve TLS on a
# distinct free port to avoid shadowing. --bg persists across reboots.
tailscale serve --bg --https=8443 http://127.0.0.1:8581   # add sudo if not the tailscale operator
tailscale serve status                                    # confirm; check the port is free first (ss -tlnp)
# -> https://homelab.tailNNNN.ts.net:8443
```

Verify end-to-end (valid cert AND real backend) before claiming done — `ssl_verify=0`
means the cert validated, and grep the title to prove it's the app, not a proxy error:

```bash
curl -sS -o /dev/null -w "HTTP %{http_code} ssl_verify=%{ssl_verify_result}\n" https://homelab.tailNNNN.ts.net:8443/
curl -sS https://homelab.tailNNNN.ts.net:8443/ | grep -oiE '<title>[^<]*</title>'
```

To remove a handler: `tailscale serve --https=8443 off`.

## What this skill is NOT for

- **Vercel deploys.** That is the `t3-turbo` stack — use `deploy-and-env`.
- **Generic Docker authoring** (image layers, multi-stage tuning) — not covered
  here; this skill is the homelab *pipeline + port governance* on top, plus the
  `docker build --check` pre-flight gate above.

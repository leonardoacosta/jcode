## Overview

Jcode Gateway exposes the authenticated Jcode protocol over an HTTP/WebSocket listener. The hostname `jcode.leonardoacosta.dev` is restricted to the trusted home LAN and Tailnet, and must not be exposed through Tailscale Funnel.

## Route

- DNS: AdGuard's declarative `homelab/adguardhome/rewrites.yaml` already maps `jcode.leonardoacosta.dev` to Traefik's LAN ingress `192.168.1.101`.
- TLS and reverse proxy: `homelab/traefik/dynamic/routes.yml` has a distinct `jcode-gateway` router for `Host(jcode.leonardoacosta.dev)` on `websecure`, with Cloudflare TLS and the `tailnet-or-lan@file` middleware. Access is limited to the Tailscale CGNAT/ULA ranges and trusted `192.168.1.0/24` home LAN clients. The backend is `http://172.20.0.1:7643`.
- Jcode binds the gateway to `172.20.0.1:7643`, the host's homelab Docker bridge address. This avoids binding it to all interfaces.
- The persistent firewall helper `scripts/firewall-jcode-gateway.sh` permits only Traefik (`172.20.0.81`) to connect to host bridge `172.20.0.1:7643`. `scripts/deploy-homelab.sh` reconciles this UFW rule at boot/deployment. Use `--remove` to revoke the rule.
- Traefik dynamic file watching picks up the hostname route without adding a Tailscale Serve entry. Do not add this hostname or port to Funnel.

The route denies clients outside the Tailnet and trusted home LAN. A host-side request routed from outside those ranges can return 403. This local host cannot verify DNS/TLS/WebSocket/pairing from a separate remote device; complete that check from an authorized Tailnet client.

## Pairing

Once route access is verified from a Tailnet or trusted-LAN client, pair the Jcode iOS/client app using `jcode pair` (or `/remote pair`). Pairing codes expire after five minutes. Do not share the code or place it in a URL/log.

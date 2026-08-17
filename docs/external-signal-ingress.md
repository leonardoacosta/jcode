# External signal ingress contract

Jcode owns external-signal admission, durable receipts, canonical lifecycle, attention evidence, and ambient delivery projections. Producers own only delivery to the configured private endpoint.

## Transport and trust

The listener is disabled by default and is separate from Command Center. It uses no HMAC, bearer token, browser session, cookie, or provider credential. Set:

```text
JCODE_EXTERNAL_SIGNAL_ENABLED=true
JCODE_EXTERNAL_SIGNAL_BIND_ADDR=<loopback-or-private-ip>:<port>
JCODE_EXTERNAL_SIGNAL_SOURCE_ID=grafana-homelab
JCODE_EXTERNAL_SIGNAL_PROJECTS=jcode=/home/nyaptor/dev/jcode
JCODE_EXTERNAL_SIGNAL_MAX_BODY_BYTES=262144
JCODE_EXTERNAL_SIGNAL_WAKES_ENABLED=false
```

Wildcard, unspecified, public IP, link-local, and non-loopback IPv6 binds fail closed. Loopback, RFC1918 IPv4, and Tailscale CGNAT `100.64.0.0/10` binds are accepted so Grafana in Docker or on the Tailnet can reach the dedicated listener without exposing Command Center unauthenticated. Network routing and firewall placement must keep the listener private, and it must not be routed through Traefik or Cloudflare. Wakes remain separately disabled for shadow rollout.

## HTTP contract

- `GET /readyz` returns readiness, source ID, and adapter version.
- `POST /v1/external-signals/grafana` accepts Grafana webhook schema version `1` with `Content-Type: application/json`.
- Content encoding is rejected. The default streaming body limit is 256 KiB and the hard configurable ceiling is 1 MiB.
- Every alert must carry exactly one consistent `jcode_project` label that maps through the explicit project registry.
- Optional `X-Jcode-Delivery-Id` supplies the delivery identity. Without it, the raw payload SHA-256 is the delivery identity.
- `202` is returned only after the raw envelope, receipt identity, processing record, canonical signal, lifecycle aggregate, and attention evidence are atomically persisted. Redelivery returns the same receipt with outcome `deduplicated`.
- `405`, `413`, `415`, `422`, and `503` retain their standard meanings. Error responses contain bounded reason codes and never echo raw payloads.

## Authority and lifecycle

The immutable raw provider envelope is retained separately from provider-neutral canonical signals. Canonical lifecycle identity is SHA-256 of source ID, canonical project key, and Grafana fingerprint. Repeated firing evidence increments one aggregate and one attention item. Resolutions older than the latest lifecycle transition cannot close a newer generation. Resolved-before-firing creates a tombstone-like resolved aggregate and a later firing reopens it as generation 1.

Durable attention evidence is written before an ambient schedule projection. Severity maps to bounded priority and timing, but ambient enablement, pause state, single-agent execution, interactive priority, and resource gates remain authoritative. A lifecycle receives at most one pending scheduled projection, so unchanged repeats do not create wake storms.

Storage is versioned at `~/.jcode/external-signals/state.json`. Rollback disables the producer route first, then `JCODE_EXTERNAL_SIGNAL_ENABLED`, while preserving the state file for deterministic re-enable and replay.

## Homelab counterpart

The exact producer-side initiative is `route-grafana-alert-lifecycle-to-jcode-ambient-inbox`. It owns the Grafana contact point, private route/firewall placement, explicit `jcode_project` labels, reversible enablement, firing/resolved delivery, and real test-alert evidence. It must send no authentication headers. This repository owns the consumer endpoint and all durable lifecycle and attention authority.

Real acceptance remains environment-owned and must prove private reachability from Grafana, failed reachability from an untrusted/public vantage point, firing, repeat, escalation, resolution, reopen, stale resolution, downtime recovery, lost-response redelivery, concurrent redelivery, and over-limit/unknown-project rejection.

# External signal ingress contract v1

Handoff for homelab initiative `route-grafana-alert-lifecycle-to-jcode-ambient-inbox`.

## Endpoint and transport

- Listener: dedicated external-signal HTTP listener, separate from Command Center.
- Method: `POST`
- Path: `/v1/external-signals/grafana`
- Readiness: `GET /readyz`
- Content type: `application/json`; content encoding is rejected.
- Authentication: none by design for this private local/homelab transport. Do not add HMAC, bearer, browser session, cookie, or provider credentials.
- Runtime configuration:
  - `JCODE_EXTERNAL_SIGNAL_ENABLED=true`
  - `JCODE_EXTERNAL_SIGNAL_BIND_ADDR=172.20.0.1:7778` for the current homelab Docker bridge deployment, or another approved loopback/RFC1918/Tailnet address and port.
  - `JCODE_EXTERNAL_SIGNAL_SOURCE_ID=grafana-homelab`
  - `JCODE_EXTERNAL_SIGNAL_PROJECTS=jcode=/home/nyaptor/dev/jcode`
  - `JCODE_EXTERNAL_SIGNAL_MAX_BODY_BYTES=262144`
  - `JCODE_EXTERNAL_SIGNAL_WAKES_ENABLED=false` for shadow rollout, then `true` when ambient scheduling is explicitly enabled.
- Bind validation: accepts loopback, RFC1918 IPv4, and Tailscale CGNAT `100.64.0.0/10`. Rejects wildcard, public IPv4, link-local IPv4, and non-loopback IPv6. This permits Docker bridge or Tailnet reachability from Grafana without exposing Command Center unauthenticated.
- Exposure rule: never publish this listener through Traefik, Cloudflare Tunnel, a public address, or a broad LAN route that is not firewall-scoped to the trusted Grafana/relay source.

## Body limit and success codes

- Default body limit: 262,144 bytes. Configurable with `JCODE_EXTERNAL_SIGNAL_MAX_BODY_BYTES`, hard-capped at 1,048,576 bytes.
- Success: `202 Accepted` after durable raw envelope, delivery receipt, processing record, canonical signal, lifecycle aggregate, and attention evidence are written.
- Duplicate delivery: `202 Accepted` with the original receipt and outcome `deduplicated`.
- `GET /readyz`: readiness, source ID, and adapter version.
- `405`, `413`, `415`, `422`, and `503` retain their standard meanings.
- Error responses contain bounded reason codes and do not echo raw payloads.

## Payload/version

Jcode accepts Grafana's native grouped webhook envelope directly. No application-specific transform is required. The central contract is: Grafana contact point sends its native JSON and includes exactly one consistent `jcode_project` label across the group or alert labels.

Grafana webhook requirements:

- Top-level `version` must be `"1"`.
- Top-level `status` must be `firing` or `resolved`.
- `alerts` count must be 1 to 100.
- Each alert `status` must be `firing` or `resolved`.
- Each alert must include a non-empty `fingerprint`.
- Exactly one project label must resolve from alert labels or `commonLabels`: `jcode_project=<project-key>`.
- The project key must exist in `JCODE_EXTERNAL_SIGNAL_PROJECTS`.

Minimal Grafana-shaped example:

```json
{
  "version": "1",
  "groupKey": "{}:{alertname=DiskFull}",
  "status": "firing",
  "receiver": "jcode",
  "groupLabels": {},
  "commonLabels": {
    "jcode_project": "jcode",
    "severity": "critical"
  },
  "commonAnnotations": {},
  "alerts": [
    {
      "status": "firing",
      "labels": {
        "alertname": "DiskFull",
        "jcode_project": "jcode",
        "severity": "critical"
      },
      "annotations": {
        "summary": "Disk is full"
      },
      "startsAt": "2026-08-17T04:00:00Z",
      "endsAt": "0001-01-01T00:00:00Z",
      "generatorURL": "http://grafana.local/alerting/list",
      "fingerprint": "abc123"
    }
  ]
}
```

Accepted response:

```json
{
  "receiptId": "rcpt_<sha-prefix>",
  "outcome": "accepted"
}
```

Duplicate response:

```json
{
  "receiptId": "rcpt_<same-receipt>",
  "outcome": "deduplicated"
}
```

## Stable identity, idempotency, and lifecycle ordering

- Stable lifecycle identity is SHA-256 of `source_id`, canonical project key, and Grafana alert fingerprint.
- Delivery idempotency identity is `X-Jcode-Delivery-Id` when present, otherwise SHA-256 of the raw payload body. Grafana or a central homelab relay should set `X-Jcode-Delivery-Id` to a stable delivery identifier when it has one. Retries of the same delivery must reuse it.
- Raw provider envelope is retained separately from provider-neutral canonical signals.
- Repeated firing evidence increments one lifecycle aggregate and one attention item. It does not create a wake storm.
- Resolutions older than the latest lifecycle transition cannot close a newer firing generation.
- Resolved-before-firing creates a resolved aggregate. A later firing reopens it as generation 1.
- Ordering owner: Jcode owns lifecycle reduction after admission using Grafana timestamps. The producer owns retry ordering and must not mutate a retry payload into a different lifecycle event under the same delivery id.

## Timeout and retry ownership

- Sender timeout budget: use a short HTTP timeout, recommended 2 seconds.
- Retry owner: Grafana or its central homelab relay owns retry and backoff.
- Retry only connection failures, timeouts, `429`, or `5xx`.
- Do not retry `413`, `415`, or `422` without changing configuration or payload.
- Lost-response redelivery is safe when the same `X-Jcode-Delivery-Id` or identical raw body is reused.

## Observability and unavailable behavior

- Durable state path: `~/.jcode/external-signals/state.json`.
- State records raw provider envelopes, delivery receipts, processing records, canonical signals, lifecycle aggregates, attention evidence, accepted count, deduplicated count, and rejected count.
- Startup logs the accepted listening URL as `External signal ingress listening on http://<addr>/v1/external-signals/grafana`.
- `/readyz` reports readiness, `sourceId`, and adapter version.
- If Jcode is unavailable, not listening, or returns `503`, Grafana or the relay should retry according to its notification retry policy. If a relay has durable storage, it should queue until delivered or retention expires.
- If configuration is invalid, Jcode refuses to start the listener and logs the rejection. Rollback disables the producer route first, then sets `JCODE_EXTERNAL_SIGNAL_ENABLED=false`, preserving the state file for deterministic re-enable and replay.

## Acceptance evidence

Implemented in:

- `crates/jcode-app-core/src/external_signal.rs`
- `crates/jcode-app-core/src/server.rs`
- `crates/jcode-app-core/src/lib.rs`

Verified by tests:

- `rejects_public_and_wildcard_binds`: rejects wildcard, public, link-local, and non-loopback IPv6 binds.
- `permits_loopback_rfc1918_and_tailscale_binds`: accepts loopback, RFC1918 Docker/LAN ranges, and Tailscale CGNAT.
- `coalesces_repeats_and_ignores_stale_resolution`: preserves lifecycle ordering and coalesces repeat firing evidence.
- `resolved_before_firing_reopens_a_new_generation`: handles resolved-before-firing and reopen behavior.

Runtime verification command used during handoff: `cargo test -p jcode-app-core external_signal -- --nocapture`.

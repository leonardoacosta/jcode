---
execution:
  version: 1
  depends_on: []
---

## Why

Today's `omni` cache warnings are all `low_read` with an unchanged checked prefix, so Jcode reports "cause unconfirmed" for every one. Two local defects hide the real cause:

1. **Stale route identity.** `RequestRoute.upstream_provider` is sampled at request start from `last_upstream_provider`, which the previous response set. The request's own serving route is never compared, so a gateway account or upstream switch cannot be classified. Compatible gateway streams also never emit `UpstreamProvider`; only OpenRouter's `provider` body field does.
2. **False timing claim.** `MissContext::timing()` always prints "within TTL", including a logged `1h 17m` idle "within estimated TTL 5m".

Installed OmniRoute 3.8.54 (read-only source review, 2026-11-19) already returns per-response route identity: `X-OmniRoute-Provider` and `X-OmniRoute-Selected-Connection-Id`. It honors an explicit `prompt_cache_key`; without one, its key hashes the prefix through the last assistant/tool message, which changes every turn. Priority combos protect the first target from affinity reordering. So existing client session affinity stays useful; removing it is out of scope.

## What Changes

- Compatible-provider responses report the gateway-observed route (provider alias and opaque connection fingerprint) for the request that produced them.
- The cache monitor compares the baseline request's serving route with the current request's serving route, so a connection switch classifies as `account_switch` or `upstream_switch` instead of `low_read`.
- Timing text says "within" only when idle is below the TTL, and "exceeded estimated TTL" otherwise. An estimated TTL never becomes `expired`.

## Scope

In: `crates/jcode-provider-openrouter-runtime` response-header capture, `crates/jcode-app-core` route binding, `crates/jcode-base/src/kv_cache_monitor.rs` classification and timing, their tests.

Out: gateway configuration or code, session-affinity key generation, compression, retention defaults, live inference beyond one isolated acceptance check, shared promotion. Connection IDs appear only as short fingerprints.

## Impact

Misses become actionable: a routed account change explains the miss and points at gateway priority or failover. Providers without these headers keep current behavior.

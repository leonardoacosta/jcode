## Preconditions

- [x] 0. Unresolved merge (`MERGE_HEAD` 10114429c, started 2026-11-18) leaves conflict markers in `openrouter_sse_stream.rs`, `openrouter_provider_impl.rs` and `agent_tests.rs`. Lanes A and B wait for the merge owner. Lane C touches no conflicted file and may proceed.

## Lane C: timing text (worker, `crates/jcode-base/src/kv_cache_monitor.rs`)

- [x] 1. Red test: estimated TTL 300s, idle 4620s, `low_read` detail contains "exceeded estimated TTL 5m" and not "within"; reason stays `low_read`.
- [x] 2. Fix `MissContext::timing()`, keeping under-TTL wording. Run `cargo test -p jcode-base kv_cache_monitor`.

## Lane A: capture gateway route (worker, `crates/jcode-provider-openrouter-runtime`)

- [x] 3. Red test with a fake SSE server returning `X-OmniRoute-Provider` and `X-OmniRoute-Selected-Connection-Id`: one route event precedes content on Chat and Responses paths; absent headers emit nothing.
- [x] 4. Read the headers after a successful status in `stream_response` and emit route identity, connection as a fingerprint. Reuse `StreamEvent::UpstreamProvider` or add one route event; no new dependency.

## Lane B: bind route to request (coordinator after A, `crates/jcode-app-core`, monitor route fields)

- [x] 5. Red test: different observed connections, same prefix, low read gives `account_switch`; same connection gives `low_read`; missing headers keep current behavior.
- [x] 6. Set the pending request's route from the route observed during that request before `finish_request`, instead of the previous response's value.

## Verification

- [x] 7. `cargo test -p jcode-base -p jcode-provider-openrouter-runtime -p jcode-app-core`, `cargo fmt --check`, clippy on touched crates.
- [x] 8. Live gateway check, closed as limited. Probed omni 3.8.54 directly: non-streaming responses carry `x-omniroute-provider` and `x-omniroute-selected-connection-id`; streaming responses carry neither (only `x-omniroute-route-class`). End-to-end jcode attribution on streaming turns is UNVERIFIED and does not work on this gateway version. Follow-up: ask upstream to emit both headers on streams.
- [x] 9. Review diff, commit scoped hunks only, archive after acceptance.

# Tasks

All tasks are implementation work. Authoring this proposal does not complete them. Do not perform live Hindsight writes or change the user's active memory configuration.

## 1. Contract and adapter

- [ ] 1.1 Pin the Hindsight HTTP API revision/schema and verify operation paths, request/response fields, authentication, memory invalidation and idempotency behavior. Record evidence and the exact transport contract in design before coding; do not infer method names from the Hermes integration README.
- [ ] 1.2 Add typed endpoint, per-world bank and separate credential-reference configuration. Resolve only from verified world identity; deny unknown/ambiguous worlds, missing configuration and cross-world requests before transport. Keep secrets out of prompts, diagnostics and serialized config values.
- [ ] 1.3 Implement the native adapter within the existing memory lifecycle. Keep local memory behavior unchanged; automatic Hindsight retain is off until explicit per-world/data-scope opt-in.
- [ ] 1.4 Add bounded deadlines, result/query limits, queue capacity and cancellation. Discard recalls after world/session generation changes; surface failures without provider, endpoint or bank fallback.
- [ ] 1.5 Add Jcode-side retain operation keys derived from stable source/turn identifiers. Reuse keys on retry, surface uncertain duplicate risk, and make no server exactly-once claim unless task 1.1 verifies deduplication.
- [ ] 1.6 Route recalled results through existing attribution with source, time, bank and provenance. Label content as untrusted evidence; do not expose it as instructions. Implement remote forget only as the verified soft-invalidation behavior and describe it as non-erasure.

## 2. Verification

- [ ] 2.1 Add fake-transport tests for endpoint/bank/credential selection, unknown world, cross-world denial, secret redaction, malformed responses, auth failure, timeout, queue saturation, cancellation, generation change, bounded results, opt-in default-off, stable retry keys and duplicate-risk status, provenance, untrusted rendering, and soft-forget failure/success semantics.
- [ ] 2.2 Run focused tests by the implemented module/test names, including `cargo test -p jcode-base hindsight_memory_scope` and any affected adapter/config crate tests. Record exact crate, filter and results after implementation; add a focused test target if the adapter belongs to a different crate.
- [ ] 2.3 Run `cargo clippy --workspace --all-targets -- -D warnings`, `git diff --check`, and `openspec validate scope-hindsight-memory-per-world --strict`. Exercise contract scenarios with fake transport; no live Hindsight bank writes.
- [ ] 2.4 Obtain independent review of the final diff, repair actionable findings, rerun affected checks, and update implementation evidence only from observed results.

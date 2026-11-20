# Design

## API contract evidence

Resolved by task 1.1. The transport contract is pinned to `vectorize-io/hindsight` commit `d56c4acdf59c41957613d399094cdf8c489b060c` (OpenAPI 0.10.2, sha256 `815112185239a60101dd56879ac3de285cb7da466477dad417e56cdd8c563ba7`), read from the committed OpenAPI schema and server source. Routes, fields, authentication, soft invalidation and idempotency are recorded in [evidence-contract.md](evidence-contract.md). The adapter refuses to send when the transport reports another revision. This check is self-attestation: the shipped HTTP transport reports the pinned constant, so it guards against a mismatched transport implementation and does not probe or verify the server's version.

Earlier text said no server idempotency contract was found. The pinned schema has one: a client `operation_id` UUID on **async** retain replays the original operation. The retain section below reflects that and still avoids claims the server did not confirm.

## Scope and dependencies

`add-attributable-memory-recall` owns provenance fields and bounded injected-memory rendering. This change reuses that path and adds remote bank/source provenance without duplicating its contract. `workspace-aware-context` owns verified world identity and target propagation. This change consumes that identity and does not redefine workspace resolution. `add-memory-retention-controls` is listed as cut, not implemented; this proposal sets remote Hindsight's explicit opt-in and honest soft-forget boundary but does not duplicate a general local-memory retention or purge policy. Align interfaces when that change is reopened.

## Isolation and secrets

Each verified world gets an explicit route containing an endpoint reference, bank identifier, and separate credential reference. Configuration stores secret references, not credential bytes. Resolve the referenced credential only at the transport boundary. Redact authorization headers and credential-bearing errors from diagnostics. A bank name alone grants no rights: the adapter selects the bank and credential only from the verified world's configuration. Unknown/ambiguous identity, missing route, missing credential or authorization failure denies access; there is no shared-bank or other-provider fallback. Cross-world selection is refused before HTTP transport. Endpoints must be `https`, except loopback hosts (`localhost`, `127.0.0.0/8`, `::1`) which may use `http`; any other `http` endpoint is denied as an incomplete route so the Bearer credential is never sent in cleartext.

World identity is "verified workspace policy" in this sense: the directory matched a configured rule by origin, and the rule's `expected_email` check did not mismatch or come back missing. A rule with no `expected_email` yields `Identity::NotChecked`, which counts as verified on origin alone. Add `expected_email` to a rule to require the stronger check.

## Retain retries

Derive a stable Jcode operation key from a stable source identifier plus turn identifier, scoped to the world and endpoint without embedding private text. Reuse it for retries of the same operation. The pinned contract dedupes only async retains that carry the key, so retain is always sent async with the key and the dedup claim is made only when the response confirms `async`. Expose possible duplicate writes when a timed-out request may have been accepted. Do not retry automatically where stable identity is absent or where the transport outcome leaves unsafe duplicate risk.

## Bounded transport and evidence handling

Use a finite connection/request deadline, bounded queue (an in-flight counter per endpoint held process-wide, because a client is built per tool call; capacity comes from the route) and bounded request/result size. Keep cancellation and workspace/session generation with each pending recall; discard responses after the target changes. Report timeout, queue rejection, authentication/configuration failure, malformed response and service unavailability distinctly enough for a user to act. No failure path changes provider, endpoint or bank silently.

Mark recalled text as untrusted evidence and carry source identifier, source/time, selected world/bank and retrieval provenance into the attribution mechanism. Provenance is metadata, not a reason to trust the content. Do not insert remote text into policy or instruction layers. Automatic retain stays disabled until explicit user opt-in for the world and data scope.

### Final transcript retain

The world is pinned when the memory agent first sees the session. The post-session `transcript` retain resolves the directory again at finalization and refuses to send unless the result equals the pinned world (a session with no pinned world never retains). The same directory re-check also runs before and after the request, so a change of world during the call discards it. A `transcript` scope writes the whole session tail, including any tool output the session read from other worlds. The scope bounds which bank receives it, not what it contains; opt in only for worlds where that is acceptable.

## Forget and Tern

The evidenced memory-unit state change is soft invalidation and excludes the item from future recall; it is not a hard-erasure promise. User-facing copy and status must state this limit and distinguish request failure from success. Whole-bank clearing/deletion is not part of this proposal.

Tern may present or edit a user-inspected artifact only. It does not resolve worlds, grant bank access, make retention decisions, or become an authority for memory contents.

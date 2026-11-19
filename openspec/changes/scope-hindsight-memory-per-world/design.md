# Design

## API contract evidence and open question

The available evidence is not sufficient to pin every method name or request schema for implementation. The read-only report inspected Hindsight revision `d56c4acdf59c41957613d399094cdf8c489b060c` and its `RecallRequest`/`RecallResult` declarations, and the version 1.2.1 public REST reference lists retain, recall, reflect and memory invalidation operations. However, the public API docs are not immutable evidence for that revision, the pinned route declarations and retain/reflect schemas were not exhaustively inspected, and no live or local API request was made. See [scratch/verify-tern-hindsight.md:9-21,41-55](../../../scratch/verify-tern-hindsight.md).

**Open question before implementation:** pin and inspect the exact HTTP API version and route/request/response/authentication contracts used by the adapter. Verify retain, recall, reflect, memory invalidation, and any idempotency support from source or an immutable API schema. Until then, operation names and wire shapes are intentionally not specified here. Do not infer server deduplication from a Jcode operation key.

## Scope and dependencies

`add-attributable-memory-recall` owns provenance fields and bounded injected-memory rendering. This change reuses that path and adds remote bank/source provenance without duplicating its contract. `workspace-aware-context` owns verified world identity and target propagation. This change consumes that identity and does not redefine workspace resolution. `add-memory-retention-controls` is listed as cut, not implemented; this proposal sets remote Hindsight's explicit opt-in and honest soft-forget boundary but does not duplicate a general local-memory retention or purge policy. Align interfaces when that change is reopened.

## Isolation and secrets

Each verified world gets an explicit route containing an endpoint reference, bank identifier, and separate credential reference. Configuration stores secret references, not credential bytes. Resolve the referenced credential only at the transport boundary. Redact authorization headers and credential-bearing errors from diagnostics. A bank name alone grants no rights: the adapter selects the bank and credential only from the verified world's configuration. Unknown/ambiguous identity, missing route, missing credential or authorization failure denies access; there is no shared-bank or other-provider fallback. Cross-world selection is refused before HTTP transport.

## Retain retries

Derive a stable Jcode operation key from a stable source identifier plus turn identifier, scoped to the world and endpoint without embedding private text. Reuse it for retries of the same operation. Because the verified evidence found no server idempotency contract, expose possible duplicate writes when a timed-out request may have been accepted. Do not retry automatically where stable identity is absent or where the transport outcome leaves unsafe duplicate risk. A future confirmed server dedupe contract may strengthen this behavior, but is not assumed.

## Bounded transport and evidence handling

Use a finite connection/request deadline, bounded queue and bounded request/result size. Keep cancellation and workspace/session generation with each pending recall; discard responses after the target changes. Report timeout, queue rejection, authentication/configuration failure, malformed response and service unavailability distinctly enough for a user to act. No failure path changes provider, endpoint or bank silently.

Mark recalled text as untrusted evidence and carry source identifier, source/time, selected world/bank and retrieval provenance into the attribution mechanism. Provenance is metadata, not a reason to trust the content. Do not insert remote text into policy or instruction layers. Automatic retain stays disabled until explicit user opt-in for the world and data scope.

## Forget and Tern

The evidenced memory-unit state change is soft invalidation and excludes the item from future recall; it is not a hard-erasure promise. User-facing copy and status must state this limit and distinguish request failure from success. Whole-bank clearing/deletion is not part of this proposal.

Tern may present or edit a user-inspected artifact only. It does not resolve worlds, grant bank access, make retention decisions, or become an authority for memory contents.

# Pinned Hindsight transport contract (task 1.1)

Read-only. No live or local Hindsight was contacted. Sources fetched with `curl` from raw.githubusercontent.com and saved under `scratch/applyall/hs/`.

| Item | Value |
| --- | --- |
| Repository | `vectorize-io/hindsight` (official) |
| Commit | `d56c4acdf59c41957613d399094cdf8c489b060c` (`release(hermes): v1.2.1`, 2026-09-30) |
| Schema | `hindsight-docs/static/openapi.json` at that commit, `info.version` 0.10.2 |
| Schema sha256 | `815112185239a60101dd56879ac3de285cb7da466477dad417e56cdd8c563ba7` |
| Server source | `hindsight-api-slim/hindsight_api/api/http.py`, `engine/memory_engine.py` at the same commit |

The Hermes integration README was not used for any path or field.

## Operations used

| Operation | Method and path | Body fields used | Response |
| --- | --- | --- | --- |
| Retain | `POST /v1/default/banks/{bank_id}/memories` | `items[]` (`content` required; `context`, `tags`, `metadata` string map used), `async: true`, `operation_id` (UUID) | `RetainResponse`: `success`, `bank_id`, `items_count`, `async`, `operation_id` |
| Recall | `POST /v1/default/banks/{bank_id}/memories/recall` | `query` (required), `max_tokens`, `budget`, `include` | `RecallResponse.results[]`: `id`, `text` required; `type`, `mentioned_at`, `occurred_start`, `document_id`, `tags`, `metadata` optional |
| Soft invalidate | `PATCH /v1/default/banks/{bank_id}/memories/{memory_id}` | `state: "invalidated"`, `reason` | memory unit JSON; 404 when absent |

Reflect (`POST .../reflect`) exists but is not used. Whole-bank deletion is not implemented.

## Authentication

`get_request_context` reads the `Authorization` header and accepts `Bearer <key>` or a bare key. The OpenAPI document declares no `securitySchemes`, and the default tenant extension performs no auth. The adapter sends `Authorization: Bearer <key>` and treats 401/403 as a credential failure.

## Idempotency (this corrects the earlier open question)

The pinned `RetainRequest.operation_id` is a client-supplied UUID. Re-submitting the same id returns the original operation and creates no new work. A different operation owning the id returns HTTP 409. It is **ignored for synchronous retain**. Therefore:

- the adapter always retains with `async: true` and a UUID v5 derived from world, endpoint, bank, source id and turn id (no content);
- dedup is claimed only when the response confirms `async: true`; otherwise the report says `NotConfirmed`;
- without both stable identifiers it sends no `operation_id` and never retries;
- a timeout after possible acceptance reports `PossiblyAccepted`.

## Soft invalidation

`UpdateMemoryRequest.state = "invalidated"` is documented as: excluded from recall, consolidation and graph maintenance, kept for audit, reversible, only world/experience facts. It is not erasure, and the adapter says so.

## Fail-closed revision check

`HindsightTransport::contract_revision()` must equal `CONTRACT_REVISION`. A mismatch refuses to send (`ContractMismatch`).

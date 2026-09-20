## Context

Jev decisions are approximately pure functions of `(model, schema, state)` — identical inputs produce identical outputs. The evaluate tool calls TypeSafe API for every request, costing ~$0.0002–0.0004 and ~300ms per call even when the same state+questions have been evaluated before. `hyperspaceai/jevcache` (11 stars) demonstrates a content-addressed cache achieving 78% recurrence in real workloads — 60–80% of an agent's decisions repeat.

The evaluate tool provides the API call path this cache wraps. The cache is a local-only, in-memory index backed by an append-only JSON-lines ledger on disk.

## Goals / Non-Goals

**Goals:**
- Deterministic SHA-256 fingerprinting of evaluate calls: `sha256(model ⊕ schema ⊕ canonical(redact(state)) ⊕ salt)`
- Redaction: strip PII-like patterns (emails, phones, UUIDs, timestamps, long digit runs) from state before hashing
- Per-schema salt: optional user-provided salt mixed into key for sensitive schemas
- Append-only JSON-lines ledger with in-memory index loaded at session start
- Session-level hit/miss telemetry (calls, hits, cost saved)
- Graceful degradation: cache skipped if ledger corrupt/unwritable

**Non-Goals:**
- Not a general-purpose LLM cache (LLMs are non-deterministic; Jev is deterministic)
- Not sharing caches across machines or publishing caches (future work)
- Not caching OpenRouter vs direct TypeSafe differently (model identity is part of key)
- Not cross-session persistence beyond ledger reload at session start
- Not encrypting the ledger (contains only fingerprints + answers, never raw state)
- Not implementing TTL or cache eviction

## Decisions

**Content-addressed fingerprinting:** The cache key is a SHA-256 hash of all inputs that determine a Jev decision: model name, question schema, state, and optional salt. Identical inputs always produce identical fingerprints → deterministic cache hits.

**Redaction before hashing:** Strip volatile fields (emails, UUIDs, timestamps, phone patterns, long digit runs) from state before canonicalization. This ensures logically identical states produce identical keys even when they differ only in runtime details. Stops at string-level; structured JSON objects pass through without string-level redaction.

**Salting:** Per-schema salt is mixed into the key for sensitive schemas. An outsider with the ledger file can read cached answers but cannot use the fingerprints to enumerate which states have been decided (without the salt). Salt is configured via `JEVCACHE_SALT_<name>` env vars.

**Append-only ledger:** JSON-lines format, one entry per line: `{fp, ts, model, answer}`. On session start, read entire file into `HashMap<String, Value>` for ~0ms lookups. Appends are sequential (single writer). No eviction in initial implementation — even 100k entries is ~20MB.

**Graceful skip, not error:** Corrupt or unwritable ledger → log warning, set `disabled` flag, all evaluate calls pass through to API. `JEVCACHE_DISABLE=true` env var forces skip. Cache failures never block the agent.

## Risks / Trade-offs

- **Risk:** Redaction may strip content that materially affects the Jev decision (e.g., an email address used as a username). Mitigation: redaction is conservative — only well-known patterns are stripped; agent can bypass cache with `JEVCACHE_DISABLE`.
- **Risk:** Ledger grows unbounded. Mitigation: acceptable in initial scope (~200 bytes/entry); eviction policy is future work.
- **Trade-off:** In-memory index on session start reads entire ledger. Justification: 100k entries ~20MB file reads in milliseconds; negligible vs session startup cost.
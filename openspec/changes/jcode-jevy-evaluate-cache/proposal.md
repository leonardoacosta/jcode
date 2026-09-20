---
execution:
  version: 1
  depends_on: ["add-jevsdk-evaluate-tool"]
---

# jcode-jevy-evaluate-cache

## Summary

Add a deterministic, local-first decision cache to the Jev `evaluate` tool. Jev decisions are approximately pure functions of `(model, schema, state)` — identical inputs produce identical outputs. The cache memoizes by a content-addressed fingerprint, so the same decision never runs — or bills — twice. A local hit returns in ~0ms for $0. A miss routes to the TypeSafe API (or OpenRouter) and caches the result.

The design follows the architecture of `jevcache.sh` (Hyperspace, 11 stars) but is re-implemented in Rust within Jcode rather than bundled as a binary.

## Motivation

Every Jevy proposal downstream of `add-jevsdk-evaluate-tool` makes repeated evaluate calls:

| Feature | Evaluate calls per cycle | Repeat potential |
|---|---|---|
| Model router | 1 per user turn | High — same prompts recur |
| Code review screener | 5 per diff hunk | Very high — same hunks on re-review |
| Compaction | 2 per tool call per session | Moderate — same transcripts compacted |
| Foreman supervision | 10 per observation cycle | Very high — same worker state across cycles |
| Shell history | 1 per Tab press | Low — history changes rapidly |

Without caching, every call costs ~$0.0002–0.0004 and ~300ms latency. JevCache reports a 78% recurrence rate in real workloads — 60-80% of an agent's decisions repeat. Without caching, a foreman observer running 30-second cycles on 3 workers generates `10 q × 3 w × 120 cycles/hr = 3,600` calls/hour, most with identical state.

A cache reverses the cost scaling: cached calls cost $0 and ~0ms. The first call pays; every repeat is free and instant.

## Actors

- **Jcode agent**: calls `evaluate(state, questions)` through the tool
- **Evaluate tool**: intercepts calls, fingerprints inputs, consults ledger
- **Evaluate ledger**: append-only log + in-memory index mapping fingerprints to answers
- **TypeSafe API / OpenRouter** (existing): called only on cache misses
- **JevCache design**: upstream reference architecture from `hyperspaceai/jevcache`

## Scope

- **Fingerprinting**: compute a deterministic key for each evaluate call:
  `sha256(model ⊕ schema ⊕ canonical(redact(state)))`
- **Redaction**: strip PII-like patterns (emails, phone numbers, long digit runs, volatile IDs/timestamps) from state before canonicalization, so logically identical states produce identical keys
- **Canonicalization**: sort JSON object keys, normalize whitespace, ensure deterministic serialization
- **Per-schema salt**: optional config for sensitive schemas — a user-provided salt is mixed into the key so outsiders cannot enumerate which states have been decided
- **Ledger storage**: append-only JSON-lines file (`~/.jcode/jev-cache/ledger.jsonl`) with an in-memory index loaded at session start
- **Session-level hit/miss telemetry**: track calls, hits, cost saved per session
- **`jcode stats` integration**: surface Jev cache statistics alongside other session stats
- **Graceful degradation**: if the ledger file is corrupt or unwritable, the cache is skipped (no errors to the agent)

## Non-scope

- Not a general-purpose LLM cache — Jev decisions only (Jev is deterministic for identical inputs; LLMs are not)
- Not sharing caches across machines or publishing caches (JevCache's publish/add model) — a future change
- Not caching OpenRouter calls differently from direct TypeSafe calls — model identity is part of the key, so different backends produce different fingerprints
- Not cross-session cache persistence in the initial implementation — the in-memory index is loaded at session start from the ledger; subsequent work can add cross-session sharing
- Not encrypting the ledger — it already contains only fingerprints and answers, never raw state
- Not implementing TTL or cache eviction — the ledger grows indefinitely; eviction policy is future work

## Behavior

### Happy path

1. Agent calls `evaluate(state="Production deploy failed.", questions={is_urgent: {type:"noul", instructions:"Is this urgent?"}})`
2. Evaluate tool computes `fingerprint = sha256(model ⊕ schema ⊕ canonical(redact(state)))`
3. In-memory index lookup: MISS
4. Call TypeSafe API → returns `{is_urgent: {noul: 0.97}}`
5. Append `{fingerprint, timestamp, model, answer}` to ledger
6. Insert into in-memory index
7. Return answer to agent
8. Agent calls same `evaluate` again (same state, same questions)
9. Fingerprint computed → HIT in memory → return cached answer in ~0ms for $0

### Edge cases

- **State differs only in volatile fields** (timestamps, IDs, file paths): redaction removes volatile fields before hashing → cache hit
- **Identical state, different schema (question types)**: schema is part of the key → cache miss, different fingerprint
- **Ledger file is corrupt**: log warning, skip cache, fall through to API call
- **Ledger file is unwritable**: log warning, skip cache for writes, continue serving from in-memory index
- **Ledger file grows very large**: initial scope does not evict — acceptable at ~200 bytes/entry, even 100k entries is ~20 MB
- **Schema salt is configured**: hash includes the salt — same state with different salt produces different key
- **Model version changes**: model is part of the key — `jev-latest` vs `jev-2` produce different fingerprints (by design: model outputs may differ)
- **Agent calls evaluate with empty state**: fingerprint computed normally; empty state is valid, cached as such
- **Concurrent evaluate calls**: in-memory index is behind a `RwLock`; ledger appends are sequential (single-writer)

### Cache bypass conditions

The cache is skipped (not consulted) when:
1. `JEVCACHE_DISABLE=true` environment variable is set
2. Ledger initialization fails (corrupt file, permission error)

## Design

### Fingerprint algorithm

```
1. redact(state):
   - Replace email patterns: /[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}/ → "[EMAIL]"
   - Replace phone patterns: /\b\d{7,15}\b/ → "[PHONE]"
   - Replace UUIDs: /[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}/ → "[UUID]"
   - Replace ISO timestamps: /\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}/ → "[TIMESTAMP]"
   - Replace long digit runs (>6 consecutive): /\b\d{7,}\b/ → "[DIGITS]"
   - Skip if state is not a string (structured JSON objects pass through without string-level redaction)

2. canonicalize(state):
   - If JSON object: serialize with sorted keys, no extra whitespace
   - If string: preserve as-is
   - Result is a canonical byte sequence

3. fingerprint(model, schema, state, salt):
   - key_material = model || "\0" || canonical_schema_json || "\0" || canonicalize(redact(state)) || "\0" || salt
   - return hex(sha256(key_material))
```

### Ledger format

Append-only JSON lines file at `~/.jcode/jev-cache/ledger.jsonl`:

```jsonl
{"fp":"a1b2c3...","ts":1726784627,"model":"jev-latest","answer":{"is_urgent":{"noul":0.97}}}
{"fp":"d4e5f6...","ts":1726784628,"model":"jev-latest","answer":{"routing":{"choice":"balanced","confidence":0.87}}}
```

Each line is a complete JSON object. The file is opened in append mode. On session start, the entire file is read into an in-memory `HashMap<String, serde_json::Value>` for ~0ms lookups.

### Rust structure

```rust
// file: source/jcode/crates/jcode-app-core/src/tool/jev_cache.rs

use std::collections::HashMap;
use std::sync::RwLock;
use sha2::{Sha256, Digest};

pub struct JevCache {
    /// Fingerprint → cached answer
    index: RwLock<HashMap<String, serde_json::Value>>,
    /// Append-only ledger path
    ledger_path: std::path::PathBuf,
    /// Optional per-schema salts (schema_name → salt)
    salts: HashMap<String, String>,
    /// Telemetry counters
    stats: RwLock<CacheStats>,
}

pub struct CacheStats {
    pub calls: u64,
    pub hits: u64,
    pub cost_saved: f64,  // approximate, based on $0.0004/decision
}

impl JevCache {
    /// Create a new cache, loading the ledger from disk.
    pub fn new(ledger_dir: &std::path::Path) -> Result<Self> { ... }

    /// Compute a fingerprint for the given evaluate call.
    pub fn fingerprint(model: &str, schema: &serde_json::Value, state: &serde_json::Value, salt: &str) -> String { ... }

    /// Check the cache. Returns Some(answer) on hit, None on miss.
    /// Increments telemetry.
    pub fn check(&self, fp: &str) -> Option<serde_json::Value> { ... }

    /// Store a new answer in the cache.
    /// Appends to the ledger file and updates the in-memory index.
    pub fn store(&self, fp: &str, model: &str, answer: &serde_json::Value) -> Result<()> { ... }

    /// Record a call that bypassed the cache (miss → API call).
    pub fn record_call(&self) { ... }

    /// Return current cache statistics.
    pub fn stats(&self) -> CacheStats { ... }
}

/// Redact PII-like patterns from a state string.
fn redact_state(state: &str) -> String { ... }

/// Canonicalize a JSON value for deterministic hashing.
fn canonicalize_json(value: &serde_json::Value) -> Vec<u8> { ... }
```

### Integration with evaluate tool

The `evaluate` tool's `invoke` method is modified:

```rust
// In tool/evaluate.rs — invoke method pseudocode

fn invoke(&self, input: EvaluateInput) -> ToolOutput {
    // 1. Check if caching is disabled
    if env::var("JEVCACHE_DISABLE").is_ok() {
        return self.call_api(input);
    }

    // 2. Compute fingerprint
    let fp = JevCache::fingerprint(
        &input.model.unwrap_or("jev-latest".into()),
        &serde_json::to_value(&input.questions).unwrap(),
        &input.state,
        &self.salt_for_schema(&input),
    );

    // 3. Check cache
    if let Some(answer) = self.cache.check(&fp) {
        return ToolOutput::ok(answer);
    }

    // 4. Cache miss — call the API
    let response = self.call_api(input)?;

    // 5. Store in cache
    let _ = self.cache.store(&fp, &model, &response.answers);

    // 6. Return answer
    ToolOutput::ok(response.answers)
}
```

### Configuration

| Env var | Default | Purpose |
|---|---|---|
| `JEVCACHE_DISABLE` | (unset) | Set to `true` to bypass the cache |
| `JEVCACHE_DIR` | `~/.jcode/jev-cache` | Ledger storage directory |
| `JEVCACHE_SALT_<schema>` | (unset) | Per-schema salt (e.g., `JEVCACHE_SALT_SECURITY_REVIEW=abc123`) |

## Dependencies

- **Depends on**: `add-jevsdk-evaluate-tool` — the evaluate tool must exist before it can be cached
- **Prior art**: `https://github.com/hyperspaceai/jevcache` (11 stars, Hyperspace) — fingerprint algorithm, redaction model, ledger format
- **Crates already in-tree**: `sha2` (0.10), `hex` (0.4), `serde_json` (1.x), no new crate dependencies

## Touched capabilities

| Capability | Effect |
|---|---|
| `tool/jev_cache.rs` (new) | Cache implementation: fingerprinting, redaction, ledger |
| `tool/evaluate.rs` | Integrated cache check/store in invoke path |
| `tool/mod.rs` | Register `jev_cache` module |
| `jcode stats` | Surface Jev cache hit rate and cost saved |
| `~/.jcode/jev-cache/` (new) | Ledger file on disk |

## Migration

No migration needed. The cache is transparent to callers — the `evaluate` tool's interface is unchanged. Cache misses route to the API exactly as before. Existing API keys and configuration are unaffected.

## Downstream impact

All Jevy proposals that depend on `add-jevsdk-evaluate-tool` inherit caching automatically:

| Downstream change | Caching benefit |
|---|---|
| `jcode-jevy-model-router` | Same prompts recur across sessions — each user's "fix this test" pattern hits |
| `jcode-jevy-review` | Same diff hunks on re-review are cached |
| `jcode-jevy-compaction` | Same transcript state across compaction cycles is cached |
| `jcode-jevy-foreman-supervision` | Same worker state across observation cycles is cached (largest beneficiary) |
| `jcode-jevy-browser-select` | Same DOM state for same page/goal is cached |
| `jcode-jevy-shell-history` | Low benefit — user history changes rapidly, but common command patterns may accumulate over sessions |

## Acceptance

1. Agent calls `evaluate(state="Test", questions={q: {type:"noul", instructions:"Is this a test?"}})` → cache miss → API call → answer returned
2. Agent calls same evaluate with identical inputs → cache hit → answer returned in ~0ms with no API call
3. Agent calls evaluate with state differing only in a UUID → cache hit (redaction strips UUID)
4. Agent calls evaluate with state differing only in a timestamp → cache hit (redaction strips timestamp)
5. Agent calls evaluate with different questions → cache miss (schema differs → different fingerprint)
6. `JEVCACHE_DISABLE=true` → cache bypassed, all calls go to API
7. Ledger file is corrupt → cache skipped gracefully, API call proceeds normally
8. `jcode stats` shows Jev cache statistics: calls, hits, hit rate, cost saved
9. Agent calls evaluate with per-schema salt configured → fingerprint includes salt
10. Concurrent evaluate calls from multiple tool invocations → no panic, correct hit/miss behavior
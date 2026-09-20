# jcode-jevy-evaluate-cache — Tasks

Dependency: `add-jevsdk-evaluate-tool` must be complete before Phase 1 begins. The evaluate tool provides the API call path that the cache wraps.

## Phase 1: Core cache implementation

### Task 1.1: Create JevCache module with fingerprinting

**Scope:** Create `source/jcode/crates/jcode-app-core/src/tool/jev_cache.rs` with:
- `JevCache` struct: `index: RwLock<HashMap<String, Value>>`, `ledger_path: PathBuf`, `salts: HashMap<String, String>`, `stats: RwLock<CacheStats>`
- `JevCache::fingerprint(model, schema, state, salt) -> String` — deterministic SHA-256 fingerprint
- `redact_state(state: &str) -> String` — strip emails, phones, UUIDs, timestamps, long digit runs
- `canonicalize_json(value: &Value) -> Vec<u8>` — sort keys, no extra whitespace
- Tests: same state → same fingerprint; different model → different fingerprint; UUID redaction; timestamp redaction

**Verification:** `cargo test` passes. Manual fingerprint test: two evaluate calls with state differing only in a UUID produce identical fingerprints.

### Task 1.2: Implement ledger read/write

**Scope:** Add to `jev_cache.rs`:
- `JevCache::new(ledger_dir: &Path) -> Result<Self>` — create ledger directory, load existing ledger into in-memory index
- `JevCache::check(&self, fp: &str) -> Option<Value>` — query in-memory index
- `JevCache::store(&self, fp: &str, model: &str, answer: &Value) -> Result<()>` — append JSON line to ledger, update index
- Graceful degradation: if ledger file is corrupt or unwritable, log warning, set `disabled` flag
- Tests: write → read round-trip; corrupt ledger → cache skipped; missing ledger → starts empty

**Verification:** Store a fingerprint/answer pair, restart (reload ledger), verify hit. Corrupt ledger file → cache disabled, no panics.

### Task 1.3: Implement telemetry counters

**Scope:** Add to `jev_cache.rs`:
- `CacheStats` struct: `calls: u64`, `hits: u64`, `cost_saved: f64`
- `JevCache::record_call(&self)` — increment calls counter
- `JevCache::stats(&self) -> CacheStats` — return current counters
- Cost saved estimate: `hits × $0.0004` (configurable via `JEVCACHE_ESTIMATED_COST_PER_CALL`, default 0.0004)
- Tests: call counter increments; hit counter increments on cache hit; cost saved is correct

**Verification:** After 10 calls with 7 hits, stats show calls=10, hits=7, cost_saved ≈ $0.0028.

## Phase 2: Integration with evaluate tool

### Task 2.1: Wire cache into evaluate tool invoke path

**Scope:** Modify `tool/evaluate.rs` invoke method:
- Instantiate `JevCache` at tool construction time (once per session)
- Before API call: compute fingerprint, check cache
- On cache hit: return cached answer, increment hit counter
- On cache miss: call API, store answer in cache, return answer
- Respect `JEVCACHE_DISABLE` env var — skip all cache operations
- Load per-schema salts from `JEVCACHE_SALT_<name>` env vars
- Tests: cache hit returns without HTTP; cache miss calls API and stores; `JEVCACHE_DISABLE` bypasses cache

**Verification:** Integration test: two evaluate calls with identical inputs → first calls API, second returns cached. `JEVCACHE_DISABLE=true` → both call API.

### Task 2.2: Register module and ensure no circular deps

**Scope:** 
- Add `mod jev_cache;` to `tool/mod.rs`
- Ensure `jev_cache.rs` has no dependency on evaluate.rs internals beyond the public `EvaluateInput` type
- Wire ledger directory: use `jcode-storage::jcode_dir()?.join("jev-cache")`

**Verification:** `cargo check` compiles. Module tree is clean. `jcode-storage` dependency is available in jcode-app-core.

## Phase 3: Observability

### Task 3.1: Surface cache stats in jcode stats

**Scope:** 
- Add Jev cache statistics to the existing stats output format
- Show: total calls, cache hits, hit rate (percentage), estimated cost saved
- Stats are per-session (reset on restart) but the ledger persists across sessions
- Format as part of existing session statistics

**Verification:** `jcode stats` shows Jev cache section with calls, hits, hit rate, cost saved. After a cache hit, the numbers update.

### Task 3.2: Add debug logging

**Scope:**
- Log cache hit/miss at `debug` level: `Jev cache HIT fp=a1b2c3...` / `Jev cache MISS fp=d4e5f6...`
- Log cache initialization: `Jev cache loaded N entries from ledger`
- Log cache disabling: `Jev cache disabled (JEVCACHE_DISABLE set)` / `Jev cache disabled (corrupt ledger)`
- Use existing Jcode logging infrastructure (tracing/log)

**Verification:** Run with `RUST_LOG=debug`, observe cache hit/miss log lines. Corrupt ledger produces a warning log line, not a panic.

## Dependency graph

```
Task 1.1 → Task 1.2 → Task 1.3 → Task 2.1 → Task 2.2 → Task 3.1 → Task 3.2
```

Phase 1 (core cache) must complete before Phase 2 (integration). Phase 3 (observability) is independent of Phase 2 but sequenced after for clarity.
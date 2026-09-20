# jcode-jevy-evaluate-cache — Tasks

Dependency: `add-jevsdk-evaluate-tool` must be complete before Phase 1 begins. The evaluate tool provides the API call path that the cache wraps.

## 1. Core cache implementation

- [x] 1.1 Create JevCache module with fingerprinting: In `source/jcode/crates/jcode-app-core/src/tool/jev_cache.rs`, implement `JevCache` struct, `fingerprint()` (SHA-256 of model+schema+canonical(state)+salt), `redact_state()` (strip emails, phones, UUIDs, timestamps, long digit runs), `canonicalize_json()` (sorted keys). Tests: same state→same fingerprint, different model→different fingerprint, UUID/timestamp redaction.
- [x] 1.2 Implement ledger read/write: `JevCache::new()` creates ledger dir and loads in-memory index, `check(fp)` queries index, `store(fp, model, answer)` appends JSON line and updates index. Graceful degradation: corrupt/unwritable ledger → log warning, disable cache. Tests: write→read round-trip, corrupt ledger→skip, missing ledger→empty.
- [x] 1.3 Implement telemetry counters: `CacheStats` (calls, hits, cost_saved), `record_call()`, `stats()`. Cost saved = hits × $0.0004 (configurable). Tests: counters increment correctly.

## 2. Integration with evaluate tool

- [x] 2.1 Wire cache into evaluate tool invoke path: Instantiate `JevCache` at tool construction. Before API call: compute fingerprint, check cache. Hit → return cached + increment counter. Miss → call API, store, return. Respect `JEVCACHE_DISABLE` env var. Load per-schema salts from env. Integration test: two identical calls → first calls API, second returns cached; `JEVCACHE_DISABLE=true` → both call API.
- [x] 2.2 Register module and ensure no circular deps: Add `mod jev_cache;` to `tool/mod.rs`. No dependency on evaluate.rs internals beyond public types. Wire ledger directory to `jcode-storage::jcode_dir()?.join("jev-cache")`. Verify `cargo check` compiles cleanly.

## 3. Observability

- [ ] 3.1 Surface cache stats in jcode stats: Show total calls, cache hits, hit rate (%), estimated cost saved. Per-session stats, ledger persists across sessions. Verify `jcode stats` shows Jev cache section.
- [x] 3.2 Add debug logging: Log cache hit/miss at debug level, cache init (N entries loaded), cache disabling. Use existing tracing/log infrastructure. Verify log lines appear with `RUST_LOG=debug`.

## Dependency graph

```
Task 1.1 → 1.2 → 1.3 → 2.1 → 2.2 → 3.1 → 3.2
```

Phase 1 (core cache) must complete before Phase 2 (integration). Phase 3 (observability) depends on Phase 2.
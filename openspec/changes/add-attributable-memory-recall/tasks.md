## 1. Implementation
- [ ] 1.1 Add `id`, `source`, `trust`, `updated_at` (serde default) to `InjectedMemoryItem` in `crates/jcode-memory-types/src/lib.rs`
- [ ] 1.2 Populate fields where `MemoryEventKind::MemoryInjected` is built in `crates/jcode-base/src/memory*.rs`
- [ ] 1.3 Include short memory id in the model-visible injected block
- [ ] 1.4 Filter inactive/superseded entries and enforce item/char caps with a dropped count

## 2. Verification
- [ ] 2.1 Unit tests: provenance populated, superseded excluded, cap truncation, old-log deserialization
- [ ] 2.2 `cargo test -p jcode-memory-types -p jcode-base memory`
- [ ] 2.3 Runtime check: `./target/selfdev/jcode run --no-update --socket /run/user/1000/jcode-mem.sock '<prompt>'` and inspect latest `memory-events-*.jsonl` for ids
- [ ] 2.4 `openspec validate add-attributable-memory-recall --strict`

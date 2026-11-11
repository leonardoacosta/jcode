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

### Visible overnight bounded currentness repair, 2026-11-09 22:08 UTC

JIT preflight-check-v2-shadow: approved task1.4 currentness subset only, actual public format_entries_for_prompt filters active but not superseded_by while pending-consumption already checks both. Clean memory-types target coordinator serialized, HEAD7abc9a0da main preserved/originHEAD dev mismatch. Synthetic direct API regression, no real corpus/provider access. G1=2,G2=1(full attribution/live/review absent),G3=1(default mismatch),G4=2(offline),G5=2(clean files),G6=1(provenance/char caps unfinished):9/12 shadow. Fix only demonstrated stale formatter result, preserve pending checks and unrelated ranking. No full attribution or model-injection acceptance claim.

### Public formatter currentness repair, 2026-11-09 22:10 UTC

New public three-formatter integration regression first returned obsolete value for legacy active=true/superseded_by=Some entry with limit1. Shared selected_entries_for_prompt now requires both active and no superseded_by before deduplication/item budgeting. Public context/display formatters now return current value, inactive/superseded-only returnsNone, limit0 unchanged. All memory-types checks and existing actual pending-consumption inactive_or_superseded regression pass. No provider call, ranking rewrite or stored-data migration. Current run task-010red/green receipts. Full provenance fields/IDs/character caps/dropped counts/live injection and review/commit/archive remain pending.

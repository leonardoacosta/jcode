## 1. Implementation
- [x] 1.1 Add `id`, `source`, `trust`, `updated_at` (serde default) to `InjectedMemoryItem` in `crates/jcode-memory-types/src/lib.rs`
- [x] 1.2 Carry structured provenance (id, source, trust, updated_at, dropped) from the formatter through `PendingMemory` and the `MemoryInjected` wire event into `MemoryEventKind::MemoryInjected`; parse the rendered block only as a degraded fallback
- [x] 1.3 Include short memory id in the model-visible injected block
- [x] 1.4 Filter inactive/superseded entries and enforce item/char caps with a dropped count

## 2. Verification
- [x] 2.1 Unit tests: provenance populated, superseded excluded, cap truncation, old-log deserialization
- [x] 2.2 `cargo test -p jcode-memory-types -p jcode-base memory`
- [x] 2.3 Runtime check: `./target/selfdev/jcode run --no-update --socket /run/user/1000/jcode-mem.sock '<prompt>'` and inspect latest `memory-events-*.jsonl` for ids
- [x] 2.4 `openspec validate add-attributable-memory-recall --strict`

### Visible overnight bounded currentness repair, 2026-11-09 22:08 UTC

JIT preflight-check-v2-shadow: approved task1.4 currentness subset only, actual public format_entries_for_prompt filters active but not superseded_by while pending-consumption already checks both. Clean memory-types target coordinator serialized, HEAD7abc9a0da main preserved/originHEAD dev mismatch. Synthetic direct API regression, no real corpus/provider access. G1=2,G2=1(full attribution/live/review absent),G3=1(default mismatch),G4=2(offline),G5=2(clean files),G6=1(provenance/char caps unfinished):9/12 shadow. Fix only demonstrated stale formatter result, preserve pending checks and unrelated ranking. No full attribution or model-injection acceptance claim.

### Public formatter currentness repair, 2026-11-09 22:10 UTC

New public three-formatter integration regression first returned obsolete value for legacy active=true/superseded_by=Some entry with limit1. Shared selected_entries_for_prompt now requires both active and no superseded_by before deduplication/item budgeting. Public context/display formatters now return current value, inactive/superseded-only returnsNone, limit0 unchanged. All memory-types checks and existing actual pending-consumption inactive_or_superseded regression pass. No provider call, ranking rewrite or stored-data migration. Current run task-010red/green receipts. Full provenance fields/IDs/character caps/dropped counts/live injection and review/commit/archive remain pending.

### Implementation + verification, 2026-11-19 (apply-all worker)

- 1.1 `InjectedMemoryItem` (jcode-memory-types) now Serialize/Deserialize with `id/source/trust/updated_at` as `Option` + serde default; `MemoryEventKind::MemoryInjected` gains `dropped: usize`.
- 1.2 (superseded by the review-fix pass below) provenance was first registered through a process-global table.
- 1.3 Block renders `N. [mem <last 8 chars of id>] content` (`short_memory_id`, no content or full id exposed). `MemoryManager::forget` also accepts a unique short id.
- 1.4 `selected_entries_for_prompt` filters inactive/superseded first, dedups, then caps at `MAX_INJECTED_ITEMS`=8 items and `MAX_INJECTED_CHARS`=4000; omitted count appended as `(N more memories omitted)` and recorded as `dropped` in the event. limit 0 is not truncation. Earlier currentness notes above preserved.
- 2.1 tests: memory-types `tests/prompt_currentness.rs` (short id, item cap, char cap, old/new item serde); jcode-base `memory/activity.rs::provenance_tests` (provenance populated, unregistered id, dropped count, legacy prompt), `memory_tests.rs::forget_accepts_short_id_from_injected_block`; existing superseded/inactive tests kept.
- 2.2 `JCODE_REMOTE_CARGO=0 scripts/dev_cargo.sh test -p jcode-memory-types -p jcode-base memory` -> 131 passed, 0 failed, 3 ignored (lib) + integration tests ok (log scratch/applyall/memory-test.log). First run also hit unrelated `jev::tests::voice_hedge_failure_waits_for_primary_and_memory_never_hedges` (not in the final run; timing-flaky, not memory code).
- Clippy: `clippy -p jcode-memory-types -p jcode-base --all-targets --no-deps -- -D warnings` exit 0. `cargo check -p jcode-tui -p jcode-app-core --tests` exit 0.
- 2.4 `openspec validate add-attributable-memory-recall --strict` -> valid.
- 2.3 CHECKED via tests/e2e scripted-provider test (coordinator decision: the `jcode` binary has no offline provider path, so a real `jcode run` was not used; no real provider touched). New `tests/e2e/memory_provenance.rs::injected_memory_logs_provenance_and_dropped_count` runs a real `Agent` turn with `MockProvider` under a private `JCODE_HOME` (tempdir, `JCODE_TEST_SESSION=1`), publishes 10 persisted entries via `set_pending_memory_for_project`, then reads `<JCODE_HOME>/logs/memory-events-*.jsonl` and asserts the `memory_injected` event: `dropped == 2`, 8 items each with `id` (a stored id), `source`, `trust`, `updated_at`, and no `[mem x]` marker in content. Command: `JCODE_REMOTE_CARGO=0 scripts/dev_cargo.sh test --test e2e memory_provenance` -> 1 passed, 0 failed (log: scratch/applyall/memory-e2e.log). Supporting fix: `memory_log.rs` `log_dir()` now uses `jcode_storage::logs_dir()` (honors `JCODE_HOME`; logger reopens when the dir changes). Previously it hardcoded `~/.jcode/logs`, so the private-home log was unreachable.

### Review-fix pass, 2026-11-19 (memory-auth-fixes worker)

Independent review found the first 1.2 design unsound (process-global side table, text re-parsing, short-id collisions, tool-path truncation). Reworked:

- Provenance is structured end to end. `format_relevant_prompt_with_items` (jcode-memory-types) returns the text plus `InjectedMemoryItem`s and `dropped`, built while rendering. `publish_scoped_memory` stores them on `PendingMemory.items/dropped`; `ServerEvent::MemoryInjected` carries `items` and `dropped` (serde default, items skipped when empty) so the remote TUI path is not text-only. `record_injected(prompt, count, age_ms, items, dropped)` consumes them. The global `INJECTION_PROVENANCE` table and `register_injection_provenance` are removed, so there is no 256-entry limit and no cross-session overwrite.
- Text parsing (`parse_injected_items`) is used only when no structured items arrive (old server, prompt-only publishers). It trusts the `(N more memories omitted)` trailer only as the final line and attaches only a visible short id, so memory text imitating the trailer cannot inflate `dropped`. Known limit: this fallback cannot recover source/trust/updated_at.
- `short_memory_id` returns the whole random tail of `mem_<ts>_<rand>` (non-matching shapes whole). `MemoryManager::forget` by short id now errors with the sorted candidate full ids when it is ambiguous and removes nothing.
- The item and char caps apply only to injection. The recall tool path (`format_entries_for_prompt`) honours only its `limit` and renders no ids or trailer.
- Old logs still deserialize: `memory_log::injected_detail` test parses a whole legacy `memory_injected` JSONL line (no `dropped`, items with only section/content) and round-trips a new one.

Evidence (all `JCODE_REMOTE_CARGO=0 scripts/dev_cargo.sh`):
- `test -p jcode-memory-types -p jcode-base --lib --tests`: memory-types all pass; jcode-base 2554 passed, 4 failed (`cloud_environment::...concurrent_manifest...`, `usage::cache::account_snapshot_tests::{cached_account_snapshot_is_scoped_fresh_and_nonblocking,pool_prefers_fresh_quota_within_priority_tier}`, `usage::anthropic_response::...response_limits_recover...`), all four pass when rerun alone (8 passed), unrelated to memory (parallel load/shared state). New tests: `provenance_tests::*`, `forget_by_colliding_short_id_is_ambiguous_and_removes_nothing`, `pending_memory_carries_provenance_past_any_process_wide_table_limit` (300 entries x 2 sessions), `prompt_currentness` short-id/tool-path/structured-items/char-cap tests, app-core `recent_recall_is_not_lowered_by_the_injection_cap`.
- `test --test e2e memory_provenance`: 1 passed. Still asserts `dropped == 2` with 8 items and full provenance through a real Agent turn.
- `clippy -p jcode-memory-types -p jcode-base -p jcode-protocol -p jcode-tui -p jcode-app-core --all-targets --no-deps -- -D warnings`: exit 0 (after fixing two `PendingMemory` literals in `agent_tests.rs` and an `#[expect(clippy::too_many_arguments)]` on `show_injected_memory_context`).

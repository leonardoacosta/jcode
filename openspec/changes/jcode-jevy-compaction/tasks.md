# jcode-jevy-compaction — Tasks

Dependency: `add-jevsdk-evaluate-tool` must be complete before Phase 1 begins. JevCompactor uses the evaluate tool to score tool call relevance.

## 1. Core compaction algorithm

- [ ] 1.1 Implement tool call collection and pairing: In `agent/compaction.rs`, `collect_tool_calls()` pairs `tool_use` with `tool_result` by `tool_use_id`. Pin calls in message[0] and newest `preserve_recent_messages` (default 4). Verify 10 calls → 10 pairs, pinned correctly, unpaired flagged.
- [ ] 1.2 Implement staged state fitting: `fit_state()` replaces tool results with short notes. Staged truncation: truncate inputs (1000→200→60 chars), abridge long texts (head+tail), collapse old messages, reduce old calls to one-liners, omit call-less messages, fold runs. Character-based token estimation. Target: fit into 25k tokens. Verify large transcript (500+ messages) fits; critical info survives.
- [ ] 1.3 Implement question batching: `batch_questions()` generates two Noul questions per non-pinned call (keep_call, keep_result), splits across concurrent requests staying under 30k token limit. Verify 50 non-pinned calls → 100 questions batched across ~3-5 requests.
- [ ] 1.4 Implement decision application and message rebuild: `decide_call()` applies keep_threshold (default 0.5): keep_result≥threshold→keep both, keep_call≥threshold→keep call+truncate result, else→remove both. `apply_decisions()` rebuilds messages, removes empties, ensures no orphans. Compute compaction ratio; ratio <25% returns Skip. Verify 100-message transcript compacts to ~50, all text intact.

## 2. Integration with CompactionManager

- [ ] 2.1 Add JevCompactor as a compaction strategy: Define `JevCompactor` struct with configurable thresholds. Implement `CompactionStrategy` trait. Add strategy selection to `CompactionManager`: try Jev first, fall back to LLM. Config: `compaction.jev_keep_threshold` (0.5), `compaction.jev_preserve_recent` (4).
- [ ] 2.2 Add graceful degradation: Jev unavailable → fall back to LLM. Timeout → retry once, then LLM. State won't fit → error, fall back to LLM. Malformed response → treat as "keep call, drop result". Single message/zero non-pinned → return unchanged. Verify each edge case triggers correct fallback, no crashes.

## 3. Testing and observability

- [ ] 3.1 Add compaction integration tests: 50-tool-call session → stale dropped, recent preserved. Critical file paths/errors survive. User/assistant text verbatim. Ratio ≥25% else Skip. LLM fallback when Jev unavailable. Single message unchanged. All below threshold → all dropped cleanly. Pinned calls always survive.
- [ ] 3.2 Add compaction metrics logging: Log at info: "JevCompactor: N calls assessed, M kept, ratio X%". Log fallback at warn. Track per-session: compactions run, jev vs llm count, calls pruned.

## Dependency graph

```
Task 1.1 → 1.2 → 1.3 → 1.4 → 2.1 → 2.2 → 3.1 → 3.2
```

Phase 1 (core algorithm) must complete before Phase 2 (integration). Phase 3 (testing/observability) depends on Phase 2.
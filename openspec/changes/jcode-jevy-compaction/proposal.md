---
execution:
  version: 1
  depends_on: ["add-jevsdk-evaluate-tool"]
---

# jcode-jevy-compaction

## Summary

Add `JevCompactor` as an alternative context compaction strategy in Jcode. Instead of asking an LLM to summarize old conversation turns (lossy, expensive, slow), Jev scores each historical tool call for continued relevance and only drops or truncates calls it is confident are obsolete. User and assistant text stays verbatim — nothing is rewritten.

The algorithm is adapted from `tamaratran/fast-jev-compaction` (4k stars, MIT).

## Motivation

Jcode's current compaction (`agent/compaction.rs`) uses LLM-based summarization. A summary is lossy: file paths, exact error messages, constraints, and command outputs can disappear even when they matter later. LLM summarization costs ~$0.02 per compaction and takes ~3 seconds. Jev-driven compaction costs ~$0.002 and takes ~500ms — and never loses critical information because it never rewrites anything.

## Actors

- **Jcode agent**: triggers compaction when context approaches token budget
- **CompactionManager**: existing compaction orchestrator in `agent/compaction.rs`
- **JevCompactor**: new compaction strategy
- **LLM Compactor** (existing): retained as fallback when Jev is unavailable or compaction ratio is too low

## Scope

- New `JevCompactor` struct implementing compaction strategy in `agent/compaction.rs`
- 5-step algorithm: pair calls → fit state → batch questions → decide → rebuild
- Token estimation without a full tokenizer (character-based heuristic)
- Staged state fitting into Jev's ~32k token request limit
- Configurable `keep_threshold` (default 0.5) and `preserve_recent_messages` (default 4)
- Integration with existing `CompactionManager` via a strategy pattern
- Fallback to LLM summarization when Jev is unavailable or compaction ratio < 25%

## Non-scope

- Not removing the LLM compactor — it remains as fallback
- Not changing when compaction triggers — same trigger conditions
- Not rewriting conversation text — only removing/pruning tool calls
- Not handling multi-session or cross-session compaction

## Behavior

### Happy path

1. Compaction is triggered (context approaching token budget)
2. `JevCompactor` is selected as the active strategy
3. Tool calls/results are collected and paired by `tool_use_id`
4. State is fitted into 25k tokens using staged truncation
5. Per-call Noul questions are batched into parallel Jev requests
6. Decisions are applied: keep both / keep call+truncate result / drop both
7. Message list is rebuilt — no orphaned results, text stays verbatim
8. Compaction ratio is computed; if < 25%, fall back to LLM summarizer

### Edge cases

- **Single message session**: no compaction needed, return unchanged
- **All calls pinned**: return unchanged
- **State won't fit even after maximum truncation**: return error, fall back to LLM
- **Jev request timeout**: retry once, then fall back to LLM
- **All calls score below threshold**: all non-pinned calls dropped — valid outcome
- **Zero non-pinned calls**: return unchanged
- **Malformed Jev response**: log warning, treat as "keep call, drop result" for safety

## Design

### Algorithm (adapted from fast-jev-compaction)

```
1. collectToolCalls(transcript)
   → Pair each tool_use with its tool_result by tool_use_id
   → Pin calls in message[0] and the newest preserve_recent_messages

2. fitState(state, maxStateTokens=25000)
   → Replace tool results with short notes: "ok, N chars (omitted)"
   → Staged truncation, each applied only if previous insufficient:
     a. Truncate tool inputs to 1000, 200, then 60 chars
     b. Abridge long texts to head+tail
     c. Collapse old non-pinned messages to "[… N chars omitted …]"
     d. Reduce old tool calls to one line: "t12 Read path=a.ts → ok 480ch"
     e. Omit old call-less messages
     f. Fold runs of old call-only messages into one entry
   → Token estimation: word/6 + digit/2 + symbol/1 (no tokenizer needed)

3. batchCalls(calls, maxRequestTokens=30000)
   → For each non-pinned call, two Noul questions:
     "keep_call": does knowing this call was made still matter?
     "keep_result": are these contents still needed and re-running wouldn't do?
   → Split questions across concurrent requests to stay under token limit

4. decideCall(result, keepThreshold=0.5)
   → keep_result ≥ threshold → keep call AND result
   → keep_call ≥ threshold → keep call, truncate result to head chars
   → else → remove call + result

5. applyDecisions(messages, decisions)
   → Rebuild message list
   → Remove messages that lost all content
   → Ensure no orphaned results (result without its call)
```

### Rust sketch

```rust
struct JevCompactor {
    keep_threshold: f64,
    preserve_recent_messages: usize,
    max_state_tokens: usize,
    max_request_tokens: usize,
}

impl JevCompactor {
    fn compact(&self, messages: &[Message]) -> CompactionResult {
        let calls = self.collect_tool_calls(messages);
        let state = self.fit_state(messages, &calls);
        let batches = self.batch_questions(&calls);
        let decisions = self.ask_jev(batches);
        let rebuilt = self.apply_decisions(messages, &calls, decisions);
        let ratio = reduction_ratio(messages.len(), rebuilt.len());
        if ratio < 0.25 { return CompactionResult::Skip; }
        CompactionResult::Applied(rebuilt, decisions)
    }
}
```

## Dependencies

- **Research**: `research/jev-ecosystem-deep-research.md`
- **Research**: `research/jev-jcode-feature-breakdown.md`
- **Prior art**: `https://github.com/tamaratran/fast-jev-compaction` (4k stars, MIT)
- **Depends on**: `add-jevsdk-evaluate-tool` — JevCompactor uses the evaluate tool to call Jev

## Touched capabilities

| Capability | Effect |
|-----------|--------|
| `agent/compaction.rs` | Add JevCompactor strategy |
| `compaction` manager | Strategy pattern for selecting compactor |
| `tool/evaluate.rs` | Used by JevCompactor for Jev calls |

## Acceptance

1. Session with 50+ tool calls is compacted — stale calls dropped, recent calls preserved
2. Critical file paths and error messages survive compaction
3. User and assistant text messages are unchanged after compaction
4. Compaction ratio ≥ 25% — otherwise returns Skip
5. Jev unavailable → falls back to LLM summarization
6. Session with only 3 messages → returns unchanged (nothing to drop)
7. All calls score below threshold → all non-pinned calls dropped cleanly
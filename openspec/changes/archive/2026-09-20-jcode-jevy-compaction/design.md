## Context

Jcode's current compaction (`agent/compaction.rs`) uses LLM-based summarization. Summaries are lossy — file paths, error messages, constraints, and command outputs can disappear. LLM summarization costs ~$0.02 and takes ~3s. `tamaratran/fast-jev-compaction` (4k stars, MIT) demonstrates Jev-driven compaction that never rewrites anything: it scores each historical tool call for continued relevance and only drops or truncates calls confident it's obsolete.

The evaluate tool provides the Jev API integration layer. The CompactionManager already exists with a strategy pattern for compaction strategies. The LLM compactor remains as fallback.

## Goals / Non-Goals

**Goals:**
- Add `JevCompactor` as alternative compaction strategy
- 5-step algorithm: pair calls → fit state → batch questions → decide → rebuild
- Token estimation without full tokenizer (character-based heuristic)
- Staged state fitting into Jev's ~32k token request limit
- Configurable keep_threshold (default 0.5) and preserve_recent_messages (default 4)
- Fallback to LLM summarization when Jev unavailable or compaction ratio < 25%

**Non-Goals:**
- Not removing the LLM compactor
- Not changing compaction trigger conditions
- Not rewriting conversation text (only removing/pruning tool calls)
- Not handling multi-session or cross-session compaction

## Decisions

**Tool-call-only compaction:** Only tool calls and results are scored for removal. User and assistant text stays verbatim. This preserves all human communication and agent reasoning while pruning potentially stale tool interactions.

**Paired call+result scoring:** Each tool_use is paired with its tool_result by tool_use_id. Two Noul questions per pair: "does knowing this call was made still matter?" (keep_call) and "are these contents still needed?" (keep_result). keep_result ≥ threshold → keep both; keep_call ≥ threshold → keep call but truncate result; else → remove both.

**Staged state fitting:** Rather than a single truncation pass, apply increasingly aggressive truncation stages only if previous stages insufficient. This preserves maximum context while respecting Jev's token limit. Character-based token estimate: chars/6 for words, chars/2 for digits, chars/1 for symbols — avoids full tokenizer dependency.

**Fail-to-LLM:** Jev unavailable → fall back to LLM summarization. Compaction ratio < 25% → skip (not worth changing the context). Malformed Jev response → treat conservatively (keep call, drop result). This ensures the system never degrades below current quality.

**Pinned messages:** First message (system prompt) and newest `preserve_recent_messages` (default 4) are always pinned and never dropped. This preserves critical context at both ends of the transcript.

## Risks / Trade-offs

- **Risk:** Jev may incorrectly drop a tool call that later becomes relevant. Mitigation: conservative threshold (0.5), pinned recent messages, LLM fallback.
- **Risk:** Character-based token estimation may be inaccurate vs real tokenization. Mitigation: staged fitting with generous margins.
- **Trade-off:** Jev compaction is faster and cheaper than LLM but can't produce summaries. Justification: we don't want summaries anyway — they're lossy. Pruning is strictly better for our use case.
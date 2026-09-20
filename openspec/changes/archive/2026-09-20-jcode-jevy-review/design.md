## Context

LLM code reviews are thorough but expensive: a full repo scan can cost $3-5 and take minutes. `devagrawal09/jev-review` (331 stars, MIT) demonstrates a two-tier approach: Jev screens everything cheaply (~$0.001 per hunk, ~300ms), LLM deeply reviews only flagged high-risk hunks. This preserves review quality while cutting costs by 10-50×.

The evaluate tool provides the Jev API integration. The existing `/review` skill provides the LLM-based review framework this screener integrates with.

## Goals / Non-Goals

**Goals:**
- Jev pre-screening layer: 5-dimension Noul risk matrix per diff hunk or source file
- Threshold-based routing: any dimension > 0.7 → full LLM review; all ≤ 0.7 → Jev-only
- Two modes: change review (git diff) and codebase scan (all files)
- Diff chunking: split large diffs into file-level hunks for independent screening
- Results integrated into existing `/review` output format with confidence scores

**Non-Goals:**
- Not replacing the full LLM reviewer (remains for flagged hunks)
- Not generating explanations (Jev provides probabilities, LLM provides explanations)
- Not running static analysis or compiler diagnostics
- Not integrating with external review tools

## Decisions

**5-dimension risk matrix:** correctness_risk, security_risk, reliability_risk, compatibility_risk, test_gap. These cover the main categories of code review concern. Any dimension > threshold (default 0.7) triggers full LLM review. This "any dimension" approach is conservative (rather than requiring multiple dimensions).

**File-level chunking:** Diffs are split into file-level hunks. Each hunk includes diff text, file path, language, and related test paths inferred by convention (`src/foo.rs` → `tests/foo_test.rs`). Large files (>8000 chars) are split at function boundaries. This keeps Jev state within token limits while preserving file-level context.

**Jev context for LLM:** When a hunk is flagged, Jev's risk scores are passed as context to the LLM reviewer. This helps the LLM focus its attention on the specific risk dimensions that Jev flagged, improving review quality.

**Graceful degradation:** Jev unavailable → all hunks to full LLM review. Per-hunk timeout → that hunk to LLM. This ensures review quality never degrades below current baseline. All-low → zero LLM cost (best case). All-high → same cost as current (no net loss).

## Risks / Trade-offs

- **Risk:** Jev may miss a real issue (false negative), routing a dangerous hunk to Jev-only. Mitigation: conservative threshold (0.7) with "any dimension" triggering; false negatives are possible but rare with calibrated probabilities.
- **Risk:** Jev may flag low-risk hunks (false positive), routing them to expensive LLM review. Mitigation: cost of false positive = cost of the LLM review that would have happened anyway.
- **Trade-off:** Lower threshold → more safety but less cost savings. Justification: 0.7 default is informed by jev-review's empirical results; configurable for users who want different trade-offs.
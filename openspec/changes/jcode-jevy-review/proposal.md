---
execution:
  version: 1
  depends_on: ["add-jevsdk-evaluate-tool"]
---

# jcode-jevy-review

## Summary

Add a Jev-driven pre-screening layer to Jcode's `/review` skill. Before running a full LLM code review, Jev screens each diff hunk or source file across five risk dimensions (correctness, security, reliability, compatibility, test coverage) with calibrated Noul probabilities. Hunks scoring above 0.7 on any dimension are routed to full LLM review. Hunks scoring below are assessed as low-risk with Jev confidence scores alone. This reduces review costs by 10-50× while maintaining coverage of high-risk changes.

The pipeline is adapted from `devagrawal09/jev-review` (331 stars, MIT).

## Motivation

LLM code reviews are thorough but expensive: a full repo scan can cost $3-5 and take minutes. Jev can answer 5 Noul questions per diff hunk in ~300ms for ~$0.001, providing calibrated risk probabilities. A two-tier approach — Jev screens everything cheaply, LLM deeply reviews only flagged hunks — preserves review quality while cutting costs by 10-50×.

## Actors

- **Jcode agent**: invokes `/review` skill on a diff or codebase
- **Jev pre-screener**: runs 5-dimension Noul risk matrix per hunk/file
- **Full LLM reviewer** (existing): deeply reviews high-risk hunks
- **Jcode user**: receives review results with confidence scores

## Scope

- New `JevReviewScreener` module in or near `tool/skill.rs`
- 5-dimension Noul risk matrix per diff hunk or source file:
  - `correctness_risk`: probability this change introduces a logic error
  - `security_risk`: probability this change introduces a vulnerability
  - `reliability_risk`: probability this change affects stability/error handling
  - `compatibility_risk`: probability this change breaks existing contracts
  - `test_gap`: probability existing tests don't adequately cover this change
- Threshold-based routing: any dimension > 0.7 → full LLM review; all ≤ 0.7 → Jev-only assessment
- Results integrated into existing review output format with confidence scores
- Change review mode (git diff) and codebase scan mode (all files)
- Diff chunking: split large diffs into file-level hunks, each independently screened

## Non-scope

- Not replacing the full LLM reviewer — it remains for flagged hunks
- Not generating explanations — Jev provides probabilities, LLM provides explanations for flagged items
- Not running static analysis or compiler diagnostics
- Not integrating with external review tools

## Behavior

### Happy path (change review)

1. Agent calls `/review` with a git diff
2. Diff is chunked into file-level hunks
3. Each hunk is sent to Jev with 5 Noul questions:
   ```
   state: {diff_hunk: "...", file_path: "src/auth.rs", related_tests: ["tests/auth_test.rs"]}
   questions: {
     correctness_risk: {type: "noul", instructions: "Could this change introduce a logic error?"},
     security_risk: {type: "noul", instructions: "Could this introduce a vulnerability?"},
     ...
   }
   ```
4. Jev returns: correctness=0.12, security=0.05, reliability=0.08, compatibility=0.15, test_gap=0.72
5. Policy: test_gap > 0.7 → flag for full LLM review
6. Flagged hunk is sent to LLM for deep review with Jev's risk scores as context
7. Low-risk hunks get Jev-only: "Low risk (confidence: correctness=0.88, security=0.95, ...)"
8. Results are presented in the existing `/review` format

### Edge cases

- **Empty diff**: return "no changes to review"
- **Single-file change with no related tests**: test_gap will likely score high, flags for LLM review
- **Binary files in diff**: skip, mark as "binary — manual review required"
- **Diff too large for Jev state**: chunk into file-level hunks, screen independently
- **All hunks score low**: return Jev-only assessment — no LLM cost
- **All hunks score high**: all routed to LLM — same quality as current, no cost saving (but no added cost either)

## Design

### Risk matrix

```rust
struct RiskMatrix {
    correctness_risk: f64,    // 0-1, Noul: "Could this introduce a logic error?"
    security_risk: f64,       // 0-1, Noul: "Could this introduce a vulnerability?"
    reliability_risk: f64,    // 0-1, Noul: "Could this affect stability or error handling?"
    compatibility_risk: f64,  // 0-1, Noul: "Could this break existing contracts or APIs?"
    test_gap: f64,            // 0-1, Noul: "Do existing tests adequately cover this change?"
}

impl RiskMatrix {
    fn needs_full_review(&self, threshold: f64) -> bool {
        self.correctness_risk > threshold
            || self.security_risk > threshold
            || self.reliability_risk > threshold
            || self.compatibility_risk > threshold
            || self.test_gap > threshold
    }
}
```

## Dependencies

- **Depends on**: `add-jevsdk-evaluate-tool` — review screener uses evaluate
- **Prior art**: `https://github.com/devagrawal09/jev-review` (331 stars, MIT)
- **Priority**: CONSIDER — medium effort, low risk

## Touched capabilities

| Capability | Effect |
|-----------|--------|
| `tool/skill.rs` | Add Jev pre-screening to `/review` skill |
| New: `review/screener.rs` | JevReviewScreener implementation |
| `tool/evaluate.rs` | Used for risk matrix calls |

## Acceptance

1. Diff with a SQL injection vulnerability → security_risk scores > 0.7, routed to LLM
2. Diff only changing comments → all dimensions score < 0.3, Jev-only assessment returned
3. Diff touching auth module without test changes → test_gap > 0.7, flagged for full review
4. Empty diff → "no changes to review" returned
5. Jev unavailable → all hunks routed to full LLM review (graceful degradation)
6. Results integrate with existing `/review` output format
7. Large multi-file diff is correctly chunked and each hunk independently screened
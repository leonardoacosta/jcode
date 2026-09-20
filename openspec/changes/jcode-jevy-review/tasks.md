# jcode-jevy-review — Tasks

Dependency: `add-jevsdk-evaluate-tool` must be complete before Phase 1 begins. The review screener uses the evaluate tool for risk assessments.

Priority: CONSIDER — medium effort, low risk.

## 1. Jev risk screening

- [ ] 1.1 Implement diff chunking for review: Create `review/screener.rs` with `chunk_diff()` that chunks diffs into file-level hunks with diff text, file path, language, related test paths (inferred by convention). Binary files flagged as skip. Max hunk size 8000 chars; split oversized files at function boundaries.
- [ ] 1.2 Implement 5-dimension risk matrix via Jev: `screen_hunk()` sends each hunk to Jev with 5 Noul: correctness_risk, security_risk, reliability_risk, compatibility_risk, test_gap. Timeout 5s per hunk. Respects Jev cache. Verify SQL injection→security high, comment changes→all low, timeout→error routed to LLM.
- [ ] 1.3 Implement threshold-based routing: `needs_full_review()`: any dimension >0.7 (configurable)→flag for LLM; all ≤0.7→Jev-only. Returns RiskMatrix with scores + routing decision.

## 2. Review integration

- [ ] 2.1 Integrate screener into /review skill: On `/review` invocation: chunk diff → screen each hunk → low-risk get Jev-only summary, flagged get full LLM review with Jev context. Final output combines both. Empty diff returns "no changes to review".
- [ ] 2.2 Implement full codebase scan mode: Scan all files (walk source tree, exclude configured patterns). Screen each file independently. Group low-risk directories: "N files low risk, see details". Verify full repo scan completes within limits.
- [ ] 2.3 Implement graceful degradation: Jev unavailable→all hunks to LLM. Per-hunk timeout→that hunk to LLM. Invalid response→hunk to LLM. All low→zero LLM cost. All high→same cost as current. Very large diff (>50 files)→sample first 50 with note. Verify no partial or missing results.

## 3. Output and observability

- [ ] 3.1 Format review output with risk scores: Jev-only: "✓ file — Low risk (correctness=X, security=X, ...)". Flagged: "⚠ file — Full review (risk=X) [LLM review follows]". Binary: "⊘ file — Binary, manual review". Summary with counts + estimated cost saved.
- [ ] 3.2 Add screening telemetry: Track files/hunks screened, LLM reviews triggered/avoided, cost saved. Log at info: "JevReview: N hunks screened, M flagged, $X saved". Surface in `jcode stats`.

## Dependency graph

```
Task 1.1 → 1.2 → 1.3 → 2.1 → 2.2 → 2.3 → 3.1 → 3.2
```

Phase 1 (core screening) must complete before Phase 2 (integration). Phase 3 (output/observability) depends on Phase 2.
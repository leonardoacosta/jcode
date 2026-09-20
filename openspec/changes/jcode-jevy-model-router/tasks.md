# jcode-jevy-model-router — Tasks

Dependency: `add-jevsdk-evaluate-tool` must be complete before Phase 1 begins. The router uses the evaluate tool for complexity classification.

Priority: CONSIDER — medium effort, medium risk.

## 1. Jev complexity classification

- [ ] 1.1 Implement prompt text extraction for routing: In `agent/provider.rs`, `extract_routing_prompt()` intercepts first request per user turn, extracts only prompt text (no source code/tool results/history/system prompts). Returns None for tool-loop continuations. Marks turn as routed. Verify correct extraction on various turn types.
- [ ] 1.2 Implement Jev complexity classification: `JevRouter::classify_complexity()` sends prompt to Jev with 4 questions: complexity Score (trivial→expert), reasoning_required Noul, tool_complexity Noul, context_size Noul. Maps scores to tiers (Fast/Balanced/Strong/Long). Timeout 5s. On failure returns KeepCurrent.

## 2. Policy engine

- [ ] 2.1 Implement deterministic routing policy rules: `route_tier()`: explicit user choice wins; Jev failure→KeepCurrent (fail-open); low confidence (<0.6)→no downgrade, cap upgrade at Balanced; large conversation (>50% budget)→refuse downgrade; all same tier→skip Jev; already cheapest→skip downgrade; unavailable tier→step upward.
- [ ] 2.2 Implement tier model resolution: Per-provider config `[provider.<name>.tiers]` with fast/balanced/strong/long keys. Resolution: unconfigured tier steps upward. No tiers configured → routing disabled. Support all active Jcode providers.

## 3. Provider integration

- [ ] 3.1 Wire JevRouter into provider dispatch pipeline: Add as middleware in provider dispatch. Intercept first request per turn → classify → apply policy → resolve tier model → route. Tool-loop continuations reuse turn's model.
- [ ] 3.2 Add routing telemetry and logging: Log routing decisions at debug: "JevRouter: prompt → Strong (complexity=4, confidence=0.82)". Track per-session: turns, tier distribution, Jev failures, cost saved estimate. Surface in `jcode stats`.
- [ ] 3.3 Implement fail-open and edge case handling: Jev unavailable→KeepCurrent (warn). Rate limited→KeepCurrent (log). Routing disabled→skip Jev entirely. Prompt exceeds limit→truncate last 32k chars. Concurrent turns→first routes, subsequent reuse. Verify no broken turns from routing failures.

## Dependency graph

```
Task 1.1 → 1.2 → 2.1 → 2.2 → 3.1 → 3.2 → 3.3
```

Phase 1 (classification) must complete before Phase 2 (policy). Phase 3 (integration) depends on Phase 2.
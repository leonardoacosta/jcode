## Context

Jcode agents currently use static model selection — either the user's chosen model or a global default. Simple queries ("what does this file do?") use the same model as complex tasks ("implement auth middleware"). `gargpratyush/jev-router` (191 stars, MIT) demonstrates Jev-driven complexity classification for per-turn routing, saving 40-60% on API costs.

The evaluate tool provides the Jev API integration. The provider dispatch pipeline (`agent/provider.rs`) already supports model selection and routing — this adds a middleware layer before it.

## Goals / Non-Goals

**Goals:**
- Jev-driven complexity classification on first request of each user turn
- Tier taxonomy: Fast (cheapest), Balanced (default), Strong (complex), Long (extended context)
- Per-provider tier model mappings (configurable)
- Deterministic policy rules in Rust code
- Fail-open: Jev failure/timeout keeps current model

**Non-Goals:**
- Not a proxy — native middleware in provider dispatch pipeline
- Not removing manual model selection (`/model` picker still works)
- Not routing tool-loop continuations differently
- Not implementing tier-specific prompt modification
- Not routing based on file types or repository context (prompt text only, for privacy)

## Decisions

**Prompt-text-only classification:** Only the user's prompt text is sent to Jev. No source code, tool results, conversation history, or system prompts. This is both privacy-preserving and sufficient — the complexity of a task is usually evident from the prompt alone.

**4-dimension classification:** complexity Score (trivial→expert), reasoning_required Noul, tool_complexity Noul, context_size Noul. The Score provides a tier anchor; the Noul questions provide nuance for edge cases (e.g., "simple task but needs multi-step reasoning" → Balanced not Fast).

**Tier mapping with boundaries:** trivial→Fast, simple→Fast/Balanced (boundary at 0.5), moderate→Balanced, complex→Strong, expert→Strong/Long (boundary at 0.5). The boundaries handle ambiguous classifications conservatively.

**Conservative policy rules:** Low confidence (<0.6) never downgrades and caps upgrades at Balanced. Large conversations (>50% token budget) refuse downgrades. Unavailable tiers step upward, never silently choose weaker model. These rules prioritize correctness over cost savings.

**Fail-open:** Jev unavailable, timeout, rate limited → KeepCurrent. The agent proceeds normally; routing is a quality-of-life improvement, not a critical path. Users never see routing errors.

## Risks / Trade-offs

- **Risk:** Jev may misclassify task complexity, routing simple tasks to weak models or complex tasks to cheap models. Mitigation: conservative policy rules (low confidence caps upgrades, large context refuses downgrades).
- **Risk:** Model quality within tiers varies by provider. Mitigation: per-provider tier mappings let users choose specific models.
- **Trade-off:** Routing adds ~300ms latency per turn (Jev call). Justification: 40-60% cost savings outweigh marginal latency for most users.
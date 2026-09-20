---
execution:
  version: 1
  depends_on: ["add-jevsdk-evaluate-tool"]
---

# jcode-jevy-model-router

## Summary

Add Jev-driven model tier selection as a provider middleware layer. On each fresh user turn, the prompt text is sent to Jev for complexity classification. Jev returns a tier recommendation (Fast/Balanced/Strong/Long), and the provider dispatcher routes to the configured model for that tier. This saves 40-60% on API costs by routing simple queries to cheap models while reserving strong models for complex tasks.

The approach is adapted from `gargpratyush/jev-router` (191 stars, MIT) but implemented as native middleware rather than a proxy.

## Motivation

Jcode agents currently use a static model selection — either the user's chosen model or a global default. Simple queries ("what does this file do?", "summarize this error") use the same model as complex tasks ("implement auth middleware", "debug race condition"). Jev can classify task complexity in ~300ms for ~$0.0002, enabling per-turn routing that saves 40-60% on API costs without degrading correctness.

## Actors

- **Jcode agent**: sends user turns through the provider dispatch layer
- **Jev Router middleware**: intercepts first request per turn, classifies complexity
- **Provider dispatcher** (`agent/provider.rs`): routes to the configured model for the selected tier
- **Jcode user**: configures tier model mappings per provider

## Scope

- New `JevRouter` middleware in `agent/provider.rs`
- Jev classification call on first request of each user turn (not tool-loop continuations)
- Tier taxonomy: Fast (cheapest), Balanced (default), Strong (complex), Long (extended context)
- Per-provider tier model mappings (configurable)
- Deterministic policy rules in code:
  - Explicit user model requests always win
  - Jev failure/timeout keeps current model (fail-open)
  - Low confidence (< 0.6) never downgrades, caps upgrades at Balanced
  - Large conversations (> 50% token budget) refuse downgrades
  - Unavailable tiers step upward rather than silently choosing weaker model
- Jev call sends ONLY the prompt text — no source code, no tool results

## Non-scope

- Not a proxy — native middleware in the provider dispatch pipeline
- Not removing manual model selection (`/model` picker still works)
- Not routing tool-loop continuations differently (same tier as the turn start)
- Not implementing tier-specific prompt modification
- Not routing based on file types or repository context (prompt text only, for privacy)

## Behavior

### Happy path

1. User sends a turn: "fix the failing test in auth.rs"
2. Jev Router middleware intercepts the request
3. Prompt text is extracted and sent to Jev:
   ```
   questions: {
     complexity: {type: "score", levels: ["trivial", "simple", "moderate", "complex", "expert"]},
     reasoning_required: {type: "noul", instructions: "Does this require multi-step reasoning?"},
     tool_complexity: {type: "noul", instructions: "Does this need multiple coordinated tool calls?"},
     context_size: {type: "noul", instructions: "Does this require large context understanding?"}
   }
   ```
4. Jev returns: complexity=moderate, reasoning_required=0.78, tool_complexity=0.45, context_size=0.31
5. Policy: moderate complexity → Balanced tier
6. Provider dispatcher routes to configured Balanced model (e.g., Sonnet/Terra)
7. All tool-loop continuations in this turn use the same model

### Edge cases

- **User explicitly selects model**: "use opus" → skip routing, use selected model
- **Jev timeout**: keep current model, log decision as "routing unavailable"
- **Jev 429 rate limit**: keep current model, log rate limit event
- **All tiers map to same model**: routing is a no-op, skip Jev call entirely
- **Prompt text is empty**: keep current model
- **Confidence is 0.5 with "complex" but task seems simple**: policy caps upgrade at Balanced when confidence < 0.6
- **Current model is already the cheapest**: skip downgrade check

## Design

### Tier configuration

```toml
# Per-provider tier mapping (config)
[provider.anthropic.tiers]
fast = "claude-haiku-4-5"
balanced = "claude-sonnet-5"
strong = "claude-opus-4-8"
long = "claude-fable-5"

[provider.openai.tiers]
fast = "gpt-5.6-luna"
balanced = "gpt-5.6-terra"
strong = "gpt-5.6-sol"
long = "gpt-6-astra"
```

### Policy rules (in Rust)

```rust
fn route_tier(jev_answer: &JevRoutingAnswer, current_model: &Model, context: &RoutingContext) -> TierDecision {
    // 1. Explicit user choice always wins
    if context.user_explicitly_selected_model {
        return TierDecision::KeepCurrent;
    }
    // 2. Fail-open: Jev unavailable
    if jev_answer.is_error() {
        return TierDecision::KeepCurrent;
    }
    // 3. Low confidence caps upgrades
    if jev_answer.confidence < 0.6 && jev_answer.tier > current_model.tier {
        return TierDecision::UpgradeTo(Balanced);
    }
    // 4. Large conversations refuse downgrades
    if context.token_usage > 0.5 && jev_answer.tier < current_model.tier {
        return TierDecision::KeepCurrent;
    }
    // 5. Normal routing
    TierDecision::Route(jev_answer.tier)
}
```

## Dependencies

- **Depends on**: `add-jevsdk-evaluate-tool` — router uses the evaluate tool
- **Prior art**: `https://github.com/gargpratyush/jev-router` (191 stars, MIT)
- **Priority**: CONSIDER — medium effort, medium risk

## Touched capabilities

| Capability | Effect |
|-----------|--------|
| `agent/provider.rs` | Add JevRouter middleware |
| Config system | New tier model mappings per provider |
| `tool/evaluate.rs` | Used for routing decision |

## Acceptance

1. Simple query ("what is 2+2?") → routed to Fast tier model
2. Complex query ("implement rate limiting middleware") → routed to Strong tier model
3. User explicitly selects model → routing skipped, user's model used
4. Jev unavailable → current model used, no interruption
5. Large conversation → downgrade refused, current model kept
6. Tool-loop continuations → same model as turn start
7. Per-provider tier mappings respected (Anthropic vs OpenAI vs OpenRouter)
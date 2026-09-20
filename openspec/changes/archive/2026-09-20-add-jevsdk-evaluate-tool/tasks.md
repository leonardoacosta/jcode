# add-jevsdk-evaluate-tool — Tasks

Dependency order matters: `#1 evaluate tool` is the foundation for all other Jev proposals. Complete it first.

## 1. Core types and HTTP client

- [x] 1.1 Define Rust types for TypeSafe API: Create `source/jcode/crates/jcode-app-core/src/tool/evaluate.rs` with Serde types matching the TypeSafe API schema. Verify with `cargo check` and round-trip serde_json test.
- [x] 1.2 Implement HTTP client for TypeSafe API: Add `reqwest`-based HTTP client with `POST /v1/systemone`, `TYPESAFE_API_KEY` Bearer auth, 10s timeout, 3 retries on 429/529, error handling for 401/5xx/timeouts. Verify with real API key; invalid key returns clear error.
- [x] 1.3 Implement response validation: Validate Choice (option in criteria, probabilities sum ~1.0), Noul (probability in [0,1]), Score (distribution sums ~1.0). Reject malformed responses with clear error messages.

## 2. Jcode tool integration

- [x] 2.1 Register evaluate as a Jcode tool: Implement `Tool` trait (`name()`, `description()`, `input_schema()`, `invoke()`) and add to `tool/mod.rs`. Verify tool appears in agent tool list and is callable.
- [x] 2.2 Add API key configuration: Support `TYPESAFE_API_KEY` and `OPENROUTER_API_KEY`. OpenRouter fallback: if TypeSafe key missing, use `https://openrouter.ai/api/alpha/decisions` with `~typesafe/jev-latest`. Clear error when neither key is set.
- [x] 2.3 Ship agent-visible usage guidance: Tool description teaches agents when to use Noul/Choice/Score, how to structure questions, how to handle confidence. Verify agent calls evaluate correctly on first attempt.

## 3. MCP bundling (alternative path)

- [x] 3.1 Bundle evaluate Go binary: Copy `itsmostafa/typesafe-mcp` evaluate binary into `jcode-bundled-servers/evaluate` with platform build step. Verify binary present in distribution.
- [x] 3.2 Auto-register evaluate MCP server: On session start, run `evaluate setup mcp` to register with detected agents. Verify new session has evaluate as MCP tool.

## Dependency graph

```
Task 1.1 → 1.2 → 1.3 → 2.1 → 2.2 → 2.3
                                  ↘ 3.1 → 3.2
```

Phase 1 must complete before Phase 2. Phase 3 is independent (MCP bundling path).
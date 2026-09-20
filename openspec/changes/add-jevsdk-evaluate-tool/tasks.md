# add-jevsdk-evaluate-tool — Tasks

Dependency order matters: `#1 evaluate tool` is the foundation for all other Jev proposals. Complete it first.

## Phase 1: Core types and HTTP client

### Task 1.1: Define Rust types for TypeSafe API

**Scope:** Create `source/jcode/crates/jcode-app-core/src/tool/evaluate.rs` with Serde types matching the TypeSafe API schema.

**Verification:** `cargo check` compiles. Types round-trip through serde_json successfully.

### Task 1.2: Implement HTTP client for TypeSafe API

**Scope:** Add `reqwest`-based HTTP client to `evaluate.rs`. Implement `POST /v1/systemone` with:
- `TYPESAFE_API_KEY` authentication (Bearer header)
- 10-second timeout
- Retry on 429/529 (3 attempts, exponential backoff)
- Error handling for 401, 5xx, network timeouts

**Verification:** Call against a real TypeSafe API key. Valid Noul/Choice/Score requests return typed answers. Invalid key returns clear error.

### Task 1.3: Implement response validation

**Scope:** Validate TypeSafe responses:
- Choice: selected option is in criteria, probabilities sum to ~1.0 (tolerance 0.02), confidence in [0,1]
- Noul: probability in [0,1]
- Score: probability distribution sums to ~1.0, score is valid level

**Verification:** Malformed responses (wrong structure, invalid probabilities) are rejected with clear error messages. Valid responses pass through.

## Phase 2: Jcode tool integration

### Task 2.1: Register evaluate as a Jcode tool

**Scope:** Add `evaluate` to `tool/mod.rs`. Implement the `Tool` trait: `name()`, `description()`, `input_schema()`, `invoke()`. 

**Verification:** Tool appears in agent tool list. `evaluate` is callable with state + questions JSON.

### Task 2.2: Add API key configuration

**Scope:** Support `TYPESAFE_API_KEY` and `OPENROUTER_API_KEY` environment/config keys. OpenRouter path: if `TYPESAFE_API_KEY` is absent but `OPENROUTER_API_KEY` is set, use `https://openrouter.ai/api/alpha/decisions` with model `~typesafe/jev-latest`.

**Verification:** Tool works with TypeSafe key. Tool works with OpenRouter key as fallback. Tool returns clear error when neither key is set.

### Task 2.3: Agent-visible usage guidance

**Scope:** Ship tool description and usage hints that teach agents:
- When to use Noul vs Choice vs Score
- How to structure questions (one narrow judgment per question)
- How to handle confidence scores
- Example patterns: "Is this urgent?", "Which file should I open?", "How severe is this?"

**Verification:** Agent calls evaluate correctly on first attempt with well-structured questions. Agent correctly interprets probabilities and confidence.

## Phase 3: MCP bundling (alternative path)

### Task 3.1: Bundle evaluate Go binary

**Scope:** Copy `itsmostafa/typesafe-mcp` evaluate binary into `jcode-bundled-servers/evaluate`. Add build step to download/compile the binary for the target platform.

**Verification:** Binary is present in the installed Jcode distribution. `evaluate --version` works.

### Task 3.2: Auto-register evaluate MCP server

**Scope:** On session start, run `evaluate setup mcp` to register the MCP server with detected agents (Claude Code, Codex, etc.). The evaluate tool then appears as an MCP tool for all agent sessions.

**Verification:** New session has evaluate available as MCP tool. Agent can call `evaluate(state, questions)` through MCP.

## Dependency graph

```
Task 1.1 → Task 1.2 → Task 1.3 → Task 2.1 → Task 2.2 → Task 2.3
                                                      ↘ Task 3.1 → Task 3.2
```

Phase 1 must complete before Phase 2. Phase 3 is independent (MCP bundling path).
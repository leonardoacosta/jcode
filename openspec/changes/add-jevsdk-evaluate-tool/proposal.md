---
execution:
  version: 1
  depends_on: []
---

# add-jevsdk-evaluate-tool

## Summary

Add a native Rust `evaluate` tool to Jcode that wraps the TypeSafe Jev System One model API (`POST /v1/systemone`). Agents gain access to typed probabilistic judgments — Noul (yes/no probability), Choice (pick from defined options), and Score (ordered level rating) — replacing every "ask LLM → parse text → hope valid" workflow with structured answers that code branches on directly.

As an alternative path, bundle the `evaluate` Go binary from `itsmostafa/typesafe-mcp` as a built-in MCP server so agents can call Jev through existing MCP infrastructure.

## Motivation

Jcode agents currently use LLM calls for simple binary or categorical judgments: "is this task urgent?", "which file should I open?", "how severe is this error?". These calls cost ~$0.003 each with ~2s latency, produce unstructured text, and require fragile parsing. Jev costs ~$0.0002 per call (~15× cheaper) with ~300ms latency (~7× faster) and returns typed probabilities that need no parsing.

The TypeSafe API is available directly (`api.typesafe.ai`) or through OpenRouter (`~typesafe/jev-latest`). The user already has API keys in the `jev` repo DB.

## Actors

- **Jcode agent**: the coding agent that calls `evaluate(state, questions)` for structured judgments
- **TypeSafe API**: the remote System One inference endpoint
- **OpenRouter** (optional): alternative API path for Jev access
- **Jcode user**: configures `TYPESAFE_API_KEY` or `OPENROUTER_API_KEY`

## Scope

- New `tool/evaluate.rs` module implementing the `evaluate` tool
- Rust Serde types for the TypeSafe API request/response schema
- HTTP client integration using `reqwest`
- Three question primitives: `Noul`, `Choice`, `Score`
- API key configuration through existing Jcode environment/config system
- Graceful degradation: return structured error when Jev is unavailable
- **Alternative path**: bundle `itsmostafa/typesafe-mcp` evaluate binary as built-in MCP server registered at session startup

## Non-scope

- Not replacing any existing tools
- Not making Jev mandatory — all integrations fail open
- Not implementing batching/retry logic beyond single HTTP call patterns
- Not implementing the TypeSafe Python/JS SDKs — direct HTTP only for Rust
- Not adding a prompt-based interface — tool takes typed questions only

## Behavior

### Happy path

1. Agent calls `evaluate` with `state` (plain text or JSON) and `questions` map
2. Tool constructs `POST /v1/systemone` request body per TypeSafe API schema
3. Tool authenticates with `TYPESAFE_API_KEY` (or `OPENROUTER_API_KEY`)
4. Tool sends HTTP request to `https://api.typesafe.ai/v1/systemone`
5. Tool deserializes response, validates answer structure
6. Tool returns structured answers keyed by question IDs to the agent

### Edge cases

- **Jev unavailable**: return `ToolOutput::error("Jev evaluation failed: {reason}")` with the HTTP status or connection error
- **Malformed questions**: return error before sending the request (local validation of required fields)
- **Invalid response**: validate response shape (choice in criteria, probabilities sum ~1, confidence 0-1); return error if invalid
- **API key missing**: return clear error instructing user to set `TYPESAFE_API_KEY`
- **Rate limit (429)**: return error with the `Retry-After` header value; agent may retry
- **Large state**: if state exceeds ~32k token estimate, return error suggesting state truncation
- **OpenRouter path**: if `OPENROUTER_API_KEY` is set and `TYPESAFE_API_KEY` is not, use `https://openrouter.ai/api/alpha/decisions` with model `~typesafe/jev-latest`

### Failure modes

| Failure | Behavior |
|---------|----------|
| Network timeout | Return error, agent decides fallback |
| Invalid API key (401) | Return error with auth guidance |
| Rate limited (429) | Return error with retry-after seconds |
| Jev model error (5xx) | Return error, caller retries or skips |
| Response validation fails | Return error with field path and expected vs actual |

## Design

### Rust type definitions

```rust
// Proposed: source/jcode/crates/jcode-app-core/src/tool/evaluate.rs

use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct EvaluateInput {
    pub state: serde_json::Value,  // plain text string or structured JSON object/array
    pub questions: std::collections::HashMap<String, Question>,
    #[serde(default)]
    pub model: Option<String>,  // defaults to "jev-latest"
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Question {
    #[serde(rename = "noul")]
    Noul { instructions: String },
    #[serde(rename = "choice")]
    Choice {
        instructions: String,
        criteria: std::collections::HashMap<String, String>,
    },
    #[serde(rename = "score")]
    Score {
        instructions: String,
        criteria: Vec<String>,
    },
}

// Request body for POST /v1/systemone
#[derive(Debug, Serialize)]
struct TypeSafeRequest {
    state: serde_json::Value,
    model: String,
    questions: std::collections::HashMap<String, Question>,
}

// Response: { answers: { <id>: Answer }, model: "jev-latest", usage: {...} }
#[derive(Debug, Deserialize)]
struct TypeSafeResponse {
    answers: std::collections::HashMap<String, Answer>,
    model: String,
    usage: Option<Usage>,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
enum Answer {
    Noul { noul: f64 },
    Choice { choice: String, probabilities: serde_json::Value, confidence: f64 },
    Score { score: f64, probabilities: serde_json::Value, confidence: f64 },
}
```

### Integration path A: Native Rust tool

- New file: `source/jcode/crates/jcode-app-core/src/tool/evaluate.rs`
- Register in `tool/mod.rs`: `mod evaluate;`
- Implement `Tool` trait with `name()`, `description()`, `input_schema()`, `invoke()`
- HTTP via `reqwest::Client` with a 10s timeout, 3 retries on 429/529

### Integration path B: Bundled MCP server

- Copy `itsmostafa/typesafe-mcp` evaluate binary into `jcode-bundled-servers/evaluate`
- Register as built-in MCP server: on session start, `evaluate setup mcp` is run
- Tool name in MCP: `evaluate` (single tool with `state` + `questions` inputs)
- Agents interact through existing MCP tool infrastructure

## Dependencies

- **Research**: `research/jev-ecosystem-deep-research.md` — full ecosystem analysis
- **Research**: `research/jev-jcode-feature-breakdown.md` — feature-by-feature plans
- **Prior art**: `https://github.com/itsmostafa/typesafe-mcp` — Go MCP server (99 stars, MIT)
- **API docs**: `https://docs.typesafe.ai/api` — TypeSafe HTTP API reference
- **API docs**: `https://docs.typesafe.ai/primitives` — question primitives

## Touched capabilities

| Capability | Effect |
|-----------|--------|
| `tool/evaluate.rs` (new) | New tool module |
| `tool/mod.rs` | Register evaluate module |
| `tool/mcp.rs` | Optional: register bundled evaluate MCP server |
| Config system | New `TYPESAFE_API_KEY` / `OPENROUTER_API_KEY` keys |
| `agent/tools.rs` | Evaluate tool appears in tool registry |

## Migration

No migration needed. This is a purely additive change. Existing tools and workflows are unaffected. The evaluate tool is optional — agents that don't call it see no change.

## Acceptance

1. Agent calls `evaluate(state="Help! Payouts failed.", questions={is_urgent: {type:"noul", instructions:"Does this convey urgency?"}})` → returns `{is_urgent: {noul: 0.91}}`
2. Agent calls `evaluate` with Choice question → returns `{choice, probabilities, confidence}`
3. Agent calls `evaluate` with Score question → returns `{score, probabilities, confidence}`
4. Agent calls `evaluate` without API key → returns clear error message
5. Agent calls `evaluate` when Jev is down → returns error with HTTP status, agent proceeds with fallback
6. Multiple question types in one call → all answers returned under correct IDs
7. Valid TypeSafe API key works; valid OpenRouter key works as fallback
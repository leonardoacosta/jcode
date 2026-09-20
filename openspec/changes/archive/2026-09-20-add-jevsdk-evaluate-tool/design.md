## Context

Jcode agents currently lack typed structured judgment capabilities. LLM calls for binary/categorical judgments cost ~$0.003/call with ~2s latency and require fragile text parsing. The TypeSafe Jev System One API provides typed probabilistic judgments (Noul, Choice, Score) at ~$0.0002/call with ~300ms latency. The repo has existing research in `research/jev-ecosystem-deep-research.md` and `research/jev-jcode-feature-breakdown.md`.

Existing Jcode tool infrastructure: tools implement the `Tool` trait (`name()`, `description()`, `input_schema()`, `invoke()`), registered in `tool/mod.rs`. HTTP calls use `reqwest`. MCP servers are registered at session startup.

## Goals / Non-Goals

**Goals:**
- Add a native Rust `evaluate` tool wrapping TypeSafe API `POST /v1/systemone`
- Support three question primitives: Noul (yes/no probability), Choice (pick from options), Score (ordered level rating)
- Two integration paths: native Rust tool AND bundled MCP server
- Fail-open: unavailable Jev returns errors, never blocks agent

**Non-Goals:**
- Not replacing any existing tools
- Not making Jev mandatory
- Not implementing retry/batching logic beyond single HTTP call patterns
- Not implementing TypeSafe Python/JS SDKs (direct HTTP only)
- Not adding a prompt-based interface (tool takes typed questions only)

## Decisions

**Rust type design:** Tagged enum for questions and answers matching the TypeSafe wire protocol exactly. Serde `#[serde(tag = "type")]` for discriminators. Separate request/response structs for clean serialization boundaries.

**Two integration paths:** Path A (native Rust tool) is the primary — direct `reqwest` HTTP, registered as standard Jcode tool. Path B (bundled MCP server) is an alternative — copy `itsmostafa/typesafe-mcp` Go binary, register as built-in MCP server. Both paths share the same TypeSafe API contract; they differ only in agent access mechanism.

**API key routing:** Prefer `TYPESAFE_API_KEY` for direct TypeSafe access. Fall back to `OPENROUTER_API_KEY` → `https://openrouter.ai/api/alpha/decisions` with model `~typesafe/jev-latest`. Clear error when neither is set.

**Response validation:** Validate before returning to agent. Choice: option in criteria, probabilities sum ~1.0 (tolerance 0.02), confidence in [0,1]. Noul: probability in [0,1]. Score: distribution sums ~1.0.

**Error surface:** Return `ToolOutput::error()` for all failures. Agent decides whether to retry or fall back. Rate limit errors include `Retry-After` value. Large state (>32k token estimate) returns truncation error.

## Risks / Trade-offs

- **Risk:** TypeSafe API is an external dependency — unavailability blocks the tool. Mitigation: fail-open, agent falls back to LLM judgments.
- **Risk:** OpenRouter path adds routing latency. Mitigation: direct TypeSafe path is preferred; OpenRouter is a fallback.
- **Trade-off:** Two integration paths increase maintenance surface. Justification: MCP path enables agent-agnostic access; native path enables tighter integration with Jcode's tool infrastructure.
- **Risk:** State size limits (~32k tokens) may surprise users. Mitigation: clear error messages with truncation guidance.
## Context

`crates/jcode-app-core/src/tool/evaluate.rs` resolves TypeSafe and OpenRouter credentials inline with a `fn api_config()` free function. Foreman duplicates similar logic with different defaults and no caching or deadline awareness. The evaluate tool also has an envelope inconsistency: cache hits return only the answers map, while cache misses return the full API response. Both callers use hardcoded URLs and default models.

A single service crate (`jcode-system-one`) with a trait, resolver, and shared client eliminates the duplication. Foreman and compaction remain unchanged; they are separate follow-up work.

## Goals / Non-Goals

**Goals:**
- One crate that any typed-Jev caller can depend on without pulling in tool or TUI crates.
- A `SystemOneService` trait with exactly `async fn evaluate(...)`.
- Provider resolution (TypeSafe direct, OpenRouter Decisions) with environment-only credentials.
- A shared, configurable reqwest client.
- Typed response validation (every question ID must have an answer).
- Evaluate migrates to the service with its cache and tool schema intact.
- Service returns identical result shapes for cache hits and misses.

**Non-Goals:**
- Do not migrate Foreman, compaction, or overnight.
- Do not change Evaluate's tool schema, cache keys, model defaults, or existing user-facing behavior.
- Do not add TOML configuration or credential setup flows.
- Do not add a new MCP server or external process.
- Do not change the API call contract (state/questions/answers).

## Decisions

1. **Own crate.** `crates/jcode-system-one`. Avoids pulling app-core dependencies into a service boundary.

2. **Trait, not enum.** `SystemOneService` trait with one method. Callers inject the impl. Allows cached, debug, and test implementations without coupling to the concrete provider resolver.

3. **Provider resolver as a free function.** `resolve_service() -> Result<ServiceConfig>` reads environment, returns endpoint, key, and default model. Not a builder or config struct because the only input is environment variables.

4. **Configurable timeout on the client.** The service exposes `with_timeout()` but defaults to `REQUEST_TIMEOUT_SECS` (10s). Callers with tighter deadlines (e.g., poke shadow assessment with 2s) construct their own client.

5. **Typed response.** `SystemOneResponse` contains `answers: HashMap<String, Value>`, `model: String`, and optional `usage`. Returns the same shape on cache hits and misses. Evaluate's cache layer wraps the service, not the other way around.

6. **No provider field in the response.** The caller already knows which provider it selected. The resolver produces a config; the config determines the URL. The response echoes the model, not the provider.

## Risks / Trade-offs

- **New crate adds compilation overhead.** Mitigated: the crate is small (~200 lines of service + resolver + client) and has no dependency on app-core, TUI, or protocol.
- **Foreman remains duplicated.** Acceptable for this scope. Foreman already has different defaults (5s timeout, no retries, hardcoded TypeSafe URL). Migrating it is a separate proposal.
- **Evaluate migration must preserve cache behavior.** The cache check happens before the service call. On a hit, the service is not called. The cache stores only answers, not the full response, but the service trait returns `SystemOneResponse`; the evaluate tool wrapper can reconstruct the full response from cached answers + known model.

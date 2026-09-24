---
execution:
  version: 1
  depends_on: []
---

## Why

The `evaluate` tool (`crates/jcode-app-core/src/tool/evaluate.rs`) and the Foreman observer (`crates/jcode-app-core/src/agent/foreman.rs`) each duplicate TypeSafe/OpenRouter endpoint selection, key precedence, and model defaults. No crate exposes a reusable System One service boundary. A modular service crate gives every typed-Jev caller one shared resolver, one HTTP client, and one testable contract without coupling to tool schemas, TUI state, or agent lifecycle.

`jev-poke-shadow-assessment` requires a clean service to call. That change depends on this one.

## What Changes

- Add `crates/jcode-system-one` with:
  - A `SystemOneService` trait: `async fn evaluate(state, questions, model_override) -> Result<SystemOneResponse>`.
  - Provider resolver (TypeSafe direct, OpenRouter Decisions) with environment-only credentials.
  - A shared reqwest client with configurable timeout (caller-specified, with a const default).
  - Typed response validation: every requested question ID must have an answer.
- Migrate the existing Evaluate tool to consume the service through its trait.
- Leave Foreman, compaction, and overnight unchanged.
- Keep evaluate's existing cache, tool schema, and public tool output contract unchanged.

### Capabilities

#### New Capabilities

- `system-one-service`: Modular crate exposing a typed System One evaluation trait and shared provider/model/credential resolver.

#### Modified Capabilities

- `evaluate`: Uses the System One service rather than inline endpoint/key/model logic. Tool schema and cache behavior unchanged.

## Impact

Touched paths: new `crates/jcode-system-one/` crate, `crates/jcode-app-core/src/tool/evaluate.rs`. Existing evaluate callers see identical tool output. No tool schema, cache key, or model behavior changes for end users. No credential, endpoint, or model default changes. No TOML changes. Foreman and compaction remain unchanged.

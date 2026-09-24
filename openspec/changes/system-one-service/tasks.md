## 1. Crate scaffolding

- [ ] 1.1 Create `crates/jcode-system-one/Cargo.toml` with dependencies (serde, serde_json, reqwest with http2, tokio, anyhow) and no dependency on jcode-app-core, jcode-tui, or jcode-protocol.
- [ ] 1.2 Add `crates/jcode-system-one/src/lib.rs` exporting the trait, resolver, types, and client constructor.

## 2. Service trait and types

- [ ] 2.1 Define `SystemOneResponse` with `answers: HashMap<String, Value>`, `model: String`, and `usage: Option<Value>`. Derive Debug, Serialize, Deserialize.
- [ ] 2.2 Define the `SystemOneService` trait: `async fn evaluate(&self, state: Value, questions: HashMap<String, Value>, model_override: Option<&str>) -> Result<SystemOneResponse>`.
- [ ] 2.3 Add unit tests for response serialization roundtrip and the trait contract with a mock implementation.

## 3. Provider resolver

- [ ] 3.1 Implement `resolve_service() -> Result<ServiceConfig>` with TypeSafe and OpenRouter credential precedence, empty/whitespace handling, and explicit provider selection.
- [ ] 3.2 Add unit tests for: TypeSafe key only, OpenRouter key only, both keys present, empty environment values, missing keys, and explicit provider selection.

## 4. HTTP client and service implementation

- [ ] 4.1 Implement `LiveSystemOneService` constructing a shared reqwest client and calling the resolved endpoint.
- [ ] 4.2 Validate every requested question ID has an answer in the response. Return typed errors for missing answers.
- [ ] 4.3 Add integration-style tests with a mock HTTP server verifying request construction, response parsing, and error handling for both provider routes.

## 5. Evaluate tool migration

- [ ] 5.1 Remove the inline `api_config()` function and hardcoded URL/retry constants from evaluate.rs.
- [ ] 5.2 Inject `SystemOneService` into `EvaluateTool`. Construct the service once at tool creation.
- [ ] 5.3 Preserve cache behavior: check cache before service call; on miss, call service, validate, cache answers, and return full response.
- [ ] 5.4 Verify cache hit/miss parity: both paths return identical response shapes.
- [ ] 5.5 Run existing evaluate tool tests and verify all pass with unchanged tool schema and output.

## 6. Acceptance and verification

- [ ] 6.1 Run `cargo test -p jcode-system-one` and `cargo test -p jcode-app-core` with the migrated evaluate.
- [ ] 6.2 Verify existing evaluate cache tests still pass.
- [ ] 6.3 Build via selfdev and confirm the evaluate tool works in a tester session with both provider routes.

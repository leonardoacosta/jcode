## 1. Backoff and health

- [x] 1.1 Replace `FAILED_CONNECT_RETRY_COOLDOWN` with per-server failure count + exponential delay in `pool.rs`; reset on success/config change; unit tests for growth, cap, reset
- [x] 1.2 Buffer last 20 stderr lines in `client.rs` stdio reader; classify failures; tests for missing-cwd and silent-exit fixtures
- [x] 1.3 Add `degraded` state after 5 failures; explicit connect/reload clears it; tests
- [x] 1.4 Dedupe `MCP_REGISTER_FAILED` notices per transition; test with shared pool and multiple sessions
- [x] 1.5 `cargo test -p jcode-base mcp` and `openspec validate mcp-failure-backoff --strict`

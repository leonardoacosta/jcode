## 1. Turn execution
- [x] 1.1 Create `crates/jcode-app-core/src/agent/turn_execution_tests.rs` and wire it with `#[cfg(test)]` in `turn_execution.rs`
- [x] 1.2 Reuse/extract the `MockProvider` pattern from `server/client_session_tests`
- [x] 1.3 Cover `run_once_capture`, `append_user_context_message`, `continue_turn_after_failover`, lease/stale checks
- [x] 1.4 `cargo test -p jcode-app-core turn_execution` passes

## 2. TUI input
- [x] 2.1 Create `crates/jcode-tui/src/tui/app/input/key_dispatch_tests.rs` using `create_test_app()`
- [x] 2.2 Cover submit, empty submit, newline, history, paste, shell command
- [x] 2.3 `cargo test -p jcode-tui key_dispatch` passes

## 3. Harness translate snapshots
- [x] 3.1 Add snapshot helper with `UPDATE_SNAPSHOTS=1` regenerate under `src/translate_tests/`
- [x] 3.2 Add request and event fixtures plus golden files
- [x] 3.3 `cargo test -p jcode-harness-api-server translate` passes; deliberate drift fails with diff

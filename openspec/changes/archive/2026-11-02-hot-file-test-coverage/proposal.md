## Why

Roadmap owner: [P01: Trusted core and acceptance/evidence baseline](../../../docs/plans/PRODUCT_ROADMAP.md#p01-trusted-core-and-acceptanceevidence-baseline).

Forensics (60d) show the highest-churn files lack direct tests: `crates/jcode-app-core/src/agent/turn_execution.rs` (78 changes, 1647 lines, no test module), `crates/jcode-tui/src/tui/app/input.rs` (96 changes, 4329 lines; only drop/inline/terminal-control-sequence submodule tests), and `crates/jcode-harness-api-server/src/translate.rs` (120 changes; 138 assertion tests in `translate_tests.rs` but no whole-frame snapshots). Regressions there break every session.

## What Changes

- Add `crates/jcode-app-core/src/agent/turn_execution_tests.rs` (wired via `#[cfg(test)] #[path] mod`, matching `turn_loops_tests.rs`/`turn_streaming_mpsc_tests.rs`) characterizing `run_once_capture`, `append_user_context_message`, `continue_turn_after_failover`, `ensure_session_lease`, `session_copy_is_stale` using the existing `MockProvider` pattern from `server/client_session_tests`.
- Add `crates/jcode-tui/src/tui/app/input/key_dispatch_tests.rs` using `create_test_app()` (`tests/support_failover/part_01.rs`) to pin submit, newline, history, paste and shell-command key paths.
- Add golden snapshot tests under `crates/jcode-harness-api-server/src/translate_tests/` for `BridgeState::api_request_to_legacy` and `legacy_event_to_api`, with fixtures in `translate_tests/snapshots/`.
- Tests only. No production behavior change.

## Open decisions

1. Snapshot tool. **Default: plain JSON golden files** compared with `serde_json::Value` equality plus an `UPDATE_SNAPSHOTS=1` regenerate path (no new dep). Alt: `insta` dev-dependency (better review UX, new dep; repo currently has none).
2. Characterize current behavior even if suspicious? **Default: yes**, pin as-is and mark questionable cases with `// CHARACTERIZATION:` comments; fixes go in separate changes.

## Impact

Test-only files in three crates; `Cargo.toml` unchanged under default decision 1.

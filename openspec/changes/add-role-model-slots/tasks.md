# Tasks

## 1. Config

- [x] 1.1 Add `agents.roles.{research,edit,compaction,websearch,council}` optional model fields.

## 2. Resolution

- [x] 2.1 Add optional `role` to swarm spawn; resolve explicit model, then role slot, then `swarm_model`, then inherit.
- [x] 2.2 Use role slots in existing independent model callers. Compaction is wired; websearch has no independent model caller, so its configured slot remains reserved.
  - [x] Compaction: select `agents.roles.compaction` on an independent provider fork before both native compaction and text-summary fallback (`crates/jcode-base/src/compaction.rs`, `generate_compaction_artifact`). Unset/blank slots retain the active model.
  - [x] Websearch applicability checked: reserved slot, no independent model caller exists. `WebSearchTool::execute` and `search_with_engine` in `crates/jcode-app-core/src/tool/websearch.rs` use HTTP retrieval backends. Native search attaches a tool to the main provider request in `crates/jcode-provider-anthropic-runtime/src/lib.rs` through `native_web_search::server_tools_for_request`, not a separate model call. Do not introduce new synthesis or change the main conversation model to claim this requirement. Clarification: slot selection applies only to an existing independent model caller. HTTP retrieval and native main-model tools do not introduce one. The approved role-resolution and read-only requirements remain unchanged.
- [x] 2.3 Apply a read-only tool profile for `research` workers.
- [x] 2.4 Report source `slot:<role>` via `show-resolved-worker-model` reporting.
- [x] 2.5 Show configured slots in `swarm list_models`.

## 3. Verification

- [x] 3.1 Unit tests for resolution precedence and `slot:<role>` source.
- [x] 3.2 Test research worker cannot call edit tools.
- [x] 3.3 Run `openspec validate add-role-model-slots --strict`.

## Verification notes (2026-11-02)

- Compaction regression reproduced before the fix: configured `slot-model` returned `main-model` (exit 101). Final `cargo test -p jcode-base --lib compaction::tests:: -- --test-threads=1`: 39 passed, including isolated subprocess cases for native/text compaction, blank/unset slots, and an unavailable model. The active provider model remains unchanged.
- `cargo test -p jcode-base --lib session:: -- --test-threads=1`: 118 passed, 2 ignored. `cargo test -p jcode-protocol --lib comm_spawn -- --test-threads=1`: 3 passed.
- Research regression reproduced before enforcement: the executor reached edit argument parsing (``missing field `file_path` ``) instead of rejecting the tool (exit 101). The new test covers valid edit rejection, mutation/escape tools, allowed reads, and disabled memory. Visible-spawn tests launch a separate test process through full, startup-stub, and remote-startup session loaders, including resume/reset checks. Final `cargo test -p jcode-app-core --lib <filter> -- --test-threads=1` gates pass: `research` (2), `server::comm_session::comm_session_tests::` (38, including research/council headless spawn), `council` (14), `concurrency_construction` (5), and `role_slot_resolution_precedence_and_source` (1).
- Commands run through `selfdev test` with an isolated `JCODE_HOME`, chained with `&&` to preserve failures. Compaction tests require `--test-threads=1`: the existing `max_context_tokens_applies_at_construction_and_reloads_before_requests` fixture changes process-global config, causing two unrelated estimate assertions to fail in parallel. Serialized execution passes all 39.
- Fresh coordinator gate `2690950cv6` exited 0: serialized compaction, council, research, and strict validation for both role slots and council. Independent reviewer `mizaru` reviewed commits `a6707c25a` and `142354496` and returned CODE REVIEW READY with no actionable role-code findings. Its sole artifact blocker was the websearch caller clarification recorded above. Archive awaits the final artifact recheck. No push, branch change, or shared-daemon reload performed.

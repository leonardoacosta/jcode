# Tasks

## 1. Config

- [x] 1.1 Add `agents.roles.{research,edit,compaction,websearch,council}` optional model fields.

## 2. Resolution

- [x] 2.1 Add optional `role` to swarm spawn; resolve explicit model, then role slot, then `swarm_model`, then inherit.
- [ ] 2.2 Use `compaction` and `websearch` slots in their internal callers when set.
  - [x] Compaction: select `agents.roles.compaction` on an independent provider fork before both native compaction and text-summary fallback (`crates/jcode-base/src/compaction.rs`, `generate_compaction_artifact`). Unset/blank slots retain the active model.
  - [ ] Websearch: reserved slot, no independent model caller exists. `WebSearchTool::execute` and `search_with_engine` in `crates/jcode-app-core/src/tool/websearch.rs` use HTTP retrieval backends. Native search attaches a tool to the main provider request in `crates/jcode-provider-anthropic-runtime/src/lib.rs` through `native_web_search::server_tools_for_request`, not a separate model call. Do not introduce new synthesis or change the main conversation model to claim this requirement. This portion remains unresolved pending a dedicated caller design or an explicit requirement clarification.
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
- `openspec validate add-role-model-slots --strict` and `git diff --check` pass. Independent final review and archive remain pending. No archive, push, branch change, or shared-daemon reload performed.

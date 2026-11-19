## 1. Config

- [x] 1.1 Extend existing `HooksConfig` compatibly (preserve legacy keys/env overrides, transformations and feedback)  (`[hooks]` with per-event entry lists: `matcher`, `path`, `command`, `timeout`, `fail_closed`, `mode`) to `crates/jcode-config-types/src/lib.rs`
  - Evidence: `cargo test -p jcode-base --lib config::tests` ok (lifecycle_hook_entries_parse_alongside_legacy_keys_and_round_trip); HookEntry+6 event fields in jcode-config-types, legacy keys untouched
- [x] 1.2 Validate matcher regex and path glob at load; report invalid entries without aborting startup
  - Evidence: hooks_lifecycle_tests::bad_entries_are_disabled_individually ok (bad regex/mode/empty/glob disabled, rest load); warnings logged at config load

## 2. Runner

- [x] 2.1 Extend the existing direct-exec hook runner: write compatible JSON payloads, enforce timeout, bound/redact output and preserve current env/argv behavior. Verify legacy keys, input transforms and feedback still work without duplicate event dispatch.
  - Evidence: `cargo test -p jcode-base --lib hooks` 38 pass incl. all legacy gate/transform tests; legacy_command_also_configured_as_lifecycle_runs_once ok
- [x] 2.2 Mode handling: `blocking` awaited with decisions, `observe` awaited and logged, `async` detached with timeout kill; per-event defaults and warning for unsupported `blocking`
  - Evidence: tests default_modes_follow_event_semantics, blocking_mode_on_a_non_blocking_event_degrades_with_a_warning, async_hook_does_not_delay_the_caller, timeout_kills_the_hook_process_group, observe_hooks_ignore_exit_two ok
- [x] 2.3 Parse Claude-compatible JSON stdout (`decision`, `reason`, `continue`, `stopReason`, `suppressOutput`, `hookSpecificOutput.permissionDecision`, `additionalContext`)
  - Evidence: tests pre_tool_use_exit_two_blocks_with_stderr_reason, pre_tool_use_json_rewrite_chains_and_deny_drops_it, json_ask_blocks_context_is_collected_and_observe_ignores_decisions, stop_hook_json_block_returns_reason_and_continue_false_halts ok
- [x] 2.4 Build Claude Code compatible payloads per event
  - Evidence: payload_is_claude_code_compatible ok; registry test asserts PostToolUse tool_name/tool_input/tool_response; SubagentStop test asserts status/message/validation/follow_up/report

## 3. Events

- [x] 3.1 `SessionStart` on new/resumed session with `source`
  - Evidence: fire_session_lifecycle_hook in agent.rs (attach and resume -> resume, else startup); compiles, clippy clean. `cargo test -p jcode-app-core --lib -- --test-threads=1 session_start_hook_reports` ok (new session source=startup, attached session source=resume)
- [x] 3.2 `PreToolUse` before dispatch in `turn_execution.rs`; exit 2 or JSON deny blocks; `ask` blocks with a clear reason (no hook approval prompt exists in jcode; spec/design amended); `fail_closed` blocks on any failure or timeout
  - Evidence: `cargo test -p jcode-app-core --lib tool::tests::registry_execute_lifecycle_hooks_gate_rewrite_and_report_tool_calls` ok (path-filtered deny skipped, rewrite applied, unfiltered exit 2 blocks); `ask` routes to block (no hook approval prompt exists), documented
- [x] 3.3 `PostToolUse` after tool result with `tool_response`
  - Evidence: same registry test: PostToolUse payload has tool_response and rewritten input
- [x] 3.4 `SubagentStop` when a swarm worker submits its completion report, payload includes report
  - Evidence: `cargo test -p jcode-app-core --lib subagent_stop_hook` ok: fires once for duplicate report, stale-generation rejected reports never reach the hook
- [x] 3.5 `Stop` at natural completion only; bounded JSON continuation obeys durable admission, user cancellation and budgets. Preserve recursion guard, track owned hook child work for settlement; `PreCompact` before compaction with trigger. Verify explicit cancellation never admits another turn.
  - Evidence: `cargo test -p jcode-app-core --lib stop_hook` 4 ok: bound=3 continuations, no block no continue, stop_hook_active after first, cancel during hook never continues; Stop admission checks cancel, migration lease, plan turn, reserve_current. PreCompact: `pre_compact_hook_reports_manual_trigger` ok (trigger=manual); auto trigger sites (messages_for_provider, overflow recovery) are wired but not runtime-tested

## 4. Tests and validation

- [x] 4.1 Unit tests: matcher/path matching, timeout kill, exit code 2 block, JSON deny/ask/additionalContext, Stop continuation, async non-await, observe ignores decisions, fail_closed, payload snapshots per event
  - Evidence: hooks_lifecycle_tests (16) + config + agent stop tests ok
- [x] 4.2 Integration test: SubagentStop hook receives worker report JSON
  - Evidence: subagent_stop_hook_receives_the_worker_report_once ok
- [x] 4.3 `openspec validate add-lifecycle-hooks --strict`
  - Evidence: `openspec validate add-lifecycle-hooks --strict` -> valid

## Review follow-up (2026-11-19, review-lifecycle-hooks.md: CHANGES REQUESTED)
- MAJOR-1 fixed: `fire_session_lifecycle_hook` maps "attach" and "resume" to source=resume (agent.rs); `session_start_hook_reports_startup_and_resume_sources` now also asserts `restore_session` reports resume.
- MAJOR-2 fixed: spec.md, design.md (Amendments) and task 3.2 now say `ask` blocks with a "requires user approval" reason; docs/HOOKS.md consistent.
- MINOR-1: spec reworded to "logged" warnings (no TUI surfacing; accepted).
- MINOR-2 fixed: `fail_closed` only blocks PreToolUse; failed/timed-out Stop hook is logged only. Test `failed_stop_hook_never_becomes_a_continuation`.
- MINOR-3: documented (spec/design/HOOKS.md): `path` filters judge the tool input as issued, not recomputed after rewrites.
- MINOR-4 accepted limit (not implemented): hook waits not raced against cancel; Esc can wait up to hook timeout. Cancel still wins after return. Documented in design.md and HOOKS.md.
- MINOR-5: documented exemption: SubagentStop children are not generation-owned (fires from the report handler, async, timeout kills the group).
- MINOR-6 tests added: `fail_closed_blocks_on_exit_one_and_bad_json_only_when_set`, `recursion_guard_is_set_on_children_and_suppresses_dispatch`, `failed_stop_hook_never_becomes_a_continuation`. Not added: cancel-during-fail-open-hook and lease-accounting-until-settled tests, entry-level 10s default kill test (config default parse test only).
- MINOR-7 fixed: hook command redacted (`redact`) in warn logs.
- NIT: 3-continuation bound stated in spec; legacy-timeout and backgrounded-child notes in design.md and HOOKS.md.
- Evidence: `scripts/dev_cargo.sh test -p jcode-base --lib -- hooks config` 299 pass; `-p jcode-config-types` 36 pass; `-p jcode-app-core --lib -- --test-threads=1 lifecycle agent_tests hooks tool::tests subagent_stop` 155 pass, 3 ignored, 0 failed (plus 1 pass for the extra filter run); `clippy -p jcode-config-types -p jcode-base -p jcode-app-core --all-targets -- -D warnings` clean; `openspec validate add-lifecycle-hooks --strict` valid.

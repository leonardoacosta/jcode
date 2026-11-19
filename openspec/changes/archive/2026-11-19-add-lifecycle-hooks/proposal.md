---
execution:
  version: 1
  depends_on: [make-worker-stop-durable]
---
# Proposal

Roadmap owner: [P08: SDK/API/tool integration contracts, including browser/voice](../../../docs/plans/PRODUCT_ROADMAP.md#p08-sdkapitool-integration-contracts-including-browservoice). Supporting: P02.

## Why

The original discovery predates the existing `HooksConfig` in `crates/jcode-config-types/src/lib.rs` and runner in `crates/jcode-base/src/hooks.rs`. Current hooks already cover session/turn/tool observers, input transformation, pre-tool gating and post-tool feedback. The remaining feature is compatible event matching, structured decisions and worker/compaction coverage through that runner, not a parallel hook system. Historical upstream compatibility notes are design inputs, not proof that every current external script works unchanged.

## What Changes

- Extend the existing `[hooks]` config compatibly with events `SessionStart`, `PreToolUse`, `PostToolUse`, `SubagentStop`, `Stop`, `PreCompact`.
- Each entry: `matcher` (tool name regex, optional `path` glob against tool file arguments), `command` (existing shell-style argv parsing/direct execution, no implicit shell), `timeout` (default 10s), `fail_closed` (default false).
- Hook receives a Claude Code compatible JSON payload on stdin (`session_id`, `transcript_path`, `cwd`, `hook_event_name`, plus event fields such as `tool_name`, `tool_input`, `tool_response`, `trigger`, `source`).
- Per-hook `mode = "blocking" | "observe" | "async"` (see `design.md`, option C). Defaults: `PreToolUse`/`Stop` blocking, `SessionStart`/`PreCompact` observe, `PostToolUse`/`SubagentStop` async.
- Explicit cancel/stop and revoked generations always override hook continuation. Only natural turn completion can invoke a bounded Stop continuation, with a one-continuation-per-completion guard and no observer recursion. Hook subprocesses retain generation ownership for settlement.
- Blocking hooks use the Claude Code contract: exit 2 blocks with stderr to the model; exit 0 with JSON stdout (`decision`, `reason`, `continue`, `stopReason`, `hookSpecificOutput.permissionDecision`, `additionalContext`); other non-zero codes are warnings unless `fail_closed`.
- `SubagentStop` payload includes the worker's completion report (status, message, validation).

## Capabilities

### New Capabilities

- `lifecycle-hooks`: Configurable shell hooks at session, tool, compaction, and subagent lifecycle events.

## Impact

Existing `HooksConfig` in `crates/jcode-config-types/src/lib.rs` and runner in `crates/jcode-base/src/hooks.rs`, with bounded call-site extensions in turn execution, compaction, session and worker completion paths. Preserve legacy keys, env/direct-exec semantics and `spawn_hook`; map event aliases without duplicate dispatch.

## Confirmed decisions

1. No `.claude/settings.json` import in v1; payload and output compatibility only.
2. Timeout kills the hook and logs a warning; it blocks only when `fail_closed = true` on a blocking hook.
3. Schema confirmed against current Claude Code, Codex, and Gemini docs; option C chosen. See `design.md`.

## Approval

Approved 2026-11-01 with recommended defaults; hook mode design C confirmed by user 2026-11-02.

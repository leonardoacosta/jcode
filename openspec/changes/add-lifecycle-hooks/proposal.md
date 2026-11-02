# Proposal

Roadmap owner: [P08: SDK/API/tool integration contracts, including browser/voice](../../../docs/plans/PRODUCT_ROADMAP.md#p08-sdkapitool-integration-contracts-including-browservoice). Supporting: P02.

## Why

jcode has one hook: `spawn_hook` (`crates/jcode-config-types/src/lib.rs:806-821`), which only wraps headed spawn launches. Users cannot run policy or notification commands at session, tool, compaction, or subagent boundaries. A concrete ask (pi workflow) is a notification when a subagent finishes; today that needs polling swarm status. Claude Code users already have hook scripts written against its stdin JSON contract; reusing them avoids a second ecosystem. Compaction lives in `crates/jcode-app-core/src/agent/compaction.rs`, tool dispatch in `crates/jcode-app-core/src/agent/turn_execution.rs`. Clean-room behavior notes: `~/.jcode/scratch/empryo-re/adjacent-behavior.md` (behavior only, no text reused).

## What Changes

- New `[hooks]` config table with events `SessionStart`, `PreToolUse`, `PostToolUse`, `SubagentStop`, `Stop`, `PreCompact`.
- Each entry: `matcher` (tool name regex, optional `path` glob against tool file arguments), `command` (shell), `timeout` (default 10s), `fail_closed` (default false).
- Hook receives a Claude Code compatible JSON payload on stdin (`session_id`, `transcript_path`, `cwd`, `hook_event_name`, plus event fields such as `tool_name`, `tool_input`, `tool_response`, `trigger`, `source`).
- Per-hook `mode = "blocking" | "observe" | "async"` (see `design.md`, option C). Defaults: `PreToolUse`/`Stop` blocking, `SessionStart`/`PreCompact` observe, `PostToolUse`/`SubagentStop` async.
- Blocking hooks use the Claude Code contract: exit 2 blocks with stderr to the model; exit 0 with JSON stdout (`decision`, `reason`, `continue`, `stopReason`, `hookSpecificOutput.permissionDecision`, `additionalContext`); other non-zero codes are warnings unless `fail_closed`.
- `SubagentStop` payload includes the worker's completion report (status, message, validation).

## Capabilities

### New Capabilities

- `lifecycle-hooks`: Configurable shell hooks at session, tool, compaction, and subagent lifecycle events.

## Impact

`crates/jcode-config-types/src/lib.rs` (new `HooksConfig`), new hook runner module in `crates/jcode-app-core/src/agent/`, call sites in `turn_execution.rs`, `compaction.rs`, session start, and swarm completion report handling (`crates/jcode-app-core/src/server/comm_session.rs`). `spawn_hook` unchanged.

## Confirmed decisions

1. No `.claude/settings.json` import in v1; payload and output compatibility only.
2. Timeout kills the hook and logs a warning; it blocks only when `fail_closed = true` on a blocking hook.
3. Schema confirmed against current Claude Code, Codex, and Gemini docs; option C chosen. See `design.md`.

## Approval

Approved 2026-11-01 with recommended defaults; hook mode design C confirmed by user 2026-11-02.

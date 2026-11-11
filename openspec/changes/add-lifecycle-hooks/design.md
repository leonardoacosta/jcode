# Design: lifecycle hooks blocking model

## Sources (fetched 2026-11-01)

- Claude Code: https://code.claude.com/docs/en/hooks
- Codex CLI: https://developers.openai.com/codex/hooks
- Gemini CLI: https://github.com/google-gemini/gemini-cli/blob/main/docs/hooks/reference.md
- pi: https://github.com/badlogic/pi-mono/blob/main/packages/coding-agent/docs/extensions.md
- Empryo v3: clean-room notes `~/.jcode/scratch/empryo-re/adjacent-behavior.md` (behavior only)

## Comparison

| Harness | Blocking events | Blocking mechanism | Payload / return | Non-blocking path | Timeout default |
| :- | :- | :- | :- | :- | :- |
| Claude Code | PreToolUse, UserPromptSubmit, Stop, SubagentStop (prevents stopping), others; PostToolUse feeds back only | Exit 2 = block, stderr to model; exit 0 + JSON `decision: "block"`, `reason`, `continue: false` + `stopReason`, `suppressOutput`, `hookSpecificOutput.permissionDecision: allow/deny/ask`, `additionalContext`; other codes = non-blocking error | JSON on stdin (`session_id`, `transcript_path`, `cwd`, `hook_event_name`, event fields) | `async: true` runs in background, timeout not enforced | 600s command; timeout = no decision unless `onFailure: "block"` |
| Codex CLI | PreToolUse, Stop (continue), PermissionRequest | Exit 2 or JSON `decision: "block"/"approve"`; legacy `notify` is fire-and-forget | JSON stdin, Claude-shaped | background hooks; `notify` program | 1s command; timeout = hook failure, non-blocking |
| Gemini CLI | BeforeTool, BeforeAgent, AfterAgent, BeforeModel | Exit 2 or JSON `decision: "deny"/"block"`, `reason`, `continue`, `systemMessage` | JSON stdin, JSON stdout | non-zero non-2 = warning | 60s |
| pi | `tool_call` | In-process TS handler returns `{ block: true, reason }` | Typed event objects | handlers awaited; no shell contract | none |
| Empryo v3 | 2 blocking events | Block result plus `once`, `if` filter, `failClosed` | JSON | other events observe-only | 10s |

Key differences: every shell-based harness converged on exit 2 + Claude-style JSON stdout; defaults differ wildly on timeout (1s to 600s) and on what a timeout means; only Claude exposes an explicit `async` flag; fail-closed is opt-in everywhere (`onFailure`, `failClosed`).

## Options

- **A. Exit code only.** Simplest. Cannot express `ask`, `additionalContext`, or Stop continuation reasons cleanly; Claude scripts that emit JSON silently lose behavior.
- **B. Exit code + JSON decision (Claude compatible).** Reuses existing scripts and the de facto standard. Every hook is still awaited, so a notification hook on `SubagentStop` adds latency to the coordinator.
- **C. B + per-hook `mode = "blocking" | "async" | "observe"`.** `blocking`: awaited, decisions honored. `observe`: awaited with timeout, output logged, decisions ignored. `async`: spawned detached, never awaited, output discarded, timeout still kills.

## Decision: C

- Latency: notification hooks (`SubagentStop`, `PostToolUse`) default to `async`, so they never stall a turn.
- Safety: only `PreToolUse` and natural-completion `Stop` honor decisions in blocking mode. Explicit user/coordinator cancellation, budgets and revoked generations take precedence, never run a continuation. Limit Stop continuation to one per completion event, preserve recursion prevention, and account hook children in generation settlement. `fail_closed` applies only to blocking mode.
- Compat: compatible JSON payload/output fields are the target, with explicit fixtures for supported forms. Current external scripts are not guaranteed unchanged across harness versions. `mode` is additive and existing Jcode keys remain supported.
- Explicitness: whether a hook can gate execution is visible in config instead of inferred per event.

Defaults: `PreToolUse`, `Stop` = `blocking`; `SessionStart`, `PreCompact` = `observe`; `PostToolUse`, `SubagentStop` = `async`. Setting `blocking` on a non-blockable event is a config warning and behaves as `observe`.

## Existing-runner compatibility and dependency

Extend `crates/jcode-base/src/hooks.rs` and current `HooksConfig`; retain legacy snake_case hook keys, env overrides, direct argv execution, pre_tool_transform and post_tool_feedback behavior. Map new event names additively with explicit precedence and tests so one event never fires twice. Do not replace the existing hook ecosystem or import external config. Consumes make-worker-stop-durable admission/generation/settlement; its dependency does not make hooks a prerequisite of worker control. External compatibility examples above are historical observations, not a current upstream guarantee.

## Confirmed decisions

1. No `.claude/settings.json` import in v1; payload/output compatibility only.
2. Timeout: kill + warning, non-blocking unless `fail_closed = true` (blocking mode only). Default 10s.

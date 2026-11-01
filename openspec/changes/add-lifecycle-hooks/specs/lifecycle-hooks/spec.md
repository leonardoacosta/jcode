## ADDED Requirements

### Requirement: Hook configuration
The config SHALL accept a `[hooks]` table keyed by event (`SessionStart`, `PreToolUse`, `PostToolUse`, `SubagentStop`, `Stop`, `PreCompact`) where each entry has a `command`, optional `matcher` and `path`, a `timeout` defaulting to 10 seconds, and `fail_closed` defaulting to false.

#### Scenario: Default timeout
- **WHEN** a hook entry omits `timeout`
- **THEN** the hook is killed after 10 seconds

#### Scenario: Invalid matcher
- **WHEN** a hook entry has an invalid matcher regex
- **THEN** that entry is disabled with a visible config warning and other hooks still load

### Requirement: Claude Code compatible payload
Each hook SHALL receive a JSON object on stdin containing `session_id`, `transcript_path`, `cwd`, and `hook_event_name`, plus event fields compatible with Claude Code hooks (`tool_name` and `tool_input` for tool events, `tool_response` for `PostToolUse`, `trigger` for `PreCompact`, `source` for `SessionStart`).

#### Scenario: PostToolUse payload
- **WHEN** a `PostToolUse` hook runs after a `bash` call
- **THEN** stdin contains `hook_event_name` `PostToolUse`, `tool_name` `bash`, `tool_input`, and `tool_response`

### Requirement: Tool matching
Tool event hooks SHALL run only when `matcher` matches the tool name and, when `path` is set, a file path argument of the tool matches the glob.

#### Scenario: Path filter excludes call
- **WHEN** a hook has `matcher = "edit"` and `path = "src/**"` and the agent edits `docs/a.md`
- **THEN** the hook does not run

### Requirement: PreToolUse blocking
A `blocking` `PreToolUse` hook exiting with code 2 SHALL block the tool call and return the hook's stderr to the model as the tool result. Other failures and timeouts SHALL block only when `fail_closed` is true.

#### Scenario: Exit code 2 blocks
- **WHEN** a `PreToolUse` hook exits 2 with stderr `protected path`
- **THEN** the tool is not executed and the model sees `protected path`

#### Scenario: Fail-open timeout
- **WHEN** a `PreToolUse` hook without `fail_closed` times out
- **THEN** the tool executes and a warning is logged

#### Scenario: Fail-closed error
- **WHEN** a `PreToolUse` hook with `fail_closed = true` exits 1
- **THEN** the tool is not executed

### Requirement: Subagent completion hook
When a swarm worker submits its completion report, the coordinator SHALL run `SubagentStop` hooks with the report (worker session, status, message, validation, follow-up) in the stdin payload.

#### Scenario: Notify on subagent finish
- **WHEN** a worker reports `completed`
- **THEN** a configured `SubagentStop` command receives JSON including that status and message

### Requirement: Hook execution modes
Each hook entry SHALL accept `mode` of `blocking`, `observe`, or `async`. Defaults SHALL be `blocking` for `PreToolUse` and `Stop`, `observe` for `SessionStart` and `PreCompact`, and `async` for `PostToolUse` and `SubagentStop`. `blocking` on any other event SHALL produce a config warning and behave as `observe`.

#### Scenario: Async hook does not delay
- **WHEN** an `async` `SubagentStop` hook sleeps 30 seconds
- **THEN** the coordinator continues immediately and the hook is killed at its timeout

#### Scenario: Observe hook decision ignored
- **WHEN** an `observe` `SessionStart` hook prints `{"decision":"block"}`
- **THEN** the session starts normally and the output is logged

#### Scenario: Blocking on unsupported event
- **WHEN** a `PostToolUse` hook sets `mode = "blocking"`
- **THEN** a config warning is shown and the hook runs as `observe`

### Requirement: JSON decision output
A `blocking` hook exiting 0 with JSON stdout SHALL be honored: `decision: "block"` or `hookSpecificOutput.permissionDecision: "deny"` blocks with `reason` returned to the model; `permissionDecision: "ask"` routes to the normal permission prompt; `additionalContext` is appended to model context. For `Stop`, `decision: "block"` SHALL continue the turn with `reason` as the next instruction. Non-JSON stdout SHALL be treated as no decision.

#### Scenario: JSON deny
- **WHEN** a `PreToolUse` hook exits 0 with `{"hookSpecificOutput":{"permissionDecision":"deny","permissionDecisionReason":"no prod"}}`
- **THEN** the tool is not executed and the model sees `no prod`

#### Scenario: Stop continuation
- **WHEN** a `Stop` hook exits 0 with `{"decision":"block","reason":"run tests"}`
- **THEN** the agent continues with `run tests`

#### Scenario: Silent success
- **WHEN** a blocking hook exits 0 with empty stdout
- **THEN** execution proceeds unchanged

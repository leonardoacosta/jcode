## ADDED Requirements

### Requirement: Hook configuration
Existing hook keys, environment overrides, input-transform and feedback semantics SHALL remain compatible through the existing runner. New event aliases SHALL NOT dispatch duplicate hooks. Commands SHALL preserve direct argv execution unless an explicitly configured command invokes a shell.
The config SHALL accept a `[hooks]` table keyed by event (`SessionStart`, `PreToolUse`, `PostToolUse`, `SubagentStop`, `Stop`, `PreCompact`) where each entry has a `command`, optional `matcher` and `path`, a `timeout` defaulting to 10 seconds, and `fail_closed` defaulting to false.

#### Scenario: Default timeout
- **WHEN** a hook entry omits `timeout`
- **THEN** the hook is killed after 10 seconds

#### Scenario: Invalid matcher
- **WHEN** a hook entry has an invalid matcher regex
- **THEN** that entry is disabled with a config warning written to the log and other hooks still load

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
A `blocking` `PreToolUse` hook exiting with code 2 SHALL block the tool call and return bounded safe stderr as its result. Other failures and timeouts SHALL block only when `fail_closed` is true (`PreToolUse` only; `fail_closed` has no effect on other events), but fail-open SHALL resume normal admission rather than override cancellation, revocation, budgets or other policy denials.

#### Scenario: Exit code 2 blocks
- **WHEN** a `PreToolUse` hook exits 2 with stderr `protected path`
- **THEN** the tool is not executed and the model sees `protected path`

#### Scenario: Fail-open timeout
- **WHEN** a `PreToolUse` hook without `fail_closed` times out
- **THEN** the tool is rechecked against normal admission and executes only if still permitted, with a safe warning logged

#### Scenario: Fail-closed error (exit 1 or timeout)
- **WHEN** a `PreToolUse` hook with `fail_closed = true` exits 1
- **THEN** the tool is not executed

### Requirement: Subagent completion hook
When a swarm worker submits its completion report, the coordinator SHALL run `SubagentStop` hooks with the report (worker session, status, message, validation, follow-up) in the stdin payload.

#### Scenario: Notify on subagent finish
- **WHEN** a worker reports `completed`
- **THEN** a configured `SubagentStop` command receives JSON including that status and message

### Requirement: Hook execution modes
Each hook entry SHALL accept `mode` of `blocking`, `observe`, or `async`. Defaults SHALL be `blocking` for `PreToolUse` and `Stop`, `observe` for `SessionStart` and `PreCompact`, and `async` for `PostToolUse` and `SubagentStop`. `blocking` on any other event SHALL produce a logged config warning and behave as `observe`.

#### Scenario: Async hook does not delay
- **WHEN** an `async` `SubagentStop` hook sleeps 30 seconds
- **THEN** the coordinator continues immediately and the hook is killed at its timeout

#### Scenario: Observe hook decision ignored
- **WHEN** an `observe` `SessionStart` hook prints `{"decision":"block"}`
- **THEN** the session starts normally and the output is logged

#### Scenario: Blocking on unsupported event
- **WHEN** a `PostToolUse` hook sets `mode = "blocking"`
- **THEN** a config warning is logged and the hook runs as `observe`

### Requirement: JSON decision output
A `blocking` hook exiting 0 with JSON stdout SHALL be honored: `decision: "block"` or `hookSpecificOutput.permissionDecision: "deny"` blocks with `reason` returned to the model; `permissionDecision: "ask"` blocks the call with a clear "requires user approval" reason, because jcode has no hook approval prompt; `additionalContext` is appended to model context. For natural-completion `Stop`, `decision: "block"` SHALL request at most one continuation per completion event, and at most 3 continuations per user turn, with `reason` as the next instruction, subject to normal admission and budgets. A failed, timed-out or crashed `Stop` hook (including with `fail_closed`) SHALL only be logged and never becomes a continuation. Explicit cancellation, durable stop or revoked generation SHALL suppress continuation regardless of hook output. Non-JSON stdout SHALL be treated as no decision.

#### Scenario: JSON deny
- **WHEN** a `PreToolUse` hook exits 0 with `{"hookSpecificOutput":{"permissionDecision":"deny","permissionDecisionReason":"no prod"}}`
- **THEN** the tool is not executed and the model sees `no prod`

#### Scenario: Stop continuation
- **WHEN** a natural-completion `Stop` hook exits 0 with `{"decision":"block","reason":"run tests"}` and normal admission permits continuation
- **THEN** the agent continues with `run tests`

#### Scenario: Cancellation overrides hook
- **WHEN** a hook requests continuation after explicit cancel or generation revocation
- **THEN** no new turn or tool execution is admitted, and owned hook processes remain accounted until settled

#### Scenario: Silent success
- **WHEN** a blocking hook exits 0 with empty stdout
- **THEN** execution proceeds unchanged

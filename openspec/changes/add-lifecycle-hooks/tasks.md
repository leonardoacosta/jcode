## 1. Config

- [ ] 1.1 Add `HooksConfig` (`[hooks]` with per-event entry lists: `matcher`, `path`, `command`, `timeout`, `fail_closed`, `mode`) to `crates/jcode-config-types/src/lib.rs`
- [ ] 1.2 Validate matcher regex and path glob at load; report invalid entries without aborting startup

## 2. Runner

- [ ] 2.1 Hook runner: spawn shell command, write JSON payload to stdin, enforce timeout, capture exit code, stdout, stderr
- [ ] 2.2 Mode handling: `blocking` awaited with decisions, `observe` awaited and logged, `async` detached with timeout kill; per-event defaults and warning for unsupported `blocking`
- [ ] 2.3 Parse Claude-compatible JSON stdout (`decision`, `reason`, `continue`, `stopReason`, `suppressOutput`, `hookSpecificOutput.permissionDecision`, `additionalContext`)
- [ ] 2.4 Build Claude Code compatible payloads per event

## 3. Events

- [ ] 3.1 `SessionStart` on new/resumed session with `source`
- [ ] 3.2 `PreToolUse` before dispatch in `turn_execution.rs`; exit 2 or JSON deny blocks; `ask` routes to permission prompt; `fail_closed` blocks on any failure or timeout
- [ ] 3.3 `PostToolUse` after tool result with `tool_response`
- [ ] 3.4 `SubagentStop` when a swarm worker submits its completion report, payload includes report
- [ ] 3.5 `Stop` when the agent turn ends (JSON block continues turn); `PreCompact` before compaction in `compaction.rs` with `trigger`

## 4. Tests and validation

- [ ] 4.1 Unit tests: matcher/path matching, timeout kill, exit code 2 block, JSON deny/ask/additionalContext, Stop continuation, async non-await, observe ignores decisions, fail_closed, payload snapshots per event
- [ ] 4.2 Integration test: SubagentStop hook receives worker report JSON
- [ ] 4.3 `openspec validate add-lifecycle-hooks --strict`

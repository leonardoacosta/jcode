## ADDED Requirements

### Requirement: Coordinator tool allowlist

While an overnight run is active, its coordinator session SHALL be allowed only: `swarm`, `todo`, `schedule`, `bg`, `read`, `agentgrep`, `conversation_search`, `session_search`, `batch`, MCP tools, `edit` and `write` whose target path ends in `.md` or lies inside the run's own directory (run bookkeeping: task cards, issue drafts, validation output), and `bash` whose command is read-only. Every other tool call SHALL be denied before execution. `batch` members SHALL be checked against the same rules. The allowlist SHALL be removed when the run ends.

#### Scenario: Code edit denied
- **WHEN** the coordinator calls `edit` on `src/main.rs`
- **THEN** the call is denied, the file is unchanged, and the result tells the coordinator to delegate the change to a worker

#### Scenario: Markdown edit allowed
- **WHEN** the coordinator calls `edit` or `write` on `notes/plan.md`
- **THEN** the call runs

#### Scenario: Run bookkeeping allowed
- **WHEN** the coordinator calls `write` on `<run_dir>/task-cards/x.json`
- **THEN** the call runs, but `<run_dir>/../escape.rs` and a `.json` outside the run directory are denied

#### Scenario: Patch tool denied
- **WHEN** the coordinator calls `apply_patch`
- **THEN** the call is denied

#### Scenario: Planning tools allowed
- **WHEN** the coordinator calls `swarm`, `todo` or a Linear MCP tool
- **THEN** the call runs

#### Scenario: Batch member denied
- **WHEN** the coordinator calls `batch` with a `read` member and an `edit` member on a `.rs` file
- **THEN** the `edit` member is denied and the `read` member runs

#### Scenario: Run ended
- **WHEN** the overnight run reaches a terminal status
- **THEN** the session's normal tool set is restored

### Requirement: Read-only coordinator bash

Coordinator `bash` SHALL run only when every command in the line is in the read-only set (`git status`, `git log`, `git diff`, `git show`, `git branch` without arguments that modify, `rg`, `grep`, `ls`, `cat`, `head`, `tail`, `wc`, `jq`, `find` without `-exec`/`-delete`, `pwd`, `echo`) and the line has no output redirection to a file, no command substitution, and no background operator. Any other command SHALL be denied.

#### Scenario: Read-only command
- **WHEN** the coordinator runs `git diff --stat && rg -n TODO src`
- **THEN** the command runs

#### Scenario: Build denied
- **WHEN** the coordinator runs `cargo build`
- **THEN** the command is denied with a delegation hint

#### Scenario: Redirect denied
- **WHEN** the coordinator runs `cat a.rs > b.rs`
- **THEN** the command is denied

### Requirement: Denial is recorded and guided

Each denied coordinator call SHALL record a `coordinator_tool_denied` overnight event with the tool name and reason, and the tool result SHALL say that edits, builds, reviews and judging go to swarm workers.

#### Scenario: Denial event
- **WHEN** a coordinator call is denied
- **THEN** events.jsonl contains `coordinator_tool_denied` with that tool name

### Requirement: Coordinator prompt states the role

Overnight coordinator prompts (supervised and visible current-session) SHALL state that the coordinator coordinates, facilitates, nudges and manages todos, and that every code change, build, review and judgement is assigned to a worker.

#### Scenario: Prompt text
- **WHEN** an overnight coordinator prompt is built
- **THEN** it contains the delegation rule

# tool-failure-recovery Specification

## Purpose
Make tool timeouts and file-edit mismatches recoverable with actionable diagnostics and safe continuation of affected work.

## Requirements

### Requirement: Foreground bash timeouts promote to background tasks

When a foreground `bash` call exceeds its timeout, the tool MUST NOT kill the command. It MUST register the still-running command as a background task and return the task id, output file path, and instructions to use the `bg` tool. This MUST hold for both the in-process and reload-persistable foreground paths.

#### Scenario: Long command exceeds foreground timeout
- **WHEN** a foreground command runs longer than its `timeout`
- **THEN** the command keeps running, the result metadata includes `timeout_promoted: true`, and the output names the task id and output file

#### Scenario: Promoted command later fails
- **WHEN** a promoted command exits non-zero
- **THEN** the background task records failed status and exit code, retrievable via `bg`

#### Scenario: Explicit background task times out
- **WHEN** a `run_in_background` task exceeds its timeout
- **THEN** the process group is killed and the task is marked timed out (no promotion)

### Requirement: Mismatch errors include nearest file context

When `edit` cannot find `old_string`, or `apply_patch` cannot find context or expected lines, the error MUST include the closest region of the current file with line numbers, bounded in size, if similarity meets the threshold. Existing exact whitespace/indentation hints MUST take precedence.

#### Scenario: Edit old_string drifted
- **WHEN** `old_string` differs from file text by one changed line
- **THEN** the error includes `Closest match near line N` and the actual line-numbered text

#### Scenario: Patch context drifted
- **WHEN** an `apply_patch` hunk's expected lines are not found but a similar region exists
- **THEN** the error includes the line-numbered closest region and no file is modified

#### Scenario: No similar region
- **WHEN** no window meets the similarity threshold
- **THEN** the error keeps the existing "use the read tool" message with no context block

#### Scenario: Indentation-only mismatch
- **WHEN** lines match after trimming
- **THEN** the existing indentation hint is returned

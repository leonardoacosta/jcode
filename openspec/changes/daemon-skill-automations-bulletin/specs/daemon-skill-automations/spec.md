## ADDED Requirements

### Requirement: Durable skill automation definitions
The system SHALL accept an installed skill and fixed frequency, capture working directory and provider/model reference, and persist enabled state and next due time without changing one-shot schedules.

#### Scenario: Create and restore
- **WHEN** the owner creates an automation for an available skill with `15m`
- **THEN** the system shows its captured directory and first due time 15 minutes later, and preserves the definition after restart.

#### Scenario: Invalid configuration
- **WHEN** creation uses an unknown skill, nonexistent directory, zero/negative/overflowing interval, unsupported unit, or interval below one minute
- **THEN** validation identifies the invalid field and no definition is persisted.

#### Scenario: Pause and change cadence
- **WHEN** the owner pauses a definition
- **THEN** no further run starts, and any current run may finish.
- **WHEN** the owner resumes or changes its frequency
- **THEN** the next due time is recalculated from the successful update time without a catch-up run.

### Requirement: Bounded daemon-owned recurrence
The daemon SHALL dispatch recurring runs independently of TUI clients and Ambient enablement, with one automation run active globally, no overlapping ticks, and a 30-minute execution deadline.

#### Scenario: Due runs while TUI is closed
- **WHEN** two enabled automations become due with Ambient disabled and no TUI connected
- **THEN** the earliest due run starts first, ties use stable IDs, and the other waits for the slot without accumulating queued ticks.

#### Scenario: Slow execution and deadline
- **WHEN** a run spans several intervals
- **THEN** its elapsed ticks are skipped rather than immediately replayed after completion.
- **WHEN** execution reaches 30 minutes
- **THEN** cancellation is requested, timeout is recorded, and the slot is not reused until termination is acknowledged.

#### Scenario: Clock changes and downtime
- **WHEN** startup finds overdue schedules or the wall clock jumps forward
- **THEN** startup skips missed runs and live scheduling coalesces elapsed ticks without bursts.
- **WHEN** the clock moves backward
- **THEN** persisted next due times prevent repeating previously claimed occurrences.

### Requirement: Safe skill execution and visible outcomes
The system SHALL resolve the skill for the captured directory on every run, use existing agent permission rules, and persist success, failure, blocked, timeout, or interrupted outcomes with session linkage and bounded final text.

#### Scenario: Completed execution
- **WHEN** a skill completes
- **THEN** the run stores its session ID, skill source identity, timestamps, final response, and success outcome without an extra summarization call.

#### Scenario: Skill removed or authorization unavailable
- **WHEN** the skill or captured directory is no longer available
- **THEN** the definition pauses with an actionable error and no agent starts.
- **WHEN** execution requires fresh human authorization
- **THEN** the run becomes blocked without granting permission or waiting indefinitely.

#### Scenario: Provider failure
- **WHEN** credentials are missing or the provider rejects a run
- **THEN** a sanitized failed outcome appears and no immediate retry loop starts.

### Requirement: Crash-safe claims and compatible storage
The system SHALL persist claims before execution, enforce a single writer, and fail closed on unusable state without changing existing Ambient or one-shot data.

#### Scenario: Restart after an uncertain run
- **WHEN** the daemon restarts with a running record
- **THEN** it marks that record interrupted, does not replay it, and schedules future intervals normally.

#### Scenario: Persistence or ownership failure
- **WHEN** state is corrupt, an unknown version, unwritable, or already owned by another daemon
- **THEN** new automation execution is blocked with a diagnostic, and the original state is preserved.

### Requirement: Explicit always-on provisioning
The system SHALL offer opt-in Linux user-service provisioning for the daemon and bulletin, distinguish daemon-lifetime from login-independent availability, and preserve unrelated services and active daemon ownership.

#### Scenario: Supervised operation
- **WHEN** the owner approves installation and required linger setup
- **THEN** exactly one managed daemon serves the bulletin and schedules tasks after closing the TUI, after logout, and after a service restart.

#### Scenario: Unsupported or denied provisioning
- **WHEN** user systemd or linger is unavailable or authorization is denied
- **THEN** the system states that login-independent operation is unavailable, without modifying system policy or claiming always-on provisioning succeeded.

#### Scenario: Disable and uninstall
- **WHEN** the owner uninstalls the managed service
- **THEN** that service stops and is disabled while automation history and unrelated services remain intact.

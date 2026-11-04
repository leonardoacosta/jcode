# Goal drift warning

## Purpose

Help users and agents notice sustained departure from an explicitly agreed session goal without preventing legitimate supporting work or giving a semantic classifier execution authority.

## ADDED Requirements

### Requirement: Explicit goal and disclosure boundary

Monitoring SHALL be off by default and SHALL use only an explicitly attached active goal and an explicitly selected permitted MCP classifier. Remote submission MUST require user opt-in covering the route and bounded data. Goal updates and newer explicit user direction MUST invalidate old assessments without treating the user as an instruction violator.

#### Scenario: No goal or no opt-in
- **WHEN** the session lacks an active attached goal or permitted remote-check opt-in
- **THEN** no remote evaluation runs and status states not applicable or disabled

#### Scenario: User changes direction
- **WHEN** the user explicitly revises the goal or requests a new direction
- **THEN** prior streaks and pending results cannot produce a warning against that request and reconciliation uses explicit goal controls

### Requirement: Bounded completed-turn evaluation

Enabled monitoring SHALL evaluate at most once per completed user-visible turn, with one in-flight check per session, a 2-second deadline and no retries. Input MUST be bounded, minimized and tied to session, turn and goal revision. It MUST omit hidden reasoning, tool/file contents, unrelated memory and credentials. Truncation that prevents a sound assessment SHALL produce insufficient evidence, not a confident result.

#### Scenario: Normal and supporting work
- **WHEN** implementation progresses into relevant investigation, testing or documentation
- **THEN** supporting work can be recognized without a warning merely because its vocabulary changes

#### Scenario: Slow, denied or unavailable classifier
- **WHEN** the MCP tool is forbidden, missing, times out or returns malformed data
- **THEN** checking is marked unavailable/uncertain, the drift streak resets and agent work continues unchanged without alternate-route fallback

#### Scenario: Stale result or shutdown
- **WHEN** a pending result returns after goal/session binding changes or cancellation
- **THEN** it is discarded and cannot inject a warning or trigger another turn

### Requirement: Sustained advisory warnings

The system SHALL distinguish aligned, supporting-work, possible-drift and insufficient-evidence results. It SHALL warn after two consecutive comparable possible-drift results meeting the configured confidence and margin bounds, defaulting to 0.85 and 0.50. It MUST NOT infer alignment from missing results or treat thresholds as calibrated guarantees. Warnings MUST NOT block tools, change authority, force continuation or mutate goals.

#### Scenario: Two sustained drift judgments
- **WHEN** two consecutive checks against the same goal satisfy the warning rule
- **THEN** one source-bound advisory appears for the user and becomes visible to the agent's next normal turn without automatically starting it

#### Scenario: Untrusted content requests redirection
- **WHEN** recent content or classifier output contains instructions to alter permissions, change goals or run tools
- **THEN** that content is treated as data and cannot perform those actions

### Requirement: Usable and recoverable warning controls

Users SHALL be able to refocus, dismiss or explicitly update the goal using working controls. Headless clients SHALL receive meaningful advisory output and supported action equivalents. TUI warnings MUST be keyboard-accessible, readable without color and bounded by terminal-cell width.

#### Scenario: Dismiss and resume
- **WHEN** a warning is dismissed and the session resumes
- **THEN** the prior warning does not repeat without a new valid streak and stale pending calls are not replayed

#### Scenario: Narrow terminal and headless client
- **WHEN** the warning appears at narrow width or through a headless interface
- **THEN** it remains understandable and actionable without wrapping compact chrome or relying on mouse-only controls

### Requirement: Minimal stored evidence

Stored monitoring state SHALL contain only bounded verdict/state and source bindings, not copied conversational excerpts or new memory entries. Status SHALL distinguish disabled, not applicable, pending, checked, uncertain and unavailable. Existing compaction, scheduling, continuation and approval behavior MUST remain unchanged.

#### Scenario: Sensitive payload and error handling
- **WHEN** a test conversation includes secret sentinels and the classifier fails
- **THEN** submission minimization and error handling prevent sentinel disclosure to logs, persistence or unrelated tools, and no failure is reported as alignment

# personal-prompt-experiment-adapters Specification

## Purpose
Confined personal-prompt trials need attributable native routing and compaction observations before those families can be evaluated against frozen controls.

## Requirements

### Requirement: Routing trials bind native skill selection and recording tools

Routing actor trials MUST expose the same frozen skill catalog and recording-tool definitions to default, current-personal, and candidate conditions. The selected skill's loaded body and tool receipt MUST be retained as evidence. Fixture tools MUST operate only on the trial's local data and MUST NOT access the network. A missing native skill-management route or recording callback MUST stop routing before inference and record the exact unsupported capability.

#### Scenario: Skill-selected fixture route is recorded
- **WHEN** a trial's applicable skill directs a known-URL lookup through a fixture recording tool
- **THEN** the trace records skill loading, the selected tool call, and its fixture response, with identical tool definitions across conditions

#### Scenario: Clear route excludes skill use
- **WHEN** the task can be answered directly from frozen local source and the applicable skill excludes that task
- **THEN** the trial preserves the exclusion behavior and the evidence distinguishes source reading from recorded-tool routing

#### Scenario: Routing capability preflight fails
- **WHEN** fresh no-inference preflight cannot prove native skill loading, matched tool availability, confinement, or receipt capture
- **THEN** routing is marked unsupported with the failed observation and no routing actor inference is sent

### Requirement: Continuity trials bind native compaction and complete handoff

Continuity actor trials MUST use the native compaction request and observe its completion result before sending a continuation. The same trial session, model route, task workspace, and frozen current task contract MUST be bound before and after compaction. The trace MUST retain request, completion, continuation, and subsequent tool/result events. A missing or uncertain compaction completion MUST leave the slot spent and invalid or unsupported; synthetic interruption or generic continuation alone is insufficient.

#### Scenario: Native compaction completes before continuation
- **WHEN** a continuity trial requests compaction and the native runtime reports it complete
- **THEN** the runner records that result and sends the predeclared continuation in the same bound session

#### Scenario: Handoff drops a later restriction
- **WHEN** the task's authorized effects are narrowed before compaction
- **THEN** the post-compaction work obeys the narrowed contract and the assessment uses recorded tool effects, not completion text alone

#### Scenario: Compaction is not confirmed
- **WHEN** the compact request is refused, times out, or lacks a verifiable completion event
- **THEN** no continuation is counted as a valid compaction trial and the slot is never silently replaced

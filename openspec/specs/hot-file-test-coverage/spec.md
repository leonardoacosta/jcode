# hot-file-test-coverage Specification

## Purpose
Protect frequently changed execution, protocol, and TUI input paths with focused characterization tests that detect behavioral regressions.

## Requirements

### Requirement: Turn execution characterization tests
The `jcode-app-core` crate SHALL include tests for `agent/turn_execution.rs` that drive a turn through a mock provider and assert on the resulting messages and returned text.

#### Scenario: Single turn captures assistant text
- **WHEN** `run_once_capture` runs against a mock provider that returns "ok"
- **THEN** it returns "ok" and the session history ends with one user and one assistant message

#### Scenario: Stale session lease is rejected
- **WHEN** the session lease is held by another owner
- **THEN** `ensure_session_lease` returns an error and no provider request is made

### Requirement: TUI input dispatch characterization tests
The `jcode-tui` crate SHALL include tests that feed key events into an `App` from `create_test_app()` and assert on input buffer and submission state.

#### Scenario: Enter submits non-empty input
- **WHEN** the user types "hi" and presses Enter
- **THEN** the input buffer is cleared and "hi" is queued for submission

#### Scenario: Enter on empty input does nothing
- **WHEN** Enter is pressed with an empty buffer
- **THEN** nothing is submitted and state is unchanged

### Requirement: Harness API translation snapshots
The `jcode-harness-api-server` crate SHALL compare full translation output for representative requests and legacy events against committed golden files.

#### Scenario: Translation matches golden file
- **WHEN** a fixture request is translated by `api_request_to_legacy`
- **THEN** the serialized output equals the committed snapshot

#### Scenario: Drift fails with a diff
- **WHEN** translation output differs from the snapshot
- **THEN** the test fails and prints the fixture name and a JSON diff, and regeneration only occurs with `UPDATE_SNAPSHOTS=1`

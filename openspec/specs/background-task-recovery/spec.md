# background-task-recovery Specification

## Purpose
Define how detached background tasks survive a server reload: reattach live tasks by pid and start-time identity, finalize dead or reused-pid tasks exactly once as lost, and suppress wakes for revoked session generations.

## Requirements

### Requirement: Background tasks survive reload
The server SHALL persist background tasks and, after reload, SHALL reattach live tasks and mark missing ones `lost`.

It SHALL reuse existing persisted task identity/manager records and retain owning execution generation. Reconciliation SHALL NOT wake revoked work or treat unknown process liveness as settled execution. Reattachment SHALL validate PID and process start identity, not PID alone.

#### Scenario: Task still running
- **WHEN** the server reloads while a background task's process is alive
- **THEN** the task SHALL remain listed and its completion SHALL be delivered to the owning session

#### Scenario: Process gone
- **WHEN** the server reloads and the task's process no longer exists
- **THEN** the task SHALL be marked `lost` with its output file still available

#### Scenario: Pid reused by another process
- **WHEN** the server reloads and the task's pid is alive but its process start time differs from the one recorded at registration
- **THEN** the task SHALL be marked `lost`, the unrelated process SHALL NOT be signalled or reaped (including by `bg cancel`), and the output file SHALL remain available
- Tests: `reattach_treats_reused_pid_as_gone_without_touching_the_process`, `cancel_never_signals_a_process_that_reused_the_pid`, `unreaped_zombie_with_reused_pid_is_not_reaped`

#### Scenario: Exit status unknown
- **WHEN** a task's process is gone and no exit status was recorded in its output
- **THEN** the task SHALL be `lost` with no exit code and SHALL NOT be reported as success; a recorded exit marker SHALL yield `completed` or `failed`
- Tests: `reattach_marks_vanished_task_lost_once_never_success`, `reattach_uses_recorded_exit_marker_for_success_and_failure`, `lost_background_task_renders_as_unknown_status_never_success`, `lost_tasks_are_terminal_unknown_and_selectable`

#### Scenario: Completion bound to a revoked generation
- **WHEN** a reattached task completes or stalls and the session generation that owned it was revoked by a stop (still unsettled, settled, or resumed under a newer generation)
- **THEN** the completion SHALL still be recorded and shown to attached clients, and no external wake, live turn or soft interrupt SHALL be started
- **AND** a task bound to a generation created after a resume SHALL wake normally
- Tests: `background_task_completion_wake_is_withheld_for_revoked_generation`, `background_task_stall_wake_is_withheld_for_revoked_generation`, `wake_is_revoked_only_for_stopped_generations`, `register_detached_task_stamps_start_time_and_owner_generation`

#### Scenario: Unsettled stop retains a reattached process
- **WHEN** a session whose stop is requested but unsettled owns a reattached live task
- **THEN** its pid SHALL stay in the session's stop barrier so the stop is reported unresolved until that process is proven gone; a settled or runnable session SHALL be left untouched
- Tests: `reattached_child_keeps_pending_stop_unresolved_and_is_idempotent`, `reattached_child_of_runnable_session_is_not_marked`

#### Scenario: Legacy status file
- **WHEN** a status file written before this change (no `pid_start_time`, no `owner_generation`) is read after reload
- **THEN** it SHALL load unchanged; if its pid is alive it SHALL be reattached, if gone it SHALL be marked `lost`
- Tests: `legacy_status_file_without_generation_fields_still_loads`, `legacy_running_file_is_reattached_when_alive_and_lost_when_dead`

#### Scenario: Single completion under concurrent reconciliation
- **WHEN** the output follower, `status`, `list`, the sweep, startup reattach and a late progress or delivery update race on one finished task
- **THEN** exactly one completion SHALL be published and the persisted status SHALL stay terminal
- Tests: `concurrent_reconcilers_publish_a_single_completion`, `progress_update_racing_finalize_yields_one_completion_and_terminal_status`, `progress_and_delivery_writers_never_overwrite_a_terminal_status`

### Requirement: Platform and legacy limits are explicit
The system SHALL treat a process whose start identity is unavailable as alive rather than gone, and the limits below are accepted for this change.

#### Scenario: Start identity unavailable
- **WHEN** the recorded or current process start time is unavailable (non-Linux target, or a legacy file)
- **THEN** the process SHALL be treated as alive and never reported gone
- Tests: `legacy_running_file_is_reattached_when_alive_and_lost_when_dead`

Accepted limits:

- Start-time identity (`platform::process_start_time`) exists on Linux only. Where either the recorded or the current start time is unavailable (macOS, Windows, and legacy `Running` files without `pid_start_time`), identity cannot be disproved and the process is treated as alive (fail open on liveness, never reported gone). On those targets a reused pid is therefore not detected and `bg cancel` may signal it. Adding macOS and Windows start-time sources is out of scope for this change.
- A legacy task with no recorded `owner_generation` cannot prove it predates a stop, so it wakes once the session is `Runnable` again, including after a resume.
- On Windows, startup reattach does not resume the output follower (`resume_reattached_task` is a no-op off Unix); such tasks are finalized only when `status`, `list` or the sweep reconciles them.

# overnight-run-bookkeeping Specification

## Purpose
TBD - created by archiving change harden-overnight-run-bookkeeping. Update Purpose after archive.

## Requirements

### Requirement: Manifest listing tolerates unreadable entries
`latest_manifest()` SHALL return the newest parseable manifest and SHALL skip manifests that cannot be read or deserialized, recording each skipped path through `crate::logging`. `load_manifest(run_id)` and `admission::read_manifest_at` SHALL keep failing for an explicitly addressed corrupt manifest.

#### Scenario: One corrupt manifest among valid runs
- **WHEN** the runs directory holds a valid older run, a newer manifest containing invalid JSON, and a valid newest run
- **THEN** `latest_manifest()` SHALL return the valid newest run and `/overnight status` SHALL render it

#### Scenario: Legacy status value
- **WHEN** a manifest's `status` is `"blocked"` or any string not matching a known variant
- **THEN** deserialization SHALL succeed with `OvernightRunStatus::Blocked` or `OvernightRunStatus::Unknown`, `label()` SHALL return `blocked` or `unknown`, and no active-run guard SHALL treat it as live

#### Scenario: Explicit load of a corrupt manifest
- **WHEN** `load_manifest("<id>")` targets a manifest with invalid JSON
- **THEN** it SHALL return an error and SHALL NOT fall back to a backup

### Requirement: TUI auto-poke exits finalize the run
Every code path that clears `App::overnight_auto_poke` SHALL record one `auto_poke_stopped` event whose summary names the reason, and when the manifest status is still `Running` SHALL persist `Completed` for a `final_done` exit or `Failed` for any other non-cancel exit, setting `completed_at`. Cancellation SHALL continue to persist `CancelRequested` through the existing path. Finalization SHALL be best-effort: a failed write SHALL surface as a system display message and SHALL NOT panic or block the UI.

#### Scenario: Final wrap completed
- **WHEN** the auto-poke loop observes the final-done condition
- **THEN** the manifest SHALL read `Completed`, `completed_at` SHALL be set, and the events log SHALL contain `auto_poke_stopped` with reason `final_done`

#### Scenario: Non-retryable turn error
- **WHEN** `stop_overnight_auto_poke_for_non_retryable_error` clears auto-poke
- **THEN** the manifest SHALL read `Failed` and the events log SHALL contain `auto_poke_stopped` with reason `non_retryable_error`

#### Scenario: Stall or poke budget exhausted
- **WHEN** the loop exits because of stalled turns or the poke budget
- **THEN** the manifest SHALL read `Failed` and the event reason SHALL be `stalled` or `poke_budget`

#### Scenario: Manifest unreadable or run mismatch
- **WHEN** the loop cannot read the latest manifest, or its `run_id` differs from the tracked run
- **THEN** auto-poke SHALL stop, a system message SHALL say why, and if the tracked run's own manifest is loadable by `run_id` it SHALL be finalized `Failed` with reason `manifest_unreadable` or `run_mismatch`

#### Scenario: User cancel
- **WHEN** `/overnight cancel` runs while auto-poke is active
- **THEN** the manifest SHALL read `CancelRequested` and the events log SHALL contain both `cancel_requested` and `auto_poke_stopped` with reason `cancelled`

### Requirement: TUI poke activity is recorded
The TUI auto-poke loop SHALL record `auto_poke_sent` with the phase label and stalled-turn count on each poke it queues, and `auto_poke_turn_error` with the error text on each turn error it inspects.

#### Scenario: Poke queued
- **WHEN** `schedule_overnight_poke_followup_if_needed` queues a poke
- **THEN** the events log SHALL contain `auto_poke_sent` with `phase` and `stalled_turns` in its data

#### Scenario: Event write fails
- **WHEN** the events file is not writable
- **THEN** the poke SHALL still be queued and the UI SHALL not panic

### Requirement: Readers surface stale supervisors
A manifest with status `Running` or `CancelRequested` whose `process_id` is not alive SHALL be labelled stale by `format_status_markdown` and the TUI progress card, naming the pid. Readers SHALL NOT rewrite the manifest. `supervisor_is_alive(&OvernightManifest) -> bool` SHALL use `kill(pid, 0)` semantics on unix (alive on `0` or `EPERM`) and SHALL return `pid != 0` elsewhere.

#### Scenario: Dead supervisor
- **WHEN** a `Running` manifest's `process_id` refers to no live process
- **THEN** `/overnight status` SHALL include `stale` and the pid, and the manifest file bytes SHALL be unchanged after rendering

#### Scenario: Live supervisor
- **WHEN** the `process_id` is the current process
- **THEN** no stale label SHALL appear

### Requirement: TUI start refuses a live duplicate
`/overnight <duration>` from the TUI SHALL refuse to start when a manifest with the same `coordinator_session_id` is `Running` or `CancelRequested` and `supervisor_is_alive` returns true, using the server path's message: `This session already has an active overnight run {run_id}. Inspect status or cancel it before starting another.`

#### Scenario: Live run in same session
- **WHEN** such a manifest exists
- **THEN** no new run directory SHALL be created and the message SHALL be shown as an error display message

#### Scenario: Stale run in same session
- **WHEN** the matching manifest is `Running` but its supervisor is not alive
- **THEN** the start SHALL proceed and a system message SHALL note the stale prior run id

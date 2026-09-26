## Why

Running a skill repeatedly currently requires manual invocation or arranging one-shot scheduled tasks. Users need durable daemon-owned recurrence and one local page showing what those automations produced, even after closing the TUI.

## What Changes

- Accept an installed skill and fixed frequency, with a captured working directory, as a persistent automation.
- Execute due automations in independent headless sessions using existing skill discovery and agent safeguards.
- Provision an opt-in lightweight HTTP listener inside the daemon, serving one HTMX page for automation controls and a chronological results bulletin.
- Keep scheduling and the page available without a TUI client. Provide explicit Linux user-service installation for restart and login-independent operation.
- Persist run outcomes, next due times, and pause state. Recover interrupted runs without automatically repeating potentially completed side effects.

## Capabilities

### New Capabilities

- `daemon-skill-automations`: Durable fixed-interval skill execution owned by the daemon.
- `automation-bulletin`: Local HTMX automation management and run-result bulletin.

### Modified Capabilities

None. Existing one-shot scheduling and Ambient behavior remain compatible.

## Scope

Actors: local owner configures schedules and reads results, daemon dispatches skills, existing agent runtime executes them. Minimum creation inputs: skill and frequency. Working directory defaults to the directory captured when the bulletin is provisioned, visibly editable at creation. Optional skill arguments are plain invocation text, not shell commands.

Proposed first-release choices, subject to approval: fixed intervals of at least one minute (`15m`, `1h`, `1d`), first run after one interval, missed ticks skipped, one active automation run globally, local-only access, Linux user-service provisioning. A day means 24 elapsed hours, not a local calendar appointment.

## Non-Goals

Cron expressions, calendar/timezone schedules, distributed scheduling, public hosting, remote authentication, desktop redesign, arbitrary shell scheduling, notifications, automatic skill installation, and guaranteed exactly-once external side effects. No service or code is installed by this proposal.

## Impact

Touches daemon startup/shutdown, headless execution, configuration, skill lookup, durable local storage, CLI provisioning, and a small embedded HTML/HTMX surface. No database or separate frontend build/runtime. HTTP dependency choice is an implementation-time check, not permission to hand-roll HTTP parsing. Linux service provisioning must preserve the current launcher's build-channel behavior and avoid competing shared daemons.

## Approval

Draft, awaiting user approval. Implementation and service installation are blocked until approval. Enabling an automation separately confirms recurring model usage and the selected skill's existing permissions. Persistent login-independent service setup requires an explicit provisioning action.

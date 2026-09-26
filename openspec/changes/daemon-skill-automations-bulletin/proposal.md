## Why

Running a skill repeatedly currently requires manual invocation or arranging one-shot scheduled tasks. Users need durable daemon-owned recurrence and one private local-or-tailnet page showing what those automations produced, even after closing the TUI.

## What Changes

- Accept an installed skill and either a fixed interval or selected weekdays/time of day/timezone, with a captured working directory, as a persistent automation.
- Execute due automations in independent headless sessions using existing skill discovery and agent safeguards.
- Provision an opt-in lightweight HTTP listener inside the daemon, serving one HTMX page for automation controls and a chronological results bulletin, accessible over Tailscale as a first-class supported network.
- Keep scheduling and the page available without a TUI client. Provide explicit Linux user-service installation for restart and login-independent operation.
- Persist run outcomes, next due times, and pause state. Recover interrupted runs without automatically repeating potentially completed side effects.

## Capabilities

### New Capabilities

- `daemon-skill-automations`: Durable interval and timezone-aware calendar skill execution owned by the daemon.
- `automation-bulletin`: Local and Tailscale-accessible HTMX automation management and run-result bulletin.

### Modified Capabilities

None. Existing one-shot scheduling and Ambient behavior remain compatible.

## Scope

Actors: owner on the host or an authorized tailnet device configures schedules and reads results, daemon dispatches skills, existing agent runtime executes them. Minimum creation inputs: skill and schedule. Calendar schedules require selected weekdays, one local time of day, and a visible IANA timezone. Working directory defaults to the directory captured when the bulletin is provisioned, visibly editable at creation. Optional skill arguments are plain invocation text, not shell commands.

First-release scope includes intervals of at least one minute (`15m`, `1h`, `1d`) and calendar schedules such as Monday–Friday at 09:00 America/Chicago or Sunday at 18:00 Europe/London. Every day is all seven weekdays. Calendar times follow their timezone through daylight-saving changes. An interval day still means 24 elapsed hours. Proposed defaults: first future occurrence, missed occurrences skipped, one active automation run globally, Linux user-service provisioning. Tailscale HTTPS access is a required delivery path, not an optional follow-up or localhost-only workaround.

## Non-Goals

Raw cron expressions, multiple times within one calendar definition (use separate automations), monthly/yearly schedules, distributed scheduling, public hosting or Tailscale Funnel, desktop redesign, arbitrary shell scheduling, notifications, automatic skill installation, and guaranteed exactly-once external side effects. No service or code is installed by this proposal.

## Impact

Touches daemon startup/shutdown, headless execution, configuration, skill lookup, durable local storage, CLI provisioning, and a small embedded HTML/HTMX surface. No database or separate frontend build/runtime. Includes managed Tailscale Serve provisioning/status/rollback, tailnet browser authentication, and second-device acceptance testing. HTTP dependency choice is an implementation-time check, not permission to hand-roll HTTP parsing. Linux service provisioning must preserve the current launcher's build-channel behavior and avoid competing shared daemons.

## Approval

Approved by the user on 2026-09-26 at 07:08:40 UTC, following revision `9635a9425`. The weekday/timezone schedule and first-class Tailscale scope, including proposed design defaults, are approved for implementation in task dependency order. Implementation has not started. Enabling an automation separately confirms recurring model usage and the selected skill's existing permissions. Persistent login-independent service setup and Tailscale Serve configuration require explicit provisioning actions. Tailscale login, HTTPS availability, and tailnet access policy are human-controlled prerequisites, never silently changed.

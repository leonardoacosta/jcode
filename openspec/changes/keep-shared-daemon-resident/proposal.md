---
execution:
  version: 1
  depends_on: [recover-missed-scheduled-runs, reattach-background-tasks-after-reload, reconcile-overnight-runs-after-restart]
---
Roadmap owner: [P05: Durable overnight, scheduling and recovery](../../../docs/plans/PRODUCT_ROADMAP.md#p05-durable-overnight-scheduling-and-recovery)

## Why

`crates/jcode-app-core/src/server.rs:561-562,580-581,1816-1874` currently gives the ordinary shared server a five-minute idle timeout and exits with `EXIT_IDLE_TIMEOUT`; the temporary-server path has a separate lifecycle monitor. The ambient/schedule loop runs as a server-owned task (`server.rs:1466-1474`), so retiring the shared process while no clients are connected can stop unattended work. The SIGTERM path unregisters and drains session saves before exiting (`server.rs:1279-1307`), but must preserve pending work through service restart and reload. Commit `b03b08db327268938ba0928b625d996bd00627b8` added idle exit to address orphaned self-dev servers; the fix must retain that cleanup for temporary servers without applying it to the shared systemd service.

The registry already exposes server `started_at`, `list_servers()` performs stale-entry cleanup, and each server record contains session names (`crates/jcode-base/src/registry.rs:15-39,119-150,226-230`). TUI and CLI users currently need a client-visible view that makes daemon and client counts and uptime observable.

## What Changes

- Keep the shared `jcode serve` daemon resident regardless of idle duration; do not return the idle-timeout exit code for that server scope.
- Keep idle exit for temporary/self-dev servers. Require owner-PID liveness checks when an owner PID is recorded; allow the configured idle timeout to retire an ownerless temporary server only after it has no clients, active workers, or pending work.
- Prevent every server scope from idle-exiting while scheduled, ambient, or background work is pending or active. Reuse durable recovery/reattachment contracts rather than introducing a second work registry.
- Handle SIGTERM and reload as orderly lifecycle transitions: stop new work admission, settle or persist pending work for existing recovery, drain session saves, unregister/clean temporary artifacts, and let the service manager restart the shared daemon without an idle-exit loop.
- Add client-visible CLI and TUI status for server count, connected-client count, and each server's uptime from registry `started_at`; omit stale servers and keep compact UI rows within terminal cell width.

## Capabilities

### New Capabilities

- `shared-server-lifecycle`: scope-aware daemon residency, safe shutdown/reload, and client-visible server/client status.

### Modified Capabilities

None.

## Impact

Primary surfaces: `crates/jcode-app-core/src/server.rs`, `crates/jcode-app-core/src/server/lifecycle.rs`, `crates/jcode-base/src/registry.rs`, and the existing CLI/TUI status commands or views. Reuse registry cleanup and existing persisted schedule/background recovery; do not add a parallel task ledger. Tests must cover shared versus temporary scope, owner PID exit, pending-work protection, systemd `Restart=always`, stale registry cleanup, reload/shutdown recovery, and narrow-terminal truncation.

## Non-goals

No systemd unit installation or deployment changes, scheduler redesign, new work persistence format, exactly-once execution guarantee, changes to ambient authorization, or Desktop UI. This proposal does not change roadmap files or implement any dependent recovery change.

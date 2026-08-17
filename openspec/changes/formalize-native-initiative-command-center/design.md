## Context

Jcode's initiative tool and app-core goal persistence are the durable authority. The Command Center web host and SolidStart application project that state; ambient schedules, Jcode runs, and Orca are linked evidence or external runtime authorities.

`docs/COMMAND_CENTER.md:7-13` already states the authority model and `:15-25` already maps the implementation seams. This change does not restate that work. It addresses what the existing document leaves ambiguous or overstates: the `goal`/`initiative` vocabulary split, the durability limits of revision and idempotency, and the status of `sync_goal_memory` as a second store.

## Goals

- Name one canonical vocabulary.
- State durability guarantees that match the code exactly.
- Give future contributors a single extension path.
- Make the no-frontend-persistence invariant checkable rather than asserted.

## Non-Goals

- Rebuild the web UI.
- Add a frontend database.
- Replace the TUI.
- Move Orca or scheduling authority into initiatives.
- Rename any Rust type, tool, or command.
- Add durable revisions, durable idempotency, or a protocol `degraded` state.

## Decisions

### D1: `initiative` is canonical externally, `goal` internally

The user-facing term in documentation, tool names, and UI copy is **initiative**. The internal Rust type prefix remains **Goal** (`Goal`, `GoalStatus`, `GoalMilestone`, `GoalStep`, `GoalUpdate`, `crate::goal::`). Documentation SHALL state once that these denote the same entity, and SHALL NOT introduce a third term.

No rename occurs in this change. `crates/jcode-app-core/src/tool/goal.rs` continues to register the `initiative` tool. A future rename, if desired, is a separate proposal with its own migration cost.

### D2: Document persistence authority at its true strength

App-core goal persistence is authoritative for **identity, status, milestones, steps, and updates**. Documentation SHALL state, with citations:

| Property | Reality | Source |
| --- | --- | --- |
| Revision | Derived from `updated_at.timestamp_millis()`; two saves in one millisecond produce the same revision; not persisted on `Goal` | `command_center.rs:528`, `jcode-task-types/src/lib.rs:110-136` |
| Idempotency | In-memory `HashMap` keyed by `(String, String)`; lost on daemon restart; a retried command re-applies | `jcode-command-center/src/lib.rs:1226`, `:1331` |
| Checkpoints | Not an entity; `checkpoint_summary` is appended to `goal.updates` as a `GoalUpdate` | `jcode-base/src/goal.rs:147-157` |
| Memory sync | `sync_goal_memory` writes a second derived store on every update; non-authoritative | `jcode-base/src/goal.rs:160`, `:624-651` |
| Step/milestone status | Free-form `String` with `default_pending_status`, unlike the 7-variant `GoalStatus` enum | `jcode-task-types/src/lib.rs:44-52`, `:88-97` |

Durable revisions and durable idempotency are tracked by `optimize-orca-command-center-orchestration` task 3.4 and are out of scope here.

### D3: Extend through native contracts

New browser views consume generated DTOs and issue public commands through the daemon. New TUI views use the same goal repository and lifecycle semantics.

**Known exception, documented not fixed:** the TUI calls `crate::goal::*` directly (`crates/jcode-tui/src/tui/app/state_ui_input_helpers.rs:1101`, `commands.rs:2461-2520`) rather than going through `GoalInitiativeRepository`, so the daemon's revision check does not apply on the TUI path. Concurrent TUI and browser writes to one initiative are last-write-wins via `save_goal` (`jcode-base/src/goal.rs:512`).

### D4: Fail-closed boundaries are documented where they exist

Real fail-closed behavior exists for Orca: `UnsupportedCapability` at `crates/jcode-command-center/src/lib.rs:1388,1405,1422,1434,1482`, and `OrcaUnavailable` at `crates/jcode-app-core/src/command_center.rs:376`. Documentation SHALL describe these and SHALL NOT claim equivalent scheduler-unavailable or browser-host-unavailable states, which do not exist.

## Known failure modes to document

- Revision collision within a single millisecond.
- Idempotency cache loss across daemon restart, causing a retried command to re-apply.
- Concurrent TUI and browser writes resolving last-write-wins.
- `sync_goal_memory` failure after a successful `save_goal`: the write reached disk but `update_goal` returns `Err`, so the caller believes it failed (`jcode-base/src/goal.rs:159-160`).

## Verification

- Focused app-core and command-center tests pass.
- Contract generation remains drift-free.
- The no-frontend-persistence check passes and demonstrably fails on an introduced storage call.
- Every durability statement in the new documentation cites a source line that supports it.

## Why

`docs/COMMAND_CENTER.md` already documents the authority model and an implementation-seam table. What it does not do is resolve three concrete defects that make the boundary unreliable in practice:

1. **The domain word is ambiguous.** Internally everything is `goal` (`GoalStatus`, `GoalMilestone`, `GoalStep`, `crate::goal::`, the TUI `/goals` command at `crates/jcode-tui/src/tui/app/commands.rs:2451`). Externally the user-facing tool is `initiative` (`crates/jcode-app-core/src/tool/goal.rs:109`, `crates/jcode-app-core/src/tool/mod.rs:273`). No document states which word is canonical or that they denote the same entity.
2. **The persistence-authority claim is stronger than the code.** Revision is derived, not stored: `crates/jcode-app-core/src/command_center.rs:528` computes `Revision(goal.updated_at.timestamp_millis())`, and the persisted `Goal` (`crates/jcode-task-types/src/lib.rs:110-136`) has no revision field. Idempotency is process-local: `crates/jcode-command-center/src/lib.rs:1226` holds `Arc<Mutex<HashMap<(String, String), CommandResult>>>`, lost on restart. Checkpoints are not entities: `crates/jcode-base/src/goal.rs:147-157` folds `checkpoint_summary` into `goal.updates`. And `crates/jcode-base/src/goal.rs:160` calls `sync_goal_memory`, writing a second derived store, which any naive "single store" statement contradicts.
3. **Status typing is inconsistent.** `GoalStatus` is a 7-variant enum (`crates/jcode-task-types/src/lib.rs:44-52`), while `GoalStep.status` and `GoalMilestone.status` are free-form `String` with a `default_pending_status` fallback (`:88-97`).

This change documents the boundary accurately, names the canonical vocabulary, and records the durability limits honestly rather than asserting guarantees the code does not provide.

## What Changes

- Declare `initiative` the canonical user-facing term and `goal` the internal type-name prefix, and state that they denote one entity.
- Document app-core goal persistence as the authority for initiative identity, status, milestones, steps, and updates, explicitly scoping out revision durability, idempotency durability, and checkpoint identity.
- Document that revision is derived from `updated_at` in milliseconds, with its collision window stated.
- Document that idempotency is process-local and lost on daemon restart, cross-referencing the durable-store work owned by `optimize-orca-command-center-orchestration` task 3.4.
- Document `sync_goal_memory` as a derived, non-authoritative projection.
- Document the `GoalStatus` enum versus free-form step/milestone status divergence as a known limitation.
- Document the Command Center web application as a browser projection with no local persistence, which is true today and asserted as a maintained invariant.
- Add contributor guidance for extending native surfaces before creating a new UI or persistence path.

## Capabilities

### New Capabilities

- `native-initiative-command-center`: Documented native initiative authority, vocabulary, and extension contract across app-core, TUI, daemon API, and web projections.

### Modified Capabilities

- `command-center-protocol`: Document derived-revision semantics, process-local idempotency scope, and the absence of a degraded-state vocabulary in the protocol today.
- `command-center-web`: Document the no-local-persistence invariant as an enforced, checkable property.

## Impact

- Updates `docs/COMMAND_CENTER.md` and adds a vocabulary and extension guide under `docs/`.
- Adds one repository check asserting the no-frontend-persistence invariant.
- May update doc comments on public DTOs without changing wire shapes.
- Does not add a second database, replace the TUI, or make Orca the source of truth.
- Does not rename any Rust type, tool, or command in this change.

## Out of Scope

- Making revisions or idempotency durable. That work is owned by `optimize-orca-command-center-orchestration` task 3.4.
- Introducing a `degraded` state into the protocol. `rg degraded crates/jcode-command-center/src/lib.rs` returns nothing; adding one is a behavioral change requiring its own proposal.
- Promoting `GoalStep.status` and `GoalMilestone.status` to a typed enum.
- Routing the TUI through `GoalInitiativeRepository`. The TUI currently calls `crate::goal::*` directly (`crates/jcode-tui/src/tui/app/state_ui_input_helpers.rs:1101`, `commands.rs:2461-2520`), bypassing the daemon revision check. This change documents that fact; changing it is separate.

## Preconditions

- Base commit: `5f8df69c0` on `dev`.
- Sequenced after `optimize-orca-command-center-orchestration` task 4.1, which owns `docs/COMMAND_CENTER.md` and `docs/COMMAND_CENTER_MIGRATION_LEDGER.md`. This change SHALL NOT be applied while that task is open.
- The Command Center vertical slice is **not** complete: `add-solidstart-command-center-vertical-slice` tasks 4.5 (Orca adapter) and 8.4 (full test/lint gate) are open, and `docs/COMMAND_CENTER.md` records the rollout gate as "Blocked, not passed". This change documents the current state and does not depend on that gate.

## Done Means

- One document states that `initiative` and `goal` denote the same entity and which is canonical in which context.
- Every durability claim in the documentation is traceable to a cited source line and does not overstate the code.
- `rg -n 'localStorage|sessionStorage|indexedDB' apps/command-center/src` returns no matches, and a repository check enforces this.
- A contributor can identify the authoritative store, mutation path, projection path, and Orca boundary from one entry point.
- No wire shape, type name, or runtime behavior changes.

## Testing

- `cargo test -p jcode-app-core command_center` passes.
- `cargo test -p jcode-command-center` passes.
- `scripts/check-command-center-contracts.sh` reports no drift.
- The new no-frontend-persistence check passes, and fails when a storage API call is introduced.
- `openspec validate formalize-native-initiative-command-center --strict --no-interactive`.

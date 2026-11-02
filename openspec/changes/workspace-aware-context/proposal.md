## Why

Roadmap owner: [P07: Memory/context quality and privacy](../../../docs/plans/PRODUCT_ROADMAP.md#p07-memorycontext-quality-and-privacy).

Forensics (60d) show heavy switching between personal, Priceless, and Brown workspaces. Tracker (Linear vs ADO), branch naming, skill routing, and Git identity rules differ per workspace, yet jcode infers none of it. The agent re-derives it each session from prompt prose, and mistakes (wrong tracker link, wrong identity) are costly.

## What Changes

- Add `[workspaces]` user config (`crates/jcode-base/src/config.rs`): ordered rules mapping git `origin` URL patterns (host + path glob) to a workspace name with hints: `tracker` (`linear`|`ado`|`none`), `branch_template`, `skills`, `expected_email`.
- Detect workspace at session start from the working dir's `origin`; credentials stripped before matching. No origin or no match: `unknown`.
- Compact workspace badge in TUI header (`crates/jcode-tui/src/tui/ui_header.rs`) following the no-wrap rule: fits display-cell width or truncates with ellipsis.
- Inject a bounded per-workspace hint block (tracker, branch template, skills) into the system prompt.
- Cached Git identity check: compare effective `user.email` with `expected_email`, cache per repo root + config mtimes, warn on mismatch. Never modifies Git config.

- Per-workspace `hooks` (merged over global `[hooks]`), per-workspace `swarm_model` and role-slot model overrides, and propagation of the resolved workspace to spawned swarm workers. Depends on `add-lifecycle-hooks` and `add-role-model-slots`.

## Capabilities

### New Capabilities
- `workspace-context`: workspace detection, badge, hints, identity check.

- `workspace-hooks-and-swarm`: Workspace-scoped hooks and swarm model overrides, and workspace propagation to workers.

## Impact

Config types, session startup, TUI header, prompt assembly. No new deps.

## Open decisions

1. **Match syntax.** Recommended default: host + path glob (`dev.azure.com/brown/**`). Alternative: regex (more power, worse ergonomics).
2. **Identity mismatch severity.** Recommended: warning badge + prompt note only. Alternative: block commits via tool gate (safer, intrusive).
3. **Re-detection timing.** Recommended: session start and working-dir change. Alternative: every turn (catches `cd`, extra git call; mitigated by cache).


## Dependencies

- `add-lifecycle-hooks` (hook schema and runner) must land before per-workspace hooks.
- `add-role-model-slots` (role-slot config; change not yet drafted in `openspec/changes/`) must land before role-slot overrides. Swarm spawn code: `crates/jcode-app-core/src/server/comm_session.rs`.

## Approval

Approved 2026-11-01 with recommended defaults for all open decisions.

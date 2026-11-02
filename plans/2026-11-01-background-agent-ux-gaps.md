# Background agent UX gaps (2026-11-01)

Read-only survey. Proposals checked for overlap: all 21 under `openspec/changes/*` (notably recover-swarm-workers-after-reload, queue-messages-to-busy-workers, return-full-swarm-reports, show-resolved-worker-model, add-lifecycle-hooks, shared-server-crash-resilience, add-swarm-file-claims). Archive names skimmed: repair-jcode-herdr-session-binding, swarm-communicate-reliability, swarm-widget-cell-width, swarm-state-async-safety, swarm-module-split.

| # | Gap | Evidence | Proposed change | Overlap checked | Effort | Value |
|---|-----|----------|-----------------|-----------------|--------|-------|
| 1 | Herdr only gets `idle`/`working`; herdr supports `blocked`, `done` | `crates/jcode-tui/src/tui/herdr.rs:31-34` (`AgentState{Idle,Working}`); `terminal_title.rs:71-76`; herdr SKILL.md:55-59 | Add `Blocked` (permission/ask-user prompt pending) and `Done` (turn finished, unseen) to `AgentState`; map in `sync_herdr_agent_state` | repair-jcode-herdr-session-binding (archived, session binding only) | S | High |
| 2 | Headless swarm workers never report to herdr | `herdr::sync` only called from TUI client `terminal_title.rs:76`; no herdr refs in `server/swarm*.rs` | Coordinator TUI reports aggregate worker state (any blocked/stalled → `blocked`); or spawn visible workers in herdr panes and let each sync | recover-swarm-workers-after-reload, show-resolved-worker-model | M | High |
| 3 | Stalled state not surfaced in info widget | watchdog flags stalls `server/swarm_watchdog.rs:1-15,59`, but widget status map `info_widget_swarm_background.rs:351` lists blocked/failed/crashed, not stalled | Propagate stall flag to member status; render ⚠ `stalled Nm` row with idle duration | none (watchdog is swarm list/status only) | S | High |
| 4 | No jump-to-worker-pane | `terminal-launch/src/lib.rs:662-669` spawns herdr panes but pane id not stored on member | Record spawned `pane_id` on swarm member; `/agents` action / keybind runs `herdr pane focus <id>` | none | M | High |
| 5 | Completion notifications are in-session only | swarm/bg completion delivered as chat notification; herdr `done` unused (gap 1) | On worker/bg task completion or failure while client unfocused: herdr `done`, terminal bell/OSC 9 desktop notify | add-lifecycle-hooks (user hooks, not built-in notify) | S | Med-High |
| 6 | Background tasks (bash run_in_background) not resumed/reattached after reload | reload path `server/reload.rs`; recover-swarm-workers-after-reload covers swarm members only | Persist bg task registry (pid, output path); on reload reattach live pids, mark dead ones `lost` | recover-swarm-workers-after-reload (workers only) | M | High |
| 7 | No unified lifecycle enum across bg tasks / swarm members | widget matches free-form strings `info_widget_swarm_background.rs:87,351` | Single `Lifecycle{Spawned,Running,Waiting,Blocked,Stalled,Done,Failed}` shared by swarm, bg tasks, herdr mapping | swarm-module-split (structure only) | M | Med |
| 8 | Blocked reason invisible | widget shows `blocked` label only (`:351`) | Show short reason (awaiting approval / waiting on dep / waiting_network) + age | queue-messages-to-busy-workers | S | Med |
| 9 | Stall nudge opt-in & silent to user | `swarm_watchdog.rs:10-13` (`swarm_stall_nudge`) | Toast in coordinator TUI with actions: nudge / stop / open pane | none | S | Med |
| 10 | Herdr resume only for foreground session | `herdr.rs:369` `resume_argv_for_session` uses client session | Workers in herdr panes report own resume argv so herdr restores them after crash | shared-server-crash-resilience | S | Med |
| 11 | Done items linger/vanish without "unseen" tracking | widget renders done rows without seen state | Mark completions unseen until viewed; badge count in header | return-full-swarm-reports | S | Med |
| 12 | No ETA/progress for long workers | bg tasks support `JCODE_PROGRESS`, swarm members don't | Let workers emit todo-derived progress (completed/total) into widget bar | none | M | Low-Med |

## Top 5
1. Herdr `blocked`/`done` states (S, high).
2. Surface stalled workers in info widget (S, high).
3. Herdr aggregate state for headless workers (M, high).
4. Jump-to-worker herdr pane (M, high).
5. Persist + reattach background tasks after reload (M, high).

Not done: `feature/herdr-terminal-browser` branch diff not reviewed in depth (terminal-browser pane, not status).

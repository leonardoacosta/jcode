## 1. States

- [ ] 1.1 Add `Done` to `AgentState` with its Herdr string (`Blocked` ships in `fix-herdr-pane-status-accuracy`)
- [ ] 1.2 Derive done from turn completion until focus, and add blocked for pending approval/ask-user prompts, extending the state function from `fix-herdr-pane-status-accuracy` (which covers only `requires_user_auth` todos)

## 2. Aggregate

- [ ] 2.1 Extend the shipped aggregate (owned running workers and wake background tasks) with blocked or stalled owned workers and finished-unseen work (blocked > working > done > idle)
- [ ] 2.2 Optional OSC 9/bell on unfocused completion or failure (config, default off)

## 3. Verify

- [ ] 3.1 Unit tests for shared projection/precedence, draining/unresolved/failed/cancelled labels and Herdr-only done-clears-on-focus; verify widget cursors and execution state are unchanged.
- [ ] 3.2 Runtime check in a Herdr pane with a headless worker that blocks and finishes
- [ ] 3.3 `openspec validate report-agent-lifecycle-to-herdr --strict`

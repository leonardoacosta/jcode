## 1. States

- [ ] 1.1 Add `Blocked` and `Done` to `AgentState` with Herdr strings
- [ ] 1.2 Derive blocked from pending approval/ask-user prompts; done from turn completion until focus

## 2. Aggregate

- [ ] 2.1 Fold owned swarm members and background tasks into the reported state (blocked > working > done > idle)
- [ ] 2.2 Optional OSC 9/bell on unfocused completion or failure (config, default off)

## 3. Verify

- [ ] 3.1 Unit tests for state precedence and done-clears-on-focus
- [ ] 3.2 Runtime check in a Herdr pane with a headless worker that blocks and finishes
- [ ] 3.3 `openspec validate report-agent-lifecycle-to-herdr --strict`

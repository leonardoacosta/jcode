## 1. Model

- [ ] 1.1 Shared lifecycle enum for swarm members and background tasks, parsing legacy strings
- [ ] 1.2 Propagate watchdog stall flag and blocked reason into member status

## 2. Widget

- [ ] 2.1 Render `stalled Nm` and blocked reason with age, display-cell truncated
- [ ] 2.2 Unseen tracking and header count
- [ ] 2.3 Stall toast with nudge / stop / open-pane

## 3. Verify

- [ ] 3.1 Widget render tests at narrow widths; stall and blocked fixtures
- [ ] 3.2 `openspec validate surface-worker-stall-and-block-reasons --strict`

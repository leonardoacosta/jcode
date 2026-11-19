## 1. Model

- [ ] 1.1 Lossless presentation projection over existing swarm/background/control types, parsing legacy strings and retaining stopped/crashed/cancelled/draining/unresolved distinctions. Verify none is displayed as successful completion.
- [ ] 1.2 Propagate watchdog stall flag and blocked reason into member status

## 2. Widget

- [ ] 2.1 Render truthful `stalled Nm` and blocked reason with age, display-cell truncated, on the upstream dedicated swarm page rather than a fork-only inline strip.
- [ ] 2.2 Completion-event IDs and per-consumer seen cursors; verify widget/page viewing does not implicitly acknowledge Herdr or alter execution.
- [ ] 2.3 Show only upstream-supported authorized stall actions; retain durable stop and owner checks, omit unavailable pane focus, and verify no nudge revives revoked work.

## 3. Verify

- [ ] 3.1 Upstream swarm page render tests at narrow widths; stall and blocked fixtures
- [ ] 3.2 Report coalescing tests prove a prerequisite transition and full report cannot be dropped or overtaken; completion cursor remains independent of execution settlement.
- [ ] 3.3 `openspec validate surface-worker-stall-and-block-reasons --strict`

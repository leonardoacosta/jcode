## 1. Model

- [ ] 1.1 Lossless presentation projection over existing swarm/background/control types, parsing legacy strings and retaining stopped/crashed/cancelled/draining/unresolved distinctions. Verify none is displayed as successful completion.
- [ ] 1.2 Propagate watchdog stall flag and blocked reason into member status

## 2. Widget

- [ ] 2.1 Render `stalled Nm` and blocked reason with age, display-cell truncated
- [ ] 2.2 Completion-event IDs and per-consumer seen cursors; verify widget/agents viewing does not implicitly acknowledge Herdr or alter execution.
- [ ] 2.3 Stall toast with authorized nudge/durable-stop; gate open-pane on verified capability/identity and test absent capability plus revoked target.

## 3. Verify

- [ ] 3.1 Widget render tests at narrow widths; stall and blocked fixtures
- [ ] 3.2 `openspec validate surface-worker-stall-and-block-reasons --strict`

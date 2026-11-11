## 1. Persist and reattach

- [ ] 1.1 Characterize existing status-file/detached-task/orphan recovery and add only missing persisted owner-generation/reattach fields through the existing manager; verify no duplicate registry and legacy status files remain readable.
- [ ] 1.2 On reload, reattach live PIDs (PID + start time match); mark others `lost`
- [ ] 1.3 Deliver generation-bound completion for reattached tasks; suppress revoked wakes, retain uncertain liveness for stop settlement, and keep unknown exit status unknown. Verify PID reuse, duplicate delivery and cancelled target cases.

## 2. Verify

- [ ] 2.1 Tests: live task reattached; dead task `lost`; PID reuse rejected
- [ ] 2.2 Runtime check across `selfdev reload`
- [ ] 2.3 `openspec validate reattach-background-tasks-after-reload --strict`

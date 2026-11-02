## 1. Persist and reattach

- [ ] 1.1 Persist task registry on change
- [ ] 1.2 On reload, reattach live PIDs (PID + start time match); mark others `lost`
- [ ] 1.3 Deliver completion/wake for reattached tasks; unknown exit status stays unknown

## 2. Verify

- [ ] 2.1 Tests: live task reattached; dead task `lost`; PID reuse rejected
- [ ] 2.2 Runtime check across `selfdev reload`
- [ ] 2.3 `openspec validate reattach-background-tasks-after-reload --strict`

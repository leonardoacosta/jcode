## 1. Persist and recover

- [ ] 1.1 Add lane/prompt/model/status to persisted member state (serde defaults)
- [ ] 1.2 On `load_runtime_state`, resume restorable workers; mark others `orphaned`
- [ ] 1.3 `retry` respawns orphan from persisted lane+prompt

## 2. Verify

- [ ] 2.1 Tests: reload resumes worker; missing session → orphaned; retry respawns; old state file loads
- [ ] 2.2 `openspec validate recover-swarm-workers-after-reload --strict`

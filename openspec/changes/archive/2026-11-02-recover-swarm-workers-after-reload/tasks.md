## 1. Persist and recover

- [x] 1.1 Add lane/prompt/model/status to persisted member state (serde defaults)
- [x] 1.2 On `load_runtime_state`, resume restorable workers; mark others `orphaned`
- [x] 1.3 `retry` respawns orphan from persisted lane+prompt

## 2. Verify

- [x] 2.1 Tests: reload resumes worker; missing session → orphaned; retry respawns; old state file loads (retry: `communicate_retry_respawns_orphaned_worker_from_spawn_spec`)
- [x] 2.2 `openspec validate recover-swarm-workers-after-reload --strict`

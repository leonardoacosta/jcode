# jcode-jevy-foreman-supervision — Tasks

Dependency: `add-jevsdk-evaluate-tool` must be complete before Phase 1 begins. The observer uses the evaluate tool for assessment calls.

Priority: CONSIDER — high effort, high risk. Phase 1 is passive observation only; Phase 2 (intervention) is deferred pending validation.

## 1. Passive observer

- [ ] 1.1 Implement worker observation state builder: Create `agent/foreman.rs` with `ObservationState` struct and `build_observation_state()`. Collects recent worker action history (tail 20), output tail (1000 chars), git status, bounded diff summary (2000 chars), changed files, elapsed time, attempt/failure counts. Returns None for inactive workers.
- [ ] 1.2 Implement 10-dimension assessment via Jev: `assess_worker()` sends 10 Noul questions (implementation_complete, tests_sufficient, requirements_satisfied, needs_verification, ready_to_finish, meaningful_progress, worker_stuck, work_off_track, agents_md_drift, needs_human) in a single request. Timeout 5s. Respects Jev cache. Verify all 10 probabilities in [0,1]; timeout returns None.
- [ ] 1.3 Implement background observer loop: `ForemanObserver` with configurable interval (default 30s). Each tick: enumerate active workers, build state, assess via Jev (parallel), log results at info level. Track last 5 cycles per worker. Phase 1 rule: log only, never intervene. Graceful shutdown on session end.
- [ ] 1.4 Register observer with swarm system: Add observer startup to session init, shutdown to session cleanup. Config: `foreman.enabled` (default false, opt-in), `foreman.interval_seconds` (30). Verify observer spawns when enabled, no interference with worker messaging.

## 2. Deterministic policy engine (deferred)

- [ ] 2.1 Implement policy engine: `PolicyEngine::decide_action()` returns PolicyAction (CONTINUE/STEER/STOP/START_VERIFIER/RETRY/FINISH/ESCALATE) based on assessment thresholds. All thresholds configurable via `foreman.thresholds.*`.
- [ ] 2.2 Implement intervention actions: STEER_WORKER sends guidance message via swarm messaging. STOP_WORKER requests worker stop. START_VERIFIER spawns independent verification agent. RETRY_WORKER restarts with same job (max retries). FINISH marks done. ESCALATE notifies coordinator. Log every intervention at warn.
- [ ] 2.3 Add intervention gating: Config gate `foreman.intervention_enabled` (default false). Per-action gates: `foreman.stop_enabled`, `foreman.steer_enabled`, etc. Minimum cycles before action: `foreman.min_cycles_before_action` (default 2). Require consistent assessment across min_cycles. Log suppressed interventions.

## Dependency graph

```
Task 1.1 → 1.2 → 1.3 → 1.4
                  ↘ 2.1 → 2.2 → 2.3
```

Phase 1 (passive observer) provides assessment foundation. Phase 2 (policy + intervention) depends on Phase 1 and deferred until assessment quality proven.
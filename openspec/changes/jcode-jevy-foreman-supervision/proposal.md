---
execution:
  version: 1
  depends_on: ["add-jevsdk-evaluate-tool"]
---

# jcode-jevy-foreman-supervision

## Summary

Add a Jev-driven background observer that periodically assesses active swarm workers. Jev answers 10 Noul questions about each worker's progress in a single request. A deterministic Rust policy decides: continue, steer, stop, verify, or escalate. Initially passive — the observer reports its assessments but does not intervene. The architecture is adapted from `thruwire/foreman` (353 stars, MIT).

## Motivation

Coding agents can get stuck, drift off-track, or complete work without adequate testing — and burn thousands of tokens before anyone notices. Foreman demonstrates that 10 fast, parallel Jev Noul questions can assess worker state with calibrated probabilities. A deterministic policy can then intervene before token waste. Jcode's swarm system has multiple active workers — a supervision layer would catch issues the harness misses.

**Priority:** CONSIDER — high effort, high risk. Jcode's swarm orchestration is more complex than Foreman's single-worker model. Start with passive observation.

## Actors

- **Foreman observer**: background task that periodically assesses swarm workers
- **Swarm workers**: active coding agents being supervised
- **TypeSafe Jev**: answers 10 Noul questions per observation cycle
- **Deterministic policy** (Rust code): decides what action to take based on Jev probabilities

## Scope

- Background observer task running at configurable intervals (default: 30s)
- 10 Noul questions per worker per observation cycle, all in one Jev request
- Observation state: original job, worker summaries, recent history tail, git status, bounded diff, AGENTS.md
- Deterministic policy with configurable thresholds
- **Initial phase**: passive only — assessments are logged/reported, no intervention
- **Phase 2** (if validated): policy can suggest steering, trigger independent verification

## Non-scope

- Not replacing Jcode's existing swarm orchestration
- Not intervening autonomously in phase 1 (passive observation only)
- Not modifying worker behavior — observer reads, doesn't write
- Not persisting supervision state beyond the session

## Behavior

### Assessment dimensions (10 Noul questions per cycle)

| Dimension | Question |
|-----------|----------|
| `implementation_complete` | Is the required implementation work complete? |
| `tests_sufficient` | Are tests adequate for this change? |
| `requirements_satisfied` | Does the repo satisfy the request as a whole? |
| `needs_verification` | Should an independent verification pass run? |
| `ready_to_finish` | Should the factory consider the job complete? |
| `meaningful_progress` | Is the current worker advancing the job? |
| `worker_stuck` | Is the worker looping, failing, or unable to advance? |
| `work_off_track` | Is work drifting from the original job? |
| `agents_md_drift` | Is worker behavior inconsistent with AGENTS.md? |
| `needs_human` | Does this need human judgment, credentials, or clarification? |

### Policy actions (when intervention is enabled)

| Action | Trigger |
|--------|---------|
| CONTINUE | No concerning scores |
| STEER_WORKER | Off-track > 0.8 but not yet stuck |
| STOP_WORKER | Stuck > 0.8 or drift > 0.8 after one steering attempt |
| START_VERIFIER | Needs verification > 0.65 and implementation > 0.75 |
| RETRY_WORKER | Worker stopped, retries remaining |
| FINISH | Ready to finish > 0.85 and requirements satisfied > 0.80 and tests sufficient > 0.75 |
| ESCALATE | Needs human > 0.80 |

### Observation format

```json
{
  "job": "Add rate limiting to the API with tests",
  "factory_status": "worker_active",
  "workers": [{
    "id": "worker-1",
    "type": "coding",
    "status": "running",
    "recent_history": ["Read auth.rs", "Edit middleware.rs", "Running tests..."],
    "output_tail": "FAILED: test_rate_limit_integration",
    "elapsed_seconds": 245
  }],
  "git_status": "M src/auth.rs\nM tests/auth_test.rs",
  "diff_summary": "Added rate limiting middleware (47 lines changed in 2 files)",
  "changed_files": ["src/auth.rs", "tests/auth_test.rs"],
  "agents_md": "Use Rust, add tests, follow existing patterns",
  "verification_results": null,
  "prior_assessment": null,
  "attempt_count": 1,
  "failure_count": 0,
  "elapsed_seconds": 245
}
```

## Dependencies

- **Depends on**: `add-jevsdk-evaluate-tool` — observer uses evaluate for Jev calls
- **Prior art**: `https://github.com/thruwire/foreman` (353 stars, MIT)
- **Priority**: CONSIDER — high effort, high risk

## Touched capabilities

| Capability | Effect |
|-----------|--------|
| `agent/turn_loops.rs` | Add background observer hook |
| New: `agent/foreman.rs` | Observer implementation, policy engine |
| `tool/evaluate.rs` | Used for assessment calls |

## Acceptance

1. Observer runs every 30s on active swarm workers
2. Assessment results are logged with all 10 dimension scores
3. Observer does not intervene in phase 1 — reports only
4. Worker stuck > 0.8 is correctly detected (looping, repeated failures)
5. Worker making progress shows low stuck and off_track scores
6. Jev unavailable → observer logs warning, skips cycle
7. Assessment state is compact and bounded (respects diff/context limits)
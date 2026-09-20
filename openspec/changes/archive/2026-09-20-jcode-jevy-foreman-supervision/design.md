## Context

Coding agents can get stuck, drift off-track, or complete work without adequate testing — and burn thousands of tokens before anyone notices. `thruwire/foreman` (353 stars, MIT) demonstrates 10 fast, parallel Jev Noul questions assessing worker state with calibrated probabilities. Jcode's swarm system runs multiple active workers — a supervision layer would catch issues the harness misses.

This is high-effort, high-risk. Phase 1 is passive observation only. Phase 2 (intervention) is deferred until Phase 1 proves assessment quality.

## Goals / Non-Goals

**Goals:**
- Background observer task at configurable intervals (default 30s) assessing active swarm workers
- 10 Noul questions per worker per observation cycle in a single Jev request
- Observation state: original job, worker summaries, recent history tail, git status, bounded diff, AGENTS.md
- **Phase 1**: passive only — assessments logged/reported, no intervention
- **Phase 2** (deferred): deterministic policy can suggest steering, trigger verification, escalate

**Non-Goals:**
- Not replacing Jcode's existing swarm orchestration
- Not intervening autonomously in phase 1
- Not modifying worker behavior (observer reads, doesn't write)
- Not persisting supervision state beyond the session

## Decisions

**Single-request assessment:** All 10 Noul questions per worker are sent in one Jev request. This avoids 10 round-trips per worker per cycle. Jev can handle the parallel questions efficiently; the additional token cost is negligible vs latency savings.

**10-dimension coverage:** The questions cover implementation completeness, test adequacy, requirements satisfaction, verification need, readiness, progress, stuckness, drift from job, AGENTS.md alignment, and human intervention need. This covers the main failure modes observed in coding agents.

**Deterministic policy in Rust:** Jev provides probabilistic assessments; a Rust policy engine makes deterministic decisions from calibrated thresholds. This avoids LLM-driven policy decisions (expensive, inconsistent) while keeping the assessment fast and cheap.

**Two-phase rollout:** Phase 1 (passive) validates that the 10 dimensions actually detect real issues before any intervention is enabled. Phase 2 adds intervention gated behind `foreman.intervention_enabled` with per-action gates and minimum-cycle consistency requirements.

**Observation bounds:** Diff summary capped at 2000 chars, recent history at 20 entries, output tail at 1000 chars. These bounds keep the Jev request within token limits while providing enough context for meaningful assessment.

## Risks / Trade-offs

- **Risk:** Assessment quality depends on the observation state being sufficiently informative. Mitigation: 10 dimensions cover multiple signals; Phase 1 validates before intervention.
- **Risk:** Observer adds latency to system (30s interval × parallel Jev calls). Mitigation: configurable interval; Phase 1 is passive so no user-visible impact.
- **Risk:** Observing workers may interfere with harness management. Mitigation: observer reads only, never writes to worker state.
- **Trade-off:** Two-phase rollout delays intervention benefits. Justification: premature intervention risks disrupting active work; validation first reduces risk.
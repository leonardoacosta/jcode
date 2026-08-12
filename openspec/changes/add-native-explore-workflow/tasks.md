## 1. Shared Workflow Preflight

- [ ] 1.1 Define repository-scoped OpenSpec, Beads, and telemetry status plus persisted one-time consent state.
- [ ] 1.2 Implement non-mutating detection and canonical initialization with consent, readiness recheck, decline persistence, and reset.
- [ ] 1.3 Test ready, missing, accepted, declined, failed, combined-prompt, repository-change, and reset paths.

## 2. Native Explore Skill

- [x] 2.1 Add native `explore` with intent, evidence, synthesis, decision-map, routing, and handoff phases.
  - Evidence: `skills/explore/SKILL.md` defines the native invocation contract, phase order, decision-map mode, and output contract, and explicitly excludes Codex/Claude-owned workflow activation.
- [ ] 2.2 Integrate todo, memory, session search, initiative, side panel, Recon query, and optional read-only swarm without a duplicate ledger.
- [ ] 2.3 Add the structured feature handoff with provenance and revision fields.

## 3. Telemetry and Efficient Execution

- [ ] 3.1 Emit best-effort workflow and efficiency telemetry.
- [x] 3.2 Encode the native-tool-first ladder, structured-output preference, direct execution, batching, timeouts, and caps.
  - Evidence: `skills/explore/SKILL.md` contains the native-tool-first efficiency ladder with typed-tool preference, batching, bounded shell fallback, timeout, and output-cap rules.
- [ ] 3.3 Test unnecessary shell fallback, unbounded output, repeated polling, and telemetry-dependent behavior.

## 4. Acceptance

- [ ] 4.1 Exercise `/explore` through the installed public REPL and prove no Codex/Claude activation.
- [ ] 4.2 Exercise local-only, Recon-backed, swarm-assisted, degraded, and durable decision-map workflows.
- [ ] 4.3 Verify handoff reuse and stale-revision rejection.
- [ ] 4.4 Run focused tests and strict OpenSpec validation.
  - Evidence: `openspec validate add-native-explore-workflow --strict` passed on 2026-08-12. Public REPL acceptance and degraded workflow exercises remain open, so acceptance is not closed.

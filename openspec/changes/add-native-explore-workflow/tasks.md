## 1. Shared Workflow Preflight

- [ ] 1.1 Define repository-scoped OpenSpec, Beads, and telemetry status plus persisted one-time consent state.
- [ ] 1.2 Implement non-mutating detection and canonical initialization with consent, readiness recheck, decline persistence, and reset.
- [ ] 1.3 Test ready, missing, accepted, declined, failed, combined-prompt, repository-change, and reset paths.

## 2. Native Explore Skill

- [x] 2.1 Add native `explore` with intent, evidence, synthesis, decision-map, routing, and handoff phases.
  - Evidence: `skills/explore/SKILL.md` defines the native invocation contract, phase order, decision-map mode, and output contract, and explicitly excludes Codex/Claude-owned workflow activation.
- [x] 2.2 Integrate todo, memory, session search, initiative, side panel, Recon query, and optional read-only swarm without a duplicate ledger.
  - Evidence: `skills/explore/SKILL.md` now defines integration rules for `todo`, `memory`, `session_search`, `initiative`, `side_panel`, Recon, optional read-only `swarm`, and explicitly forbids a second ledger.
- [x] 2.3 Add the structured feature handoff with provenance and revision fields.
  - Evidence: `skills/explore/SKILL.md` now includes a YAML feature handoff schema with destination, topic, revision, freshness check, provenance, evidence, assumptions, alternatives, scope, decisions, surfaces, dependencies, conflicts, edge cases, done means, limitations, and recommended action. It also requires `/feature` to reject or refresh stale handoffs.

## 3. Telemetry and Efficient Execution

- [ ] 3.1 Emit best-effort workflow and efficiency telemetry.
- [x] 3.2 Encode the native-tool-first ladder, structured-output preference, direct execution, batching, timeouts, and caps.
  - Evidence: `skills/explore/SKILL.md` contains the native-tool-first efficiency ladder with typed-tool preference, batching, bounded shell fallback, timeout, and output-cap rules.
- [ ] 3.3 Test unnecessary shell fallback, unbounded output, repeated polling, and telemetry-dependent behavior.

## 4. Acceptance

- [x] 4.1 Exercise `/explore` through the installed public REPL and prove no Codex/Claude activation.
  - Evidence: `./target/debug/jcode run --no-update --tool-profile none '/explore local-only degraded-path contract probe. Do not use model routing. Return the skill name and whether model routing was used.'` returned `Skill: /explore` and `Model routing used: No.` on 2026-08-12. The shell wrapper reported the known harness/zsh `read-only variable: status` issue after output, but the public skill resolution evidence was produced.
- [ ] 4.2 Exercise local-only, Recon-backed, swarm-assisted, degraded, and durable decision-map workflows.
  - Evidence: Locally possible contract coverage was added for local-only, Recon-backed, swarm-assisted, degraded, and durable decision-map routes in `skills/explore/SKILL.md`. Full live workflow exercise remains open because it requires real slash-run exploration sessions and optional integrations outside this scoped change.
- [x] 4.3 Verify handoff reuse and stale-revision rejection.
  - Evidence: `skills/explore/SKILL.md` now requires revision metadata and states that native `/feature` must reject or refresh a handoff when repository root, confirmed revision, critical evidence freshness, or selected destination no longer matches the current request.
- [x] 4.4 Run focused tests and strict OpenSpec validation.
  - Evidence: `openspec validate add-native-explore-workflow --strict` passed on 2026-08-12. `cargo test -p jcode-base skill:: -- --nocapture` passed on 2026-08-12, covering slash invocation parsing/resolution including colon-bearing skills, multi-word registered names, unknown fallback, and file-drop rejection. The shell wrapper again reported the known harness/zsh `read-only variable: status` issue only after both commands printed explicit zero exits.

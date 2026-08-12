## 1. Shared Scheduling Core

- [ ] 1.1 Define and validate versioned schedule and wave-plan contracts with revision, provenance, dependency, conflict, frontier, lineage, and extension fields.
- [ ] 1.2 Implement authoritative feature and queue resolution with stale, ambiguous, unsupported, cyclic, and legacy-input rejection.
- [ ] 1.3 Implement path, claim, repository, workspace, external-system, deployment, and mutable-resource conflict analysis.
- [ ] 1.4 Reuse shared OpenSpec/Beads consent and telemetry preflight without duplicate prompts.

## 2. Native Apply

- [x] 2.1 Add native `apply` slash skill and preserve the selected feature argument.
  - Evidence: `skills/apply/SKILL.md` defines the native `/apply` invocation contract and requires preserving the selected feature argument exactly.
- [ ] 2.2 Implement single-feature preflight, execution-path selection, bounded task ownership, and verification recipes.
- [ ] 2.3 Implement complete feature review, persistence, settlement, archive, and truthful closeout adapters for the active repository authority.

## 3. Native Apply All

- [x] 3.1 Add native `apply:all` slash skill and require an explicit selected queue.
  - Evidence: `skills/apply:all/SKILL.md` defines the native `/apply:all` contract and explicit queue requirement. `crates/jcode-base/src/skill.rs` now parses colon-bearing skill invocations, with `cargo test -p jcode-base skill::tests::parse_invocation -- --nocapture` passing on 2026-08-12.
- [ ] 3.2 Construct dependency- and conflict-safe waves and dispatch only proven-independent features concurrently.
- [ ] 3.3 Pause transitive dependents after failure, continue unrelated valid branches, and recompute the ready frontier.
- [ ] 3.4 Implement queue-level integration gates and partial outcome reporting.

## 4. Orchestration and Runtime Authority

- [ ] 4.1 Implement observable risk and topology scoring with direct, reviewed, light-swarm, deep-DAG, and durable-initiative selection.
- [ ] 4.2 Implement review tiers with cross-provider review starting at high risk and human approval for critical risk.
- [ ] 4.3 Implement Jcode durable-authority and Orca runtime-authority identity, capability, receipt, and cleanup contracts.
- [ ] 4.4 Implement declared Jcode-native fallback and fail-closed behavior for unmet Orca-dependent capabilities.

## 5. Recovery, Evidence, and Projection

- [ ] 5.1 Implement interruption reconstruction from repository authority, frozen schedules, Git, initiatives, Orca receipts, and fresh verification.
- [ ] 5.2 Implement attempt-scoped idempotency, retry lineage, stale-input invalidation, and duplicate-mutation prevention.
- [ ] 5.3 Bind review and verification receipts to unchanged requirements, diffs, and artifacts.
- [x] 5.4 Add bounded side-pane execution state, authorized actions, durable evidence links, and compact terminal events.
  - Evidence: `skills/apply/SKILL.md` and `skills/apply:all/SKILL.md` require bounded `side_panel` projection, authorized actions, compact terminal output, and durable evidence links.
- [x] 5.5 Emit best-effort telemetry and enforce native-tool-first, structured-output, timeout, batching, and output-cap policies.
  - Evidence: `skills/apply/SKILL.md` and `skills/apply:all/SKILL.md` require best-effort workflow telemetry and typed-tool-first, structured-output, batching, timeout, and source-side cap policies.

## 6. Acceptance

- [ ] 6.1 Exercise native `/apply` and `/apply:all` activation through installed Jcode public interfaces.
  - Evidence: `./target/debug/jcode run --no-update --socket /run/user/1000/jcode-skill-acceptance-4.sock --tool-profile none '/apply add-native-explore-workflow'` and `./target/debug/jcode run --no-update --socket /run/user/1000/jcode-skill-acceptance.sock --tool-profile none '/apply:all add-native-explore-workflow add-native-feature-workflow'` both resolved through public run interface on 2026-08-12 and reached expected degraded no-tool paths while preserving selected arguments. Installed REPL/TUI activation remains open.
- [ ] 6.2 Exercise explicit queue selection, dependency order, conflicts, cycles, stale schedules, invalid inputs, and no implicit queue broadening.
- [ ] 6.3 Exercise feature failure with dependent pauses, independent continuation, retry lineage, and partial settlement.
- [ ] 6.4 Exercise all risk tiers, same-provider normal review, high-risk cross-provider review, critical approval, and review invalidation.
- [ ] 6.5 Exercise Orca-supervised, Jcode-native fallback, missing-capability, interruption, resume, cancellation, and cleanup paths.
- [ ] 6.6 Exercise telemetry degradation and bounded side-pane and terminal behavior under large queues.
- [ ] 6.7 Run focused repository tests, contract drift checks, public workflow acceptance, and strict OpenSpec validation.
  - Evidence: `cargo test -p jcode-base skill:: -- --nocapture` and `openspec validate add-native-apply-workflows --strict` passed on 2026-08-12. The focused skill tests cover slash invocation parsing/resolution including colon-bearing `/apply:all`, multi-word registered names, unknown fallback, and file-drop rejection. Public workflow acceptance, contract drift checks, Mac/Orca/external capability gates, and full-suite coverage remain open.

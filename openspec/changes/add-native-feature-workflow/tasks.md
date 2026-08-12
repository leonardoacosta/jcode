## 1. Intake and Refinement

- [x] 1.1 Add native `feature` slash skill and direct invocation intake.
  - Evidence: `skills/feature/SKILL.md` defines the native `/feature` invocation and intake contract, preserves trailing command text, and explicitly excludes `codex-feature`/Claude-owned workflow activation.
- [ ] 1.2 Consume and freshness-check the native explore handoff with selective refresh.
- [ ] 1.3 Implement uncertainty classification and focused user-decision blocking.

## 2. Surface Inventory and Authority

- [ ] 2.1 Inventory consumers, touched paths, dependencies, conflicts, compatibility, and material edge cases.
- [ ] 2.2 Implement singular authority selection for repository-declared systems, OpenSpec, approved alternatives, and degraded initiatives.
- [x] 2.3 Reuse shared one-time integration consent without duplicate prompts or state.
  - Evidence: `skills/feature/SKILL.md` requires running the shared workflow preflight from `explore` instead of declaring a separate consent prompt or state store.

## 3. Authoring and Traceability

- [x] 3.1 Author requirements, scenarios, exclusions, decisions, done means, verification, and expected results through the selected authority.
  - Evidence: `skills/feature/SKILL.md` authoring contract lists requirements, user-observable scenarios, scope, exclusions, decisions, rejected alternatives, assumptions, dependencies, conflicts, touched paths, edge cases, done means, and expected results.
- [x] 3.2 Generate independently executable implementation tasks with complete requirement traceability.
  - Evidence: `skills/feature/SKILL.md` requires independently executable tasks that map to requirements and requirement-specific verification commands with expected outcomes.
- [x] 3.3 Produce the explicit implementation handoff and external-gate report.
  - Evidence: `skills/feature/SKILL.md` output contract requires external gates and one explicit next implementation action for native `/apply`.

## 4. Review, Telemetry, and Efficiency

- [ ] 4.1 Add deterministic authority validation and digest-bound independent semantic review with mutation invalidation.
- [ ] 4.2 Emit best-effort workflow, authority, review, degradation, efficiency, and completion telemetry.
- [x] 4.3 Enforce native-tool-first, structured-output, bounded batching, timeouts, and output caps.
  - Evidence: `skills/feature/SKILL.md` review section requires native typed tools, structured output, batching, timeouts, and source-side caps before shell fallback.

## 5. Acceptance

- [ ] 5.1 Exercise direct `/feature` and explore-fed `/feature` through public Jcode interfaces.
- [ ] 5.2 Exercise OpenSpec, alternate-authority, degraded, setup-declined, conflict, and stale-handoff paths.
- [ ] 5.3 Prove artifact mutation invalidates review and validation evidence.
- [ ] 5.4 Run focused tests and strict OpenSpec validation.
  - Evidence: `openspec validate add-native-feature-workflow --strict` passed on 2026-08-12. Public interface, degraded, conflict, stale-handoff, and mutation-invalidation acceptance paths remain open.

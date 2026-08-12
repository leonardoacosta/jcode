## 1. Intake and Refinement

- [x] 1.1 Add native `feature` slash skill and direct invocation intake.
  - Evidence: `skills/feature/SKILL.md` defines the native `/feature` invocation and intake contract, preserves trailing command text, and explicitly excludes `codex-feature`/Claude-owned workflow activation.
- [x] 1.2 Consume and freshness-check the native explore handoff with selective refresh.
  - Evidence: `skills/feature/SKILL.md` now requires repository identity, revision, path, evidence, timestamp, dependency, and assumption freshness checks, selective refresh of stale/missing handoff fields, invalidated-assumption reporting, and explore-fed public activation acceptance. Public degraded probe `./target/debug/jcode run --no-update --tool-profile none --json 'Native /explore handoff for /feature: ...'` completed on 2026-08-12 and preserved the handoff fields while reporting no-tool degradation.
- [x] 1.3 Implement uncertainty classification and focused user-decision blocking.
  - Evidence: `skills/feature/SKILL.md` classifies uncertainties into discoverable facts, safe reversible defaults, user-only judgments, and later evidence-dependent actions; it blocks authoring on unresolved critical user-only judgments and now requires recording each uncertainty disposition, evidence/default, rejected alternatives, blocking question, or later gate in the selected authority.

## 2. Surface Inventory and Authority

- [x] 2.1 Inventory consumers, touched paths, dependencies, conflicts, compatibility, and material edge cases.
  - Evidence: `skills/feature/SKILL.md` requires inventorying callers, routes, commands, APIs, UI components, schemas, data flows, migrations, integrations, operations, tests, docs, permissions, deployment surfaces, active work, touched paths, dependencies, conflicts, compatibility behavior, edge cases, exclusions, and verification mapping. It now explicitly requires active local-change and planning-work conflict checks and forbids touching unrelated lanes or model-routing files.
- [x] 2.2 Implement singular authority selection for repository-declared systems, OpenSpec, approved alternatives, and degraded initiatives.
  - Evidence: `skills/feature/SKILL.md` now requires recording exactly one selected authority, rejected authorities, and selection reason. It covers repository-declared authority, OpenSpec with strict validation, approved alternate authority with its validator/consistency check, and degraded durable initiative plus one Markdown design artifact after setup decline/failure without a shadow ledger.
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

- [x] 4.1 Add deterministic authority validation and digest-bound independent semantic review with mutation invalidation.
  - Evidence: `skills/feature/SKILL.md` now requires stable artifact digests before review, semantic review bound to those digests, deterministic validation evidence with exact command/result/timestamp/digests, checklist coverage for traceability and handoff quality, and rerunning affected evidence after any authoritative artifact mutation. Current authoritative digests on 2026-08-12: `skills/feature/SKILL.md` `28c53648ffdf3b77e13899fa15526e95e05f24fe708643c0577b67cd88a96d17`; spec `21bada169371e6bbeeec4352130004c346a3cfc10bfa4e53d30ef55d74fd7458`; tasks `c1aa35f6a10c624709a47b05ebd4e18b52e2d7f7f220e27dbcea856ddd5af326` before this evidence update.
- [x] 4.2 Emit best-effort workflow, authority, review, degradation, efficiency, and completion telemetry.
  - Evidence: `skills/feature/SKILL.md` requires telemetry checks every invocation plus best-effort intake, authority, setup, phase, review, efficiency, degradation, and completion observations when supported. Telemetry failure is explicitly non-blocking and cannot weaken review.
- [x] 4.3 Enforce native-tool-first, structured-output, bounded batching, timeouts, and output caps.
  - Evidence: `skills/feature/SKILL.md` review section requires native typed tools, structured output, batching, timeouts, and source-side caps before shell fallback.

## 5. Acceptance

- [ ] 5.1 Exercise direct `/feature` and explore-fed `/feature` through public Jcode interfaces.
  - Evidence: direct degraded public probe `./target/debug/jcode run --no-update --tool-profile none --json '/feature direct public invocation acceptance probe: preserve this tail and report degraded if no tools'` completed on 2026-08-12, resolved through the public run interface, preserved the tail text, and reported no-tool degradation. Explore-fed degraded public probe `./target/debug/jcode run --no-update --tool-profile none --json 'Native /explore handoff for /feature: destination=/feature; ...'` completed on 2026-08-12, preserved handoff fields, invoked `/feature`, and reported no-tool degradation. Direct repository-backed execution and true prior-session explore-fed handoff acceptance remain blocked by the local public no-tool probe profile and are still open.
- [ ] 5.2 Exercise OpenSpec, alternate-authority, degraded, setup-declined, conflict, and stale-handoff paths.
  - Evidence: local contract coverage for OpenSpec, alternate-authority, degraded, setup-declined, conflict, and stale-handoff paths is present in `skills/feature/SKILL.md`. Degraded public direct and explore-fed paths were exercised on 2026-08-12. Runtime exercise of setup-declined, alternate-authority, conflict, and stale-handoff repository-backed paths remains open because those scenarios require seeded repository states or user consent fixtures not locally available in this lane.
- [ ] 5.3 Prove artifact mutation invalidates review and validation evidence.
  - Evidence: `skills/feature/SKILL.md` now contains the mutation-invalidation contract and digest-bound evidence requirements. A destructive mutation/rerun proof against generated user artifacts remains open because no safe disposable authoritative feature artifact fixture exists in this lane.
- [ ] 5.4 Run focused tests and strict OpenSpec validation.
  - Evidence: `openspec validate add-native-feature-workflow --strict` passed on 2026-08-12 after the local contract updates. `cargo test -p jcode-base skill:: -- --nocapture` passed on 2026-08-12, covering slash invocation parsing/resolution used by public feature activation. Public degraded direct/explore-fed execution passed. Repository-backed direct/explore-fed, setup-declined, alternate-authority, conflict, stale-handoff, and mutation-invalidation fixture acceptance paths remain open.

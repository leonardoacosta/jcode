## Context

Claude's feature flow contributes discovery fast paths, ambiguity categories, touched-path conflict analysis, incremental review, and planning-system synchronization. Codex contributes decision classification, surface inventory, edge-case completeness, explicit defaults, freshness checks, independent review, and fail-closed readiness. Native Jcode supplies todo, initiative, side panel, swarm review, repository tools, memory/session context, and telemetry.

The workflow should adapt to the repository rather than mandate OpenSpec or Beads everywhere. It must still offer one-time initialization when those integrations are absent and preserve exactly one durable authority for the feature contract.

## Goals / Non-Goals

**Goals:**

- Create a native `/feature` skill that authors an implementation-ready contract.
- Reuse verified `/explore` context without rediscovery.
- Resolve uncertainty and inventory all material surfaces before authoring.
- Select one repository authority and validate its artifacts.
- Make telemetry, shell efficiency, and independent review observable requirements.

**Non-Goals:**

- Implement application code.
- Require a single planning product in every repository.
- Import Codex-specific verifiers, Claude preprocessors, telemetry scripts, or Beads/OpenSpec policy verbatim.
- Claim readiness from inspection alone.

## Decisions

### 1. Ship a native skill named `feature`

The skill registry provides the slash command directly. Rejected: aliasing or renaming `codex-feature`, because its OpenSpec and verifier ownership is not repository-neutral Jcode behavior.

### 2. Reuse the shared environment preflight

The companion explore change owns repository integration and telemetry detection. `/feature` invokes the same preflight and saved consent state. It never creates a second setup preference or asks again after a recorded decline.

### 3. Support direct and handoff-fed intake

A native explore handoff is freshness-checked using repository identity, revisions, paths, evidence IDs, and timestamps. Valid fields seed refinement. Stale or missing fields are selectively refreshed. Direct invocation performs equivalent intake rather than requiring prior `/explore`.

### 4. Resolve four uncertainty classes

Discoverable facts are investigated; safe reversible defaults are recorded with rejected alternatives; user-only judgments produce one focused question at a turn boundary; later evidence-dependent actions become terminal gates or separate dependencies. Critical unresolved judgments block authoring.

### 5. Inventory surfaces and material cases

Before scope selection, inventory callers, routes, components, schemas, integrations, operational paths, tests, documentation, compatibility surfaces, and active changes. Each material case becomes a requirement scenario or an explicit exclusion with defined behavior and verification.

### 6. Choose exactly one durable authority

Selection order is: repository-declared authority; initialized OpenSpec; explicitly approved existing issue/planning system; durable Jcode initiative with attached Markdown design in degraded mode. Todo remains session-local. Side panel remains a view. The workflow never mirrors the full task set into multiple systems.

### 7. Separate semantic and deterministic validation

Deterministic validation uses the selected authority's non-interactive validator. Semantic review checks traceability, consistency, edge cases, executability, freshness, dependencies, and scope. Independent swarm review may supply evidence, but the coordinator owns the verdict. Any artifact mutation invalidates prior review.

### 8. Detect telemetry and optimize execution

Emit best-effort start, intake source, phase outcome, authority, setup status, review, shell-efficiency, and completion events. Prefer typed tools, native searches, structured CLI output, and batched direct execution. Record degradation rather than routing based on telemetry availability.

### 9. Produce an explicit implementation handoff

Report authority and artifact references, requirements, tasks, touched paths, dependencies, preconditions, validation commands and expected results, review evidence, unresolved external gates, and the single next implementation action.

## Risks / Trade-offs

- **[Risk] Repository-neutral routing becomes ambiguous** → require one explicit authority selection and report why.
- **[Risk] Degraded initiative becomes a shadow tracker** → use it only when no accepted repository authority exists and never mirror tasks elsewhere.
- **[Risk] Handoff evidence is stale** → freshness-check each referenced path/revision and selectively refresh.
- **[Risk] Review is performative** → bind semantic evidence to unchanged artifact digests and rerun after mutation.
- **[Risk] Shell-heavy discovery wastes tokens** → enforce native-tool-first and structured output contracts.
- **[Risk] Telemetry absence hides behavior** → report capability status in the final evidence matrix.

## Migration Plan

1. Land or depend on the shared preflight and explore handoff schema.
2. Add native `feature` intake and uncertainty classification.
3. Add authority selection and artifact adapters.
4. Add semantic review, deterministic validation, telemetry, and efficiency evidence.
5. Add direct and explore-fed public acceptance workflows.
6. Roll back by disabling the skill; repository artifacts remain under their existing authority.

## Open Questions

None.

---
name: feature
description: Native Jcode feature-authoring workflow. Use to turn a clarified outcome or /explore handoff into one decision-complete, repository-authoritative implementation contract with traceable requirements, tasks, validation, review, and an explicit /apply handoff.
---

# Native Feature

Treat `/feature` as Jcode's native proposal-authoring workflow. It refines a requested outcome, selects exactly one durable authority, authors implementation-ready artifacts, validates them, and hands off one explicit next implementation action. It does not implement application code and does not activate `codex-feature` or Claude-owned workflows.

## Invocation and intake

- Preserve all trailing slash-command text as the feature description.
- Run the shared workflow preflight from `explore`: repository identity, OpenSpec readiness, Beads readiness, telemetry detection, one-time setup consent, degraded-mode reporting, and reset semantics.
- Accept either direct intake or a native `/explore` handoff.
- Freshness-check handoff repository identity, revision stamps, referenced paths, evidence IDs, timestamps, dependencies, and assumptions.
- Reuse current handoff fields and selectively refresh only stale or missing evidence. Report invalidated assumptions.

### Public activation checks

Use the public Jcode slash-skill surface when validating this workflow. A direct `/feature ...` probe must resolve the `feature` skill, keep the command tail as the feature description, and must not route to `codex-feature`, `codex_feature`, `Claude`, or compatibility commands. An explore-fed probe must pass the structured native `/explore` handoff through session context and then resolve `/feature` without requiring the user to restate settled discovery.

If the public runtime lacks repository tools, network, model credentials, or setup consent, treat the run as degraded acceptance only when it still proves slash resolution, native skill loading, truthful limitation reporting, and no foreign workflow activation. Do not count a degraded probe as repository-backed authoring acceptance.

## Refinement gate

Before authoring authoritative artifacts, classify every material uncertainty as one of:

- Discoverable fact: investigate and cite evidence.
- Safe reversible default: choose a default, record rejected alternatives, and expose it for correction.
- User-only judgment: ask one focused turn-boundary question and block authoring until answered.
- Later evidence-dependent action: represent as a terminal gate, prerequisite, or follow-on feature.

Do not author while critical user-only judgments remain unresolved.

Record the disposition for every material uncertainty in the selected authority before writing final requirements. The record must name the class, evidence or default chosen, rejected alternatives when a safe default is used, and the blocking question or later gate when no local answer is valid.

## Surface and case inventory

Inventory affected and compatible surfaces before selecting scope:

- Callers, routes, commands, APIs, UI components, schemas, data flows, migrations, integrations, operations, tests, docs, permissions, deployment surfaces, and active work.
- Touched paths, dependencies, conflicts, existing consumers, compatibility behavior, material edge cases, and explicit exclusions.
- Every material case must become a requirement scenario or an exclusion with defined behavior and verification.

Also check active local changes and known open planning work for conflicts before authoring. If another lane owns a touched surface, either exclude that surface, declare a dependency, or block with the conflict. Do not modify unrelated lanes, model-routing files, or authority artifacts selected by another workflow.

## Authority selection

Choose exactly one durable authority:

1. Repository-declared authority, if present.
2. OpenSpec, when initialized and not superseded by repository policy.
3. An explicitly approved existing issue or planning system.
4. A durable Jcode `initiative` plus attached design artifact only in degraded mode after setup was declined or failed.

`todo` remains session-local and `side_panel` remains a view. Never mirror the full contract into multiple systems.

Authority selection is singular and auditable. Record the selected authority, rejected authorities, and reason. When OpenSpec is selected, create or update exactly one change and run strict validation. When an approved alternate authority is selected, name the approving signal and use that system's non-interactive validator or consistency check. When no authority exists and setup is declined or failed, use one durable Jcode `initiative` plus one attached Markdown design artifact, label the result degraded, and do not create a shadow OpenSpec or task ledger.

## Authoring contract

The selected authority must capture:

- Requirements with user-observable scenarios.
- Scope, exclusions, decisions, rejected alternatives, assumptions, dependencies, conflicts, touched paths, edge cases, done means, and expected results.
- Implementation tasks that are independently executable and map to one or more requirements.
- Requirement-specific verification commands or checks with expected outcomes.
- External gates and preconditions that cannot be resolved during authoring.

## Review, validation, and telemetry

- Run the authority's deterministic non-interactive validator when available, such as strict OpenSpec validation for OpenSpec changes.
- Complete independent semantic review for traceability, consistency, edge cases, executability, freshness, dependencies, and scope.
- Bind validation and review evidence to unchanged artifact bytes or digests. Any artifact mutation invalidates affected evidence and requires rerun.
- Check telemetry every invocation and emit best-effort intake, authority, setup, phase, review, efficiency, degradation, and completion observations when supported. Telemetry failure never weakens review.
- Prefer native typed tools, structured output, batching, timeouts, and source-side caps. Use shell only when no typed surface exists.

### Digest-bound evidence

Before review, compute or record stable digests for all authoritative artifacts. Semantic review evidence must include those digests and a checklist covering requirement-to-scenario, requirement-to-task, requirement-to-check, edge cases, dependencies, conflicts, exclusions, degraded limits, and implementation handoff. Deterministic validation evidence must include the exact command or native validator, result, timestamp, and artifact digests. If any authoritative artifact changes after either evidence record is captured, mark the affected evidence stale and rerun it before reporting readiness.

### Degraded path contract

In degraded mode, still produce a complete local contract when possible: requirements, scenarios, decisions, exclusions, tasks, checks, expected results, dependencies, gates, and one next action. Prefix readiness with the degraded authority limitation and list which repository integrations were unavailable, declined, or failed. Degraded mode may prepare implementation guidance, but it must not claim repository-authoritative readiness unless the selected degraded authority is explicitly accepted for this repository.

## Output contract

End only after refinement, authoring, validation, and review succeed, or report the concrete blocker. Include:

- Selected authority and artifact references.
- Requirements, tasks, checks, expected results, touched paths, dependencies, gates, review evidence, and validation evidence.
- Handoff provenance and freshness status when an explore handoff was used.
- One explicit next implementation action for native `/apply`.

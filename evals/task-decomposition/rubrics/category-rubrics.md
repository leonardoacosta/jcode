# Category Rubrics

Use these rubrics alongside deterministic support evidence from `score-artifacts` and `extract-evidence`. The structured rubric JSON uses five planning-quality dimensions scored 1 to 5:

- Fidelity
- Scope lock
- Blast-radius discovery
- Risk and dependency ordering
- Verification executability

The historical OpenSpec change is reference evidence. It is not a reproduction target, and exact artifact shape or token overlap should not dominate the semantic score.

## Shared axes

- Fidelity: the plan preserves the user request, domain intent, and fixture intent contract.
- Scope lock: the plan includes essential work while avoiding unrelated expansion and accidental narrowing.
- Blast-radius discovery: the plan names affected routes, packages, data, auth, config, tests, docs, and operational surfaces deeply enough for the fixture.
- Risk and dependency ordering: tasks are sequenced so prerequisites, compatibility windows, migration risks, rollback, security/privacy concerns, and ownership boundaries are explicit.
- Verification executability: acceptance checks match observable behavior, main workflows, edge cases, integration boundaries, and likely failure modes.

## Category emphasis

### Free design/product choices

- Identifies the product decision space and trade-offs.
- Preserves hard constraints while allowing multiple viable designs.
- Defines user-facing acceptance criteria rather than only implementation steps.
- Names the surfaces where design choices will propagate.

### Business/domain logic

- Models actors, states, permissions, and domain invariants.
- Separates business rules from UI or infrastructure mechanics.
- Includes edge cases, conflict states, and lifecycle transitions.

### Infra/platform/config

- Names runtime boundaries, environment ownership, startup failure modes, and rollback.
- Avoids secret exposure and avoids local-only assumptions.
- Includes deploy, packaging, and bootstrap verification where relevant.

### Test strategy/E2E remediation

- Separates test infrastructure, fixtures, selectors, data ownership, and app defects.
- Defines flake controls, cleanup, reporting, and failure triage.
- Avoids hiding failures through filtering unless explicitly justified.

### Data/schema/migration

- Names source of truth, migration sequence, compatibility windows, and integrity checks.
- Handles backfills, dual reads/writes, index rebuilds, or cutover gates where applicable.

### Auth/security/permissions

- Identifies trust boundaries, credentials, scopes, session state, and abuse cases.
- Defines fail-closed behavior and audit evidence.

### Observability/telemetry

- Separates events, logs, traces, metrics, dashboards, and drains.
- Requires receipt proof before retiring existing telemetry paths.

### Developer tooling/agent integration

- Defines tool contracts, compatibility, rollback, provenance, and operator workflow.
- Avoids coupling to one local machine state unless explicitly scoped.

### Refactor/dead-code/entropy cleanup

- Distinguishes proven dead code from public API or framework entry points.
- Includes baseline/regression checks and exception discipline.

### UI/UX polish

- Captures the intended perceptual change, responsive states, accessibility, and visual regression checks.
- Keeps subjective polish bounded to named surfaces.

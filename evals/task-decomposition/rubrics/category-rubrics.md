# Category Rubrics

Use these rubrics alongside the deterministic `score-artifacts` output. Each axis is scored 0 to 2.

## Shared axes

- Artifact completeness: proposal, design, tasks, and delta specs exist and are internally consistent.
- Scope fit: the plan includes the essential work without unrelated expansion.
- Dependency ordering: tasks are sequenced so prerequisites are explicit.
- Verification quality: acceptance checks match observable behavior and likely failure modes.
- Risk handling: blockers, security/privacy, migration, and rollback risks are named where relevant.

## Category emphasis

### Free design/product choices

- Identifies the product decision space and trade-offs.
- Preserves hard constraints while allowing multiple viable designs.
- Defines user-facing acceptance criteria rather than only implementation steps.

### Business/domain logic

- Models actors, states, permissions, and domain invariants.
- Separates business rules from UI or infrastructure mechanics.
- Includes edge cases and conflict states.

### Infra/platform/config

- Names runtime boundaries, environment ownership, startup failure modes, and rollback.
- Avoids secret exposure and avoids local-only assumptions.
- Includes deploy or packaging verification where relevant.

### Test strategy/E2E remediation

- Separates test infrastructure, fixtures, selectors, data ownership, and app defects.
- Defines flake controls, cleanup, reporting, and failure triage.
- Avoids hiding failures through filtering unless explicitly justified.

### Data/schema/migration

- Names source of truth, migration sequence, compatibility windows, and integrity checks.
- Handles backfills, dual reads/writes, or cutover gates where applicable.

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

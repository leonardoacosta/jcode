# Advisory Ledger Format

> Demoted from `rules/CORE.md` by `prune-core-stale-and-rescope-narrow` (2026-07-25).
> Governs `advisor-plans/` — a surface `CLAUDE.md` § 2 already describes as historical,
> never the primary path. Consulted when authoring or auditing an advisory ledger, not per turn.

### Advisory Ledger Format

> Relocated from `openspec/AGENTS.md` § "Advisory Ledger Canon" ahead of the OpenSpec CLI 1.4.1
> migration (`openspec-migration-execute`, cc-vop7r). The routing-level summary already exists
> in global `CLAUDE.md` § 2 "Advisory spine", but the literal ledger schema and status-string
> vocabulary below do not appear anywhere else.

Any advisory-plans directory MUST carry an index ledger — one row per plan, this schema:

| Plan | Title | Priority | Effort | Risk | Depends on | Status |

Status vocabulary (exact strings): `TODO | IN PROGRESS | DONE (ref) | BLOCKED (reason) |
REJECTED (rationale)`. A `DONE` row MUST cite a commit SHA or spec/change slug — a bare `DONE`
with no ref is not a valid close (`advisor-plans/README.md` is the working exemplar).

`advisor-plans/` is cc's standing advisory home — the ledger above lives there. `plans/` is
plan-mode scratch (auto-generated names, no ledger, swept by `scripts/bin/plans-sweep`) and is
explicitly **not** a second execution ledger: genuine cc advisory content found there gets
promoted into `advisor-plans/` with a ledger row, it is never left to accumulate its own
parallel tracking scheme in place.

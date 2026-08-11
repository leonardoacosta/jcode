
# Record-Keeping Doctrine (R1-R6)

The citable home for "how do we record evaluations so they are both historical and shareable."
Distilled from `docs/research/evaluation-intelligence-session-2026-07-21.md` § 5 — that doc is
the dated research record explaining *why* these six patterns were named; this file is the
current-state doctrine every profile's memory-home section (and any future one) cites instead of
restating. Do not fork a parallel copy of any rule below — cite this file by rule ID (R1-R6).

| # | Rule | Binding statement | Repo precedent |
|---|------|--------------------|-----------------|
| R1 | Append-only record + generated view | The source of record is append-only (JSONL, `history[]`, or a database table); anything human-facing is regenerable from it and never hand-edited. | repository-local decision log plus generated index |
| R2 | ADRs supersede, never edit | A structural decision gets one numbered record — context/options/outcome/consequences — filed under `docs/adr/`. A later decision supersedes an earlier one with a new record; it never edits or deletes the old one. The decision log IS the evolution timeline. | `docs/adr/0001-user-tag-is-hitl.md` |
| R3 | Progressive disclosure + staleness re-verify | An index line (target, verdict, date) stays cheap to scan and links to the full record; readers pay for depth only when they descend. A record older than the domain's staleness horizon (default >30 days) is re-verified before its verdict is reused, never trusted as-is. | MEMORY.md index discipline + `rules/CORE.md`'s >30-day re-verify rule |
| R4 | Current-state vs evolution split + tombstones | Present truth lives in one place (`openspec/specs/`, a profile, CLAUDE.md); how-it-got-here lives in another (`changes/archive/`, ADRs, ledgers). A rejected or superseded approach gets a tombstone row recording the negative result, so it isn't relitigated from scratch. | Failure-mode tombstones (global `CLAUDE.md` § 3) + `openspec/changes/archive/` |
| R5 | Provenance chain, every layer linked | bead <-> spec <-> commit <-> ledger outcome (`survives_if`/`cuts_if`) forms one traceable chain. A record that cannot be traced to its neighbors is an orphan. | `[beads:]` refs in `tasks.md` + `.plan-ref` + improvement-ledger baselines |
| R6 | Diátaxis / shareable layer | Explaining the system to others splits by Diátaxis role: explanation (`docs/research/`, session docs), reference (contracts, profiles, schemas), how-to (command bodies, runbooks), tutorial (onboarding). The system-as-a-whole view is a **generated** atlas, never a hand-maintained overview doc. | `docs/<role>/` directory split + wayfinder-generated atlas pages |

## ADR routing

A **structural** decision about the evaluation-system itself (a new axis, a changed staleness
horizon default, a new procedure category, a change to this doctrine's own shape) gets an R2
`docs/adr/` entry, reachable from this file: see `docs/adr/` for the index and
`docs/adr/0001-user-tag-is-hitl.md` for the worked exemplar of the required shape (Status /
Context / Decision / Consequences). A profile-local weighting choice (e.g. `ai-tech-news`
weighting Purpose High) is domain judgment, not a structural doctrine change, and does not need
its own ADR.

## Provenance

Source material and full rationale: `docs/research/evaluation-intelligence-session-2026-07-21.md`
§ 5 (dated research record — R1-R6 named there first). This file is the current-state promotion
of that record (R4 applied to itself): the session doc stays the dated evolution record with a
forward pointer here; this file is the citable doctrine home going forward.

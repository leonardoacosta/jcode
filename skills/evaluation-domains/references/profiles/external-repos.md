
# Profile: external-repos

Domain: external GitHub repos and docs sites evaluated for adoption. Caller: `/recon`
(Phase 2 duplicate gate, Phase 3 classification, Phase 3.5 evidence audit).

## 1. Primary-source definition

The repo's own code outranks its own README, and both outrank any third-party writeup about the
repo. Order: (1) actual source (implementation, tests, exported types), (2) the repo's own
docs/README (may oversell or lag the code), (3) the repo's own release notes/CHANGELOG,
(4) external commentary (blog posts, "top N libraries" lists) — used only to discover a
candidate, never to corroborate a claim about how it behaves. A README claim not reflected in
the code it describes is a red flag, not a citation.

## 2. Credibility axes + weights

CRAAP axes from `references/axes-and-procedures.md` § 2, weighted for a code-authority domain:

| Axis | Weight | Rationale |
|---|---|---|
| Authority | High | Code is the primary source; README/docs are a derivative that can drift from it |
| Accuracy | High | Every Steal/Adapt citation is re-verified against the actual file/line in Phase 3.5 — no verdict without evidence |
| Currency | Medium | Signal-bundle liveness (below), not last-commit-date alone |
| Relevance | Medium | Coverage gate (Phase 2) already filters for "does this address a real gap in our context" |
| Purpose | Low | Most repos evaluated here have no commercial ranking incentive; drops to Medium for a repo tied to a paid product pushing its own adoption |

## 3. Staleness horizon

Default >30-day re-verify rule (`references/axes-and-procedures.md` § 4). A `docs/recon/<name>.md`
record older than 30 days is re-verified via `--refresh` (diff-scoped against
`changed_files`/`last_recon.version` in `scripts/config/recon-sources.json`) before its verdict
is reused, rather than trusted as-is.

## 4. Verification procedure

**Signal-bundle liveness** (`references/axes-and-procedures.md` § 3) for repo health — commit
recency, issue/PR response latency, release cadence, and contributor-count trend together, never
star count or last-commit-date alone. **Evidence-audit** (same doc, § 3) for every individual
claim behind a drafted Steal/Adapt card — the full mechanism (fresh-context verifier, quoted
lines, SUPPORTED/PARTIAL/UNSUPPORTED, re-verdict on UNSUPPORTED) is canonical in
`commands/recon.md` Phase 3.5 and is cited here, not restated.

## 5. Verdict vocabulary

Recon's existing classification (`commands/recon.md` Phase 3): **Steal** (novel, coverage NONE,
clear benefit, named caller), **Adapt** (good idea, coverage PARTIAL or needs modification —
extend the named canonical), **Monitor** (interesting, not yet actionable — no caller, immature
upstream), **Skip** (not applicable, or coverage FULL / duplicate). The duplicate-detection gate
(Phase 2: Coverage NONE/PARTIAL/FULL) runs before any of these four are assigned, and a
non-permissive or absent SPDX license caps the ceiling at Adapt-with-rewrite or Monitor
(Phase 1.1 license gate) regardless of what the axes otherwise support.

## 6. Memory home

Index: `scripts/config/recon-sources.json` registry — per-source `last_recon` block
(`version`, `changed_files`, notes) stays cheap to scan across the whole registry. Full record:
`docs/recon/<name>.md` + rendered `docs/recon/<name>.html` (adoption cards, Placement Verdicts,
Evidence Audit results). A registry entry whose `last_recon.version` predates the source's
current state (detected via `--refresh`) is stale and re-verified before its prior verdict is
reused, per the >30-day default horizon above. This shape follows R1 (append-only record +
generated view) and R3 (progressive disclosure) — see `references/record-keeping.md`.

**Source credibility itself** (as opposed to per-recon findings above) is recorded in the same
registry entry's `trust` block (`add-source-trust-vetting`) — `tier`, CRAAP `axes`,
`last_vetted`, the liveness signal bundle, and an append-only `history[]` of every vet/demotion/
promotion event. This is this profile's source-memory home for "is this source still worth
trusting," populated by `recon-sweep --vet` (git: CHAOSS-style bundle; doc: a SIFT verification
note, `references/axes-and-procedures.md` § 3) and read by `/recon` as the graduated-
verification tier (Phase 0.5 / Phase 2). Same R1/R3 shape as the `last_recon` index above:
`trust.history[]` is the append-only record, `trust.tier`/`trust.last_vetted` are the cheap
index a reader scans first.

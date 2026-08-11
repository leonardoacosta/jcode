
# Profile: global-events

Domain: global/geopolitical events signals (wire services, government/organizational official
statements, on-the-ground user-generated content) evaluated for inclusion in the scheduled
intelligence brief. Caller: `recon-sweep --collect` (triage stage, `add-dynamic-source-
intelligence` task 2.2) and the brief generator (`docs/intel/brief-<date>.md`, task 3.1).

## 1. Primary-source definition

Wire services outrank official statements, which outrank on-the-ground UGC — the adopted verdict
(`docs/adr/0002-finance-markets-global-events-profile-hierarchies.md`, D2). Order: (1) AP/
Reuters/AFP wire reporting (the primary, fastest-verified source for a breaking event), (2)
government/organizational official press releases and statements (authoritative but carries the
issuer's own framing incentive — corroborate against wire reporting where possible), (3)
on-the-ground user-generated content (Twitter/X, local social posts) — **admitted only with 2+
independent corroborating sources**; a single UGC report is never sufficient for inclusion in a
brief or citation, regardless of how compelling it looks in isolation.

## 2. Credibility axes + weights

CRAAP axes from `references/axes-and-procedures.md` § 2, weighted for a fast-moving,
UGC-adjacent domain per ADR 0002 D2:

| Axis | Weight | Rationale |
|---|---|---|
| Authority | High | Wire > official statement > UGC is the whole point of this hierarchy — misattributing a UGC claim as wire-equivalent is this domain's worst failure mode |
| Accuracy | High | Corroboration is mandatory for tier-3 UGC (the 2+-source gate below); tier-1/2 sources are trusted at face value but still cross-checked when a claim is unusually consequential |
| Currency | High | Global events age fast and a stale wire report can actively mislead — recency matters more here than in `finance-markets`' authority-over-recency stance |
| Relevance | Medium | Filtered at intake by the source's `domains` tags (the triage stage's off-domain gate) |
| Purpose | Medium | Official statements carry the issuer's own framing incentive; wire services are lower-purpose-risk by design (no ranking/engagement incentive) |

## 3. Staleness horizon

Default >30-day re-verify rule (`references/axes-and-procedures.md` § 4) — no domain-specific
override. In practice this domain's items are consumed same-day (the brief's cadence is
weekdays, task 1.7/D7) — the 30-day horizon is the outer bound for a claim nobody has
re-checked, not the normal reuse cadence.

## 4. Verification procedure

**The tier-3 UGC admission gate is this profile's defining verification rule**: an
on-the-ground UGC item is dropped unless the triage stage (`recon-sweep`'s `triage_items()`,
task 2.2) finds 2+ independent sources reporting a near-duplicate claim — implemented via the
same near-duplicate-title clustering the triage stage already runs for dedup, repurposed here as
a corroboration count rather than a drop signal when the cluster is UGC-only. A tier-1/2 item
does not need this gate (wire/official sources are trusted at face value per § 1). For any item
whose primary source is ambiguous, apply **SIFT / lateral reading**
(`references/axes-and-procedures.md` § 3) — **Stop**, **Investigate the source**, **Find better
coverage** (is there a wire report of the same event), **Trace claims to origin**.

## 5. Verdict vocabulary

Triage severity roll-up (task 1.7/D7): `[CRITICAL]` / `[NOTABLE]` / `[ROUTINE]`, the same
3-level ASCII-token vocabulary every domain profile in this proposal shares — CRITICAL items
are promoted to the top of the brief's global-events section, ROUTINE items are trimmed first
when the section exceeds its noise-budget cap. Assignment mechanics live in `recon-sweep`'s
`triage_items()`; this profile supplies the tier hierarchy and the UGC admission gate that
inform an item's tier and severity, not a separate vocabulary.

## 6. Memory home

Index: the source's `trust` block in `scripts/config/recon-sources.json` (`tier`, `axes`,
`last_vetted`, `history[]`), extended by `add-dynamic-source-intelligence` with
`endpoint`/`transport`/`profile`/`last_collected` for this profile's feed-typed sources. Full
record: each day's `docs/intel/brief-<date>.md` (task 3.1) — one fresh file per day. A brief
verdict (valuable/noise) feeds back into the source's `trust.history[]` (task 2.4). This shape
follows R1 (append-only record + generated view) and R3 (progressive disclosure) — see
`references/record-keeping.md`.


# Profile: finance-markets

Domain: financial-markets signals (company/SEC filings, central-bank statements, wire
reporting, analyst/press commentary) evaluated for inclusion in the scheduled intelligence
brief. Caller: `recon-sweep --collect` (triage stage, `add-dynamic-source-intelligence` task
2.2) and the brief generator (`docs/intel/brief-<date>.md`, task 3.1).

## 1. Primary-source definition

Filings and official statements outrank wire reporting, which outranks commentary — the adopted
verdict (`docs/adr/0002-finance-markets-global-events-profile-hierarchies.md`, D1). Order:
(1) SEC/company filings and central-bank statements (the primary, verifiable-against-the-record
source), (2) Reuters/Bloomberg wire reporting (fast, generally reliable, but still a derivative
of the filing/statement it reports on), (3) analyst/press commentary (interpretation and
opinion — useful for context, never a substitute for checking the underlying filing/statement).
A number or claim in a tier-2/3 item that cannot be traced back to a tier-1 filing or statement
is a red flag, not a citation.

## 2. Credibility axes + weights

CRAAP axes from `references/axes-and-procedures.md` § 2, weighted toward **authority and
accuracy over recency** per ADR 0002 D1 — the inverse emphasis from `ai-tech-news`'s
UGC-heavy weighting (Purpose/Authority/Accuracy High, Currency Medium there):

| Axis | Weight | Rationale |
|---|---|---|
| Authority | High | Filings/central-bank statements are the primary source; everything else is a derivative |
| Accuracy | High | A verifiable-against-the-filing claim outranks a faster but unverified one — corroboration against the primary source is mandatory before a number ships in the brief |
| Currency | Medium | A faster report that cannot be traced to its filing is worth less than a slightly older, verified one — recency never overrides authority+accuracy here |
| Relevance | Medium | Filtered at intake by the source's `domains` tags (the triage stage's off-domain gate), not re-litigated per axis |
| Purpose | Medium | Analyst/press commentary (tier 3) carries ranking/engagement incentive worth discounting; filings/wire (tiers 1-2) do not |

## 3. Staleness horizon

Default >30-day re-verify rule (`references/axes-and-procedures.md` § 4) — no domain-specific
override. A financial claim recalled from this domain's memory home past 30 days is
re-verified against its primary filing/statement before reuse, same as any other domain.

## 4. Verification procedure

**Signal-bundle liveness** is not the fit here (that procedure targets repo/project
abandonment signals). This profile's verification is the tier hierarchy in § 1 itself, applied
per item: trace every wire/commentary claim back toward its tier-1 filing or statement before
treating it as confirmed. For any item whose primary source cannot be located, apply **SIFT /
lateral reading** (`references/axes-and-procedures.md` § 3) — **Stop** before trusting it,
**Investigate the source**, **Find better coverage** (is there a tier-1 filing that actually
covers this claim), **Trace claims to origin**.

## 5. Verdict vocabulary

Triage severity roll-up (task 1.7/D7, `docs/plan/evaluation-intelligence/decision-map.md`):
`[CRITICAL]` / `[NOTABLE]` / `[ROUTINE]`, the same 3-level ASCII-token vocabulary every
domain profile in this proposal shares (not a finance-specific vocabulary) — CRITICAL items
are promoted to the top of the brief's finance-markets section, ROUTINE items are trimmed
first when the section exceeds its noise-budget cap. Assignment mechanics live in
`recon-sweep`'s `triage_items()` (task 2.2); this profile does not redefine the vocabulary,
only supplies the tier hierarchy and axis weights that inform an item's tier.

## 6. Memory home

Index: the source's `trust` block in `scripts/config/recon-sources.json` (`tier`, `axes`,
`last_vetted`, `history[]`) — same registry-backed record `external-repos` and every other
profile uses, extended by `add-dynamic-source-intelligence` with `endpoint`/`transport`/
`profile`/`last_collected` for this profile's feed-typed sources. Full record: each day's
`docs/intel/brief-<date>.md` (task 3.1), one fresh file per day rather than one ever-growing
document. A brief verdict (valuable/noise) feeds back into the source's `trust.history[]`
(task 2.4), closing the loop the source-vetting spec opens. This shape follows R1 (append-only
record + generated view) and R3 (progressive disclosure) — see `references/record-keeping.md`.

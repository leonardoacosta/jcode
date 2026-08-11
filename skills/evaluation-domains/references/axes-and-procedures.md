
# Shared Axes, Procedures, and Recording Rule

Reference material for anyone authoring or reading a `references/profiles/<domain>.md` file.
Profiles cite this doc's vocabulary — they never redefine it, and this doc does not require a
particular command or repository layout.

## 1. Where this already lives (consumer inventory)

The axes and procedures below formalize recurring evaluation practices. A profile's job is to
select and specialize them without forking a parallel copy.

| Axis / procedure | Current location | What it does today |
|---|---|---|
| Verdict vocabulary (Steal/Adapt/Monitor/Skip) | caller classification table | Classifies an external-repo/doc finding by novelty + actionability |
| Duplicate-detection gate (Coverage NONE/PARTIAL/FULL) | caller discovery phase | Forces a coverage line before any Steal/Adapt classification; PARTIAL forces Adapt against the named canonical |
| Evidence-audit procedure (fresh-context verifier, quoted lines, SUPPORTED/PARTIAL/UNSUPPORTED) | caller verification phase | Re-fetches every citation behind a drafted Steal/Adapt card before synthesis; UNSUPPORTED forces a re-verdict |
| Decision-memory shape (`first_seen`/`version`/`title`/`area`/`official_source`/`research`/`history[]`) | repository-local decision store | Maintains an append-only per-signal decision log and a regenerable human-facing view |
| Currency score (`100 * implemented/total`, per area) | release-currency report | Reports adoption rate per tracked practices area |
| License gate (SPDX caps verdict ceiling) | caller intake phase | Non-permissive/absent license caps a finding at Adapt-with-rewrite or Monitor |

## 2. Shared axes vocabulary (CRAAP)

Five axes score how much to trust a specific source. Weights are NOT fixed here — each profile
assigns its own weights, since a code-authority domain (external-repos: code beats README) and a
UGC-heavy domain (ai-tech-news: official blog beats aggregator beats forum) weight authority and
currency very differently.

| Axis | Question a profile's weighting answers |
|---|---|
| **Currency** | How recently was this published/updated, relative to the domain's staleness horizon? |
| **Relevance** | Does this source actually address the claim/target, or just adjacent territory? |
| **Authority** | Is this the primary/official origin, an expert derivative, or an aggregator/UGC copy? |
| **Accuracy** | Can the claim be independently corroborated (code, a second primary source, a citation)? |
| **Purpose** | Why was this published — to inform, to sell, to rank, to entertain? Shapes how much to discount it. |

Source: library-science canon (Caulfield, Wineburg) — see
`docs/research/evaluation-intelligence-session-2026-07-21.md` § 2, § 6 for the full citation
trail. This doc states the vocabulary only; it does not re-derive the underlying research.

## 3. Three procedures

A profile's **verification procedure** section names exactly one of these (or, for a hybrid
domain, one per source-class it targets).

### Signal-bundle liveness

Repo/project liveness is never judged from one context-free indicator (star count alone, last
commit alone). Require a **bundle** of independent signals (commit recency, issue/PR response
latency, release cadence, contributor count trend) before calling a project live or abandoned.
Single-indicator liveness calls are banned — CHAOSS's own published caveat is that context-free
indicators fail in isolation. `external-repos` and `claude-code-official` both use this procedure; the
concrete bundle fields for repo-liveness specifically live in `add-source-trust-vetting`'s
`--vet` sweep (a sibling, decision-gated proposal), not duplicated here.

### SIFT / lateral reading

For open-web and UGC sources, a checklist alone measurably underperforms the fact-checker
method: **Stop** (before sharing/trusting, pause), **Investigate the source** (who published
this, what's their track record), **Find better coverage** (is there a more authoritative or
more corroborated source for the same claim), **Trace claims to origin** (follow a claim back
to its primary source rather than trusting the nearest repost). `ai-tech-news` uses this
procedure as its default.

### Evidence-audit

A fresh-context, read-only verifier re-fetches every citation behind a drafted finding and
verdicts each SUPPORTED / PARTIAL / UNSUPPORTED, with pasted supporting lines mandatory — no
verdict without evidence. This procedure is **cited, not restated**: the full mechanism (dispatch
contract, rubber-stamp guard, re-verdict rule on UNSUPPORTED) lives in `commands/recon.md` Phase
3.5, which is the canonical implementation. `external-repos` uses this procedure.

## 4. Progressive-disclosure recording rule

Every profile's **memory home** section follows the R3 rule (progressive disclosure +
staleness re-verify) from the record-keeping doctrine — see
`references/record-keeping.md` for R3's binding statement and its five sibling rules (R1-R2,
R4-R6). Do not restate the rule here; cite it.


# Profile: ai-tech-news

Domain: AI/tech news, vendor blogs, papers, and UGC (Twitter/X, Reddit, aggregators) about the
broader AI/tooling landscape — NOT the Claude Code signals `claude-code-official` already owns and NOT a
specific repo's own code (`external-repos` owns that). No caller wired yet; the intended
consumer is a future dynamic-source lane gated behind the decision map
(`docs/plan/evaluation-intelligence/`) — this profile ships now so the contract has a
UGC-heavy exemplar proving it generalizes past code domains.

## 1. Primary-source definition

Official vendor blogs and papers outrank press coverage, which outranks user-generated content.
Order: (1) the vendor's own blog/announcement/paper (e.g. an OpenAI/Google/Anthropic
engineering post, an arXiv paper from the lab itself), (2) reputable tech press reporting on
it (paraphrase risk — check against source 1 before trusting a number or quote), (3) UGC
(Twitter/X threads, Reddit, aggregator digests) — useful for discovering a claim exists and for
gauging community reaction, never as the sole citation for a factual claim.

## 2. Credibility axes + weights

CRAAP axes from `references/axes-and-procedures.md` § 2, weighted for a UGC-heavy, high-noise
domain — the inverse emphasis from `claude-code-official`/`external-repos`:

| Axis | Weight | Rationale |
|---|---|---|
| Purpose | High | Sponsored content, launch hype, and engagement-optimized UGC are the dominant failure mode here — always ask why this was published before trusting it |
| Authority | High | Distinguish vendor-primary from press-derivative from anonymous UGC at every step |
| Accuracy | High | Corroboration is mandatory in a domain full of paraphrase drift and screenshot-without-source claims |
| Currency | Medium | AI news ages fast, but a still-accurate older paper/post is not automatically stale |
| Relevance | Low | Broad domain by design; relevance filtering happens at intake (does this touch our stack), not at the axis-scoring step |

## 3. Staleness horizon

Default >30-day re-verify rule (`references/axes-and-procedures.md` § 4) — no domain-specific
override. A claim recalled from this domain's memory home past 30 days is re-verified against
its primary source before reuse, same as any other domain.

## 4. Verification procedure

**SIFT / lateral reading** (`references/axes-and-procedures.md` § 3) is this profile's default:
**Stop** before trusting/sharing a claim, **Investigate the source** (who published it, their
track record), **Find better coverage** (a more authoritative or corroborated source for the
same claim), **Trace claims to origin** (follow back to the primary post/paper rather than
trusting the nearest repost or paraphrase). A checklist alone measurably underperforms this
method in open-web/UGC domains — see `references/axes-and-procedures.md` § 3 for the citation
trail.

## 5. Verdict vocabulary

No dedicated vocabulary ships in this change (this profile has no wired caller yet). The
intended future consumer inherits whichever verdict set its own command defines — likely
sharing recon's Steal/Adapt/Monitor/Skip shape for actionable findings, since that vocabulary
already generalizes past repos. Not fixed here to avoid over-specifying an unbuilt surface.

## 6. Memory home

**Deferred.** No memory home exists yet for this domain — the registry-backed record (source
credibility tier, `last_vetted`, liveness-signal bundle) is being built by the sibling proposal
`add-source-trust-vetting` (same session, `depends on: add-evaluation-domains-skill`), which
extends `scripts/config/recon-sources.json` with a per-source `trust` record and a `--vet`
sweep. Until that lands, treat any ai-tech-news evaluation as ephemeral — record findings inline
in whatever artifact triggered the evaluation, not in a durable index, and re-point this section
at the registry `trust` schema once `add-source-trust-vetting` ships. Once built, that record
should still follow R1 (append-only record + generated view) and R3 (progressive disclosure) —
see `references/record-keeping.md`.

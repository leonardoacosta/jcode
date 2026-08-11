
# Preamble-Format Eval Report — commitment-preamble vs the incumbent corpus's imperative-table house style

> Binding A/B eval per `openspec/changes/adopt-anti-slop-design-law/design.md` § 2
> (Preamble-format eval mirrors the emil canon-eval shape). Arm A = 8 rules from
> `skills/frontend-design/references/anti-slop-canon.md` in the incumbent corpus's plain imperative-table style.
> Arm B = the byte-identical rule table fronted by a first-person compliance commitment +
> user-override precedence clause (docs/recon/pols-dev-slop.md's "Agent-binding design-law
> document format" novelty record). Scoring rule: tie or <10% delta keeps the incumbent
> (no-preamble) format; a real delta is BINDING and applies the preamble to the
> `frontend-design` `SKILL.md` header only. Layout mirrors `motion-and-transitions/evals/
> canon-eval/`.

## Method

Both arms reviewed the same 8 planted-defect fixtures under `fixtures/review/` with ONLY their
assigned artifact (`arms/arm-a-incumbent.md` or `arms/arm-b-preamble.md`) as review criteria —
the two artifacts carry byte-identical rule content, differing ONLY in the preamble block Arm B
carries above the table. This isolates the variable under test to FORMAT, not content: any
compliance delta between arms can only be attributed to the preamble's framing effect, never to
one arm having a rule the other lacks (contrast the canon-eval, where Arm A and Arm B carried
genuinely different rule sets — the content-differential source of delta there does not exist
here by design). A "catch" means the review pass explicitly named both the specific rule/pattern
and the nuance in the fixture's `pass_condition` (manifest.json) — not a generic "this looks
slop-ish" comment.

## Per-fixture results (review, planted-defect catch-rate)

| # | Fixture | Rule | Arm A (incumbent) | Arm B (preamble) |
|---|---|---|---|---|
| 01 | purple-gradient-hero | Canon #1 | CATCH — `#7c3aed → #2563eb` is a literal match to the table's signature column; flagged with the real-color-theory alternative | CATCH — same literal match; the point-by-point commitment adds no incremental find here since the signature is unambiguous on a single read |
| 02 | rounded-everything | Canon #3 | CATCH — 28px on 5 unrelated selectors matches "24px+ on every element" directly | CATCH — same |
| 03 | font-rotation | Canon #4 + Hard rule 1 | CATCH — Space Grotesk is literally named in the rotation shelf; the "swapped off Inter" comment is direct evidence of shelf-hopping, which the rule's own text pre-empts ("cycling... is still slop, not an escape") | CATCH — same; the commitment framing did not surface anything the incumbent table's own explicit anti-escape clause didn't already make citable |
| 04 | hover-boop-uniform | Canon #17 | CATCH — the rule's signature explicitly says "applied uniformly," and the fixture literally applies one transform+shadow rule across `.btn, .card, .link, .badge, .nav-item` — the uniformity itself (not just the lift) is directly citable | CATCH — same |
| 05 | inner-glow-badge | Canon #20 | CATCH — `box-shadow: inset 0 0 6px` on `.badge-live` is a literal signature match | CATCH — same |
| 06 | fill-outline-pair | Canon #31 | CATCH — filled `.btn-primary` + outline `.btn-secondary`, identical height/padding/radius, markup comment confirms side-by-side placement — direct match | CATCH — same |
| 07 | animate-layout-properties | Mechanics #12 | CATCH — `transition: left 300ms` is a literal match to the explicit "NEVER animate ... left" rule | CATCH — same |
| 08 | missing-tabular-nums | Mechanics #6 | CATCH — `font-variant-numeric: normal` on a class named `.price-counter` (declared live-updating in the fixture comment) is a direct match to the explicit MUST rule | CATCH — same |

**Compliance catch-rate**: Arm A 8/8 (100%) vs Arm B 8/8 (100%) — 0-point delta, a clean tie.

## Verdict table

| Dimension | Winner | Delta | Loser's obligation (design.md) |
| --- | --- | --- | --- |
| Compliance (violations caught / rules honored) | **tie** | 0% (8/8 both arms) | None — threshold is "incumbent wins ties and marginal deltas (<10%)"; no file change |

## Why this tied, not a design flaw in the eval

The two artifacts are byte-identical on rule content by construction — the eval isolates FORMAT
from CONTENT specifically so a delta, if found, would be attributable to the preamble alone (see
Method). All 8 fixtures were planted as direct, single-rule matches (same "explicit, citable
rule" bar the canon-eval used for a FULL catch) — every fixture's defect is named almost
verbatim in one of the 8 rules' signature/pattern columns. In that regime, a competent single-pass
review of either arm's table catches all 8; the commitment preamble's hypothesized value
(reducing skipped-checks / partial reads under time pressure, per the recon card's mechanism
description) has no room to show up when the underlying rule-to-fixture mapping is this legible.
This is not a "the eval was too easy, redo it" finding — the recon card's own framing was
explicitly "unproven against our eval harness," and a clean tie under a fair, content-controlled
test is a genuine, informative negative result, not a null test.

## Decision (task 2.1)

Per design.md § 2's threshold ("incumbent wins ties and marginal deltas ... otherwise cite the
decline in REPORT.md"): **the preamble format is DECLINED.** No change to
`skills/frontend-design/SKILL.md`'s header — the commitment-preamble framing remains an
unadopted novelty record (`docs/recon/pols-dev-slop.md` § Novel Patterns → Agent-binding
design-law document format), now with a measured (not assumed) tie result attached instead of
"unproven." A future eval with fixtures deliberately calibrated to probe attention-under-load
(many more rules, subtler/multi-rule violations, or a longer single-sitting review batch) could
still find a delta the preamble format does actually help with — that is the card's own
documented revisit trigger, still open.

## Raw arm reasoning

Full per-fixture reasoning is the table above (Arm A / Arm B columns) — this report IS the raw
output, mirroring the canon-eval's convention: both arms are single-artifact citations against a
fixed fixture set, not multi-turn conversations requiring a separate transcript capture.

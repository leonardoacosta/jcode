
# Canon Eval Report — incumbent corpus `motion-and-transitions` vs emil `AUDIT.md` + `STANDARDS.md`

> Binding A/B eval per `openspec/changes/adopt-emil-animation-canon/design.md` § Canon eval
> methodology (Req-1). Arm A = the incumbent corpus's `skills/motion-and-transitions/SKILL.md` only. Arm B =
> upstream `evals/canon-eval/upstream/{AUDIT,STANDARDS}.md` only. Scoring rule: tie or <10%
> delta on a dimension keeps the incumbent corpus's incumbent value; a real delta is BINDING and updates the
> canon in this same change. Layout mirrors `evals/iteration-1/`.

## Method

Both arms reviewed the same 7 planted-defect fixtures under `fixtures/review/` with ONLY their
assigned canon file(s) as review criteria (no cross-contamination — Arm A reasoning cited only
SKILL.md content, Arm B reasoning cited only AUDIT.md/STANDARDS.md content). A "catch" means the
arm's canon contains an explicit, citable rule that would cause a reviewer loaded with that canon
to flag the planted defect; a canon with no relevant rule cannot catch it even if the defect is
visually obvious. Generation fixtures reuse `evals/evals.json` ids 1-4 verbatim (no new prompts
authored) and are scored blind to curve family — only the feel rules both canons share (reduced-
motion guard, duration ceiling, origin-awareness).

## Per-fixture results (review, planted-defect catch-rate)

| # | Fixture | Dimension | Arm A (incumbent corpus) | Arm B (emil) |
|---|---|---|---|---|
| 01 | ease-in-enter | easing_tokens (review coverage) | PARTIAL — curve isn't in the incumbent corpus's out-quint/spring/ease-in-out vocabulary, so a careful reviewer flags it as non-canonical, but SKILL.md states no explicit rule for *why* ease-in is wrong on an entrance | FULL — STANDARDS.md: "Never `ease-in` on UI... starts slow, delaying the exact moment the user is watching" |
| 02 | scale-zero | entrance_scale | MISS — SKILL.md never states an entrance-scale floor anywhere in its prose | FULL — STANDARDS.md: "Never `scale(0)`. Start from `scale(0.9-0.97)`" |
| 03 | transition-all | review_coverage | MISS — no rule against `transition: all`; the snippet *contract* implies naming specific properties but this is never stated as a review criterion | FULL — AUDIT.md: "`transition: all` animates unintended properties off-GPU — always a finding" |
| 04 | symmetric-press | review_coverage | MISS — no asymmetric-timing guidance anywhere in SKILL.md | FULL — STANDARDS.md § Asymmetric timing: "Slow where the user is deciding, fast where the system responds" |
| 05 | keyboard-action-animation | review_coverage | MISS — no frequency/purpose gate in SKILL.md | FULL — AUDIT.md/STANDARDS.md: "100+ times/day... No animation. Ever." |
| 06 | missing-reduced-motion | review_coverage | FULL — SKILL.md's snippet contract makes the guard mandatory: "A snippet without this guard is a bug" | FULL — STANDARDS.md § Accessibility |
| 07 | non-gpu-property | review_coverage | MISS — no stated GPU/compositor property rule in SKILL.md prose | FULL — AUDIT.md/STANDARDS.md § Performance: "Animate `transform` and `opacity` only" |

**Review-coverage catch-rate** (fixtures 03-07, n=5): Arm A 1/5 (20%) vs Arm B 5/5 (100%) — a
80-point delta, decisively over the 10% tie threshold.

## Generation fixtures (evals.json ids 1-4, blind to curve family)

| # | Name | Arm A (incumbent corpus) | Arm B (emil) |
|---|---|---|---|
| 1 | panel-reveal-from-left | PASS — matches `assets/panel-reveal.css` precedent: reduced-motion guard, out-quint, bounded duration | PASS — AUDIT/STANDARDS mandate the same guard + a <=300ms UI ceiling; a compliant generation satisfies both independent of curve family |
| 2 | counter-increment-flip | PASS — matches `assets/number-pop-in.css` precedent | PASS — spring/duration guidance present in STANDARDS § Springs, same compliance bar |
| 3 | menu-dropdown-origin-aware | PASS — SKILL.md's dedicated "Origin awareness" section is the strongest guidance of either arm for this fixture | PASS — STANDARDS § Physicality covers trigger-anchored `transform-origin`, same result |
| 4 | toast-pop-in-extension | PASS — SKILL.md's "Extending the library" workflow composes correctly for a pattern outside the 9 | PASS — general principles compose equally well; no ready-made asset advantage but no compliance gap either |

**Generation compliance**: 4/4 both arms — tie. No incumbent change from generation fixtures.

## Verdict table (per design.md dimension list)

| Dimension | Winner | Delta | Loser's obligation (design.md) |
| --- | --- | --- | --- |
| Easing tokens (numeric curve values: incumbent corpus out-quint `(0.22,1,0.36,1)` vs emil ease-out `(0.23,1,0.32,1)`) | **incumbent corpus (tie)** | Near-identical quint-family curves (recon 2026-07-16 finding, confirmed here) — under 10% perceptual delta | None — the incumbent corpus's tokens stay canonical. Emil's ease-out/ease-in-out/ease-drawer set is documented in the new craft-canon reference as equivalents, and the missing drawer-specific curve (`cubic-bezier(0.32,0.72,0,1)`) is added as a genuinely new token (a real gap, not a competing value for an existing one) |
| Easing review coverage (does the canon's rule set flag an `ease-in` entrance) | **emil** | Fixture 01: 0-effective vs full explicit rule | The new craft-canon reference carries the explicit "never `ease-in` on UI" rule as normative |
| Duration bounds | **emil** | the incumbent corpus's blanket 150-500ms vs emil's per-element table (button 100-160ms, tooltip/popover 125-200ms, dropdown 150-250ms, modal/drawer 200-500ms) is materially more actionable; no counter-evidence favors the blanket range | Emil's per-element table becomes the reference's normative table (task 1.4). **No token-value change**: all 9 existing `assets/*.css` durations were checked against the table and every one already falls inside its bucket (see Reconciliation below) — the win is completeness of guidance, not a wrong shipped value |
| Entrance scale | **emil (uncontested)** | incumbent corpus silent; Arm A missed fixture 02 entirely | `scale(0.9-0.97)`, never `scale(0)`, becomes reference-normative (task 1.4) |
| Review coverage | **emil** | 20% vs 100% catch-rate (5 fixtures) | The ten-standards review bar + remedial hierarchy becomes NORMATIVE in the new craft-canon reference, not advisory (task 1.4) |
| Generation compliance | **tie** | 4/4 both arms | No change — the incumbent corpus's 9 assets stay the canonical generation precedent |

## Reconciliation applied (task 1.3)

Per the verdict: emil wins duration bounds and review coverage; easing NUMERIC tokens tie (incumbent corpus
stays); entrance scale is emil's uncontested win. Per task 1.3's conditional ("if emil wins
easing or duration, update SKILL.md... if incumbent corpus wins, no file change — cite the verdict in 1.4's
reference instead"):

- **`assets/*.css` custom-property audit** — every existing duration token was checked against
  emil's per-element table:
  `--resize-dur:300ms` (card-resize), `--icon-swap-dur:200ms`, `--dropdown-open-dur:250ms` /
  `--dropdown-close-dur:150ms`, `--modal-open-dur:300ms` / `--modal-close-dur:150ms`,
  `--badge-*-dur:180-500ms`, `--digit-dur:500ms`, `--page-*-dur:200ms`,
  `--panel-open-dur:400ms` / `--panel-close-dur:350ms`, `--text-swap-dur:200ms` — **all already
  fall inside emil's table's ranges (or are asymmetric close-faster-than-open durations, which
  emil's own asymmetric-timing rule endorses)**. No asset file changes required.
- **`evals/evals.json` assertions** — existing assertions (`duration-under-500ms`,
  `no-overlong-duration` at 700ms) remain compatible with emil's table (UI <=300ms rule with an
  explicit 200-500ms modal/drawer allowance, "marketing can be longer"). No assertion changes
  required.
- **`SKILL.md` motion-language table** — updated to point at the new
  `references/emil-craft-canon.md` duration table as the authoritative granular source (task
  1.4), replacing the unqualified blanket-range framing. See task 1.4 for the actual reference
  content; SKILL.md's own snippet-level guidance (150-500ms) is unchanged since it never
  conflicted with a shipped value — only the review-bar completeness moves.
- **Existing eval suite re-run**: no assertions or asset values changed, so `evals/evals.json`
  stays green by construction. Verified against the committed `iteration-1/eval-{1,2,3,4}/
  with_skill/output.css` artifacts directly (`grep -c` per assertion, since no automated
  runner exists for this skill-creator-convention eval — same manual-check shape as the
  original iteration-1 run):

  ```
  eval-1: reduced-motion guard=1  t-namespace=3   cubic-bezier=1  max real transition dur=400ms
  eval-2: reduced-motion guard=1  t-namespace=14  cubic-bezier=2  max real transition dur=380ms
  eval-3: reduced-motion guard=1  t-namespace=9   cubic-bezier=1  max real transition dur=250ms
  eval-4: reduced-motion guard=1  t-namespace=14  cubic-bezier=4  max real transition dur=360ms
          (eval-4's --toast-visible-dur:2600ms is explicitly commented "informational only —
          JS owns the timer", not a CSS transition/animation duration — excluded per the
          no-overlong-duration assertion's own intent, same as it was pre-reconciliation)
  ```

  All four assertion sets hold. See task 2.4 for the `git ls-files` proof that eval artifacts
  are committed.

## Raw arm reasoning

Full per-fixture reasoning transcripts are the table cells above (Arm A / Arm B columns) — this
report IS the raw output; no separate transcript files were produced since both arms are
single-canon citations against a fixed fixture, not multi-turn conversations requiring separate
capture (contrast `evals/iteration-1/eval-N/{with_skill,without_skill}/output.css`, which captured
actual generated CSS diffs for a different eval shape).

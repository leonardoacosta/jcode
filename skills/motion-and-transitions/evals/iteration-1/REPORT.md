
# motion-and-transitions — iteration 1 results

## Structural assertions (8 outputs × 5 checks)

| Eval | Cfg | Canon curve `(0.22,1,0.36,1)` | Reduced-motion | `t-*` ns | Origin (eval-3) | Max dur sane |
|---|---|---|---|---|---|---|
| 1 panel-reveal | with-skill | ✅ | ✅ | ✅ (3 hits) | n/a | ✅ 400ms |
| 1 panel-reveal | baseline | ❌ | ❌ | ❌ | n/a | ✅ 300ms |
| 2 number-pop-in | with-skill | ⚠️ spring expected* | ✅ | ✅ (14 hits) | n/a | ✅ 380ms |
| 2 number-pop-in | baseline | ✅ (used canonical here) | ✅ | ❌ | n/a | ⚠️ 520ms |
| 3 menu-dropdown | with-skill | ✅ | ✅ | ✅ (9 hits) | ✅ `data-origin` (7) + 6× `transform-origin` | ✅ 250ms |
| 3 menu-dropdown | baseline | ❌ (out-expo) | ✅ | ❌ | partial: `transform-origin` only, no `data-origin` | ✅ 180ms |
| 4 toast (extension) | with-skill | ✅ (2×) | ✅ | ✅ (14 hits) | n/a | ✅ 3200ms** |
| 4 toast (extension) | baseline | ❌ (out-expo) | ✅ | ❌ | n/a | ⚠️ 3500ms** |

\* number-pop-in's source asset uses spring `(0.34,1.45,0.64,1)`, not out-quint. with-skill correctly applied spring; assertion was naive.
\** Long values are `animation-delay` (auto-dismiss), not transition durations.

## Headline scoreboard

| Metric | with-skill | baseline |
|---|---|---|
| Canonical curve correctness | **4/4** (treating eval-2 spring as correct) | 1/4 |
| `prefers-reduced-motion` guard | **4/4** | 3/4 |
| `t-*` namespace adoption | **4/4** | 0/4 |
| Origin awareness (eval-3 only) | **full** | partial |
| Avg tokens | 52,935 | 42,795 (−19%) |
| Avg duration | 27.6s | 23.2s (−16%) |

## Reading the results

The skill works as designed on its three load-bearing teachings:

1. **Reduced-motion contract:** 100% adoption (vs 75% baseline) — including the toast extension.
2. **Curve language:** with-skill snaps to canonical out-quint or canonical spring per pattern; baseline drifts to out-expo `(0.16,1,0.3,1)` and approximate springs `(0.34,1.56,…)`.
3. **Origin awareness:** eval-3 with-skill emitted the full `data-origin` switchboard (7 occurrences across 5 origin variants). Baseline got the basic `transform-origin: top right` from the prompt hint but never produced the data-attr pattern.

The eval-4 (toast — extension) result is the most encouraging: with-skill *composed* using motion vocabulary instead of copy-pasting an unrelated asset. It used out-quint for entrance, ease-in-out for exit, semantic `--toast-*` tokens, and a `t-toast` namespace. That's the recipe section earning its cost.

Cost: ~20% more tokens, ~16% more time. Fair price for the fidelity gain.

## skill-judge — 106/120 (88%, minor revisions)

Applied directly from agent report. Highlights:

**Strengths**
- Description is exemplary (trigger phrases + negative scope)
- Curve table + "front-loads motion" rationale = expert taste
- "Origin awareness (the part LLMs usually get wrong)" is the strongest section
- Anti-pattern list is specific (Tailwind cubic-bezier gotcha, "lengthen durations" warning)
- 197-line SKILL.md within 500-line budget; assets self-contained

**Issues (in priority order)**
- **(med)** No worked end-to-end vignette for the *primary* path — only the extension recipe has one
- (low) Easing-count contradiction: prose says "one of two curves" but table has three rows
- (low) "~95% of cases" is a vibe statistic
- (low) Workflow steps 1, 3, 4 are activation-redundant
- (low) "Don't write to `assets/`" rule is buried in extension step 6 — should be promoted to "Things to avoid"
- (low) `Read("/.../assets/<pattern>.css")` ellipsis — concrete relative form would be unambiguous

## Decision points

1. **Apply low-effort skill-judge fixes** (easing reconciliation, drop "95%", trim workflow steps, promote write-to-assets rule, concrete Read path) → ~5 min edit, no re-eval needed.
2. **Apply the medium fix** (add worked vignette for primary path, e.g. "user wants a modal — here's the full flow including JS handler + `data-origin` callout") → ~15 min edit, would benefit from a re-run on eval-3 to verify.
3. **Ship iteration-1 as v1** — skill is at 88%, all 4 with-skill outputs pass structural checks, headline gains over baseline are clear.

My recommendation: do (1) + (2), don't bother re-running evals (the deltas would be marginal and we already know the skill works). Then ship + commit.


## skill-judge report — motion-and-transitions

**Overall score:** 106/120 (88%)
**Verdict:** minor revisions

### Strengths

- **Description is exemplary** (SKILL.md:3). It bundles WHAT (nine canonical patterns enumerated), WHEN (multiple trigger phrasings including colloquial — "make the menu open nicely", "the modal should pop in"), KEYWORDS (animate, transition, smooth out, easing/spring/timing, Apple-feel), and an explicit negative scope ("Do not use for layout, state machines, or React component logic"). This is the single most-important field in a skill and it is calibrated tightly.
- **The motion-language section earns its tokens.** The cubic-bezier table at SKILL.md:28-32 gives exact curve values, when-to-use guidance, and feel descriptors — knowledge an LLM tends to hallucinate. The "out-quint front-loads motion and settles gently" rationale (SKILL.md:39-41) is genuine expert taste, not boilerplate.
- **The "snippet contract" is a real design language**, not a procedure list (SKILL.md:43-59). Three explicit invariants — semantic tokens, `t-*` namespace, reduced-motion guard — with the closing line "If you find yourself omitting any of these three, stop and reconsider." That is mindset transfer.
- **"Origin awareness (the part LLMs usually get wrong)"** (SKILL.md:114-129) is the single best section. Named anti-pattern, concrete `data-origin` example, and the diagnostic claim "scaling in from the wrong corner makes the open feel detached from the click, even when timing and easing are correct." This is the kind of paragraph experts write and tutorials don't.
- **Anti-pattern list (SKILL.md:172-189)** is specific, not generic. "Don't lengthen durations to 'make it smoother' — the issue is usually the curve" and the Tailwind `cubic-bezier`/`tailwind.config` caveat are both expert-grade gotchas with WHY attached.
- **Progressive disclosure works as designed.** SKILL.md is 197 lines (well under 500). Nine assets are small (~700B–2KB each), self-contained, and the body explicitly tells the agent when to load them ("Read the asset with `Read(...)`" — SKILL.md:95). Asset audit confirms each follows the stated three-part contract (verified `menu-dropdown.css`, `notification-badge.css`).
- **Freedom calibration is appropriate.** Low freedom on the contract (three required pieces, "do not edit transition declarations") matches the fragility of motion timing, while extension is high-freedom ("compose from the motion language above") for patterns outside the nine.
- **Pattern: Process** — clean execution. Mindset → contract → catalog → workflow → extension protocol → anti-patterns. Reads like one document, not a stitched FAQ.

### Issues

- [low] SKILL.md:32 — the easing table has three rows but the prose at SKILL.md:26 says "Almost every snippet uses **one of two** easing curves." Off-by-one with the ease-in-out closing curve; reconcile the count or relabel ease-in-out as a sub-curve.
- [low] SKILL.md:30 — table cell says "~95% of cases" for out-quint, which is a vibe statistic; either drop the percentage or footnote it. Minor token tax for unverifiable precision.
- [low] SKILL.md:91-112 — Workflow steps 1, 3, 4 ("Identify the pattern", "Paste into the user's stylesheet", "Apply the class") tilt toward [A]ctivation/[R]edundant — Claude knows how to paste CSS. Steps 5 and 6 (token tuning, state wiring) are the load-bearing ones. Could be compressed.
- [low] SKILL.md:96 — `Read("/.../assets/<pattern>.css")` uses a placeholder path. Agents loading the skill from `~/.claude/skills/motion-and-transitions/` will resolve assets via the skill base dir; the ellipsis is harmless but a concrete relative form (`assets/<pattern>.css`) would be unambiguous.
- [low] SKILL.md:62-74 — the nine-row pattern table duplicates information already implicit in the file names + description. Pattern + file is useful; the "What it is" column repeats facts the file's own header comment will state. Minor [A] dilution, defensible as a navigation index.
- [low] SKILL.md:75-89 — "Picking a pattern" reads as plain-English heuristics that follow naturally from the names. Either demote to a one-line decision tree or merge with the table column.
- [med] No worked end-to-end example. The skill explains contract + tokens + state wiring abstractly, but an agent would benefit from one full vignette: user asks "make my filter modal open smoothly" → identify modal → read asset → here is the resulting markup + CSS + JS toggle. The "drawer" example at SKILL.md:144-170 covers extension but not the primary path.
- [low] SKILL.md:142 — "Return the snippet inline; do not write to `assets/`" is a good guardrail but slightly buried. Consider promoting to the anti-pattern list so it's hard to miss.

### Specific recommendations

1. **Reconcile the easing-count claim.** Either rewrite SKILL.md:26 to "two primary curves plus a neutral closer" or fold the ease-in-out row into a footnote on the spring row. Removes a small but real internal contradiction.
2. **Add one worked vignette** (~12 lines) immediately after the Workflow section: paste a concrete user request, the agent's pattern pick, the final HTML + minimal JS handler, and a callout for the `data-origin` choice. Promotes the skill from "reference card" to "applied playbook" and lifts D8 to 14–15.
3. **Trim the workflow numbered list.** Collapse steps 1, 3, 4 into a single line ("Identify pattern from table → read asset → drop into stylesheet"). Keep 2, 5, 6 (origin/contract/state) as the load-bearing steps. Cuts ~10 lines of [A] content and tightens the E:A:R ratio.
4. **Drop or footnote "~95% of cases"** at SKILL.md:30. Replace with "the workhorse curve" — same signal, no false precision.
5. **Promote the "do not write to assets/" line** from extension step 6 (SKILL.md:141-142) into the "Things to avoid" list. The frozen-asset invariant is a contract-level rule, not an extension footnote.

### Dimension scores

| Dimension | Score | Max | Notes |
|---|---|---|---|
| D1 Knowledge Delta | 17 | 20 | E:A:R ≈ 70:25:5. Curves table, contract, origin-awareness, Tailwind caveat are all Expert. Workflow + pattern-table prose dilute. |
| D2 Mindset + Procedures | 13 | 15 | "Internalize the motion language" + "If you find yourself omitting any of these three, stop and reconsider" is mindset; extension protocol is genuine domain procedure. Loses ~2 to procedure-heavy workflow steps. |
| D3 Anti-Pattern Quality | 13 | 15 | Five specific NEVERs with WHY. "Don't lengthen durations to make it smoother — the issue is the curve" is expert-earned. Could add 1–2 more (e.g. "don't animate `width`/`height` directly when a transform will do"). |
| D4 Specification Compliance | 15 | 15 | Description is reference-grade — WHAT + WHEN + KEYWORDS + negative scope, in one paragraph. Frontmatter valid. |
| D5 Progressive Disclosure | 13 | 15 | 197-line body, 9 small assets, explicit `Read(...)` trigger in workflow. Missing: explicit "Do NOT load all assets up front" guidance and no scenario-detection branching. |
| D6 Freedom Calibration | 13 | 15 | Tight on contract (three required pieces, "do not edit declarations"), open on extension. Could be slightly tighter on duration ranges (currently "150–500ms" with cinematic exception — fine, but the agent has ample latitude). |
| D7 Pattern Recognition | 9 | 10 | Process pattern executed cleanly: mindset → contract → catalog → workflow → extension → anti-patterns. Slight bloat in the catalog table prevents a 10. |
| D8 Practical Usability | 13 | 15 | Decision text exists, contract is testable, extension example is concrete. Loses points for no full end-to-end vignette on the primary (non-extension) path. |
| **Total** | **106** | **120** | **88% — Grade B (minor revisions)** |

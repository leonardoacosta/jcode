# Agent Naming — 2026-07-11 Decision Record

> Moved from global `CLAUDE.md` § 8 by the CLAUDE.md-split pattern (`rules/TOOLING.md` §
> CLAUDE.md-Split Pattern) — historical justification for a naming-convention call, consulted
> only when re-litigating agent-suffix vocabulary. `CLAUDE.md` § 8 keeps the live naming rule
> plus a one-line pointer here.

advisor-plans 032 (move 6) proposed renaming `-auditor`/`-validator`/`tdd-*` agents to the
strict 5-suffix set (`-analyst/-architect/-engineer/-reviewer/-specialist`); advisor-plans 033
(move 13) proposed this amendment instead, arguing renaming ~10 agents across every dispatch
site (including the `SubagentStart` matcher regex in `settings.json`) is high blast-radius
churn for zero behavior change — the same anti-pattern CLAUDE.md's Named Failure Modes #18
(invented agent names) warns against, just self-inflicted via a mass rename instead of a
hallucination. Leo chose the amendment. See `advisor-plans/032-cowork-consultant-roadmap.md`
§ Move 6 and `advisor-plans/033-cowork-audit-2-structural-integrity.md` Part B for full
rationale.

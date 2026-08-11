---
name: verification-before-completion
description: Use when about to claim work is complete, fixed, or passing, before committing or creating PRs - requires running verification commands and confirming output before making any success claims; evidence before assertions always
source: ~/.agents/skills@2026-07-13
---


# Verification Before Completion

## The Iron Law

```
NO COMPLETION CLAIMS WITHOUT FRESH VERIFICATION EVIDENCE
```

If you haven't run the verification command in this message, you cannot claim it passes.
Evidence before claims, always — identify what command proves the claim, run it fresh, read the
full output, then state the claim WITH that evidence. Skip any step and it's not verification.

The value of this skill isn't the principle above — every model already "knows" evidence beats
assertion. The value is the tables below and in `references/`: they name the specific
rationalizations and specific insufficient-evidence patterns that let a claim slip through anyway.

## Common Failures (what a claim actually requires)

| Claim | Requires | Not Sufficient |
|-------|----------|----------------|
| Tests pass | Test command output: 0 failures | Previous run, "should pass" |
| Linter clean | Linter output: 0 errors | Partial check, extrapolation |
| Build succeeds | Build command: exit 0 | Linter passing, logs look good |
| Bug fixed | Test original symptom: passes | Code changed, assumed fixed |
| Regression test works | Red-green cycle verified | Test passes once |
| Agent completed | VCS diff shows changes | Agent reports "success" |
| Requirements met | Line-by-line checklist | Tests passing |

## Red Flags - STOP

- Using "should", "probably", "seems to"
- Expressing satisfaction before verification ("Great!", "Perfect!", "Done!")
- About to commit/push/PR without verification
- Trusting agent success reports
- Relying on partial verification
- Thinking "just this once"
- **ANY wording implying success without having run verification**

MANDATORY: Read [references/rationalization-red-flags.md](references/rationalization-red-flags.md)
before treating any excuse as a genuine exception to the rule above — it has the full
excuse -> reality table, including the "different words so it doesn't apply" case, which is
the one that catches paraphrases the list above doesn't spell out verbatim.

## Key Patterns

**Tests:**
```
✅ [Run test command] [See: 34/34 pass] "All tests pass"
❌ "Should pass now" / "Looks correct"
```

**Regression tests (TDD Red-Green):**
```
✅ Write → Run (pass) → Revert fix → Run (MUST FAIL) → Restore → Run (pass)
❌ "I've written a regression test" (without red-green verification)
```

**Build:**
```
✅ [Run build] [See: exit 0] "Build passes"
❌ "Linter passed" (linter doesn't check compilation)
```

**Requirements:**
```
✅ Re-read plan → Create checklist → Verify each → Report gaps or completion
❌ "Tests pass, phase complete"
```

**Agent delegation:**
```
✅ Agent reports success → Check VCS diff → Verify changes → Report actual state
❌ Trust agent report
```

MANDATORY: Read [references/worked-example.md](references/worked-example.md) when you need a
concrete end-to-end illustration of the identify -> run fresh -> read full output -> claim chain
— walks a real claim ("endpoint returns 404") from hollow source-reading through an actual curl
verification, and shows how the same chain shape carries over to build/test claims.

## When To Apply

Always, before: any completion/success claim, any expression of satisfaction, committing,
PR creation, moving to the next task, or delegating to (and trusting) another agent's report.
Applies to exact phrases, paraphrases, and implications — not just the literal words above.

## The Bottom Line

Run the command. Read the output. THEN claim the result. No shortcuts.

## References

- [references/rationalization-red-flags.md](references/rationalization-red-flags.md) — MANDATORY before accepting any excuse as an exception: full excuse -> reality table
- [references/worked-example.md](references/worked-example.md) — MANDATORY when you need a concrete verification-chain walkthrough

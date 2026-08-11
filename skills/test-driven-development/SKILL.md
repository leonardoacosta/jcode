---
name: test-driven-development
description: Use when implementing any feature or bugfix, before writing implementation code
source: ~/.agents/skills@2026-07-13
---


# Test-Driven Development (TDD)

## The Iron Law

```
NO PRODUCTION CODE WITHOUT A FAILING TEST FIRST
```

Wrote code before the test? Delete it — don't keep it "as reference," don't "adapt" it while
writing tests, don't look at it. Delete means delete. Implement fresh from tests.

## Why Knowing the Rule Isn't Enough

Every model can recite RED-GREEN-REFACTOR. That was never the failure mode. The failure mode is
a single agent, mid-task, under its own pressure to "just solve it," writing test and
implementation together and then rationalizing that the ordering doesn't matter this once —
because nothing external forces it to prove RED actually happened. A model self-certifying its
own test-first claim is unreliable by construction: it has every incentive to rationalize
("tests after achieve the same goal," "too simple to test") and no check on whether it did.

**What actually enforces this in this repo**: the `tdd-integration` three-agent loop
(`tdd-test-writer` -> `tdd-implementer` -> `tdd-refactorer`). Each agent runs in its own isolated
context — `tdd-implementer` never sees whether RED genuinely failed, only what
`tdd-test-writer` reported, and the orchestrator withholds the next dispatch until the previous
phase's return proves its claim. No single agent can self-certify its own phase transition.
Reach for that loop whenever the harness has it wired (`Skill({ skill: "tdd-integration" })`);
this skill is the discipline the loop mechanically enforces, for the cases where you're running
it solo.

## The Cycle, Briefly

1. **RED** — one minimal test for one behavior. Run it (`npm test path/to/test.test.ts`).
   Confirm it fails for the RIGHT reason (feature missing, not a typo). Passes immediately?
   You're testing existing behavior — fix the test, not the code.
2. **GREEN** — simplest code that passes. No added features, no "while I'm here" scope, no
   options nobody asked for. Run the test again; confirm it passes AND nothing else broke.
3. **REFACTOR** — clean up only with green tests as a safety net (remove duplication, improve
   names, extract helpers). No new behavior.
4. **Repeat** — one loop per acceptance criterion, never batched. Batching hides which
   criterion a given RED/GREEN pair is actually proving.

## Common Rationalizations

| Excuse | Reality |
|--------|---------|
| "Too simple to test" | Simple code breaks. Test takes 30 seconds. |
| "I'll test after" | Tests passing immediately prove nothing. |
| "Tests after achieve same goals" | Tests-after = "what does this do?" Tests-first = "what should this do?" |
| "Already manually tested" | Ad-hoc ≠ systematic. No record, can't re-run. |
| "Deleting X hours is wasteful" | Sunk cost fallacy. Keeping unverified code is technical debt. |
| "Keep as reference, write tests first" | You'll adapt it. That's testing after. Delete means delete. |
| "Need to explore first" | Fine. Throw away exploration, start with TDD. |
| "Test hard = design unclear" | Listen to test. Hard to test = hard to use. |
| "TDD will slow me down" | TDD faster than debugging. Pragmatic = test-first. |
| "Existing code has no tests" | You're improving it. Add tests for existing code. |

## Red Flags - STOP and Start Over

- Code before test
- Test after implementation
- Test passes immediately
- Can't explain why test failed
- Rationalizing "just this once"
- "Keep as reference" or "adapt existing code"
- "Already spent X hours, deleting is wasteful"
- "This is different because..."

**All of these mean: delete code, start over with TDD.**

## Verification Checklist

Before marking work complete:

- [ ] Every new function/method has a test
- [ ] Watched each test fail before implementing
- [ ] Each test failed for the expected reason (feature missing, not a typo)
- [ ] Wrote minimal code to pass each test
- [ ] All tests pass, output pristine (no errors, warnings)
- [ ] Tests use real code (mocks only if unavoidable)
- [ ] Edge cases and errors covered

Can't check all boxes? You skipped TDD. Start over.

## When Stuck

| Problem | Solution |
|---------|----------|
| Don't know how to test | Write the wished-for API, write the assertion first. Ask. |
| Test too complicated | Design too complicated — simplify the interface. |
| Must mock everything | Code too coupled — use dependency injection. |
| Test setup huge | Extract helpers. Still complex? Simplify design. |

## Debugging Integration

Bug found? Write a failing test reproducing it first, then follow the cycle above. Never fix a
bug without a test — see `systematic-debugging` Phase 4.

## Testing Anti-Patterns

When adding mocks or test utilities, read @testing-anti-patterns.md to avoid: testing mock
behavior instead of real behavior, adding test-only methods to production classes, and mocking
without understanding dependencies.

## Related Skills

| Skill | When |
|---|---|
| `tdd-integration` | The 3-agent isolation mechanism that actually enforces this discipline when the loop is wired |
| `systematic-debugging` | Phase 4 uses this cycle to create the failing test case that proves a fix |
| `verification-before-completion` | Same self-certification problem, one layer up — evidence before any completion claim |

---
name: systematic-debugging
description: Use when encountering any bug, test failure, or unexpected behavior, before proposing fixes
source: ~/.agents/skills@2026-07-13
---


# Systematic Debugging

## The Iron Law

```
NO FIXES WITHOUT ROOT CAUSE INVESTIGATION FIRST
```

If Phase 1 below isn't complete, you cannot propose fixes.

## Effort Gates

Adaptive depth based on `${CLAUDE_EFFORT}` (v2.1.120+):

| Effort | Protocol |
|---|---|
| `low` | 3-step fast path: (1) capture the exact error, (2) reproduce minimally, (3) form one hypothesis and test it. Skip the full protocol for trivial single-file bugs. |
| `medium` and above | Full protocol below. Root-cause discipline is non-negotiable at default effort and higher. |

Low effort signals "quick answer wanted" — bypassing the full protocol risks symptom-fixing but
matches intent. Default to the full protocol when effort is unset.

## The Real Discipline: When to Stop Fixing and Question the Architecture

The phases below are ordinary root-cause hygiene — most of it you already do. The part worth
internalizing is the escalation rule, because it's the one part that isn't obvious in the
moment: **3+ failed fix attempts on the same bug is a different problem than "try one more."**

If each fix reveals new shared state or coupling in a different place, requires "massive
refactoring" to implement, or creates new symptoms elsewhere — that's not a failed hypothesis,
that's a wrong architecture. STOP before Fix #4. Question whether the pattern is fundamentally
sound, or whether you're sticking with it through sheer inertia. Discuss with the user before
attempting more fixes; don't quietly keep patching.

## The Four Phases (condensed)

1. **Root Cause** — read the full error/stack trace (it usually names the fix), reproduce
   reliably (not reproducible? gather more data, don't guess), check what actually changed
   (git diff/log, new deps, config, environment).

   **In multi-component systems** (CI -> build -> signing, API -> service -> DB), add boundary
   logging BEFORE guessing which layer breaks — log what enters/exits each boundary, run once,
   read where it actually breaks, then investigate that specific component:
   ```bash
   echo "=== Secrets available in workflow: ===";  echo "IDENTITY: ${IDENTITY:+SET}${IDENTITY:-UNSET}"
   echo "=== Env vars in build script: ===";        env | grep IDENTITY || echo "not in environment"
   echo "=== Keychain state: ===";                  security list-keychains; security find-identity -v
   ```
   This reveals which layer fails (e.g. secrets -> workflow OK, workflow -> build broken) instead
   of guessing at the wrong component. For errors deep in a call stack, trace backward — see
   `root-cause-tracing.md` in this directory.

2. **Pattern** — find a working example of the same pattern elsewhere in this codebase, read it
   completely (not skimmed), and list every difference from the broken instance — don't assume
   "that can't matter."

3. **Hypothesis** — state one hypothesis ("I think X because Y"), test the smallest possible
   change, one variable at a time. Didn't work? Form a NEW hypothesis — don't stack fixes on top
   of an unconfirmed one.

4. **Implementation** — write a failing test reproducing the bug first (`test-driven-development`
   skill), make the single fix, verify it and that nothing else broke. If it doesn't hold, apply
   the escalation rule above rather than trying Fix #4.

## Red Flags - STOP and Follow Process

- "Quick fix for now, investigate later" / "just try changing X and see"
- Multiple changes at once, then run tests
- "It's probably X, let me fix that" (without having traced data flow)
- **"One more fix attempt" when 2+ have already failed**
- **Each fix reveals a new problem in a different place**

## Common Rationalizations

| Excuse | Reality |
|--------|---------|
| "Issue is simple, don't need process" | Simple issues have root causes too. |
| "Emergency, no time for process" | Systematic debugging is faster than guess-and-check thrashing. |
| "Multiple fixes at once saves time" | Can't isolate what worked — causes new bugs. |
| "Reference too long, I'll adapt the pattern" | Partial understanding guarantees bugs. Read it completely. |
| "I see the problem, let me fix it" | Seeing symptoms ≠ understanding root cause. |
| "One more fix attempt" (after 2+ failures) | 3+ failures = architectural problem, not a hypothesis to retry. |

## When Investigation Reveals No Fixable Root Cause

If it's genuinely environmental, timing-dependent, or external: document what you investigated,
implement appropriate handling (retry, timeout, error message), add monitoring for next time.
Most "no root cause" verdicts are an incomplete investigation, not a real one — check you
actually finished Phase 1 before landing here.

## Supporting Techniques (this directory)

- **`root-cause-tracing.md`** — trace a bug backward through the call stack to its origin
- **`defense-in-depth.md`** — add validation at multiple layers after finding root cause
- **`condition-based-waiting.md`** — replace arbitrary timeouts with condition polling

## Related Skills

- `test-driven-development` — creating the failing test case in Phase 4
- `verification-before-completion` — verify the fix worked before claiming it did

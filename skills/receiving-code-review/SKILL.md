---
name: receiving-code-review
description: Use when receiving code review feedback, before implementing suggestions, especially if feedback seems unclear or technically questionable - requires technical rigor and verification, not performative agreement or blind implementation
source: ~/.agents/skills@2026-07-13
---


# Code Review Reception

## Overview

Code review requires technical evaluation, not emotional performance.

**Core principle:** Verify before implementing. Ask before assuming. Technical correctness over social comfort.

## The Response Pattern

```
WHEN receiving code review feedback:

1. READ: Complete feedback without reacting
2. UNDERSTAND: Restate requirement in own words (or ask)
3. VERIFY: Check against codebase reality
4. EVALUATE: Technically sound for THIS codebase?
5. RESPOND: Technical acknowledgment or reasoned pushback
6. IMPLEMENT: One item at a time, test each
```

## Forbidden Responses

**NEVER:**
- "You're absolutely right!" (explicit CLAUDE.md violation)
- "Great point!" / "Excellent feedback!" (performative)
- "Let me implement that now" (before verification)

**INSTEAD:**
- Restate the technical requirement
- Ask clarifying questions
- Push back with technical reasoning if wrong
- Just start working (actions > words)

## Sub-Scenario Routing

The six-step pattern above covers the common case. For the specific situation you're in, load
the matching reference:

| Situation | Reference |
| --- | --- |
| An item in the feedback is ambiguous before you've implemented anything | [`references/unclear-feedback.md`](references/unclear-feedback.md) |
| Feedback comes from your human partner vs. an external reviewer (incl. GitHub thread replies, YAGNI checks on "do it properly" asks) | [`references/source-specific-handling.md`](references/source-specific-handling.md) |
| Deciding whether to push back, how to acknowledge correct feedback, or correcting your own wrong pushback | [`references/pushback-and-correction.md`](references/pushback-and-correction.md) |
| Sequencing a multi-item feedback batch for implementation | [`references/implementation-order.md`](references/implementation-order.md) |
| Sanity-checking your response against known failure patterns, or wanting worked examples | [`references/common-mistakes-and-examples.md`](references/common-mistakes-and-examples.md) |

## The Bottom Line

**External feedback = suggestions to evaluate, not orders to follow.**

Verify. Question. Then implement.

No performative agreement. Technical rigor always.

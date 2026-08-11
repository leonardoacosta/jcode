
# Plan-Writing Anti-Patterns (Beyond Placeholders)

> Read this when: you're about to run the Self-Review anti-pattern scan (see
> [`self-review-checklist.md`](self-review-checklist.md) step 2), or want to sanity-check a task
> you just wrote before moving to the next one.

Beyond placeholders (see main `SKILL.md` § No Placeholders), these failure modes are subtler
because the plan *looks* complete — the gap only surfaces once an engineer is implementing it:

- **Vague acceptance criteria** ("should work correctly", "handle appropriately", "reasonable
  performance"). The engineer and whoever reviews their PR will each independently guess at the
  missing threshold — usually different guesses — and the mismatch surfaces as rework only after
  the code is already written to the wrong interpretation.
- **No rollback/rollout note on any schema- or infra-affecting task.** A plan that specifies the
  forward migration but not how to reverse it turns a bad deploy into an incident instead of a
  revert — the engineer executing under time pressure has no fallback to reach for.
- **Hidden reverse dependencies in task ordering** (Task 5 assumes a type or function Task 8
  defines). A subagent-driven or sequential executor runs tasks in the order given, not the order
  that would actually compile — an undeclared reverse dependency produces a confusing failure
  that reads as the engineer's mistake, not the plan's.
- **Test instructions that name a framework or assertion style not actually used in this
  codebase.** The engineer either fights the plan to match reality or silently substitutes their
  own judgment — either way it defeats the "assume zero context" premise the whole plan depends
  on to be followed literally.


# Self-Review Checklist

> Read this when: you've just finished writing the complete plan and are about to check it
> against the spec before offering the Execution Handoff (or before dispatching the Optional
> Independent Subagent Review).

After writing the complete plan, look at the spec with fresh eyes and check the plan against it.
This is a checklist you run yourself — not a subagent dispatch.

**1. Spec coverage:** Skim each section/requirement in the spec. Can you point to a task that
implements it? List any gaps.

**2. Anti-pattern scan:** Search your plan for red flags from both "No Placeholders" (main
`SKILL.md`) and [`anti-patterns.md`](anti-patterns.md) — vague acceptance criteria, missing
rollback notes, hidden reverse dependencies, mismatched test frameworks. Fix them.

**3. Type consistency:** Do the types, method signatures, and property names you used in later
tasks match what you defined in earlier tasks? A function called `clearLayers()` in Task 3 but
`clearFullLayers()` in Task 7 is a bug.

If you find issues, fix them inline. No need to re-review — just fix and move on. If you find a
spec requirement with no task, add the task.

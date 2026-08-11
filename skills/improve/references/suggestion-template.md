# Suggestion Contract

Write every selected suggestion for an executor with zero context from the audit session.

## Contract shape

```markdown
- **Suggestion**: <one imperative sentence>
  - **Reasoning**: <verified file:line evidence and why it matters now>
  - **Definition of Done**: mechanical (<command and expected result>);
    behavior (<observable runtime outcome>); done-when (<canonical completion state>)
  - **Watch out**: <blast radius, false-positive conditions, review focus>
  - **Route**: attach `<existing-id>` | feature `<slug>` | ad-hoc task | research/decision map
```

## Required execution context

- **Base commit and drift check**: record the audited revision. Tell the executor to stop if cited
  locations or assumptions have changed enough to invalidate the steps.
- **Evidence**: include exact paths, symbols, and short current-state excerpts verified directly by
  the advisor.
- **Exemplar**: cite a real `path:line` whose structure or convention should be followed. State
  explicitly when no suitable precedent exists.
- **Boundaries**: list files in scope, related files excluded from scope, and the reason for each
  important exclusion.
- **Ordered steps**: make dependencies explicit and give each step a verification command plus its
  expected result.
- **Test plan**: name the behavior to prove, where the test belongs, and an existing test to copy
  structurally when one exists.
- **STOP conditions**: name concrete contradictions, repeated gate failures, missing authority, or
  changed assumptions that require escalation instead of improvisation.
- **Maintenance note**: identify future changes likely to interact with this work.

## Route selection

- `attach`: the outcome is already owned; add evidence to that owner.
- `feature`: the work changes a capability, spans coordinated files, needs a design choice, or
  benefits from staged gates.
- `ad-hoc task`: the fix is bounded and needs no design decision.
- `research/decision map`: uncertainty prevents an honest implementation contract.

Never create state merely because a suggestion ranked highly. Wait for explicit capture or
execution intent, then use the repository's canonical OpenSpec and Beads workflow.

## Quality check

Before routing, confirm that a fresh executor can answer all of these from the contract alone:

- What exact behavior is wrong or missing, and where is the evidence?
- What is in scope and out of scope?
- What should be implemented first, and what must already be complete?
- Which command proves structural success, and what observable behavior proves runtime success?
- What condition closes the OpenSpec change or tracker task?
- When must execution stop and return for a decision?

If any answer depends on conversation context, add it to the contract.

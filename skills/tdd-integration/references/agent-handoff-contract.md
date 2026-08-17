# Agent Handoff Contract (RED-GREEN-REFACTOR)

> Deep-dive for `tdd-integration` SKILL.md § The Three Agents + Handoff Contract. MANDATORY read
> before dispatching the loop for the first time in a session — the field names below are the
> actual contract each agent is graded against, not illustrative examples.

## The Three Agents + Handoff Contract

| Phase | Agent | Job | Prohibited | Returns |
|---|---|---|---|---|
| RED | `tdd-test-writer` | Write **one** failing test, prove it fails | Touching implementation; multiple tests | `test_path`, `test_name`, `failure_mode`, `failure_output`, `summary`, `implementation_target` |
| GREEN | `tdd-implementer` | Minimum code to pass | Touching the test; refactoring | `test_path`, `files_modified[]`, `test_output`, `summary`, `noteworthy` |
| REFACTOR | `tdd-refactorer` | Evaluate cleanup; act only on clear win | Speculative refactors; multi-file rewrites | `decision` (refactored/skipped), `reasoning`, `files_modified[]`, `test_output`, `summary` |

The orchestrator splices these returns into spec/journal records as appropriate.

## Orchestrator Workflow

```
for each acceptance_criterion in spec:
  red_result = Agent(tdd-test-writer, criterion)
  assert red_result.failure_mode in ["assertion_error", "missing_import", "missing_function"]
  green_result = Agent(tdd-implementer, red_result)
  assert green_result.test_output contains "PASS" or equivalent
  refactor_result = Agent(tdd-refactorer, green_result)
  if refactor_result.decision == "refactored":
    assert refactor_result.test_output contains passing tests for the package
  log {criterion, red_result, green_result, refactor_result}
```

Each `assert` line above is a real gate, not pseudocode flavor — it's the mechanical form of the
Gate Rules in SKILL.md § Gate Rules. If an agent's return is missing a required field or the
asserted condition doesn't hold, do not advance to the next phase; re-dispatch the current phase
with the gap as the signal.

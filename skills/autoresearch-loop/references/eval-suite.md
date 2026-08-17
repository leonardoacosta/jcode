# Evaluation Suite Contract

Build the suite before candidate mutation. Give it a stable identity and a digest over every input
that can affect the oracle, including prompts, assertions, weights, fixtures, metric configuration,
and ordering where ordering changes behavior.

## Freeze the suite

Record the suite identity and digest in the run descriptor, then treat the suite as immutable for
the run. Changing a prompt, assertion, weight, fixture, metric input, or digest invalidates further
comparisons. Preserve the current audit trail and prepare a new suite and run instead.

The digest covers the effective bytes, not merely a filename or revision label. Verify those bytes
again before each evaluation. If the suite cannot be reconstructed from its recorded identity and
digest, stop the run rather than accepting an approximate oracle.

## Prefer discriminating checks

- Connect each assertion to the target behavior and hypothesis it can evaluate.
- Include non-regression checks for behavior the candidate must preserve.
- Prefer objective observations over stylistic impressions in the automated oracle.
- Keep human judgment for the terminal review rather than silently converting it into a score.
- Reject a suite that can pass without exercising the selected atom.

Record the baseline behavior of the unmodified target against the same frozen inputs. Assertions
may have different declared weights, but neither their meaning nor their weights can change after
the run begins.

## Prepare a new suite deliberately

When existing coverage is inadequate, end any active run before editing suite inputs. Describe the
behavioral gap, add or repair the smallest discriminating prompt, assertion, or fixture, review the
metric and ordering effects, and freeze a new suite digest. Measurements from the earlier suite
remain in its audit trail and are not compared to measurements from the new suite.

The frozen suite is an evaluation input, never a candidate-edit surface.

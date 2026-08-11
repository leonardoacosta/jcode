---
name: autoresearch-loop
description: Optimize one bounded instruction or reference atom against a frozen evaluation suite with an auditable KEEP/REVERT ratchet. Use when prose behavior needs measured iterative improvement, explicit budgets, restoration after regressions, and human review; route deterministic code behavior to ordinary tests instead.
---

# Autoresearch Loop

Use this method to improve one instruction atom without turning experimentation into an unbounded
rewrite. The method owns selection, immutability, ratcheting, and review outcomes. A harness adapter
supplies execution and isolation mechanics.

## Select one eligible atom

Classify the proposed target before spending evaluation budget:

- **L1 — deterministic behavior:** route code, schemas, and mechanically provable behavior to
  ordinary deterministic tests instead of an autoresearch run.
- **L2 — bounded instruction behavior:** accept one mutable atom with a bounded diff, a frozen
  suite, and one accountable owner. This is the default eligible level.
- **L3 — orchestration behavior:** refuse multi-atom or interacting workflow changes unless a
  separately reviewed adapter contract provides an equivalent bounded oracle and isolation
  boundary. Identify a smaller L2 atom when possible.

Do not start when ownership, restoration, or the evaluation oracle is ambiguous.

## Prepare optional research

Research is optional. Start a no-research run whenever the frozen suite already provides an
adequate oracle. When better public evidence would materially improve hypotheses or a future
suite, complete the preparation sequence in
[the harness-adapter contract](references/harness-adapter.md) before freezing the run descriptor.

Research can shape hypotheses or preparation of a new suite, but it never supplies live evaluation
input. Only frozen suite measurements participate in the KEEP/REVERT oracle.

## Run the method

1. Name the target atom, accountable owner, hypothesis, and expected behavioral effect.
2. Optionally prepare immutable research evidence, then prepare and freeze the evaluation suite according to
   [the evaluation-suite contract](references/eval-suite.md).
3. Persist the run descriptor and confirm adapter capabilities using
   [the harness-adapter contract](references/harness-adapter.md).
4. Measure the unmodified target to establish the first comparison baseline.
5. Apply one bounded candidate, evaluate it against the frozen suite, and record measurements.
6. Classify the candidate with [the ratchet contract](references/ratchet.md). KEEP advances the
   baseline; REVERT restores the last KEEP bytes.
7. Stop at the declared plateau or budget boundary, then hand the cumulative result to a human.

## Preserve the run boundary

- Keep exactly one mutable atom and one accountable owner for the entire run.
- Treat the frozen suite and every oracle-affecting descriptor input as immutable after the first
  candidate mutation.
- Compare candidates only through the declared metric and non-regression rule.
- Persist both KEEP and REVERT outcomes so interruption recovery never depends on chat history.
- Require human review before integrating cumulative KEEPs into the target branch.

Start a new run when the target baseline, suite bytes, adapter version, metric, or another frozen
input changes.

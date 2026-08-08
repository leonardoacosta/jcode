# Refine Task Decomposition Eval Rubric

## Why

The task-decomposition eval currently treats the historical OpenSpec change as a gold artifact and exposes deterministic token overlap as a prominent score. The first pilot showed that this can mislead the evaluation: Jcode can produce a coherent, valid plan while scoring poorly because the candidate artifact shape or reconstructed prompt scope differs from the historical change. The experiment should instead judge whether Jcode preserved the user's intent, locked scope correctly, and understood blast radius.

## What Changes

- Reframe historical OpenSpec proposal commits as reference evidence, not reproduction targets.
- Add an intent contract for each fixture that records user intent, hard scope boundaries, expected blast-radius surfaces, non-goals, and ambiguity traps.
- Replace the primary rubric dimensions with planning-quality dimensions: fidelity, scope lock, blast-radius discovery, risk/dependency ordering, and executable verification.
- Add deterministic evidence extraction that supports reviewer judgment without becoming the final judge.
- Update documentation so future pilots explain when low artifact overlap indicates a shape or prompt mismatch rather than poor task decomposition.

## Non-goals

- Do not run new live evals in this change.
- Do not require model-based judging.
- Do not remove the existing artifact-overlap report immediately; demote it to supporting evidence.
- Do not make Jcode reproduce historical proposal structure or exact file counts.

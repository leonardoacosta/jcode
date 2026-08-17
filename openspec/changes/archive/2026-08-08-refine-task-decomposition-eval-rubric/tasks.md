# Tasks

## Batch 1: Contract and schema

- [x] Add `intent_contract` to the fixture schema.
- [x] Populate intent contracts for the pilot fixtures first.
- [x] Add catalog validation coverage for missing or malformed contract fields.

## Batch 2: Rubric transition

- [x] Update rubric documentation to make fidelity, scope lock, blast-radius discovery, risk/dependency ordering, and verification executability the primary dimensions.
- [x] Update rubric score validation to accept the new dimensions.
- [x] Add or update sample rubric score records for the new scoring model.

## Batch 3: Evidence support

- [x] Demote artifact token overlap in docs to supporting evidence.
- [x] Add deterministic evidence extraction for historical changed surfaces and candidate surface mentions.
- [x] Report non-goal violations and ambiguity-trap warnings as evidence for human review.

## Batch 4: Verification and closeout

- [x] Run Python compilation and unit tests for the eval script.
- [x] Run fixture catalog validation and prompt catalog validation.
- [x] Run rubric score validation on a new planning-quality sample.
- [x] Run OpenSpec strict validation.
- [x] Commit the completed change with only owned paths.

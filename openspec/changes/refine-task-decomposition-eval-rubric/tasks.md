# Tasks

## Batch 1: Contract and schema

- [ ] Add `intent_contract` to the fixture schema.
- [ ] Populate intent contracts for the pilot fixtures first.
- [ ] Add catalog validation coverage for missing or malformed contract fields.

## Batch 2: Rubric transition

- [ ] Update rubric documentation to make fidelity, scope lock, blast-radius discovery, risk/dependency ordering, and verification executability the primary dimensions.
- [ ] Update rubric score validation to accept the new dimensions.
- [ ] Add or update sample rubric score records for the new scoring model.

## Batch 3: Evidence support

- [ ] Demote artifact token overlap in docs to supporting evidence.
- [ ] Add deterministic evidence extraction for historical changed surfaces and candidate surface mentions.
- [ ] Report non-goal violations and ambiguity-trap warnings as evidence for human review.

## Batch 4: Verification and closeout

- [ ] Run Python compilation and unit tests for the eval script.
- [ ] Run fixture catalog validation and prompt catalog validation.
- [ ] Run rubric score validation on a new planning-quality sample.
- [ ] Run OpenSpec strict validation.
- [ ] Commit the completed change with only owned paths.

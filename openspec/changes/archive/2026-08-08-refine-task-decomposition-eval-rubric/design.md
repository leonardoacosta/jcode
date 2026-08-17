# Design

## Overview

This change turns the eval from a reproduction test into a planning-quality test. A fixture still points to a historical OpenSpec change, but that change is treated as one source of evidence about the real blast radius and constraints. The evaluator scores the candidate against an explicit fixture intent contract and uses deterministic repo evidence to support, not replace, semantic review.

## Fixture intent contract

Each fixture gains an `intent_contract` block with:

- `user_intent`: concise statement of what the original user wanted.
- `scope_boundaries`: required in-scope and out-of-scope boundaries.
- `expected_blast_radius`: routes, packages, config, data, auth, tests, docs, or operational surfaces the candidate should consider.
- `non_goals`: tempting expansions the candidate should avoid.
- `ambiguity_traps`: likely ways an agent might under-scope, over-scope, or mistake artifact shape for intent.
- `reference_notes`: how to use the historical OpenSpec commit as evidence without treating it as a reproduction target.

The contract is fixture-local and reviewable in git. It lets reconstructed prompts be imperfect while still making the intended evaluation target explicit.

## Scoring model

Rubric records replace the previous five dimensions with five planning-quality dimensions:

1. `fidelity`: Does the plan preserve the user request and domain intent?
2. `scope_lock`: Does it include required work while avoiding unauthorized expansion or narrowing?
3. `blast_radius`: Does it identify affected surfaces and integration boundaries deeply enough?
4. `risk_dependency_ordering`: Does it sequence prerequisites, risks, rollback, migration, and permission concerns correctly?
5. `verification_executability`: Are acceptance checks observable, ordered, and realistic for the affected surfaces?

Each dimension remains scored 1-5 with reviewer notes. The aggregate score is semantic.

## Deterministic evidence

The existing `score-artifacts` output stays available but is renamed or documented as support evidence. Future implementation should add an evidence extractor that compares a candidate plan against fixture contract terms and historical reference surfaces:

- Changed files and directories in the reference commit.
- OpenSpec specs and tasks touched by the reference change.
- Route, package, env/config, auth/permission, database, test, and docs surfaces inferred from paths.
- Candidate mentions of expected blast-radius surfaces and non-goal violations.

The extractor should emit facts and gaps, not a final pass/fail verdict.

## Failure handling

- Missing `intent_contract` fields fail catalog validation.
- Rubric records using old dimensions fail after the transition window or are accepted only with an explicit legacy mode.
- A candidate can score well with low overlap if it satisfies the intent contract and blast-radius evidence.
- A candidate with high overlap can still score poorly if it misses scope lock, risk, or verification needs.

## Verification

Verification should include schema validation, fixture catalog validation, rubric score validation for a sample planning-quality record, tests for missing contract rejection, docs checks, and OpenSpec strict validation.

# Evaluate Task Decomposition Quality

## Why

Jcode needs evidence that it decomposes, delegates, and plans work well before we rely on orchestration choices by default. We already mined validated historical OpenSpec proposal-only commits across diverse categories, but the fixture set is only scratch data. We need a repo-native eval surface that can preserve those situations, materialize base checkouts, and score generated proposal artifacts against gold OpenSpec proposal commits.

## What Changes

- Add a task-decomposition eval fixture catalog covering free design/product, business logic, infrastructure/config, E2E/test strategy, data migration, auth/security, observability, agent tooling, refactor/entropy, and UI polish categories.
- Add a fixture schema and validation command so the catalog can be checked without network or external packages.
- Add materialization support for creating isolated checkouts at each fixture's base commit from caller-provided local repo roots.
- Add scoring support for generated OpenSpec artifacts against each fixture's gold proposal commit using artifact completeness and content-overlap heuristics.
- Document category rubrics and the intended staged runner path without requiring live model execution in the first landing.

## Non-goals

- Do not run live model evaluations in this change.
- Do not vendor private project repositories or fixture source trees.
- Do not require non-stdlib Python packages.
- Do not make exact text match the primary quality measure.

# Adopt Fallow in a TypeScript monorepo

Use this template to plan a repository-local Fallow rollout. Replace `{WORKSPACE}` with the target
workspace scope and adjust commands for the repository's package manager.

## Context

The repository does not yet enforce dead-code, dependency, duplication, complexity, and boundary
regressions with Fallow. Adoption should establish a snapshot of existing findings and reject only
new regressions unless cleanup is explicitly in scope.

## Proposed changes

1. Copy `templates/fallowrc-t3-baseline.json` to `.fallowrc.json` and replace the example
   `{WORKSPACE}` scope.
2. Tune ignore patterns only for generated files and documented first-pass exceptions.
3. Create `.fallow/baseline.json` with `--save-regression-baseline`.
4. Add the `templates/ci-step.yml` gate to the existing CI workflow.
5. Add package scripts for local audit and CI regression checks.

## Tasks

- [ ] Add and review `.fallowrc.json`.
- [ ] Record the initial regression baseline.
- [ ] Add the CI job using the repository's package manager.
- [ ] Prove a synthetic unused export makes the regression gate fail.
- [ ] Remove the synthetic export and run the repository's normal quality gates.
- [ ] Document intentional deviations from the shared baseline.

## Acceptance criteria

The repository has a reviewed Fallow configuration, a committed baseline, and a CI check that
fails for newly introduced findings without failing solely for pre-existing debt.

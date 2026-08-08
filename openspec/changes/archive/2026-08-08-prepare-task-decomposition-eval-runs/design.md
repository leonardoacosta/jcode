# Design

## Overview

This change adds the missing pre-run layer between static fixtures and live evaluation. The layer stays deterministic and local: it validates that a fixture has enough prompt and repository context to be run later, but it does not materialize checkouts or invoke models.

## Components

### Prompt catalog

`evals/task-decomposition/prompts/catalog.json` records prompt metadata separately from fixture metadata so the fixture catalog remains stable and source-path-free. Each prompt record includes:

- `fixture_id`
- `kind`: `original` or `reconstructed`
- `confidence`: `high`, `medium`, or `low`
- `source`
- `prompt`
- `notes`

The first catalog entries cover the three intended pilot categories: free design, infra, and business logic.

### Prepare-run command

`prepare-run` validates the runnable envelope for one fixture and baseline mode. It verifies the local repo mapping, base and gold commits, prompt metadata, baseline mode, and output path. It emits JSON describing the run plan and explicitly marks `will_materialize: false` and `will_run_model: false`.

### Rubric score JSON

A rubric score JSON record captures human semantic review after a candidate exists. The validator checks fixture ID, baseline mode, reviewer, required dimensions, 1-5 score ranges, and dimension notes. This prepares review bookkeeping without requiring a model judge.

## Failure handling

- Missing prompt metadata fails `prepare-run` before any checkout or model call.
- Existing output paths fail preparation to avoid accidental overwrite.
- Unknown baseline modes fail through argparse and validation.
- Rubric score files fail with a structured JSON failure report.

## Verification

Verification stays pre-run only: unit tests, catalog validation, prompt validation, dry-run preparation against local repos, rubric score validation, OpenSpec strict validation, and Python compilation.

# Design: Task Decomposition Evals

## Overview

The eval suite is intentionally file-first. A fixture catalog records historical OpenSpec proposal-only commits, a Python utility validates and materializes them from local repo roots, and scoring compares a candidate output directory to the gold OpenSpec artifacts at the proposal commit.

This keeps the first landing deterministic and cheap while leaving room for later live model runs.

## Artifacts

```text
evals/task-decomposition/
  README.md
  fixtures/catalog.json
  fixtures/schema.json
  rubrics/*.md
scripts/eval_task_decomposition.py
```

## Fixture model

Each fixture has:

- stable `id`
- `category`
- `project`
- optional `remote`
- `base_commit`
- `gold_proposal_commit`
- `change_slug`
- `expected_artifacts`
- `notes`

The catalog does not store absolute local repo paths. The CLI accepts `--repo-root project=/path/to/repo` so private or local-only source locations remain operator supplied.

## CLI behavior

`validate-catalog` checks schema-level structure, unique IDs, commit hash format, required artifacts, and category coverage.

`materialize` clones from a supplied local repo root with `git clone --shared`, checks out the base commit, and writes `.jcode-eval-fixture.json` into the output checkout. It never deletes an existing output path.

`score-artifacts` compares a candidate OpenSpec change directory to the gold change directory from the source repo at `gold_proposal_commit`. The score combines required artifact presence, per-artifact token overlap, category rubric presence, and penalties for missing core files.

## Rationale

- Use Python stdlib to avoid adding runtime dependencies to the Rust repository.
- Require caller-supplied local repo roots to avoid encoding private machine paths in source.
- Start with deterministic fixture and scoring plumbing before adding live model invocation.
- Keep exact text similarity as a supporting signal rather than the main quality claim.

## Risks

- Some fixtures reference private repositories and cannot materialize without local access.
- Token overlap is not a semantic judge. Human or model-judge rubrics should be layered on later.
- Historical gold proposals are useful references, not perfect ground truth.

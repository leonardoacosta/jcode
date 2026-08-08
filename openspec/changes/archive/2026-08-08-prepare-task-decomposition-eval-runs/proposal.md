# Prepare Task Decomposition Eval Runs

## Why

The task-decomposition eval suite can catalog, materialize, and score fixtures, but it is not yet ready to execute faithful Jcode-vs-OpenSpec comparisons. Before running evals, operators need checked prompt metadata, explicit baseline modes, a dry-run preparation command that proves a fixture can be run without invoking models, and a structured place to record human rubric scores.

## What Changes

- Add pilot prompt metadata for the design, infra, and business-logic fixtures, with source, confidence, and reconstructed prompt text.
- Add a `prepare-run` command that validates fixture, repository, prompt, baseline mode, and output path without materializing or running a model.
- Add baseline mode names for gold OpenSpec, Jcode without OpenSpec, Jcode with OpenSpec, and Jcode with OpenSpec plus orchestration.
- Add rubric score JSON validation for human semantic review dimensions.
- Document the pre-run workflow and fix the archived eval spec purpose.

## Non-goals

- Do not run live evals or call Jcode from the eval script.
- Do not mine all original prompts in this change.
- Do not require non-stdlib Python packages.
- Do not replace semantic review with token overlap.

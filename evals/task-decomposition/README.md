# Task Decomposition Evals

This suite preserves historical OpenSpec proposal situations for evaluating how well Jcode decomposes ambiguous work into proposals, designs, tasks, and delta specs.

## Validate the catalog

```bash
python3 scripts/eval_task_decomposition.py validate-catalog
```

This is offline and uses only Python standard library modules.

## Materialize a fixture

Provide the local source repository explicitly. The catalog intentionally does not encode absolute machine paths.

```bash
python3 scripts/eval_task_decomposition.py materialize \
  --fixture free-design-otaku-staff-console \
  --repo-root otaku-odyssey=/home/you/dev/otaku-odyssey \
  --output "$JCODE_SCRATCH_DIR/evals/free-design-otaku-staff-console"
```

The command refuses to overwrite an existing output path.

## Validate prompt metadata

Prompt records live separately from fixture metadata so original or reconstructed prompts can be improved without changing repository/commit facts.

```bash
python3 scripts/eval_task_decomposition.py validate-prompt-catalog
```

Each prompt record declares its fixture, whether it is `original` or `reconstructed`, confidence, source, prompt text, and notes. The initial prompt catalog covers the three pilot categories: free design/product, infra/platform/config, and business/domain logic.

## Prepare a run without running evals

Use `prepare-run` before a real eval to validate the local inputs and baseline mode. It checks the fixture, prompt metadata, local repository, base commit, gold proposal commit, output path, and selected baseline mode. It does not create a checkout and does not call Jcode or any model.

```bash
python3 scripts/eval_task_decomposition.py prepare-run \
  --fixture free-design-otaku-staff-console \
  --repo-root otaku-odyssey=/home/you/dev/otaku-odyssey \
  --baseline-mode jcode-openspec \
  --output "$JCODE_SCRATCH_DIR/evals/free-design-otaku-staff-console/jcode-openspec"
```

Supported baseline modes:

- `openspec-gold`: historical gold proposal reference.
- `jcode-no-openspec`: Jcode planning without OpenSpec guidance.
- `jcode-openspec`: Jcode planning with OpenSpec guidance.
- `jcode-openspec-orchestrated`: Jcode planning with OpenSpec guidance and orchestration.

## Score generated artifacts

After an evaluator creates an OpenSpec change directory for the same fixture, score it against the historical gold proposal commit:

```bash
python3 scripts/eval_task_decomposition.py score-artifacts \
  --fixture free-design-otaku-staff-console \
  --repo-root otaku-odyssey=/home/you/dev/otaku-odyssey \
  --candidate path/to/openspec/changes/generated-change
```

The score is deterministic and heuristic. It checks required artifact presence and token overlap with the gold OpenSpec artifacts. It is not a semantic judge.

## Validate human rubric scores

After a candidate exists, a reviewer can record semantic scores in JSON and validate the structure before aggregation:

```bash
python3 scripts/eval_task_decomposition.py validate-rubric-score \
  --score path/to/rubric-score.json
```

Rubric scores use five 1-5 dimensions: requirement coverage, decomposition quality, risk handling, scope control, and executability. The validator checks every score and note and emits the computed average.

## Recommended evaluation loop

1. Materialize the fixture at `base_commit`.
2. Give Jcode the fixture prompt or reconstructed situation.
3. Ask Jcode to create an OpenSpec proposal, design, tasks, and delta specs.
4. Run `score-artifacts` against the generated change directory.
5. Apply the category rubric in `rubrics/` for qualitative review.

## Fixture categories

- Free design/product choices
- Business/domain logic
- Infra/platform/config
- Test strategy/E2E remediation
- Data/schema/migration
- Auth/security/permissions
- Observability/telemetry
- Developer tooling/agent integration
- Refactor/dead-code/entropy cleanup
- UI/UX polish

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

## Score generated artifacts

After an evaluator creates an OpenSpec change directory for the same fixture, score it against the historical gold proposal commit:

```bash
python3 scripts/eval_task_decomposition.py score-artifacts \
  --fixture free-design-otaku-staff-console \
  --repo-root otaku-odyssey=/home/you/dev/otaku-odyssey \
  --candidate path/to/openspec/changes/generated-change
```

The score is deterministic and heuristic. It checks required artifact presence and token overlap with the gold OpenSpec artifacts. It is not a semantic judge.

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

# Model Routing Evaluation

This directory contains offline, stdlib-only tournament scaffolding for comparing Jcode model routes by workflow role.

## Safe offline commands

```bash
python3 scripts/eval_model_routing.py validate
python3 scripts/eval_model_routing.py dry-run-cost
python3 scripts/eval_model_routing.py plan-blocks
python3 scripts/eval_model_routing.py smoke-ready
python3 scripts/eval_model_routing.py selection-report
```

These commands do not schedule trials and report `provider_traffic: false` where applicable. Paid provider execution is intentionally absent from this implementation.

## Evidence flow

1. Freeze a descriptor under `experiments/`.
2. Validate budgets, route availability, pricing completeness, isolation, retry, cache, judge, and stop conditions.
3. Use `dry-run-cost` to get conservative provider and total bounds.
4. Use `plan-blocks` for deterministic provider-aware attempt planning.
5. Append terminal attempt events with `append-event` and replay aggregates with `replay`.
6. Create an immutable non-canonical local bundle with `bundle`.
7. Use `publish-recon` only as a fail-closed adapter until the authoritative Recon command exists.

## Promotion policy

`selection-report` preserves qualification-only finalist logic, keeps holdout blind, records unavailable command integrations as `acceptance_blocked`, and never mutates production routing. A separate reviewed routing-policy change and human approval are required for any promotion.

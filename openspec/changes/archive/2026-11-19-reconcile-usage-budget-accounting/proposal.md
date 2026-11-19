---
execution:
  version: 1
  depends_on: [make-worker-stop-durable]
---
Roadmap owner: [P06: Provider/auth/usage-budget reliability](../../../docs/plans/PRODUCT_ROADMAP.md#p06-providerauthusage-budget-reliability)

## Why

Overnight preflight projects usage with fixed heuristics (`overnight.rs:446-527`, e.g. `delta_max = hours * 7.0 * max_agents_guidance / 2.0` at line 484) and never compares the projection to observed usage at run end. OAuth usage recording exists for OpenAI (`crates/jcode-provider-openai-runtime/src/openai_usage_recording.rs:5-57`) but there is no per-run aggregate across resumed sessions and swarm workers, so the morning report cannot say what a run actually consumed versus what was estimated.

## What Changes

- Aggregate provider-reported usage per overnight run across coordinator, resumed sessions and run-owned swarm workers.
- Consume retained run/session/generation ownership from durable worker control, with event/request dedup across retries, restart and resumed workers. Post-request accounting is observation, not authoritative pre-send monetary reservation or proof of hard currency caps.
- At run end, record `usage_reconciled`: projected range, observed usage, and source (`reported` or `estimated`) per provider.
- Render it in the morning report using the info widget's limit-bar renderer.

## Non-scope

- Enforcement (`enforce-overnight-cancellation-and-limits`).
- Billing/currency conversion.

## Capabilities

### New Capabilities

- `usage-budget-reconciliation`

## Impact

`overnight.rs` report, usage recording hooks, `jcode-usage-types`, review HTML.

## Resolved defaults

1. Providers without usage reporting. Options: (a) omit; (b) show estimated with label. Selected default: (b), honesty requirement favors labelled estimates over silent gaps.

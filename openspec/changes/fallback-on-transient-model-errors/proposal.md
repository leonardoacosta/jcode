---
execution:
  version: 1
  depends_on: [classify-provider-auth-failures]
---
# Proposal: Retire custom automatic model fallback

Roadmap owner: [P06: Provider/auth/usage-budget reliability](../../../docs/plans/PRODUCT_ROADMAP.md#p06-providerauthusage-budget-reliability).

## Why

This change previously proposed and implemented a custom `model_fallback` chain with automatic model switching. The approved upstream v1.0.1 adoption retires that fork-only behavior where upstream supersedes it. The existing implementation and tests remain historical evidence for reconciliation, not an active behavior contract. Retirement must not weaken shared auth/quota classification or route/account no-substitution safeguards.

## What Changes

- Remove the fork-only automatic `model_fallback` chain and its config, health/cooling state, cycle behavior, and automatic model substitution during upstream integration.
- Preserve shared provider failure categories and ensure quota/billing exhaustion is not reclassified as a transient signal for fallback.
- Preserve explicit user-directed model selection/manual recovery only where the upstream interface continues to support it. Do not promise the fork's `fallback_switch` behavior if upstream removes it.
- Do not substitute provider, account, gateway, or model identity as a recovery side effect. Keep auth and quota safeguards authoritative.
- Retire this behavior explicitly through source and test reconciliation. Do not mark the change superseded, archive with `--skip-specs`, or treat previously checked tasks as proof that retirement is complete.

## Capabilities

### Modified Capabilities

- `model-fallback-chain`: Retire the custom automatic fallback chain and retain provider auth/quota and no-substitution safeguards.

## Impact

Remove the fork-only `model_fallback` config, chain execution and health tracking, and fallback-only transcript behavior where upstream replaces it. Retain shared auth/quota classification, route/account protections, and supported manual recovery.

## Decision and boundary

Approved retirement is part of the full upstream v1.0.1 integration. Existing automatic fallback behavior is removed explicitly, not hidden through a supersession marker or skipped spec synchronization. Provider auth/quota safeguards, same-route/identity rules, and confirmed billing/quota classification remain required. This change does not close the separate external gateway acceptance gate.

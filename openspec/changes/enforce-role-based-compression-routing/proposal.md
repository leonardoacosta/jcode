---
execution:
  version: 1
  depends_on: []
---

## Why

### Current correction, 2026-11-03

The user clarified that combo/model endpoints must select their predefined compression in OmniRoute. Jcode must not tell the gateway which profile to use. This supersedes client role-based profile selection described below. Preserve the prior implementation and acceptance notes as history, not as the current request contract.

The bounded Jcode correction removes compression override headers, forced cache bypass, and requested-policy notices from both Chat and Responses requests, including forks and legacy header configuration. Route/model selection and complete review-output instructions remain unchanged. No live gateway setting, provider account, or credential change is included. Gateway per-combo assignment and applied-plan acceptance remain separate unfinished work.

The user approved consistent role-based input compression across the existing Pi5 (`omnipi`) and local Omni (`omni`) routes on 2026-11-02. Model aliases select routing combos but do not prove which compression pipeline ran. Both gateways have no explicit compression-profile assignments and empty effective `compression.comboOverrides`; their global fallback policies differ.

## What Changes

- Select a request-scoped conservative, optimized-trial, or exact input policy from durable task identity, not model-family guesses or prompt text.
- Root conversations, memory extraction, and unclassified chat use conservative session-dedup then lite. Explicit bounded coding workers use optimized-trial RTK aggressive then Caveman ultra with existing protected-content rules. Final independent review and Jev decisions use exact input.
- Bind the two existing gateway routes to equivalent named profiles and a conservative fallback. Keep Pi5 primary and local Omni manually selected. Do not introduce automatic cross-gateway or direct-provider substitution.
- Mark direct routes as lacking gateway compression, without claiming a profile ran. Preserve explicit direct-provider exceptions.
- Verify the actual gateway resolver and receipts, including stored combo mode versus effective override behavior. Exact review must not force ultra-terse output styles.
- Verify one real production TypeSafe Jev evaluation with synthetic nonsecret input and no cross-provider fallback. Missing credentials or failed transport are blockers, not passing evidence.

## Capabilities

### New Capabilities

- `role-based-compression-routing`: explicit role-based gateway request policy, safe fallback, and observable enforcement.

### Modified Capabilities

None. Existing `route-jev-supervision-explicitly` remains outside implementation scope.

## Impact

Jcode provider request transport and fork isolation, agent task classification, memory sidecar calls, council review output policy, and existing Homelab OmniRoute patch/configuration deployment surfaces. Pi5 infrastructure uses its repository-owned deployment path. No new credentials, provider accounts, dependencies, branches, or worktrees. Preserve unrelated changes and existing contributor identity. Source checkout is on `main` while remote HEAD names `dev`; preserve the checkout and do not switch branches or publish unrelated history.

## Authorization and preflight

The user approved the role/profile recommendation and requested DeepSeek workers. Synthetic TypeSafe Jev inference is explicitly authorized. Approval does not bypass repository build, review, deployment, archive, secret-handling, or publication gates. DeepSeek primary-combo workers failed before tool calls with gateway HTTP 503; a bounded attempt through the existing alternate DeepSeek Azure combo retains Pi5 routing. Coordinator continues independent work.

Implementation may proceed after this change validates. Deployment requires fresh final-diff gates, exact target/image/config verification, scoped rollback, and applicable repository gates. Unknown runtime proof remains incomplete. Tasks in this change are the authoritative cross-repository acceptance ledger; repository-local deployment artifacts must link their scope without duplicating completion claims.

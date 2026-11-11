---
execution:
  version: 1
  depends_on: []
---

## Why

### Current correction, 2026-11-03

The user clarified that combo/model endpoints must select their predefined compression in OmniRoute. Jcode must not tell the gateway which profile to use. This supersedes client role-based profile selection described below. Preserve the prior implementation and acceptance notes as history, not as the current request contract.

The bounded Jcode correction removes compression override headers, forced cache bypass, and requested-policy notices from both Chat and Responses requests, including forks and legacy header configuration. Route/model selection and complete review-output instructions remain unchanged. No live gateway setting, provider account, or credential change is included. Gateway per-combo assignment and applied-plan acceptance remain separate unfinished work.

Historical 2026-11-02 approval concerned role profiles on existing Pi5/local gateways. The later explicit 2026-11-03 correction and inherited-default selection supersede client profile selection. Existing aliases and labels alone still do not prove an applied compression pipeline.

## What Changes

- Current contract: Jcode sends the selected combo/model without role-selected compression or forced cache-bypass overrides. Compression selection belongs to the endpoint's inherited defaults. Preserve Chat/Responses forks, route identity and complete review-output instructions.
- Historical role/profile selection below is superseded by the 2026-11-03 correction and is not executable work. Do not restore root/coding/reviewer override headers from old checked tasks.
- Both gateways retain their user-selected inherited defaults and conservative fallback. Differentiated per-combo mapping remains recommendation-only unless separately selected; local Omni stays manually selected and no automatic cross-gateway/direct-provider substitution is allowed.
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

Historical role/profile approval and worker-route attempts are retained in task evidence for attribution, not current authority to select client compression. Earlier synthetic-inference permission does not authorize a new disclosure or paid call during this reconciliation. No model/provider override was used for the failed reconciliation workers.

Native artifact validation does not authorize implementation or deployment. A later selected execution request and fresh task-specific preflight remain required; deployment additionally needs fresh final-diff gates, exact target/image/config verification, scoped rollback, and applicable repository gates. Unknown runtime proof remains incomplete. Tasks in this change are the authoritative cross-repository acceptance ledger; repository-local deployment artifacts must link their scope without duplicating completion claims.

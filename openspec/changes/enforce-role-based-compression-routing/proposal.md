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

- Keep the existing endpoint-owned compression contract: Jcode sends the selected combo/model without role-selected compression or forced cache-bypass overrides. Compression selection belongs to endpoint inherited defaults. Preserve Chat/Responses forks, route identity, complete review-output instructions, and full swarm report/report-completion semantics.
- Adopt upstream compatible HTTP compression transport and retire Jev-specific compression/cache overrides only where they duplicate or conflict with that transport. Do not restore role selector headers or request cache controls from historical checked tasks.
- Preserve explicit direct-provider routes and their no-substitution rules. The external gateway applied-policy/receipt gate remains separate and open; transport tests do not close it.
- Keep historical gateway and TypeSafe Jev evidence as evidence only. Do not add a new live provider call or configuration mutation to this bounded reconciliation.

## Capabilities

### New Capabilities

- `role-based-compression-routing`: explicit role-based gateway request policy, safe fallback, and observable enforcement.

### Modified Capabilities

None. Existing `route-jev-supervision-explicitly` remains outside implementation scope.

## Impact

Jcode provider HTTP compression/Jev request transport, route preservation, complete report delivery and report completion. No new credentials, provider accounts, dependencies, branches, or worktrees. Preserve unrelated changes and existing contributor identity. Source checkout remains in place.

## Authorization and preflight

Historical role/profile approval and worker-route attempts are retained in task evidence for attribution, not current authority to select client compression. Earlier synthetic-inference permission does not authorize a new disclosure or paid call during this reconciliation. No model/provider override was used for the failed reconciliation workers.

Native artifact validation does not authorize implementation or deployment. A later selected execution request and fresh task-specific preflight remain required; deployment additionally needs fresh final-diff gates, exact target/image/config verification, scoped rollback, and applicable repository gates. Unknown runtime proof remains incomplete. Tasks in this change are the authoritative cross-repository acceptance ledger; repository-local deployment artifacts must link their scope without duplicating completion claims.

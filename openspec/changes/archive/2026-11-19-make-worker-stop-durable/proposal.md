---
execution:
  version: 1
  depends_on: []
---
# Proposal

Roadmap owner: [P04: Swarm collaboration and handoffs](../../../docs/plans/PRODUCT_ROADMAP.md#p04-swarm-collaboration-and-handoffs). Supporting boundary: P08 session-control consumers. User selected existing-daemon scope on 2026-11-07. Proposal authoring was planning-only. On 2026-11-10 the user explicitly authorized Apply and eligible dependent plans, scoped local commits on `main`, GPT 6.1 Sol independent review, and local OpenSpec closeout after all gates pass. Shared-daemon activation, push, merge, deployment, real microphone/login flows and disclosure of private data to external providers remain excluded.

## Why

A worker recorded successful edits after `swarm stop` returned `Stopped agent`; global discovery later exposed the same ID in another swarm while local control rejected it. Stop currently removes membership without awaiting active execution, so ownership transfer and completion reports can outpace actual settlement ([investigation](../../../docs/research/worker-session-control-2026-11-07.md)).

## What Changes

- Extend the existing daemon's control machinery, persisted session/swarm state and active-turn registry into one authority for all sessions hosted by that daemon. No additional manager process, database or cross-host service.
- Retain stable session identity and owner authorization separately from swarm membership. Resolve exact session IDs daemon-wide before authorization; aliases remain swarm-scoped. Discovery or messaging never grants cancellation authority.
- Persist a monotonic execution generation and stop intent before acknowledging stop. Block automatic registration, queued wakes, recovery and native mutating-tool admission from restoring a revoked generation.
- Distinguish `stop_requested`, `draining`, `stopped` and `unresolved`. Return terminal stop success only after tracked execution and owned tool work settle. Preserve claims and forbid ownership transfer while settlement is unproved. Existing arbitrary shell/MCP/external effects are not silently treated as cancellable or rolled back.
- Bind accepted work, completion reports and `await_members` caches to an execution generation. Old reports remain readable as history but cannot satisfy a newer assignment. Missing membership is not completion or quiescence.
- Reconcile persisted stop/owner/generation state after restart without rerunning cancelled work. Explicit authorized resume can admit a new generation only after the predecessor settles.
- Expose request/generation/settlement evidence through existing swarm status and stop results without prompts, file contents or secrets.
- **BREAKING semantics:** `Stopped agent` no longer means membership removal, and unknown members no longer satisfy completion waits. Additive protocol fields remain backward readable; old clients must receive explicit pending/error responses rather than false success.

## Capabilities

### New Capabilities

- `worker-session-control`: daemon-scoped identity/ownership, durable stop admission, observable settlement, safe resume and diagnostic control results.

### Modified Capabilities

- `swarm-state-persistence`: preserve stop intent, owner and generations across recovery; distinguish runnable recovery from explicit cancellation.
- `swarm-communicate-reliability`: generation-bound reports/waits and truthful stop deadlines/results.
- `swarm-file-claims`: release only after execution and tracked mutation settlement, not mere membership removal or an orphan label.

## Impact

- Existing `crates/jcode-app-core/src/server/` control, lightweight registration, live-turn, recovery, swarm persistence/claims and wait paths; `turn_cancel_registry.rs`; agent tool admission and owned tool/background accounting.
- `crates/jcode-swarm-core`, protocol/session storage and swarm tool response schemas. SDK/harness consumers need compatibility checks, not a separate control authority or Desktop application changes.
- Extend existing tests and public-interface isolated daemon acceptance. No new runtime dependency is selected. Keep existing async persistence ordering and user data.
- Existing `add-swarm-handoff-contracts` retains revision/check/budget report enrichment. This proposal owns generation identity and wait freshness because stop/reassignment needs them even without that enrichment. Consumer proposals explicitly depend on this foundation where they require its control semantics; this foundation does not depend on its consumers. Existing SDK contract tests can progress first and gain additive generation cases. Serialize overlapping schema edits and revalidate both sides, rather than adding a false prerequisite for UI or report enrichment.
- Archived recovery, claims and communicate changes are implemented baseline, not unexecuted prerequisites. `depends_on: []` introduces no implementation dependency on unrelated roadmap phases.

## Scope and delivery boundary

Required: all sessions on the existing daemon, owner-preserving exact-ID resolution, durable stop, no stale readmission, safe generation-bound resume/waits and honest unresolved work. Excluded: multi-daemon/host coordination, new permission grants, universal shell sandboxing, killing unrelated OS processes, rollback of external effects, application task recovery and shared daemon restart/deployment.

A later Apply must first reproduce the selected failure on an isolated socket, run final-diff gates and public acceptance, obtain independent review, commit verified work and archive through the native workflow. Artifact validity is not implementation acceptance. This authoring run makes no runtime behavior claim or issue/PR publication.

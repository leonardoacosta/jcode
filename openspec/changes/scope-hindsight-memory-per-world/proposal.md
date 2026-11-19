---
execution:
  version: 1
  depends_on: [add-attributable-memory-recall, workspace-aware-context]
---
Roadmap owner: [P07: Memory/context quality and privacy](../../../docs/plans/PRODUCT_ROADMAP.md#p07-memorycontext-quality-and-privacy)

## Why

Jcode's existing memory attribution and verified workspace routing own separate parts of safe memory use, but neither defines a native remote-memory adapter. The research found that a Hindsight bank ID selects a route, not an authorization boundary; retain retries may duplicate data; and Hindsight forget/invalidation is soft delete ([scratch/verify-tern-hindsight.md:16-21,57-61](../../../scratch/verify-tern-hindsight.md)). Workspace identity is a verified policy boundary, while Tern sessions are presentation groupings, not workspaces ([scratch/agent-workflow-mining.md:122-140](../../../scratch/agent-workflow-mining.md)).

## What Changes

- Add a native adapter for an explicitly configured external or local Hindsight HTTP endpoint, integrated with the existing memory lifecycle rather than exposed as an unscoped MCP server.
- Bind each verified world to its own named bank and distinct credential reference. Treat bank selection as routing only; deny cross-world recall and never route an unknown world to a shared/default bank.
- Keep automatic retention disabled until the user opts in. Generate Jcode-side operation keys from stable source and turn identifiers to make retry handling deterministic without claiming server-side exactly-once writes.
- Treat recalled content as untrusted evidence. Preserve source, time, bank and provenance in the existing attribution path; do not elevate recalled text to instructions.
- Bound request timeouts, result sizes and work queues. Surface failure and cancellation; never silently switch provider, endpoint or bank.
- Document forget as exclusion from future recall through soft invalidation, not guaranteed hard erasure.

## Capabilities

### New Capabilities

- `hindsight-memory-scope`: configured Hindsight transport, per-world bank/credential isolation, safe recall/retain behavior and visible operational state.

### Modified Capabilities

None. Existing attribution and workspace capabilities remain their owners; this change depends on them rather than duplicating their requirements.

## Impact

Expected surfaces include typed configuration and secret references, the memory adapter/lifecycle, verified world resolution, prompt provenance and user-visible status. No code, live configuration, bank, credentials or endpoint is changed by this proposal.

## Non-goals

No Hindsight server installation, automatic dual-write, default/shared bank, cross-world access, per-project policy, Tern authorization or memory policy, or claim of hard deletion. Tern is only an optional presentation surface for inspecting/editing artifacts; it does not own memory or access policy.

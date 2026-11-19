---
execution:
  version: 1
  depends_on: [fallback-on-transient-model-errors, classify-provider-auth-failures, surface-worker-stall-and-block-reasons, enforce-role-based-compression-routing, remote-client-link-copy]
---
# Proposal: Adopt upstream v1.0.1

## Why

The fork is based at `dc00085917c130dccce7fd423c1495a44231f4cb` and upstream v1.0.1 is `10114429c8133994861d6c915a2483d397c0f7c2`. The approved update is a full history merge, not a squash, cherry-pick series, or replacement of the fork's history. The change must retain valuable fork behavior while removing custom code that duplicates upstream behavior or causes conflicting policy.

## Approved delivery contract

- Merge the full upstream v1.0.1 history into the existing `main` checkout, in place. Do not create a branch or worktree and do not rewrite or discard either history.
- Preserve unrelated custom features and attribution. Resolve conflicts by comparing both histories and their tests; do not treat upstream presence alone as permission to delete fork behavior.
- Use swarms where parallel work has disjoint file/resource ownership. Keep one integration coordinator for shared files, merge state, test results, and promotion. Run independent review agents against the final candidate after the gates.
- Build a release candidate (RC) artifact immutably and test that exact artifact in isolated runtime/test surfaces. Promote only that tested artifact and only if all required gates are green. Never promote a later rebuild under the same RC identity.
- Use scoped local commits on `main` as changes are verified. No force operation or history rewrite. Push, publication, and deployment are not part of this contract.

## Adopt upstream behavior

Adopt upstream agent-to-session binding, HTTP compression transport, and the dedicated swarm page with report coalescing. Retire fork-only inline swarm strip controls, custom automatic model fallback (`model_fallback` and `model-fallback-chain`), and Jev-specific compression/cache overrides where upstream supersedes them. Preserve the fork's thin named-provider session affinity, long but bounded gateway `Retry-After` handling, complete swarm reports and report-completion semantics, orphan explicit retry, and existing stop/owner safety. Keep remote routing, `/links`, and `/mouse`; remote clipboard uses OSC 52 only, with an explicit native override.

Preserve council/file claims/role slots and other custom features that do not conflict. Do not expand or mark complete `make-worker-stop-durable`; its acceptance remains unfinished. The external gateway acceptance gate and physical-device remote-client gate remain open, independent gates.

## Non-scope and safety

- No canonical spec, archived change, roadmap, source, or release artifact is changed by this planning-only proposal.
- Do not activate an untested binary on the shared daemon. Do not send external traffic or change gateway configuration as a substitute for isolated acceptance.
- Do not close external gateway or physical-device gates based on static checks, mocks, or another gate's result.
- Do not use a supersession marker or `--skip-specs` to bypass the required explicit retirement of fallback behavior.

## Capabilities

### New Capabilities

- `upstream-v1-0-1-adoption`: Preserve the approved merge, reconciliation, immutable-RC, isolated-test, and green-only promotion contract.

## Dependencies

This integration coordinates the active fallback retirement, provider auth safeguards, swarm status/reporting, endpoint-owned compression, and remote client link/clipboard changes. The listed changes remain owners of their respective acceptance until their scoped contracts are reconciled. `make-worker-stop-durable` remains an external unfinished prerequisite for stop behavior, not a task this integration completes.
